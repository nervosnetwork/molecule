use case::CaseExt;
use proc_macro2 as m4;

use crate::ast::HasName as _;
use std::{collections::HashSet, sync::OnceLock};

static RUST_KEYWORDS: OnceLock<HashSet<&'static str>> = OnceLock::new();

fn rust_keyword() -> &'static HashSet<&'static str> {
    RUST_KEYWORDS.get_or_init(|| {
        IntoIterator::into_iter([
            // strick
            "as",
            "struct",
            "break",
            "const",
            "continue",
            "crate",
            "else",
            "enum",
            "extern",
            "false",
            "fn",
            "for",
            "if",
            "impl",
            "in",
            "let",
            "loop",
            "match",
            "mod",
            "move",
            "mut",
            "pub",
            "ref",
            "return",
            "self",
            "Self",
            "static",
            "super",
            "trait",
            "true",
            "type",
            "unsafe",
            "use",
            "where",
            "while",
            // 2018+
            "async",
            "await",
            "dyn",
            // reserved
            "abstract",
            "become",
            "box",
            "do",
            "final",
            "macro",
            "override",
            "priv",
            "typeof",
            "unsized",
            "virtual",
            "yield",
            // 2018+
            "try",
            // weak
            "macro_rules",
            "union",
            "'static",
            // 2024+
            "gen",
        ])
        .collect()
    })
}

pub(super) fn module_ident(name: &str) -> m4::Ident {
    match name {
        "self" | "Self" | "super" | "crate" | "_" => ident_new(&format!("{name}_")),
        _ if rust_keyword().contains(name) => m4::Ident::new_raw(name, m4::Span::call_site()),
        _ => ident_new(name),
    }
}

fn sanitize_identifier(name: &str) -> String {
    let keywords = rust_keyword();
    let snake_name = name.to_snake();
    if keywords.contains(snake_name.as_str()) {
        snake_name + "_"
    } else {
        snake_name
    }
}

pub(super) fn usize_lit(num: usize) -> m4::Literal {
    m4::Literal::usize_unsuffixed(num)
}

pub(crate) fn ident_new(ident: &str) -> m4::Ident {
    m4::Ident::new(ident, m4::Span::call_site())
}

pub(crate) fn ident_name(name: &str, suffix: &str) -> m4::Ident {
    let span = m4::Span::call_site();
    m4::Ident::new(&format!("{}{}", name, suffix).to_camel(), span)
}

pub(super) fn entity_name(name: &str) -> m4::Ident {
    ident_name(name, "")
}

pub(super) fn reader_name(name: &str) -> m4::Ident {
    ident_name(name, "Reader")
}

// Rust normalizes schema spelling (feeRate and fee_rate both become FeeRate).
// Private import aliases keep the original, case-sensitive declaration identity.
fn type_ref(typ: &crate::ast::TopDecl, suffix: &str) -> m4::TokenStream {
    let name = typ.name();
    let symbol = ident_name(name, suffix);
    if typ.is_byte() {
        quote::quote!(molecule::prelude::#symbol)
    } else if typ.imported_depth() == 0 {
        quote::quote!(self::#symbol)
    } else {
        let alias = import_alias(name, suffix);
        quote::quote!(#alias)
    }
}

pub(super) fn import_alias(name: &str, suffix: &str) -> m4::Ident {
    ident_new(&format!("__Mol{suffix}_{name}"))
}

pub(super) fn entity_ref(typ: &crate::ast::TopDecl) -> m4::TokenStream {
    type_ref(typ, "")
}

pub(super) fn reader_ref(typ: &crate::ast::TopDecl) -> m4::TokenStream {
    type_ref(typ, "Reader")
}

pub(super) fn entity_union_name(name: &str) -> m4::Ident {
    ident_name(name, "Union")
}

pub(super) fn reader_union_name(name: &str) -> m4::Ident {
    ident_name(name, "UnionReader")
}

pub(super) fn union_item_name(name: &str) -> m4::Ident {
    ident_name(name, "")
}

pub(super) fn builder_name(name: &str) -> m4::Ident {
    ident_name(name, "Builder")
}

pub(crate) fn field_name(name: &str) -> m4::Ident {
    let span = m4::Span::call_site();
    m4::Ident::new(&sanitize_identifier(name), span)
}

pub(super) fn emitted_type_names(decl: &crate::ast::TopDecl, lazy: bool) -> Vec<String> {
    use crate::ast::{HasName as _, TopDecl};
    let name = decl.name();
    let mut names = vec![ident_name(name, "").to_string()];
    if lazy {
        if matches!(decl, TopDecl::FixVec(_) | TopDecl::DynVec(_)) {
            names.push(ident_name(name, "Iterator").to_string());
            names.push(ident_name(name, "IteratorRef").to_string());
        }
    } else {
        names.push(ident_name(name, "Reader").to_string());
        names.push(ident_name(name, "Builder").to_string());
        match decl {
            TopDecl::Union(_) => {
                names.push(ident_name(name, "Union").to_string());
                names.push(ident_name(name, "UnionReader").to_string());
            }
            TopDecl::FixVec(v) => {
                names.push(ident_name(name, "Iterator").to_string());
                if !v.item().typ().is_byte() {
                    names.push(ident_name(name, "ReaderIterator").to_string());
                }
            }
            TopDecl::DynVec(_) => {
                names.push(ident_name(name, "Iterator").to_string());
                names.push(ident_name(name, "ReaderIterator").to_string());
            }
            _ => {}
        }
    }
    names
}

pub(crate) fn check_names(ast: &crate::ast::Ast, lazy: bool) -> std::io::Result<()> {
    use crate::ast::TopDecl;
    use std::collections::HashMap;
    let major = ast.major_decls();
    let mut local_types: HashMap<String, &str> = HashMap::new();
    for decl in &major {
        let primary = ident_name(decl.name(), "").to_string();
        if primary == "Self" {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "E_RUST_TYPE_COLLISION: {} generates reserved Rust keyword `Self`",
                    decl.name()
                ),
            ));
        }
        for generated_type in emitted_type_names(decl.as_ref(), lazy) {
            if let Some(previous) = local_types.insert(generated_type.clone(), decl.name()) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!(
                        "E_RUST_TYPE_COLLISION: {} collides with {previous} as `{generated_type}`",
                        decl.name()
                    ),
                ));
            }
        }
    }

    for decl in &major {
        let (fields, reserved): (&[crate::ast::FieldDecl], &[&str]) = match decl.as_ref() {
            TopDecl::Union(union) => {
                let mut seen_variants = HashSet::new();
                for item in union.items() {
                    let variant = ident_name(item.typ().name(), "").to_string();
                    if !seen_variants.insert(variant.clone()) {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!(
                                "E_RUST_VARIANT_COLLISION: union {} generates duplicate variant `{variant}`",
                                decl.name()
                            ),
                        ));
                    }
                }
                continue;
            }
            TopDecl::Struct(record) => (
                record.fields(),
                if lazy { &["verify"] } else { &["as_reader"] },
            ),
            TopDecl::Table(table) => (
                table.fields(),
                if lazy {
                    &["verify"]
                } else {
                    &[
                        "as_reader",
                        "total_size",
                        "field_count",
                        "count_extra_fields",
                        "has_extra_fields",
                    ]
                },
            ),
            _ => continue,
        };
        let mut seen_fields = HashSet::new();
        for field in fields {
            let generated = sanitize_identifier(field.name());
            if reserved.contains(&generated.as_str()) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!(
                        "E_RUST_FIELD_COLLISION: {}.{} collides with generated method `{generated}`",
                        decl.name(),
                        field.name()
                    ),
                ));
            }
            if !seen_fields.insert(generated.clone()) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!(
                        "E_RUST_FIELD_COLLISION: {}.{} collides with another field as `{generated}`",
                        decl.name(),
                        field.name()
                    ),
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn func_name(name: &str) -> m4::Ident {
    let span = m4::Span::call_site();
    m4::Ident::new(&sanitize_identifier(name), span)
}

pub(super) fn entity_iterator_name(name: &str) -> m4::Ident {
    ident_name(name, "Iterator")
}

pub(super) fn reader_iterator_name(name: &str) -> m4::Ident {
    ident_name(name, "ReaderIterator")
}

pub(super) fn entity_new_unchecked(
    typ: &crate::ast::TopDecl,
    expr: m4::TokenStream,
) -> m4::TokenStream {
    if typ.is_byte() {
        quote::quote!(molecule::prelude::Byte::new_unchecked(#expr))
    } else {
        let inner = entity_ref(typ);
        quote::quote!(<#inner as molecule::prelude::Entity>::new_unchecked(#expr))
    }
}

pub(super) fn reader_new_unchecked(
    typ: &crate::ast::TopDecl,
    expr: m4::TokenStream,
) -> m4::TokenStream {
    if typ.is_byte() {
        quote::quote!(molecule::prelude::ByteReader::new_unchecked(#expr))
    } else {
        let inner = reader_ref(typ);
        quote::quote!(<#inner<'r> as molecule::prelude::Reader<'r>>::new_unchecked(#expr))
    }
}

pub(super) fn entity_as_slice(typ: &crate::ast::TopDecl, expr: m4::TokenStream) -> m4::TokenStream {
    if typ.is_byte() {
        quote::quote!(molecule::prelude::Byte::as_slice(#expr))
    } else {
        quote::quote!(molecule::prelude::Entity::as_slice(#expr))
    }
}

pub(super) fn entity_as_bytes(typ: &crate::ast::TopDecl, expr: m4::TokenStream) -> m4::TokenStream {
    if typ.is_byte() {
        quote::quote!(molecule::prelude::Byte::as_bytes(*#expr))
    } else {
        quote::quote!(molecule::prelude::Entity::as_bytes(#expr))
    }
}

pub(super) fn reader_as_slice(typ: &crate::ast::TopDecl, expr: m4::TokenStream) -> m4::TokenStream {
    if typ.is_byte() {
        quote::quote!(molecule::prelude::ByteReader::as_slice(#expr))
    } else {
        quote::quote!(molecule::prelude::Reader::as_slice(#expr))
    }
}

pub(super) fn reader_verify(typ: &crate::ast::TopDecl, slice: m4::TokenStream) -> m4::TokenStream {
    if typ.is_byte() {
        quote::quote!(molecule::prelude::ByteReader::verify(#slice, compatible))
    } else {
        let inner = reader_ref(typ);
        quote::quote!(<#inner<'_> as molecule::prelude::Reader<'_>>::verify(#slice, compatible))
    }
}
