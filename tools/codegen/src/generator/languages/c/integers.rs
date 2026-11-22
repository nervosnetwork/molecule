use crate::{
    ast::{Ast, HasName},
    integer::ByteOrder,
};
use std::io::{self, Write};

pub(super) fn generate<W: Write>(writer: &mut W, ast: &Ast) -> io::Result<()> {
    for decl in ast.major_decls() {
        let Some(spec) = ast.integer(decl.name()) else {
            continue;
        };
        let name = decl.name();
        let bits = spec.bits;
        let typ = format!("{}int{bits}_t", if spec.signed { "" } else { "u" });
        let width = spec.byte_width();
        let little = spec.byte_order == ByteOrder::Little;
        if bits > 64 {
            let typ = format!("mol_{typ}");
            let count = width / 8;
            writeln!(writer, "MOLECULE_API_DECORATOR mol_errno MolReader_{name}_read_value(const mol_seg_t *input, {typ} *output) {{")?;
            writeln!(writer, "    if (!output) return MOL_ERR_DATA;\n    return mol_integer_read_limbs(input, {count}, {little}, output->limbs);\n}}")?;
            writeln!(writer, "MOLECULE_API_DECORATOR mol_errno MolBuilder_{name}_write_value({typ} value, mol_seg_t *output) {{\n    return mol_integer_write_limbs(value.limbs, {count}, {little}, output);\n}}")?;
            continue;
        }
        writeln!(writer, "MOLECULE_API_DECORATOR mol_errno MolReader_{name}_read_value(const mol_seg_t *input, {typ} *output) {{\n    if (!output) return MOL_ERR_DATA;\n    uint64_t value;\n    mol_errno error = mol_integer_read_u64(input, {width}, {little}, &value);\n    if (error != MOL_OK) return error;")?;
        if spec.signed {
            // For a negative value, the complement fits in int64_t even at INT64_MIN.
            writeln!(writer, "    *output = value <= INT{bits}_MAX ? ({typ})value : ({typ})(-1 - (int64_t)(UINT{bits}_MAX - value));")?;
        } else {
            writeln!(writer, "    *output = ({typ})value;")?;
        }
        writeln!(writer, "    return MOL_OK;\n}}")?;
        let parameter = if spec.signed { "int64_t" } else { "uint64_t" };
        writeln!(writer, "MOLECULE_API_DECORATOR mol_errno MolBuilder_{name}_write_value({parameter} value, mol_seg_t *output) {{")?;
        let value = if spec.signed && bits < 64 {
            writeln!(
                writer,
                "    if (value < INT{bits}_MIN || value > INT{bits}_MAX) return MOL_ERR_DATA;"
            )?;
            format!("(uint{bits}_t)value")
        } else {
            "(uint64_t)value".into()
        };
        writeln!(
            writer,
            "    return mol_integer_write_u64({value}, {width}, {little}, output);\n}}"
        )?;
    }
    Ok(())
}
