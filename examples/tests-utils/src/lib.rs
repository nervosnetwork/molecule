extern crate proc_macro;

pub(crate) mod bytes;
pub(crate) mod generator;
pub(crate) mod types;

use std::{collections::HashMap, fs, io::Read};

use quote::quote;

use crate::generator::GenTest;

struct InputFiles {
    schema: String,
    tests: String,
}

impl syn::parse::Parse for InputFiles {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content =
            <syn::punctuated::Punctuated<syn::LitStr, syn::Token![,]>>::parse_terminated(input)?;
        if content.len() == 2 {
            Ok(InputFiles {
                schema: content.first().unwrap().value(),
                tests: content.last().unwrap().value(),
            })
        } else {
            Err(syn::Error::new(
                input.span(),
                "expected two files: the schema file and the test vectors",
            ))
        }
    }
}

#[proc_macro]
pub fn load_tests(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as InputFiles);
    let expanded = {
        let ast = {
            use codegen::ast::HasName;
            let filepath = &input.schema;
            let ast = codegen::Parser::parse(filepath);
            ast.decls()
                .into_iter()
                .map(|decl| (decl.name().to_owned(), decl))
                .collect::<HashMap<_, _>>()
        };
        let test_data = {
            let filepath = &input.tests;
            let mut file = fs::File::open(filepath)
                .unwrap_or_else(|err| panic!("failed to open tests from {}: {}", filepath, err));
            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .unwrap_or_else(|err| panic!("failed to load tests from {}: {}", filepath, err));
            contents
        };
        let all: types::All = serde_yaml::from_str(&test_data)
            .unwrap_or_else(|err| panic!("failed to parse tests: {}", err));
        let test = all
            .iter()
            .fold(Vec::with_capacity(all.len()), |mut tests, any| {
                tests.append(&mut any.gen_test(&ast, tests.len()));
                tests
            });
        quote!( #( #test )*)
    };
    expanded.into()
}
