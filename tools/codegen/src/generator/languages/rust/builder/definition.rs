use proc_macro2 as m4;
use quote::quote;

use super::super::utilities::{builder_name, entity_ref, entity_union_name, field_name, usize_lit};
use crate::ast;

pub(in super::super) trait DefBuilder {
    fn def_builder(&self) -> m4::TokenStream;
}

impl DefBuilder for ast::Option_ {
    fn def_builder(&self) -> m4::TokenStream {
        let builder = builder_name(self.name());
        let inner = entity_ref(self.item().typ());
        quote!(
            #[derive(Clone, Debug, Default)]
            pub struct #builder (pub(crate) ::core::option::Option<#inner>);
        )
    }
}

impl DefBuilder for ast::Union {
    fn def_builder(&self) -> m4::TokenStream {
        let builder = builder_name(self.name());
        let entity_union = entity_union_name(self.name());
        quote!(
            #[derive(Clone, Debug, Default)]
            pub struct #builder (pub(crate) #entity_union);
        )
    }
}

impl DefBuilder for ast::Array {
    fn def_builder(&self) -> m4::TokenStream {
        let builder = builder_name(self.name());
        let inner = entity_ref(self.item().typ());
        let item_count = usize_lit(self.item_count());
        quote!(
            #[derive(Clone)]
            pub struct #builder (pub(crate) [#inner; #item_count]);

            impl ::core::fmt::Debug for #builder {
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    write!(f, "{}({:?})", Self::NAME, &self.0[..])
                }
            }

            impl ::core::default::Default for #builder {
                fn default() -> Self {
                    Self(::core::array::from_fn(|_| ::core::default::Default::default()))
                }
            }
        )
    }
}

impl DefBuilder for ast::Struct {
    fn def_builder(&self) -> m4::TokenStream {
        def_builder_for_struct_or_table(self.name(), self.fields())
    }
}

impl DefBuilder for ast::FixVec {
    fn def_builder(&self) -> m4::TokenStream {
        def_builder_for_vector(self.name(), self.item().typ())
    }
}

impl DefBuilder for ast::DynVec {
    fn def_builder(&self) -> m4::TokenStream {
        def_builder_for_vector(self.name(), self.item().typ())
    }
}

impl DefBuilder for ast::Table {
    fn def_builder(&self) -> m4::TokenStream {
        def_builder_for_struct_or_table(self.name(), self.fields())
    }
}

fn def_builder_for_struct_or_table(self_name: &str, inner: &[ast::FieldDecl]) -> m4::TokenStream {
    let builder = builder_name(self_name);
    let fields = inner.iter().map(|f| {
        let field_name = field_name(f.name());
        let field_type = entity_ref(f.typ());
        quote!(#field_name: #field_type,)
    });
    quote!(
        #[derive(Clone, Debug, Default)]
        pub struct #builder { #( pub(crate) #fields )* }
    )
}

fn def_builder_for_vector(self_name: &str, item: &ast::TopDecl) -> m4::TokenStream {
    let builder = builder_name(self_name);
    let inner = entity_ref(item);
    quote!(
        #[derive(Clone, Debug, Default)]
        pub struct #builder (pub(crate) molecule::prelude::Vec<#inner>);
    )
}
