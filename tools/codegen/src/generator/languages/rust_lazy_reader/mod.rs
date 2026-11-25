use crate::ast;
use crate::ast::HasName;
use crate::generator::ident_name;
use quote::quote;
use std::io;

use super::rust;

mod generator;

pub(crate) struct Generator;

impl super::LanguageGenerator for Generator {
    fn generate<W: io::Write>(output: &mut W, ast: &ast::Ast) -> io::Result<()> {
        rust::utilities::check_names(ast, true)?;
        generator::check_legacy_integer_types(ast)?;
        let imported = generator::ImportedHelpers::collect(ast);
        writeln!(output, "extern crate alloc;")?;
        writeln!(output, "{}", rust::imports(ast, true)?)?;

        for decl in ast.major_decls() {
            match decl.as_ref() {
                ast::TopDecl::Option_(ref i) => i.gen_rust(output, ast, &imported)?,
                ast::TopDecl::Union(ref i) => i.gen_rust(output, ast, &imported)?,
                ast::TopDecl::Array(ref i) => i.gen_rust(output, ast, &imported)?,
                ast::TopDecl::Struct(ref i) => i.gen_rust(output, ast, &imported)?,
                ast::TopDecl::FixVec(ref i) => i.gen_rust(output, ast, &imported)?,
                ast::TopDecl::DynVec(ref i) => i.gen_rust(output, ast, &imported)?,
                ast::TopDecl::Table(ref i) => i.gen_rust(output, ast, &imported)?,
                ast::TopDecl::Primitive(_) => unreachable!(),
            };
        }

        imported.emit(output, ast)?;

        if !ast.integers().is_empty() {
            writeln!(output, "{}", super::integers::rust(ast, true)?)?;
        }

        Ok(())
    }
}

trait LazyReaderGenerator: HasName {
    fn gen_rust<W: io::Write>(
        &self,
        output: &mut W,
        _ast: &ast::Ast,
        _imported: &generator::ImportedHelpers<'_>,
    ) -> io::Result<()> {
        let name = ident_name(self.name(), "");
        let q = quote! {
            pub struct #name { pub cursor : molecule::lazy_reader::Cursor }

            impl ::core::convert::From<molecule::lazy_reader::Cursor> for #name {
                fn from(cursor: molecule::lazy_reader::Cursor) -> Self {
                    Self { cursor }
                }
            }
        };
        writeln!(output, "{}", q)?;
        Ok(())
    }
}
