use std::path::Path;

use pest::error::Error as PestError;

use crate::{ast::raw as ast, parser};

pub(crate) trait PairsUtils {
    fn next_string(&mut self) -> String;
    fn next_usize(&mut self) -> usize;
    fn next_item(&mut self) -> ast::ItemDecl;
    fn next_items(&mut self) -> Vec<ast::ItemDecl>;
    fn next_fields(&mut self) -> Vec<ast::FieldDecl>;
    fn next_import<P: AsRef<Path>>(
        &mut self,
        imported_base: &P,
        imported_depth: usize,
    ) -> ast::ImportStmt;
    fn next_should_be_none(self);
}

pub(crate) trait ParserUtils {
    fn preprocess<P: AsRef<Path>>(path: &P) -> Result<ast::Ast, PestError<parser::Rule>>;
}
