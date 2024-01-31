use crate::ast;
use crate::ast::HasName;
use proc_macro2::Ident;
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

fn ident_new(name: &str) -> Ident {
    Ident::new(name, proc_macro2::Span::call_site())
}

fn ident_new_camel(name: &str) -> Ident {
    let mut camel_case = String::new();
    let mut capitalize_next = true;

    for c in name.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            camel_case.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            camel_case.push(c);
        }
    }

    ident_new(&camel_case)
}

pub trait LazyReaderGenerator: HasName {
    fn gen_rust<W: io::Write>(&self, output: &mut W) -> io::Result<()> {
        let name = ident_new(self.name());
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
