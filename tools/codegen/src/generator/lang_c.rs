use std::io;

use super::Generator;
use crate::{ast::verified as ast, VERSION};

impl Generator {
    pub(crate) fn generate_c<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        writeln!(writer, "// Generate by Molecule {}", VERSION)?;
        writeln!(writer)?;
        writeln!(writer, r#"#include "molecule.h""#)?;
        writeln!(writer)?;
        for decl in &self.ast.decls[..] {
            match decl.typ {
                ast::TopDeclType::Option_(ref info) => {
                    gen_option(writer, &decl.name, info)?;
                }
                ast::TopDeclType::Union(ref info) => {
                    gen_union(writer, &decl.name, info)?;
                }
                ast::TopDeclType::Array(ref info) => {
                    gen_array(writer, &decl.name, info)?;
                }
                ast::TopDeclType::Struct(ref info) => {
                    gen_struct(writer, &decl.name, info)?;
                }
                ast::TopDeclType::FixedVector(ref info) => {
                    gen_fix_vec(writer, &decl.name, info)?;
                }
                ast::TopDeclType::DynamicVector(ref info) => {
                    gen_dyn_vec(writer, &decl.name, info)?;
                }
                ast::TopDeclType::Table(ref info) => {
                    gen_table(writer, &decl.name, info)?;
                }
                ast::TopDeclType::Atom => unreachable!(),
            };
        }
        Ok(())
    }
}

/*
 * Utilities
 */

fn gen_option<W>(writer: &mut W, origin_name: &str, _info: &ast::Option_) -> io::Result<()>
where
    W: io::Write,
{
    let type_name = "MolOption";
    let macro_sig = format!("MOL_{}()", origin_name);
    write!(writer, "#define")?;
    write!(writer, " {:64}", macro_sig)?;
    write!(writer, " {:9}", type_name)?;
    writeln!(writer)
}

fn gen_union<W>(writer: &mut W, origin_name: &str, _info: &ast::Union) -> io::Result<()>
where
    W: io::Write,
{
    let type_name = "MolUnion";
    let macro_sig = format!("MOL_{}()", origin_name);
    write!(writer, "#define")?;
    write!(writer, " {:64}", macro_sig)?;
    write!(writer, " {:9}", type_name)?;
    writeln!(writer)
}

fn gen_array<W>(writer: &mut W, origin_name: &str, info: &ast::Array) -> io::Result<()>
where
    W: io::Write,
{
    let type_name = "MolArray";
    let macro_sig = format!("MOL_{}(idx)", origin_name);
    write!(writer, "#define")?;
    write!(writer, " {:64}", macro_sig)?;
    write!(
        writer,
        " {:9},{},{},idx",
        type_name, info.item_count, info.item_size,
    )?;
    writeln!(writer)
}

fn gen_struct<W>(writer: &mut W, origin_name: &str, info: &ast::Struct) -> io::Result<()>
where
    W: io::Write,
{
    let type_name = "MolStruct";
    let mut offset = 0;
    let total_size: usize = info.field_size.iter().sum();
    for (field, field_size) in info.inner.iter().zip(info.field_size.iter()) {
        let macro_sig = format!("MOL_{}_{}()", origin_name, field.name);
        write!(writer, "#define")?;
        write!(writer, " {:64}", macro_sig)?;
        write!(
            writer,
            " {:9},{},{},{}",
            type_name, total_size, offset, field_size
        )?;
        writeln!(writer)?;
        offset += field_size;
    }
    Ok(())
}

fn gen_fix_vec<W>(writer: &mut W, origin_name: &str, info: &ast::FixedVector) -> io::Result<()>
where
    W: io::Write,
{
    let type_name = "MolFixVec";
    let macro_sig = format!("MOL_{}(idx)", origin_name);
    write!(writer, "#define")?;
    write!(writer, " {:64}", macro_sig)?;
    write!(writer, " {:9},{},idx", type_name, info.item_size)?;
    writeln!(writer)
}

fn gen_dyn_vec<W>(writer: &mut W, origin_name: &str, _info: &ast::DynamicVector) -> io::Result<()>
where
    W: io::Write,
{
    let type_name = "MolDynVec";
    let macro_sig = format!("MOL_{}(idx)", origin_name);
    write!(writer, "#define")?;
    write!(writer, " {:64}", macro_sig)?;
    write!(writer, " {:9},idx", type_name)?;
    writeln!(writer)
}

fn gen_table<W>(writer: &mut W, origin_name: &str, info: &ast::Table) -> io::Result<()>
where
    W: io::Write,
{
    let type_name = "MolTable";
    let field_count = info.inner.len();
    for (field_index, field) in info.inner.iter().enumerate() {
        let macro_sig = format!("MOL_{}_{}()", origin_name, field.name);
        write!(writer, "#define")?;
        write!(writer, " {:64}", macro_sig)?;
        write!(writer, " {:9},{},{}", type_name, field_count, field_index)?;
        writeln!(writer)?;
    }
    Ok(())
}
