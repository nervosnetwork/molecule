use proc_macro2 as m4;
use quote::quote;

use super::utilities::usize_lit;
use crate::ast::verified as ast;

pub(super) trait DefConstants {
    fn def_constants(&self) -> m4::TokenStream;
}

impl DefConstants for ast::Option_ {
    fn def_constants(&self) -> m4::TokenStream {
        quote!()
    }
}

impl DefConstants for ast::Union {
    fn def_constants(&self) -> m4::TokenStream {
        let item_count = usize_lit(self.inner.len());
        quote!(
            pub const ITEM_COUNT: usize = #item_count;
        )
    }
}

impl DefConstants for ast::Array {
    fn def_constants(&self) -> m4::TokenStream {
        let total_size = usize_lit(self.total_size());
        let item_size = usize_lit(self.item_size);
        let item_count = usize_lit(self.item_count);
        quote!(
            pub const TOTAL_SIZE: usize = #total_size;
            pub const ITEM_SIZE: usize = #item_size;
            pub const ITEM_COUNT: usize = #item_count;
        )
    }
}

impl DefConstants for ast::Struct {
    fn def_constants(&self) -> m4::TokenStream {
        let total_size = usize_lit(self.total_size());
        let field_size = self.field_size.iter().map(|x| usize_lit(*x));
        let field_count = usize_lit(self.inner.len());
        quote!(
            pub const TOTAL_SIZE: usize = #total_size;
            pub const FIELD_SIZE: [usize; #field_count]= [ #( #field_size, )* ];
            pub const FIELD_COUNT: usize = #field_count;
        )
    }
}

impl DefConstants for ast::FixVec {
    fn def_constants(&self) -> m4::TokenStream {
        let item_size = usize_lit(self.item_size);
        quote!(
            pub const ITEM_SIZE: usize = #item_size;
        )
    }
}

impl DefConstants for ast::DynVec {
    fn def_constants(&self) -> m4::TokenStream {
        quote!()
    }
}

impl DefConstants for ast::Table {
    fn def_constants(&self) -> m4::TokenStream {
        let field_count = usize_lit(self.inner.len());
        quote!(
            pub const FIELD_COUNT: usize = #field_count;
        )
    }
}
