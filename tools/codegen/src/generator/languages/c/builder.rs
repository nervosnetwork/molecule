use std::io;

use super::utilities::IdentPrefix;
use crate::ast::verified::{self as ast, DefaultContent, HasName};

pub(super) trait GenBuilder: IdentPrefix + DefaultContent {
    fn gen_builder_interfaces_internal<W: io::Write>(&self, writer: &mut W) -> io::Result<()>;

    fn gen_builder_function_build<W: io::Write>(&self, _writer: &mut W) -> io::Result<()> {
        Ok(())
    }

    fn gen_builder_interfaces<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        self.gen_builder_interfaces_internal(writer)?;
        self.define_builder_macro(writer, "_clear(b)", "mol_builder_discard(b)")?;
        Ok(())
    }

    fn gen_builder_functions<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        self.gen_builder_function_build(writer)?;
        Ok(())
    }

    fn gen_builder_interface_build<W: io::Write>(
        &self,
        writer: &mut W,
        func_name_opt: Option<&str>,
    ) -> io::Result<()> {
        if let Some(ref func_name) = func_name_opt {
            let macro_content = format!("{}(b)", func_name);
            self.define_builder_macro(writer, "_build(b)", &macro_content)?;
        } else {
            self.define_builder_function(writer, "_build", "(mol_builder_t)", "mol_seg_res_t")?;
        }
        Ok(())
    }

    fn gen_default<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        let default_content = self.default_content();
        let constant_name = format!(
            "const uint8_t {}[{}]",
            self.default_constant(),
            default_content.len()
        );
        write!(writer, "{:64} =  {{", constant_name)?;
        if default_content.len() > 4 {
            for (index, byte) in default_content.into_iter().enumerate() {
                if index % 12 == 0 {
                    writeln!(writer)?;
                    write!(writer, "{:4}", "")?;
                } else {
                    write!(writer, " ")?;
                }
                if byte == 0 {
                    write!(writer, "____,")?;
                } else {
                    write!(writer, "0x{:02x},", byte)?;
                }
            }
            writeln!(writer)?;
        } else {
            let mut not_first = false;
            for byte in default_content.into_iter() {
                if not_first {
                    write!(writer, ", ")?;
                } else {
                    not_first = true;
                }
                if byte == 0 {
                    write!(writer, "____")?;
                } else {
                    write!(writer, "0x{:02x}", byte)?;
                }
            }
        }
        writeln!(writer, "}};")?;
        Ok(())
    }
}

fn calculate_capacity(used: usize) -> usize {
    used.next_power_of_two()
}

impl GenBuilder for ast::Option_ {
    fn gen_builder_interfaces_internal<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        {
            self.define_builder_macro(
                writer,
                "_init(b)",
                "mol_builder_initialize_fixed_size(b, 0)",
            )?;
        }
        {
            self.define_builder_macro(writer, "_set(b, p, l)", "mol_option_builder_set(b, p, l)")?;
        }
        self.gen_builder_interface_build(writer, Some("mol_builder_finalize_simple"))?;
        Ok(())
    }
}

impl GenBuilder for ast::Union {
    fn gen_builder_interfaces_internal<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        {
            let id = 0;
            let default = &self.inner[id].typ;
            let (len, name) = if default.is_atom() {
                (1, "NULL".to_owned())
            } else {
                let len = default.default_content().len();
                let name = default.default_constant();
                (len, format!("&{}", name))
            };
            let data_capacity = calculate_capacity(molecule::NUMBER_SIZE + len);
            let macro_content = format!(
                "mol_union_builder_initialize(b, {}, {}, {}, {})",
                data_capacity, id, name, len
            );
            self.define_builder_macro(writer, "_init(b)", &macro_content)?;
        }
        for (item_id, item) in self.inner.iter().enumerate() {
            let (macro_sig_tail, macro_content) = if item.typ.is_atom() {
                (
                    format!("_set_{}(b, p)", item.typ.name()),
                    format!("mol_union_builder_set_byte(b, {}, p)", item_id),
                )
            } else {
                (
                    format!("_set_{}(b, p, l)", item.typ.name()),
                    format!("mol_union_builder_set(b, {}, p, l)", item_id),
                )
            };
            self.define_builder_macro(writer, &macro_sig_tail, &macro_content)?;
        }
        self.gen_builder_interface_build(writer, Some("mol_builder_finalize_simple"))?;
        Ok(())
    }
}

impl GenBuilder for ast::Array {
    fn gen_builder_interfaces_internal<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        {
            let macro_content = format!(
                "mol_builder_initialize_fixed_size(b, {})",
                self.total_size()
            );
            self.define_builder_macro(writer, "_init(b)", &macro_content)?;
        }
        for i in 0..self.item_count {
            let macro_sig_tail = format!("_set_nth{}(b, p)", i);
            let item_offset = self.item_size * i;
            let macro_content = if self.typ.is_atom() {
                format!("mol_builder_set_byte_by_offset(b, {}, p)", item_offset)
            } else {
                format!(
                    "mol_builder_set_by_offset(b, {}, p, {})",
                    item_offset, self.item_size
                )
            };
            self.define_builder_macro(writer, &macro_sig_tail, &macro_content)?;
        }
        self.gen_builder_interface_build(writer, Some("mol_builder_finalize_simple"))?;
        Ok(())
    }
}

impl GenBuilder for ast::Struct {
    fn gen_builder_interfaces_internal<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        {
            let macro_content = format!(
                "mol_builder_initialize_fixed_size(b, {})",
                self.total_size()
            );
            self.define_builder_macro(writer, "_init(b)", &macro_content)?;
        }
        let mut field_offset = 0;
        for (f, field_size) in self.inner.iter().zip(self.field_size.iter()) {
            let macro_sig_tail = format!("_set_{}(b, p)", f.name);
            let macro_content = if f.typ.is_atom() {
                format!("mol_builder_set_byte_by_offset(b, {}, p)", field_offset)
            } else {
                format!(
                    "mol_builder_set_by_offset(b, {}, p, {})",
                    field_offset, field_size
                )
            };
            self.define_builder_macro(writer, &macro_sig_tail, &macro_content)?;
            field_offset += field_size;
        }
        self.gen_builder_interface_build(writer, Some("mol_builder_finalize_simple"))?;
        Ok(())
    }
}

impl GenBuilder for ast::FixVec {
    fn gen_builder_interfaces_internal<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        {
            let data_capacity = calculate_capacity(self.item_size * 16);
            let macro_content = format!("mol_fixvec_builder_initialize(b, {})", data_capacity);
            self.define_builder_macro(writer, "_init(b)", &macro_content)?;
        }
        {
            let macro_content = if self.typ.is_atom() {
                "mol_fixvec_builder_push_byte(b, p)".to_owned()
            } else {
                format!("mol_fixvec_builder_push(b, p, {})", self.item_size)
            };
            self.define_builder_macro(writer, "_push(b, p)", &macro_content)?;
        }
        self.gen_builder_interface_build(writer, Some("mol_fixvec_builder_finalize"))?;
        Ok(())
    }
}

impl GenBuilder for ast::DynVec {
    fn gen_builder_interfaces_internal<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        {
            let data_capacity = calculate_capacity(self.typ.default_content().len() * 16);
            let number_capacity = calculate_capacity(molecule::NUMBER_SIZE * 16);
            let macro_content = format!(
                "mol_builder_initialize_with_capacity(b, {}, {})",
                data_capacity, number_capacity
            );
            self.define_builder_macro(writer, "_init(b)", &macro_content)?;
        }
        {
            self.define_builder_macro(
                writer,
                "_push(b, p, l)",
                "mol_dynvec_builder_push(b, p, l)",
            )?;
        }
        self.gen_builder_interface_build(writer, Some("mol_dynvec_builder_finalize"))?;
        Ok(())
    }
}

impl GenBuilder for ast::Table {
    fn gen_builder_interfaces_internal<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        {
            let data_capacity = calculate_capacity(self.default_content().len() * 4);
            let macro_content = format!(
                "mol_table_builder_initialize(b, {}, {})",
                data_capacity,
                self.inner.len()
            );
            self.define_builder_macro(writer, "_init(b)", &macro_content)?;
        }
        for (i, f) in self.inner.iter().enumerate() {
            let (macro_sig_tail, macro_content) = if f.typ.is_atom() {
                (
                    format!("_set_{}(b, p)", f.name),
                    format!("mol_table_builder_add_byte(b, {}, p)", i),
                )
            } else {
                (
                    format!("_set_{}(b, p, l)", f.name),
                    format!("mol_table_builder_add(b, {}, p, l)", i),
                )
            };
            self.define_builder_macro(writer, &macro_sig_tail, &macro_content)?;
        }
        self.gen_builder_interface_build(writer, None)?;
        Ok(())
    }

    fn gen_builder_function_build<W: io::Write>(&self, o: &mut W) -> io::Result<()> {
        let func_name = format!("{}_build", self.builder_prefix());
        let offset = molecule::NUMBER_SIZE * (self.inner.len() + 1);
        w!(o, "mol_seg_res_t {} (mol_builder_t builder) {{ ", func_name);
        w!(o, "    mol_seg_res_t res;                                 ");
        w!(o, "    res.errno = MOL_OK;                                ");
        w!(o, "    mol_num_t offset = {};                     ", offset);
        if !self.inner.is_empty() {
            w!(o, "    mol_num_t len;                                     ");
        }
        w!(o, "    res.seg.size = offset;                             ");
        for (i, f) in self.inner.iter().enumerate() {
            let li = i * 2 + 1;
            let len = f.typ.default_content().len();
            w!(o, "    len = builder.number_ptr[{}];              ", li);
            w!(o, "    res.seg.size += len == 0 ? {} : len;      ", len);
        }
        w!(o, "    res.seg.ptr = (uint8_t*)malloc(res.seg.size);      ");
        w!(o, "    uint8_t *dst = res.seg.ptr;                        ");
        w!(o, "    mol_pack_number(dst, &res.seg.size);               ");
        w!(o, "    dst += MOL_NUM_T_SIZE;                             ");
        for (i, f) in self.inner.iter().enumerate() {
            let li = i * 2 + 1;
            let len = f.typ.default_content().len();
            w!(o, "    mol_pack_number(dst, &offset);                 ");
            w!(o, "    dst += MOL_NUM_T_SIZE;                         ");
            w!(o, "    len = builder.number_ptr[{}];              ", li);
            w!(o, "    offset += len == 0 ? {} : len;            ", len);
        }
        if !self.inner.is_empty() {
            w!(o, "    uint8_t *src = builder.data_ptr;                   ");
        }
        for (i, f) in self.inner.iter().enumerate() {
            let li = i * 2 + 1;
            let oi = i * 2;
            let len = f.typ.default_content().len();
            w!(o, "    len = builder.number_ptr[{}];              ", li);
            w!(o, "    if (len == 0) {{                               ");
            w!(o, "        len = {};                             ", len);
            if f.typ.is_atom() {
                w!(o, "        *dst = 0;                              ");
            } else {
                let name = f.typ.default_constant();
                w!(o, "        memcpy(dst, &{}, len);           ", name);
            }
            w!(o, "    }} else {{                                     ");
            w!(o, "        mol_num_t of = builder.number_ptr[{}]; ", oi);
            w!(o, "        memcpy(dst, src+of, len);                  ");
            w!(o, "    }}                                             ");
            w!(o, "    dst += len;                                    ");
        }
        w!(o, "    mol_builder_discard(builder);                      ");
        w!(o, "    return res;                                        ");
        w!(o, "}}                                                     ");
        Ok(())
    }
}

impl GenBuilder for ast::TopDecl {
    fn gen_builder_interfaces_internal<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        match self {
            ast::TopDecl::Option_(ref i) => i.gen_builder_interfaces_internal(writer),
            ast::TopDecl::Union(ref i) => i.gen_builder_interfaces_internal(writer),
            ast::TopDecl::Array(ref i) => i.gen_builder_interfaces_internal(writer),
            ast::TopDecl::Struct(ref i) => i.gen_builder_interfaces_internal(writer),
            ast::TopDecl::FixVec(ref i) => i.gen_builder_interfaces_internal(writer),
            ast::TopDecl::DynVec(ref i) => i.gen_builder_interfaces_internal(writer),
            ast::TopDecl::Table(ref i) => i.gen_builder_interfaces_internal(writer),
            ast::TopDecl::Atom(_) => unreachable!(),
        }
    }

    fn gen_builder_function_build<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        match self {
            ast::TopDecl::Option_(ref i) => i.gen_builder_function_build(writer),
            ast::TopDecl::Union(ref i) => i.gen_builder_function_build(writer),
            ast::TopDecl::Array(ref i) => i.gen_builder_function_build(writer),
            ast::TopDecl::Struct(ref i) => i.gen_builder_function_build(writer),
            ast::TopDecl::FixVec(ref i) => i.gen_builder_function_build(writer),
            ast::TopDecl::DynVec(ref i) => i.gen_builder_function_build(writer),
            ast::TopDecl::Table(ref i) => i.gen_builder_function_build(writer),
            ast::TopDecl::Atom(_) => unreachable!(),
        }
    }
}
