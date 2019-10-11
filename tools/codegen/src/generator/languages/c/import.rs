use std::io;

use crate::ast::verified::{self as ast};

pub(super) trait GenImport {
    fn gen_import<W: io::Write>(&self, writer: &mut W) -> io::Result<()>;
}

impl GenImport for ast::ImportStmt {
    fn gen_import<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        write!(writer, "#include \"")?;
        for _ in 0..self.depth {
            write!(writer, "../")?;
        }
        for p in &self.path[..] {
            write!(writer, "{}/", p)?;
        }
        writeln!(writer, "{}.h\"", self.name)
    }
}
