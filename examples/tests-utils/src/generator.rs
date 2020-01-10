use std::{collections::HashMap, rc::Rc};

use proc_macro2 as m4;
use quote::quote;

use codegen::ast::{self, HasName};

use crate::types;

type Ast = HashMap<String, Rc<ast::TopDecl>>;

mod utils {
    use proc_macro2 as m4;

    pub(crate) fn ident_new(ident: &str) -> m4::Ident {
        m4::Ident::new(ident, m4::Span::call_site())
    }
    pub(crate) fn entity_name(ident: &str) -> m4::Ident {
        if ident == "byte" {
            m4::Ident::new("Byte", m4::Span::call_site())
        } else {
            m4::Ident::new(ident, m4::Span::call_site())
        }
    }
}

pub(crate) trait GenTest {
    fn gen_test(&self, ast: &Ast, id: usize) -> Vec<m4::TokenStream>;
    fn assert_stmt(&self) -> m4::TokenStream {
        quote!(
            assert_eq!(
                result.as_slice(),
                expected.as_slice(),
                "\nexpect:\n  struct: {}\n  data: {:#x};\nactual:\n  struct: {}\n  data: {:#x}\n",
                result,
                result,
                expected,
                expected,
            );
        )
    }
}

impl GenTest for types::Any {
    fn gen_test(&self, ast: &Ast, id: usize) -> Vec<m4::TokenStream> {
        match self {
            Self::Union(inner) => inner.gen_test(ast, id),
            Self::Option_(inner) => inner.gen_test(ast, id),
            Self::Array(inner) => inner.gen_test(ast, id),
            Self::StructOrTable(inner) => inner.gen_test(ast, id),
            Self::Vector(inner) => inner.gen_test(ast, id),
        }
    }
}

impl GenTest for types::Option_ {
    fn gen_test(&self, ast: &Ast, id: usize) -> Vec<m4::TokenStream> {
        let test_name = utils::ident_new(&format!("test_{}", id));
        let name = utils::entity_name(self.name());
        let set_stmt = if let Some(item) = self.item() {
            let decl = ast.get(self.name()).unwrap();
            let item_type = if let ast::TopDecl::Option_(ref decl) = decl.as_ref() {
                utils::entity_name(decl.item().typ().name())
            } else {
                panic!("Error: type for {} is incorrect", self.name());
            };
            let item_data = item.ts();
            quote!( .set(Some(#item_type::from_slice(#item_data).unwrap())) )
        } else {
            quote!()
        };
        let expected = self.expected().ts();
        let assert_stmt = self.assert_stmt();
        let test = quote!(
            #[test]
            fn #test_name() {
                let result = #name::new_builder()
                    #set_stmt
                    .build();
                let expected = #name::from_slice(#expected).unwrap();
                #assert_stmt
            }
        );
        vec![test]
    }
}

impl GenTest for types::Union {
    fn gen_test(&self, _: &Ast, id: usize) -> Vec<m4::TokenStream> {
        let test_name = utils::ident_new(&format!("test_{}", id));
        let name = utils::entity_name(self.name());
        let set_stmt = if let Some(item) = self.item() {
            let item_type = utils::entity_name(item.typ());
            let item_data = item.data().ts();
            quote!( .set(#item_type::from_slice(#item_data).unwrap()) )
        } else {
            quote!()
        };
        let expected = self.expected().ts();
        let assert_stmt = self.assert_stmt();
        let test = quote!(
            #[test]
            fn #test_name() {
                let result = #name::new_builder()
                    #set_stmt
                    .build();
                let expected = #name::from_slice(#expected).unwrap();
                #assert_stmt
            }
        );
        vec![test]
    }
}

impl GenTest for types::Array {
    fn gen_test(&self, ast: &Ast, id: usize) -> Vec<m4::TokenStream> {
        let test_name = utils::ident_new(&format!("test_{}", id));
        let name = utils::entity_name(self.name());
        let decl = ast.get(self.name()).unwrap();
        let item_type = if let ast::TopDecl::Array(ref decl) = decl.as_ref() {
            vec![utils::entity_name(decl.item().typ().name()); self.data().len()]
        } else {
            panic!("Error: type for {} is incorrect", self.name());
        };
        let (item_func, item_data) = self.data().iter().fold(
            (
                Vec::with_capacity(self.data().len()),
                Vec::with_capacity(self.data().len()),
            ),
            |(mut item_func, mut item_data), (index, data)| {
                item_func.push(utils::ident_new(&format!("nth{}", index)));
                item_data.push(data.ts());
                (item_func, item_data)
            },
        );
        let expected = self.expected().ts();
        let assert_stmt = self.assert_stmt();
        let test = quote!(
            #[test]
            fn #test_name() {
                let result = #name::new_builder()
                    #( .#item_func(#item_type::from_slice(#item_data).unwrap()) )*
                    .build();
                let expected = #name::from_slice(#expected).unwrap();
                #assert_stmt
            }
        );
        vec![test]
    }
}

impl GenTest for types::StructOrTable {
    fn gen_test(&self, ast: &Ast, id: usize) -> Vec<m4::TokenStream> {
        let test_name = utils::ident_new(&format!("test_{}", id));
        let name = utils::entity_name(self.name());
        let decl = ast.get(self.name()).unwrap();
        let field_type_dict = if let ast::TopDecl::Struct(ref decl) = decl.as_ref() {
            decl.fields()
                .iter()
                .map(|field| (field.name(), field.typ().name()))
                .collect::<HashMap<_, _>>()
        } else if let ast::TopDecl::Table(ref decl) = decl.as_ref() {
            decl.fields()
                .iter()
                .map(|field| (field.name(), field.typ().name()))
                .collect::<HashMap<_, _>>()
        } else {
            panic!("Error: type for {} is incorrect", self.name());
        };
        let (field_func, field_type, field_data) = self.data().iter().fold(
            (
                Vec::with_capacity(self.data().len()),
                Vec::with_capacity(self.data().len()),
                Vec::with_capacity(self.data().len()),
            ),
            |(mut field_func, mut field_type, mut field_data), (name, field)| {
                field_func.push(utils::ident_new(name));
                let field_type_name = field_type_dict.get(&name.as_str()).unwrap();
                field_type.push(utils::entity_name(field_type_name));
                field_data.push(field.ts());
                (field_func, field_type, field_data)
            },
        );
        let expected = self.expected().ts();
        let assert_stmt = self.assert_stmt();
        let test = quote!(
            #[test]
            fn #test_name() {
                let result = #name::new_builder()
                    #( .#field_func(#field_type::from_slice(#field_data).unwrap()) )*
                    .build();
                let expected = #name::from_slice(#expected).unwrap();
                #assert_stmt
            }
        );
        vec![test]
    }
}

impl GenTest for types::Vector {
    fn gen_test(&self, ast: &Ast, id: usize) -> Vec<m4::TokenStream> {
        let test_name = utils::ident_new(&format!("test_{}", id));
        let name = utils::entity_name(self.name());
        let decl = ast.get(self.name()).unwrap();
        let item_type = if let ast::TopDecl::FixVec(ref decl) = decl.as_ref() {
            vec![utils::entity_name(decl.item().typ().name()); self.data().len()]
        } else if let ast::TopDecl::DynVec(ref decl) = decl.as_ref() {
            vec![utils::entity_name(decl.item().typ().name()); self.data().len()]
        } else {
            panic!("Error: type for {} is incorrect", self.name());
        };
        let item_data = self.data().iter().map(|item| item.ts());
        let expected = self.expected().ts();
        let assert_stmt = self.assert_stmt();
        let test = quote!(
            #[test]
            fn #test_name() {
                let result = #name::new_builder()
                    #( .push(#item_type::from_slice(#item_data).unwrap()) )*
                    .build();
                let expected = #name::from_slice(#expected).unwrap();
                #assert_stmt
            }
        );
        vec![test]
    }
}
