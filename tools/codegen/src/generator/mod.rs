use std::io;

use crate::ast;

#[cfg(feature = "compiler-plugin")]
use crate::ir::{self, ToIntermediate as _};

mod languages;

pub use languages::Language;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Target {
    Language(Language),
    #[cfg(feature = "compiler-plugin")]
    Intermediate(ir::Format),
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
            Target::Intermediate(format) => format.generate(writer, &self.ast.to_ir()),
        }
    }
}
