use case::CaseExt;
use proc_macro2 as m4;

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
