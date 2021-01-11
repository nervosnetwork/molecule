#![recursion_limit = "256"]

pub mod ast;
pub(crate) mod compiler;
pub(crate) mod generator;
pub(crate) mod parser;
pub(crate) mod utils;

#[cfg(feature = "compiler-plugin")]
mod ir;

pub use compiler::Compiler;
pub use generator::Language;
pub use parser::Parser;

#[cfg(feature = "compiler-plugin")]
pub use ir::Format as IntermediateFormat;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const C_API_VERSION_MIN: &str = "0.7.0";
