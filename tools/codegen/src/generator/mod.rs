use std::{io, path::Path};

use crate::{Ast, Parser};

mod languages;

pub use languages::Language;

#[derive(Debug)]
pub(crate) struct Generator {
    ast: Ast,
}

impl Generator {
    pub(crate) fn new<P: AsRef<Path>>(path: &P) -> Self {
        let ast = Parser::parse(path);
        Self { ast }
    }

    pub(crate) fn generate<W: io::Write>(&self, lang: Language, writer: &mut W) -> io::Result<()> {
        lang.generate(writer, &self.ast)
    }
}
