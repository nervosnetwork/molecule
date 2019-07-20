use pest::{error::Error as PestError, Parser as _};

use crate::{ast, utils::PairsUtils as _};

mod inner;
pub(crate) use inner::Rule;

pub(crate) struct Parser;

impl Parser {
    pub(crate) fn parse(input: &str) -> ast::verified::Ast {
        let ast_raw = Self::preprocess(input).unwrap();
        ast::verified::Ast::new(ast_raw)
    }

    fn preprocess(input: &str) -> Result<ast::raw::Ast, PestError<inner::Rule>> {
        let mut eoi = false;
        let mut ast = ast::raw::Ast::default();
        let mut file = inner::Parser::parse(inner::Rule::grammar, input)?;
        let grammar = file
            .next()
            .unwrap_or_else(|| panic!("grammar should only have one pair"));
        if file.peek().is_some() {
            panic!("grammar should only have only one pair");
        }
        for pair in grammar.into_inner() {
            match pair.as_rule() {
                inner::Rule::import_stmt => {
                    let mut pair = pair.into_inner();
                    let node = pair.next_import();
                    pair.next_should_be_none();
                    ast.add_import(node);
                    // TODO
                    panic!("sorry, current version doesn't support `import` statements");
                }
                inner::Rule::option_decl => {
                    let mut pair = pair.into_inner();
                    let node = ast::raw::OptionDecl {
                        name: pair.next_string(),
                        typ: pair.next_string(),
                    };
                    pair.next_should_be_none();
                    ast.add_decl(node);
                }
                inner::Rule::union_decl => {
                    let mut pair = pair.into_inner();
                    let node = ast::raw::UnionDecl {
                        name: pair.next_string(),
                        inner: pair.next_items(),
                    };
                    pair.next_should_be_none();
                    ast.add_decl(node);
                }
                inner::Rule::array_decl => {
                    let mut pair = pair.into_inner();
                    let node = ast::raw::ArrayDecl {
                        name: pair.next_string(),
                        typ: pair.next_string(),
                        length: pair.next_usize(),
                    };
                    pair.next_should_be_none();
                    ast.add_decl(node);
                }
                inner::Rule::struct_decl => {
                    let mut pair = pair.into_inner();
                    let node = ast::raw::StructDecl {
                        name: pair.next_string(),
                        inner: pair.next_fields(),
                    };
                    pair.next_should_be_none();
                    ast.add_decl(node);
                }
                inner::Rule::vector_decl => {
                    let mut pair = pair.into_inner();
                    let node = ast::raw::VectorDecl {
                        name: pair.next_string(),
                        typ: pair.next_string(),
                    };
                    pair.next_should_be_none();
                    ast.add_decl(node);
                }
                inner::Rule::table_decl => {
                    let mut pair = pair.into_inner();
                    let node = ast::raw::TableDecl {
                        name: pair.next_string(),
                        inner: pair.next_fields(),
                    };
                    pair.next_should_be_none();
                    ast.add_decl(node);
                }
                inner::Rule::EOI => {
                    if eoi {
                        panic!("grammar could not have more than one EOI");
                    }
                    eoi = true;
                }
                _ => {
                    unreachable!();
                }
            }
        }
        if !eoi {
            panic!("grammar should have one EOI");
        }
        Ok(ast)
    }
}
