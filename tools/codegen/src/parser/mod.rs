use std::{ffi, fs, io::Read as _, path::Path};

use pest::{error::Error as PestError, Parser as _};
use same_file::is_same_file;

use crate::{ast, utils::PairsUtils as _};

mod inner;
pub(crate) use inner::Rule;

pub(crate) struct Parser;

impl Parser {
    pub(crate) fn parse<P: AsRef<Path>>(path: &P) -> ast::verified::Ast {
        let ast_raw = Self::preprocess(path).unwrap();
        ast::verified::Ast::new(ast_raw)
    }

    fn preprocess<P: AsRef<Path>>(path: &P) -> Result<ast::raw::Ast, PestError<inner::Rule>> {
        let mut ast = ast::raw::Ast::default();
        let mut imported_depth = 0;
        ast.namespace = path
            .as_ref()
            .file_stem()
            .and_then(ffi::OsStr::to_str)
            .unwrap()
            .to_owned();

        Self::preprocess_single(&mut ast, path, imported_depth)?;

        let mut path_bufs = Vec::new();

        let mut imports = Vec::new();

        while !ast.imports.is_empty() {
            imported_depth += 1;
            while !ast.imports.is_empty() {
                let stmt = ast.imports.remove(0);
                let mut path_buf = stmt.imported_base.clone();
                path_buf.pop();
                for _ in 0..stmt.depth {
                    path_buf.push("..");
                }
                for p in &stmt.path[..] {
                    path_buf.push(p);
                }
                path_buf.push(&stmt.name);
                path_buf.set_extension("mol");
                let path_new = path_buf.as_path();
                if is_same_file(path, &path_new).unwrap() {
                    panic!("found cyclic dependencie");
                }

                if path_bufs
                    .iter()
                    .any(|ref path_old| is_same_file(&path_old, &path_new).unwrap())
                {
                    continue;
                } else {
                    imports.push(stmt);
                    Self::preprocess_single(&mut ast, &path_new, imported_depth)?;
                    path_bufs.push(path_buf);
                }
            }
        }

        ast.imports = imports;

        Ok(ast)
    }

    fn preprocess_single<P: AsRef<Path>>(
        ast: &mut ast::raw::Ast,
        path: &P,
        imported_depth: usize,
    ) -> Result<(), PestError<inner::Rule>> {
        let buffer = {
            let mut buffer = String::new();
            let mut file_in = fs::OpenOptions::new().read(true).open(&path).unwrap();
            file_in.read_to_string(&mut buffer).unwrap();
            buffer
        };
        let mut file_content = inner::Parser::parse(inner::Rule::grammar, &buffer)?;
        let grammar = file_content
            .next()
            .unwrap_or_else(|| panic!("grammar should only have one pair"));
        if file_content.peek().is_some() {
            panic!("grammar should only have only one pair");
        }
        let mut eoi = false;
        for pair in grammar.into_inner() {
            if eoi {
                panic!("grammar should have only one EOI");
            }
            match pair.as_rule() {
                inner::Rule::import_stmt => {
                    let mut pair = pair.into_inner();
                    let node = pair.next_import(path, imported_depth);
                    pair.next_should_be_none();
                    ast.add_import(node);
                }
                inner::Rule::option_decl => {
                    let mut pair = pair.into_inner();
                    let node = ast::raw::OptionDecl {
                        name: pair.next_string(),
                        typ: pair.next_string(),
                        imported_depth,
                    };
                    pair.next_should_be_none();
                    ast.add_decl(node);
                }
                inner::Rule::union_decl => {
                    let mut pair = pair.into_inner();
                    let node = ast::raw::UnionDecl {
                        name: pair.next_string(),
                        inner: pair.next_items(),
                        imported_depth,
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
                        imported_depth,
                    };
                    pair.next_should_be_none();
                    ast.add_decl(node);
                }
                inner::Rule::struct_decl => {
                    let mut pair = pair.into_inner();
                    let node = ast::raw::StructDecl {
                        name: pair.next_string(),
                        inner: pair.next_fields(),
                        imported_depth,
                    };
                    pair.next_should_be_none();
                    ast.add_decl(node);
                }
                inner::Rule::vector_decl => {
                    let mut pair = pair.into_inner();
                    let node = ast::raw::VectorDecl {
                        name: pair.next_string(),
                        typ: pair.next_string(),
                        imported_depth,
                    };
                    pair.next_should_be_none();
                    ast.add_decl(node);
                }
                inner::Rule::table_decl => {
                    let mut pair = pair.into_inner();
                    let node = ast::raw::TableDecl {
                        name: pair.next_string(),
                        inner: pair.next_fields(),
                        imported_depth,
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
            panic!("grammar should have only one EOI");
        }
        Ok(())
    }
}
