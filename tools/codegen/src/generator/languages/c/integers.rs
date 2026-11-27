use crate::{
    ast::{Ast, HasName},
    integer::{ByteOrder, IntegerSpec},
};
use std::io::{self, Write};

fn value_type(spec: IntegerSpec) -> String {
    let base = format!("{}int{}_t", if spec.signed { "" } else { "u" }, spec.bits);
    if spec.bits > 64 {
        format!("mol_{base}")
    } else {
        base
    }
}

fn writer_param_type(spec: IntegerSpec) -> String {
    if spec.bits > 64 {
        value_type(spec)
    } else if spec.signed {
        "int64_t".into()
    } else {
        "uint64_t".into()
    }
}

fn reader_sig(name: &str, spec: IntegerSpec) -> String {
    let typ = value_type(spec);
    format!(
        "MOLECULE_API_DECORATOR mol_errno MolReader_{name}_read_value(const mol_seg_t *input, {typ} *output)"
    )
}

fn builder_sig(name: &str, spec: IntegerSpec) -> String {
    let parameter = writer_param_type(spec);
    format!(
        "MOLECULE_API_DECORATOR mol_errno MolBuilder_{name}_write_value({parameter} value, mol_seg_t *output)"
    )
}

fn declared_integers(ast: &Ast) -> Vec<(String, IntegerSpec)> {
    ast.major_decls()
        .into_iter()
        .filter_map(|decl| {
            ast.integer(decl.name())
                .map(|&spec| (decl.name().to_owned(), spec))
        })
        .collect()
}

fn check_symbol_collisions(ast: &Ast) -> io::Result<()> {
    if ast.integers().is_empty() {
        return Ok(());
    }
    let mut macros = std::collections::HashSet::new();
    for decl in ast.decls() {
        match decl.as_ref() {
            crate::ast::TopDecl::Struct(record) => {
                for field in record.fields() {
                    macros.insert(format!("MolReader_{}_get_{}", decl.name(), field.name()));
                    macros.insert(format!("MolBuilder_{}_set_{}", decl.name(), field.name()));
                }
            }
            crate::ast::TopDecl::Table(table) => {
                for field in table.fields() {
                    macros.insert(format!("MolReader_{}_get_{}", decl.name(), field.name()));
                    macros.insert(format!("MolBuilder_{}_set_{}", decl.name(), field.name()));
                }
            }
            crate::ast::TopDecl::Union(union) => {
                for item in union.items() {
                    macros.insert(format!(
                        "MolBuilder_{}_set_{}",
                        decl.name(),
                        item.typ().name()
                    ));
                }
            }
            _ => {}
        }
    }
    for name in ast.integers().keys() {
        for sym in [
            format!("MolReader_{name}_read_value"),
            format!("MolBuilder_{name}_write_value"),
        ] {
            if macros.contains(&sym) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("E_C_SYMBOL_COLLISION: integer accessor `{sym}` collides with a field macro"),
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn spec_guard(name: &str, spec: IntegerSpec) -> String {
    let sign = if spec.signed { "I" } else { "U" };
    let order = if spec.bits == 8 {
        ""
    } else if spec.byte_order == ByteOrder::Little {
        "LE"
    } else {
        "BE"
    };
    format!("MOL_INTEGER_{sign}{}{order}_{name}", spec.bits)
}

pub(super) fn generate<W: Write>(writer: &mut W, ast: &Ast) -> io::Result<()> {
    check_symbol_collisions(ast)?;
    for (name, spec) in declared_integers(ast) {
        let api = format!("MOL_INTEGER_API_{name}");
        let kind = spec_guard(&name, spec);
        writeln!(writer, "#ifndef {api}\n#define {api}\n#define {kind}")?;
        w!(writer, "{};", reader_sig(&name, spec));
        w!(writer, "{};", builder_sig(&name, spec));
        write_accessors(writer, &name, spec)?;
        writeln!(writer, "#elif !defined({kind})\n#error \"incompatible Molecule integer declarations for {name}\"\n#endif")?;
    }
    Ok(())
}

fn write_accessors<W: Write>(writer: &mut W, name: &str, spec: IntegerSpec) -> io::Result<()> {
    let bits = spec.bits;
    let width = spec.byte_width();
    let little = spec.byte_order == ByteOrder::Little;
    w!(writer, "{} {{", reader_sig(name, spec));
    w!(writer, "    if (!output) return MOL_ERR_DATA;");
    if bits > 64 {
        let count = width / 8;
        w!(
            writer,
            "    return mol_integer_read_limbs(input, {count}, {little}, output->limbs);"
        );
        w!(writer, "}}");
        w!(writer, "{} {{", builder_sig(name, spec));
        w!(
            writer,
            "    return mol_integer_write_limbs(value.limbs, {count}, {little}, output);"
        );
        w!(writer, "}}");
        return Ok(());
    }
    let typ = value_type(spec);
    w!(writer, "    uint64_t value;");
    w!(
        writer,
        "    mol_errno error = mol_integer_read_u64(input, {width}, {little}, &value);"
    );
    w!(writer, "    if (error != MOL_OK) return error;");
    if spec.signed {
        // For a negative value, the complement fits in int64_t even at INT64_MIN.
        w!(writer, "    if (value <= (uint64_t)INT{bits}_MAX) {{");
        w!(writer, "        *output = ({typ})value;");
        w!(writer, "    }} else {{");
        w!(
            writer,
            "        *output = ({typ})(-1 - (int64_t)((uint64_t)UINT{bits}_MAX - value));"
        );
        w!(writer, "    }}");
    } else {
        w!(writer, "    *output = ({typ})value;");
    }
    w!(writer, "    return MOL_OK;");
    w!(writer, "}}");
    w!(writer, "{} {{", builder_sig(name, spec));
    let value = if spec.signed && bits < 64 {
        w!(
            writer,
            "    if (value < INT{bits}_MIN || value > INT{bits}_MAX) return MOL_ERR_DATA;"
        );
        format!("(uint{bits}_t)value")
    } else {
        "(uint64_t)value".into()
    };
    w!(
        writer,
        "    return mol_integer_write_u64({value}, {width}, {little}, output);"
    );
    w!(writer, "}}");
    Ok(())
}
