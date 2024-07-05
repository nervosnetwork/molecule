use crate::ast;
use crate::ast::HasName;
use crate::generator::ident_name;
use quote::quote;
use std::io;

mod generator;

pub(crate) struct Generator;

impl super::LanguageGenerator for Generator {
    fn generate<W: io::Write>(output: &mut W, ast: &ast::Ast) -> io::Result<()> {
        writeln!(
            output,
            r#"
        extern crate alloc;
        use molecule::lazy_reader::{{Cursor, Error, NUMBER_SIZE}};
        use core::convert::TryInto;"#
        )?;

        for import in ast.imports() {
            writeln!(output, "use super::{}::*; ", &import.name())?;
        }

        for decl in ast.major_decls() {
            match decl.as_ref() {
                ast::TopDecl::Option_(ref i) => i.gen_rust(output)?,
                ast::TopDecl::Union(ref i) => i.gen_rust(output)?,
                ast::TopDecl::Array(ref i) => i.gen_rust(output)?,
                ast::TopDecl::Struct(ref i) => i.gen_rust(output)?,
                ast::TopDecl::FixVec(ref i) => i.gen_rust(output)?,
                ast::TopDecl::DynVec(ref i) => i.gen_rust(output)?,
                ast::TopDecl::Table(ref i) => i.gen_rust(output)?,
                ast::TopDecl::Primitive(_) => unreachable!(),
            };
        }

        Ok(())
    }
}

pub trait LazyReaderGenerator: HasName {
    fn gen_rust<W: io::Write>(&self, output: &mut W) -> io::Result<()> {
        let name = ident_name(self.name(), "");
        let q = quote! {
            pub struct #name { pub cursor : Cursor }

            impl From<Cursor> for #name {
                fn from(cursor: Cursor) -> Self {
                    Self { cursor }
                }
            }
        };
        writeln!(output, "{}", q)?;
        Ok(())
    }
}
