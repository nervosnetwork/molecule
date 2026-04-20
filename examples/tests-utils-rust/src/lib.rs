extern crate proc_macro;

use std::{
    collections::HashMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
    rc::Rc,
};

use quote::quote;

use tests_loader::GenRustTest as _;

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
        let resolve_path = |filepath: &str| {
            let path = Path::new(filepath);
            if path.is_absolute() {
                return path.to_path_buf();
            }
            std::env::var_os("CARGO_MANIFEST_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("."))
                .join(path)
        };
        let ast = {
            use codegen::ast::HasName;
            let filepath = resolve_path(&input.schema);
            let ast = codegen::Parser::parse(&filepath);
            ast.decls()
                .iter()
                .map(|decl| (decl.name().to_owned(), Rc::clone(decl)))
                .collect::<HashMap<_, _>>()
        };
        let test_data = {
            let filepath = resolve_path(&input.tests);
            let mut file = fs::File::open(&filepath).unwrap_or_else(|err| {
                panic!("failed to open tests from {}: {}", filepath.display(), err)
            });
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap_or_else(|err| {
                panic!("failed to load tests from {}: {}", filepath.display(), err)
            });
            contents
        };
        let all: tests_loader::TestSet = serde_yaml::from_str(&test_data)
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
