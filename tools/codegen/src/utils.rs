use std::str::FromStr;

use pest::iterators::Pairs;

use crate::{ast, parser};

pub(crate) trait PairsUtils {
    fn next_string(&mut self) -> String;
    fn next_usize(&mut self) -> usize;
    fn next_items(&mut self) -> Vec<ast::raw::ItemDecl>;
    fn next_fields(&mut self) -> Vec<ast::raw::FieldDecl>;
    fn next_import(&mut self) -> ast::raw::ImportStmt;
    fn next_should_be_none(self);
}

impl<'i> PairsUtils for Pairs<'i, parser::Rule> {
    fn next_string(&mut self) -> String {
        self.next().unwrap().as_str().to_owned()
    }

    fn next_usize(&mut self) -> usize {
        usize::from_str(self.next().unwrap().as_str()).unwrap()
    }

    fn next_items(&mut self) -> Vec<ast::raw::ItemDecl> {
        let mut ret = Vec::new();
        for item in self {
            if item.as_rule() != parser::Rule::item_decl {
                unreachable!()
            }
            let mut pair = item.into_inner();
            let node = ast::raw::ItemDecl {
                typ: pair.next_string(),
            };
            pair.next_should_be_none();
            ret.push(node);
        }
        ret
    }

    fn next_fields(&mut self) -> Vec<ast::raw::FieldDecl> {
        let mut ret = Vec::new();
        for field in self {
            if field.as_rule() != parser::Rule::field_decl {
                unreachable!()
            }
            let mut pair = field.into_inner();
            let node = ast::raw::FieldDecl {
                name: pair.next_string(),
                typ: pair.next_string(),
            };
            pair.next_should_be_none();
            ret.push(node);
        }
        ret
    }

    fn next_import(&mut self) -> ast::raw::ImportStmt {
        let mut path = Vec::new();
        let mut depth = 0;
        if let Some(inner) = self.next() {
            if inner.as_rule() != parser::Rule::path {
                unreachable!()
            }
            let mut pair = inner.into_inner();
            loop {
                if let Some(inner) = pair.peek() {
                    if inner.as_rule() == parser::Rule::path_super {
                        pair.next();
                        depth += 1;
                        continue;
                    }
                }
                break;
            }
            for inner in pair {
                path.push(inner.as_str().to_owned())
            }
        }
        ast::raw::ImportStmt {
            name: path.pop().unwrap(),
            path,
            depth,
        }
    }

    fn next_should_be_none(mut self) {
        if self.next().is_some() {
            unreachable!()
        }
    }
}
