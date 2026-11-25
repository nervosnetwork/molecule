use proc_macro2 as m4;
use quote::quote;

use super::utilities::{emitted_type_names, ident_name, import_alias, module_ident};
use crate::ast;
use crate::ast::{HasName, TopDecl};

pub(crate) fn module_path_tokens(path: &ast::verified::ModulePath) -> m4::TokenStream {
    let mut stmt = quote!(super::);
    for _ in 0..path.path_supers {
        stmt = quote!(#stmt super::);
    }
    for part in &path.paths {
        let part = module_ident(part);
        stmt = quote!(#stmt #part::);
    }
    let name = module_ident(&path.name);
    quote!(#stmt #name)
}

fn legacy_imports(ast: &ast::Ast) -> m4::TokenStream {
    let mut tokens = m4::TokenStream::new();
    for import in ast.imports() {
        let module = module_path_tokens(&import.module_path());
        tokens.extend(quote!(#[allow(unused_imports)] use #module::*;));
    }
    tokens
}

// A detached v1 layout has no owner paths. A glob is sufficient only when
// the declarations in scope agree on what the generated name denotes.
fn ambiguous_v1_names(ast: &ast::Ast, lazy: bool) -> std::collections::HashSet<String> {
    use std::collections::HashSet;
    let mut names = HashSet::new();
    let mut ambiguous = HashSet::new();
    for decl in ast.decls().iter().filter(|decl| decl.imported_depth() <= 1) {
        for symbol in emitted_type_names(decl, lazy) {
            if !names.insert(symbol.clone()) {
                ambiguous.insert(symbol);
            }
        }
    }
    ambiguous
}

pub(crate) fn imports(ast: &ast::Ast, lazy: bool) -> std::io::Result<m4::TokenStream> {
    let mut tokens = m4::TokenStream::new();
    if lazy && ast.has_decl_origins() {
        return Ok(tokens);
    }
    let mut ambiguous = std::collections::HashSet::new();
    if !ast.has_decl_origins() {
        tokens = legacy_imports(ast);
        ambiguous = ambiguous_v1_names(ast, lazy);
    }
    let mut used = std::collections::HashSet::new();
    let mut include = |mut dep: &TopDecl, union_payload: bool| -> std::io::Result<()> {
        if lazy {
            while let TopDecl::Option_(o) = dep {
                dep = o.item().typ();
            }
            if matches!(dep, TopDecl::FixVec(v) if v.item().typ().is_byte())
                || matches!(dep, TopDecl::Array(a) if !union_payload && a.item().typ().is_byte())
            {
                return Ok(());
            }
        }
        if dep.is_byte() || dep.imported_depth() == 0 || !used.insert(dep.name().to_owned()) {
            return Ok(());
        }
        let name = ident_name(dep.name(), "");
        let reader = ident_name(dep.name(), "Reader");
        if (!ast.has_decl_origins() && dep.imported_depth() > 1)
            || ambiguous.contains(&name.to_string())
            || (!lazy && ambiguous.contains(&reader.to_string()))
        {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!(
                "E_RUST_TYPE_COLLISION: v1 IR cannot identify the module for {}; generate from its schema or source-aware IR", dep.name()
            )));
        }
        if !lazy {
            let module = ast
                .decl_origin(dep.name())
                .map(module_path_tokens)
                .unwrap_or_else(|| quote!(self));
            let alias = import_alias(dep.name(), "");
            let reader_alias = import_alias(dep.name(), "Reader");
            tokens.extend(quote!(use #module::{#name as #alias, #reader as #reader_alias};));
        }
        Ok(())
    };
    for decl in ast.major_decls() {
        match decl.as_ref() {
            TopDecl::Option_(o) if !lazy => include(o.item().typ(), false)?,
            TopDecl::Array(a) => include(a.item().typ(), false)?,
            TopDecl::FixVec(v) => include(v.item().typ(), false)?,
            TopDecl::DynVec(v) => include(v.item().typ(), false)?,
            TopDecl::Union(u) => u.items().iter().try_for_each(|i| include(i.typ(), true))?,
            TopDecl::Struct(s) => s
                .fields()
                .iter()
                .try_for_each(|f| include(f.typ(), false))?,
            TopDecl::Table(t) => t
                .fields()
                .iter()
                .try_for_each(|f| include(f.typ(), false))?,
            _ => {}
        }
    }
    Ok(tokens)
}
