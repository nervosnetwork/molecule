use proc_macro2 as m4;
use quote::quote;

use super::utilities::ident_new;
use crate::ast::verified as ast;

pub(super) trait GenImport {
    fn import_crate(&self) -> m4::TokenStream;
}

impl GenImport for ast::ImportStmt {
    fn import_crate(&self) -> m4::TokenStream {
        let mut stmt = quote!(use super::);
        for _ in 0..self.depth {
            stmt = quote!(#stmt super::);
        }
        for part in &self.path[..] {
            let part = ident_new(&part);
            stmt = quote!(#stmt #part::);
        }
        let name = ident_new(&self.name);
        quote!(#stmt #name::*;)
    }
}
