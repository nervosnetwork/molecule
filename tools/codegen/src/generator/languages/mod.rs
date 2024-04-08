use std::{convert::TryFrom, fmt, io};

use crate::ast;

mod c;
mod rust;
mod rust_lazy_reader;

#[derive(Debug, Clone, Copy)]
pub enum Language {
    C,
    Rust,
    RustLazyReader,
}

pub(super) trait LanguageGenerator {
    fn generate<W: io::Write>(writer: &mut W, ast: &ast::Ast) -> io::Result<()>;
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::C => write!(f, "C"),
            Self::Rust => write!(f, "Rust"),
            Self::RustLazyReader => write!(f, "Rust(Lazy Reader)"),
        }
    }
}

impl TryFrom<&str> for Language {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "c" => Ok(Self::C),
            "rust" => Ok(Self::Rust),
            "rust-lazy-reader" => Ok(Self::RustLazyReader),
            lang => Err(format!("unsupported language: [{}]", lang)),
        }
    }
}

impl Language {
    pub(crate) fn extension(self) -> &'static str {
        match self {
            Self::C => "h",
            Self::Rust => "rs",
            Self::RustLazyReader => "rs",
        }
    }

    pub(crate) fn generate<W: io::Write>(self, writer: &mut W, ast: &ast::Ast) -> io::Result<()> {
        match self {
            Self::C => c::Generator::generate(writer, ast),
            Self::Rust => rust::Generator::generate(writer, ast),
            Self::RustLazyReader => rust_lazy_reader::Generator::generate(writer, ast),
        }
    }
}
