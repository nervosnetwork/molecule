use std::{convert::TryFrom, io};

use crate::ast::verified as ast;

mod c;
mod go;
mod rust;

#[derive(Debug, Clone, Copy)]
pub enum Language {
    C,
    Rust,
    Go,
}

pub(super) trait LanguageGenerator {
    fn generate<W: io::Write>(writer: &mut W, ast: &ast::Ast) -> io::Result<()>;
}

impl TryFrom<&str> for Language {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "c" => Ok(Language::C),
            "rust" => Ok(Language::Rust),
            "go" => Ok(Language::Go),
            lang => Err(format!("unsupport language: [{}]", lang)),
        }
    }
}

impl Language {
    pub(crate) fn extension(&self) -> &str {
        match *self {
            Language::C => "h",
            Language::Rust => "rs",
            Language::Go => "go",
        }
    }

    pub(crate) fn generate<W: io::Write>(self, writer: &mut W, ast: &ast::Ast) -> io::Result<()> {
        match self {
            Language::C => c::Generator::generate(writer, ast),
            Language::Rust => rust::Generator::generate(writer, ast),
            Language::Go => go::Generator::generate(writer, ast),
        }
    }
}
