use std::path::Path;

use crate::{ast::raw as ast, parser::ParseError};

pub(crate) trait PairsUtils {
    fn next_string(&mut self) -> String;
    fn next_usize(&mut self) -> Result<usize, ParseError>;
    fn next_item(&mut self) -> ast::ItemDecl;
    #[allow(dead_code)]
    fn next_items(&mut self) -> Vec<ast::ItemDecl>;
    fn next_custom_union_items(&mut self) -> Result<Vec<ast::CustomUnionItemDecl>, ParseError>;
    fn next_fields(&mut self) -> Vec<ast::FieldDecl>;
    fn next_import<P: AsRef<Path>>(
        &mut self,
        imported_base: &P,
        imported_depth: usize,
    ) -> ast::ImportStmt;
    fn next_should_be_none(self);
}
