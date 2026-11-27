use std::io;

use crate::ast;

#[cfg(feature = "compiler-plugin")]
use crate::ir::{self, ToIntermediate as _};

mod languages;

pub(crate) use languages::rust::utilities::{field_name, ident_name};
pub use languages::Language;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Target {
    Language(Language),
    #[cfg(feature = "compiler-plugin")]
    Intermediate(ir::Format),
    #[cfg(feature = "compiler-plugin")]
    WireIntermediate(ir::Format),
}

#[derive(Debug)]
pub(crate) struct Generator {
    ast: ast::Ast,
}

impl Target {
    pub(crate) fn extension(self) -> &'static str {
        match self {
            Self::Language(lang) => lang.extension(),
            #[cfg(feature = "compiler-plugin")]
            Self::Intermediate(format) => format.extension(),
            #[cfg(feature = "compiler-plugin")]
            Self::WireIntermediate(format) => format.extension(),
        }
    }
}

impl Generator {
    pub(crate) fn new(ast: ast::Ast) -> Self {
        Self { ast }
    }

    pub(crate) fn generate<W: io::Write>(&self, target: Target, writer: &mut W) -> io::Result<()> {
        match target {
            Target::Language(lang) => lang.generate(writer, &self.ast),
            #[cfg(feature = "compiler-plugin")]
            Target::Intermediate(format) => {
                let bytes = format
                    .serialize_ast(&self.ast)
                    .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
                writer.write_all(&bytes)
            }
            #[cfg(feature = "compiler-plugin")]
            Target::WireIntermediate(format) => format.generate(writer, &self.ast.to_ir()),
        }
    }
}
