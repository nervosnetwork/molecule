use std::io;

use quote::quote;

use crate::{ast, VERSION};

pub(crate) mod utilities;

mod builder;
mod entity;
mod reader;

/// Constants for `{ Entity, Reader }`
mod display;

/// Constants for `{ Entity, Reader, Builder }`
mod constants;

/// Constants for `{ Entity, Reader }`
mod properties;

/// Constants for `{ Entity, Reader }`
mod getters;

/// Iterator for `{ Union } x { Entity, Reader }`
mod iterator;

/// Enumerator for `{ Vector } x { Entity, Reader }`
mod enumerator;

mod import;
use import::GenImport as _;

mod generator;
use generator::Generator as _;

pub(crate) struct Generator;

impl super::LanguageGenerator for Generator {
    fn generate<W: io::Write>(writer: &mut W, ast: &ast::Ast) -> io::Result<()> {
        writeln!(writer, "// Generated by Molecule {}", VERSION)?;
        writeln!(writer)?;
        let code = quote!(
            use molecule::prelude::*;
        );
        write!(writer, "{}", code)?;
        let imports = ast.imports();
        if !imports.is_empty() {
            writeln!(writer)?;
            for import in imports {
                let code = import.import_crate();
                write!(writer, "{}", code)?;
            }
        }
        writeln!(writer)?;
        for decl in ast.major_decls() {
            match decl.as_ref() {
                ast::TopDecl::Option_(ref i) => i.generate(writer)?,
                ast::TopDecl::Union(ref i) => i.generate(writer)?,
                ast::TopDecl::Array(ref i) => i.generate(writer)?,
                ast::TopDecl::Struct(ref i) => i.generate(writer)?,
                ast::TopDecl::FixVec(ref i) => i.generate(writer)?,
                ast::TopDecl::DynVec(ref i) => i.generate(writer)?,
                ast::TopDecl::Table(ref i) => i.generate(writer)?,
                ast::TopDecl::Primitive(_) => unreachable!(),
            };
        }
        Ok(())
    }
}
