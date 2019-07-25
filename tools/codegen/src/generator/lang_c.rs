use std::io;

use super::Generator;

impl Generator {
    pub(crate) fn generate_c<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        writeln!(writer)
    }
}
