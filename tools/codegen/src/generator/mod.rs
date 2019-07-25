use std::io;

use crate::{Ast, Parser};

mod lang_c;
mod lang_rust;

#[derive(Debug, Clone, Copy)]
pub enum Language {
    C,
    Rust,
}

#[derive(Debug)]
pub(crate) struct Generator {
    ast: Ast,
}

impl Language {
    pub(crate) fn extension(&self) -> &str {
        match *self {
            Language::C => "h",
            Language::Rust => "rs",
        }
    }
}

impl Generator {
    pub(crate) fn new(input: &str) -> Self {
        let ast = Parser::parse(input);
        Self { ast }
    }
    pub(crate) fn generate<W>(&self, lang: Language, writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        match lang {
            Language::C => self.generate_c(writer),
            Language::Rust => self.generate_rust(writer),
        }
    }
}
