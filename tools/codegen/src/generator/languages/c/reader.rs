use std::io;

use super::utilities::IdentPrefix;
use crate::ast;

pub(super) trait GenReader: IdentPrefix {
    fn gen_reader_interfaces_internal<W: io::Write>(&self, writer: &mut W) -> io::Result<()>;

    fn gen_reader_interfaces<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        self.gen_reader_interfaces_internal(writer)?;
        Ok(())
    }

    fn gen_reader_functions<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        self.gen_reader_function_verify(writer)?;
        Ok(())
    }

    fn gen_reader_function_verify<W: io::Write>(&self, _writer: &mut W) -> io::Result<()> {
        Ok(())
    }
}

impl GenReader for ast::Option_ {
    fn gen_reader_interfaces_internal<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        {
            self.define_reader_function(
                writer,
                "_verify",
                "(const mol_seg_t*, bool)",
                "mol_errno",
            )?;
        }
        {
            self.define_reader_macro(writer, "_is_none(s)", "mol_option_is_none(s)")?;
        }
        Ok(())
    }

    fn gen_reader_function_verify<W: io::Write>(&self, o: &mut W) -> io::Result<()> {
        let func_name = format!("{}_verify", self.reader_prefix());
        let api_decorator = self.api_decorator();
        w!(
            o,
            "{} mol_errno {} (const mol_seg_t *input, bool compatible) {{",
            api_decorator,
            func_name
        );
        if self.item().typ().is_byte() {
            w!(o, "    if (input->size > 1) {{                            ");
            w!(o, "        return MOL_ERR;                                ");
        } else {
            let f = format!("{}_verify", self.item().typ().reader_prefix());
            w!(o, "    if (input->size != 0) {{                           ");
            w!(o, "        return {}(input, compatible);               ", f);
        }
        w!(o, "    }} else {{                                         ");
        w!(o, "        return MOL_OK;                                 ");
        w!(o, "    }}                                                 ");
        w!(o, "}}                                                     ");
        Ok(())
    }
}

impl GenReader for ast::Union {
    fn gen_reader_interfaces_internal<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        {
            self.define_reader_function(
                writer,
                "_verify",
                "(const mol_seg_t*, bool)",
                "mol_errno",
            )?;
        }
        {
            self.define_reader_macro(writer, "_unpack(s)", "mol_union_unpack(s)")?;
        }
        Ok(())
    }

    fn gen_reader_function_verify<W: io::Write>(&self, o: &mut W) -> io::Result<()> {
        let func_name = format!("{}_verify", self.reader_prefix());
        let api_decorator = self.api_decorator();
        w!(
            o,
            "{} mol_errno {} (const mol_seg_t *input, bool compatible) {{",
            api_decorator,
            func_name
        );
        w!(o, "    if (input->size < MOL_NUM_T_SIZE) {{               ");
        w!(o, "        return MOL_ERR_HEADER;                         ");
        w!(o, "    }}                                                 ");
        w!(o, "    mol_num_t item_id = mol_unpack_number(input->ptr); ");
        w!(o, "    mol_seg_t inner;                                   ");
        w!(o, "    inner.ptr = input->ptr + MOL_NUM_T_SIZE;           ");
        w!(o, "    inner.size = input->size - MOL_NUM_T_SIZE;         ");
        w!(o, "    switch(item_id) {{                                 ");
        for (item_id, item) in self.items().iter().enumerate() {
            w!(o, "        case {}:                              ", item_id);
            if item.typ().is_byte() {
                w!(o, "            return inner.size == 1 ? MOL_OK : MOL_ERR; ");
            } else {
                let f = format!("{}_verify", item.typ().reader_prefix());
                w!(o, "            return {}(&inner, compatible);          ", f);
            }
        }
        w!(o, "        default:                                       ");
        w!(o, "            return MOL_ERR_UNKNOWN_ITEM;               ");
        w!(o, "    }}                                                 ");
        w!(o, "}}                                                     ");
        Ok(())
    }
}

impl GenReader for ast::Array {
    fn gen_reader_interfaces_internal<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        {
            let macro_content = format!("mol_verify_fixed_size(s, {})", self.total_size());
            self.define_reader_macro(writer, "_verify(s, c)", &macro_content)?;
        }
        for i in 0..self.item_count() {
            let macro_sig_tail = format!("_get_nth{}(s)", i);
            let item_offset = self.item_size() * i;
            let macro_content = format!(
                "mol_slice_by_offset(s, {}, {})",
                item_offset,
                self.item_size()
            );
            self.define_reader_macro(writer, &macro_sig_tail, &macro_content)?;
        }
        Ok(())
    }
}

impl GenReader for ast::Struct {
    fn gen_reader_interfaces_internal<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        {
            let macro_content = format!("mol_verify_fixed_size(s, {})", self.total_size());
            self.define_reader_macro(writer, "_verify(s, c)", &macro_content)?;
        }
        let mut field_offset = 0;
        for (f, field_size) in self.fields().iter().zip(self.field_sizes().iter()) {
            let macro_sig_tail = format!("_get_{}(s)", f.name());
            let macro_content = format!("mol_slice_by_offset(s, {}, {})", field_offset, field_size);
            self.define_reader_macro(writer, &macro_sig_tail, &macro_content)?;
            field_offset += field_size;
        }
        Ok(())
    }
}

impl GenReader for ast::FixVec {
    fn gen_reader_interfaces_internal<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        {
            let macro_content = format!("mol_fixvec_verify(s, {})", self.item_size());
            self.define_reader_macro(writer, "_verify(s, c)", &macro_content)?;
        }
        {
            self.define_reader_macro(writer, "_length(s)", "mol_fixvec_length(s)")?;
        }
        {
            let macro_content = format!("mol_fixvec_slice_by_index(s, {}, i)", self.item_size());
            self.define_reader_macro(writer, "_get(s, i)", &macro_content)?;
        }
        if self.item().typ().is_byte() {
            self.define_reader_macro(writer, "_raw_bytes(s)", "mol_fixvec_slice_raw_bytes(s)")?;
        }
        Ok(())
    }
}

impl GenReader for ast::DynVec {
    fn gen_reader_interfaces_internal<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        {
            self.define_reader_function(
                writer,
                "_verify",
                "(const mol_seg_t*, bool)",
                "mol_errno",
            )?;
        }
        {
            self.define_reader_macro(writer, "_length(s)", "mol_dynvec_length(s)")?;
        }
        {
            self.define_reader_macro(writer, "_get(s, i)", "mol_dynvec_slice_by_index(s, i)")?;
        }
        Ok(())
    }

    fn gen_reader_function_verify<W: io::Write>(&self, o: &mut W) -> io::Result<()> {
        let func_name = format!("{}_verify", self.reader_prefix());
        let api_decorator = self.api_decorator();
        let f = format!("{}_verify", self.item().typ().reader_prefix());
        w!(
            o,
            "{} mol_errno {} (const mol_seg_t *input, bool compatible) {{",
            api_decorator,
            func_name
        );
        w!(o, "    if (input->size < MOL_NUM_T_SIZE) {{               ");
        w!(o, "        return MOL_ERR_HEADER;                         ");
        w!(o, "    }}                                                 ");
        w!(o, "    uint8_t *ptr = input->ptr;                         ");
        w!(o, "    mol_num_t total_size = mol_unpack_number(ptr);     ");
        w!(o, "    if (input->size != total_size) {{                  ");
        w!(o, "        return MOL_ERR_TOTAL_SIZE;                     ");
        w!(o, "    }}                                                 ");
        w!(o, "    if (input->size == MOL_NUM_T_SIZE) {{              ");
        w!(o, "        return MOL_OK;                                 ");
        w!(o, "    }}                                                 ");
        w!(o, "    if (input->size < MOL_NUM_T_SIZE * 2) {{           ");
        w!(o, "        return MOL_ERR_HEADER;                         ");
        w!(o, "    }}                                                 ");
        w!(o, "    ptr += MOL_NUM_T_SIZE;                             ");
        w!(o, "    mol_num_t offset = mol_unpack_number(ptr);         ");
        w!(o, "    if (offset % 4 > 0 || offset < MOL_NUM_T_SIZE*2) {{");
        w!(o, "        return MOL_ERR_OFFSET;                         ");
        w!(o, "    }}                                                 ");
        w!(o, "    mol_num_t item_count = offset / 4 - 1;             ");
        w!(o, "    if (input->size < MOL_NUM_T_SIZE*(item_count+1)) {{");
        w!(o, "        return MOL_ERR_HEADER;                         ");
        w!(o, "    }}                                                 ");
        w!(o, "    mol_num_t end;                                     ");
        w!(o, "    for (mol_num_t i=1; i<item_count; i++) {{          ");
        w!(o, "        ptr += MOL_NUM_T_SIZE;                         ");
        w!(o, "        end = mol_unpack_number(ptr);                  ");
        w!(o, "        if (offset > end) {{                           ");
        w!(o, "            return MOL_ERR_OFFSET;                     ");
        w!(o, "        }}                                             ");
        w!(o, "        mol_seg_t inner;                               ");
        w!(o, "        inner.ptr = input->ptr + offset;               ");
        w!(o, "        inner.size = end - offset;                     ");
        w!(o, "        mol_errno errno = {}(&inner, compatible);   ", f);
        w!(o, "        if (errno != MOL_OK) {{                        ");
        w!(o, "            return MOL_ERR_DATA;                       ");
        w!(o, "        }}                                             ");
        w!(o, "        offset = end;                                  ");
        w!(o, "    }}                                                 ");
        w!(o, "    if (offset > total_size) {{                        ");
        w!(o, "        return MOL_ERR_OFFSET;                         ");
        w!(o, "    }}                                                 ");
        w!(o, "    mol_seg_t inner;                                   ");
        w!(o, "    inner.ptr = input->ptr + offset;                   ");
        w!(o, "    inner.size = total_size - offset;                  ");
        w!(o, "    return {}(&inner, compatible);                  ", f);
        w!(o, "}}                                                     ");
        Ok(())
    }
}

impl GenReader for ast::Table {
    fn gen_reader_interfaces_internal<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        {
            self.define_reader_function(
                writer,
                "_verify",
                "(const mol_seg_t*, bool)",
                "mol_errno",
            )?;
        }
        {
            self.define_reader_macro(
                writer,
                "_actual_field_count(s)",
                "mol_table_actual_field_count(s)",
            )?;
        }
        {
            let macro_content = format!("mol_table_has_extra_fields(s, {})", self.fields().len());
            self.define_reader_macro(writer, "_has_extra_fields(s)", &macro_content)?;
        }
        for (i, f) in self.fields().iter().enumerate() {
            let macro_sig_tail = format!("_get_{}(s)", f.name());
            let macro_content = format!("mol_table_slice_by_index(s, {})", i);
            self.define_reader_macro(writer, &macro_sig_tail, &macro_content)?;
        }
        Ok(())
    }

    fn gen_reader_function_verify<W: io::Write>(&self, o: &mut W) -> io::Result<()> {
        let func_name = format!("{}_verify", self.reader_prefix());
        let api_decorator = self.api_decorator();
        let fc = self.fields().len();
        w!(
            o,
            "{} mol_errno {} (const mol_seg_t *input, bool compatible) {{",
            api_decorator,
            func_name
        );
        w!(o, "    if (input->size < MOL_NUM_T_SIZE) {{               ");
        w!(o, "        return MOL_ERR_HEADER;                         ");
        w!(o, "    }}                                                 ");
        w!(o, "    uint8_t *ptr = input->ptr;                         ");
        w!(o, "    mol_num_t total_size = mol_unpack_number(ptr);     ");
        w!(o, "    if (input->size != total_size) {{                  ");
        w!(o, "        return MOL_ERR_TOTAL_SIZE;                     ");
        w!(o, "    }}                                                 ");
        if self.fields().is_empty() {
            w!(o, "    if (input->size == MOL_NUM_T_SIZE) {{              ");
            w!(o, "        return MOL_OK;                                 ");
            w!(o, "    }}                                                 ");
        }
        w!(o, "    if (input->size < MOL_NUM_T_SIZE * 2) {{           ");
        w!(o, "        return MOL_ERR_HEADER;                         ");
        w!(o, "    }}                                                 ");
        w!(o, "    ptr += MOL_NUM_T_SIZE;                             ");
        w!(o, "    mol_num_t offset = mol_unpack_number(ptr);         ");
        w!(o, "    if (offset % 4 > 0 || offset < MOL_NUM_T_SIZE*2) {{");
        w!(o, "        return MOL_ERR_OFFSET;                         ");
        w!(o, "    }}                                                 ");
        w!(o, "    mol_num_t field_count = offset / 4 - 1;            ");
        w!(o, "    if (field_count < {}) {{                       ", fc);
        w!(o, "        return MOL_ERR_FIELD_COUNT;                    ");
        w!(o, "    }} else if (!compatible && field_count > {}) {{", fc);
        w!(o, "        return MOL_ERR_FIELD_COUNT;                    ");
        w!(o, "    }}                                                 ");
        w!(o, "    if (input->size < MOL_NUM_T_SIZE*(field_count+1)){{");
        w!(o, "        return MOL_ERR_HEADER;                         ");
        w!(o, "    }}                                                 ");
        w!(o, "    mol_num_t offsets[field_count+1];                  ");
        w!(o, "    offsets[0] = offset;                               ");
        w!(o, "    for (mol_num_t i=1; i<field_count; i++) {{         ");
        w!(o, "        ptr += MOL_NUM_T_SIZE;                         ");
        w!(o, "        offsets[i] = mol_unpack_number(ptr);           ");
        w!(o, "        if (offsets[i-1] > offsets[i]) {{              ");
        w!(o, "            return MOL_ERR_OFFSET;                     ");
        w!(o, "        }}                                             ");
        w!(o, "    }}                                                 ");
        w!(o, "    if (offsets[field_count-1] > total_size) {{        ");
        w!(o, "        return MOL_ERR_OFFSET;                         ");
        w!(o, "    }}                                                 ");
        if !self.fields().is_empty() {
            w!(o, "    offsets[field_count] = total_size;                 ");
            if self.fields().iter().any(|field| !field.typ().is_byte()) {
                w!(o, "        mol_seg_t inner;                               ");
                w!(o, "        mol_errno errno;                               ");
            }
            for (i, field) in self.fields().iter().enumerate() {
                let j = i + 1;
                if field.typ().is_byte() {
                    w!(o, "        if (offsets[{}] - offsets[{}] != 1) {{   ", j, i);
                    w!(o, "            return MOL_ERR_DATA;                       ");
                    w!(o, "        }}                                             ");
                } else {
                    let f = format!("{}_verify", field.typ().reader_prefix());
                    w!(o, "        inner.ptr = input->ptr + offsets[{}];       ", i);
                    w!(o, "        inner.size = offsets[{}] - offsets[{}];  ", j, i);
                    w!(o, "        errno = {}(&inner, compatible);             ", f);
                    w!(o, "        if (errno != MOL_OK) {{                        ");
                    w!(o, "            return MOL_ERR_DATA;                       ");
                    w!(o, "        }}                                             ");
                }
            }
        }
        w!(o, "    return MOL_OK;                                     ");
        w!(o, "}}                                                     ");
        Ok(())
    }
}

impl GenReader for ast::TopDecl {
    fn gen_reader_interfaces_internal<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        match self {
            ast::TopDecl::Option_(ref i) => i.gen_reader_interfaces_internal(writer),
            ast::TopDecl::Union(ref i) => i.gen_reader_interfaces_internal(writer),
            ast::TopDecl::Array(ref i) => i.gen_reader_interfaces_internal(writer),
            ast::TopDecl::Struct(ref i) => i.gen_reader_interfaces_internal(writer),
            ast::TopDecl::FixVec(ref i) => i.gen_reader_interfaces_internal(writer),
            ast::TopDecl::DynVec(ref i) => i.gen_reader_interfaces_internal(writer),
            ast::TopDecl::Table(ref i) => i.gen_reader_interfaces_internal(writer),
            ast::TopDecl::Primitive(_) => unreachable!(),
        }
    }

    fn gen_reader_function_verify<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        match self {
            ast::TopDecl::Option_(ref i) => i.gen_reader_function_verify(writer),
            ast::TopDecl::Union(ref i) => i.gen_reader_function_verify(writer),
            ast::TopDecl::Array(ref i) => i.gen_reader_function_verify(writer),
            ast::TopDecl::Struct(ref i) => i.gen_reader_function_verify(writer),
            ast::TopDecl::FixVec(ref i) => i.gen_reader_function_verify(writer),
            ast::TopDecl::DynVec(ref i) => i.gen_reader_function_verify(writer),
            ast::TopDecl::Table(ref i) => i.gen_reader_function_verify(writer),
            ast::TopDecl::Primitive(_) => unreachable!(),
        }
    }
}
