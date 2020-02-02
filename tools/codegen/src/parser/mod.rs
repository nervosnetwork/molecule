use std::path::Path;

use crate::{ast, utils::ParserUtils as _};

mod inner;
pub(crate) use inner::{Parser as InnerParser, Rule};

pub struct Parser;

impl Parser {
    pub fn parse<P: AsRef<Path>>(path: &P) -> ast::Ast {
        let ast_raw = Self::preprocess(path).unwrap();
        ast::Ast::complete(ast_raw)
    }
}
