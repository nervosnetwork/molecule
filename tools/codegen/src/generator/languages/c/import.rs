use std::io;

use crate::ast;

pub(super) fn write_imports<W: io::Write>(writer: &mut W, ast: &ast::Ast) -> io::Result<()> {
    for import in ast.imports() {
        write!(writer, "#include \"")?;
        for _ in 0..import.path_supers() {
            write!(writer, "../")?;
        }
        for p in import.paths() {
            write!(writer, "{}/", p)?;
        }
        writeln!(writer, "{}.h\"", import.name())?;
    }
    Ok(())
}
