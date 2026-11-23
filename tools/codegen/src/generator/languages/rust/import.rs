use proc_macro2 as m4;
use quote::quote;

use super::utilities::ident_new;
use crate::ast;

pub(crate) trait GenImport {
    fn import_crate(&self) -> m4::TokenStream;
}

pub(crate) fn module_path_tokens(path: &ast::verified::ModulePath) -> m4::TokenStream {
    let mut stmt = quote!(super::);
    for _ in 0..path.path_supers {
        stmt = quote!(#stmt super::);
    }
    for part in &path.paths {
        let part = ident_new(part);
        stmt = quote!(#stmt #part::);
    }
    let name = ident_new(&path.name);
    quote!(#stmt #name)
}

impl GenImport for ast::ImportStmt {
    fn import_crate(&self) -> m4::TokenStream {
        let path = ast::verified::ModulePath::new(
            self.name().to_owned(),
            self.paths().to_owned(),
            self.path_supers(),
        );
        let module = module_path_tokens(&path);
        quote!(use #module::*;)
    }
}
