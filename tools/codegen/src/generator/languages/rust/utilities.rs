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

fn emitted_type_names(decl: &crate::ast::TopDecl, lazy: bool) -> Vec<String> {
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

pub(crate) struct RustNamePlan {
    pub(crate) explicit_imports: Vec<(crate::ast::verified::ModulePath, String)>,
    pub(crate) qualified_lazy: std::collections::HashMap<String, crate::ast::verified::ModulePath>,
}

pub(crate) fn analyze_names(ast: &crate::ast::Ast, lazy: bool) -> std::io::Result<RustNamePlan> {
    use crate::ast::{HasName as _, TopDecl};
    use std::collections::{BTreeSet, HashMap};

    let major = ast.major_decls();
    let mut local_decls = HashSet::new();
    let mut local_types: HashMap<String, &str> = HashMap::new();
    for decl in &major {
        local_decls.insert(decl.name());
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

    // Track Rust type names brought in by direct glob imports so we can detect
    // glob ambiguities across direct imports (and disambiguate when origins are known).
    let mut direct_import_types: HashMap<String, HashSet<&str>> = HashMap::new();
    for decl in ast.decls() {
        if local_decls.contains(decl.name()) {
            continue;
        }
        let is_direct = if ast.has_decl_origins() {
            ast.decl_origin(decl.name())
                .is_some_and(|(_, direct)| direct)
        } else {
            true
        };
        if is_direct {
            for generated_type in emitted_type_names(decl.as_ref(), lazy) {
                direct_import_types
                    .entry(generated_type)
                    .or_default()
                    .insert(decl.name());
            }
        }
    }

    let mut explicit_set: BTreeSet<(crate::ast::verified::ModulePath, String)> = BTreeSet::new();
    let mut qualified_lazy: HashMap<String, crate::ast::verified::ModulePath> = HashMap::new();
    let mut imported_refs: HashMap<String, String> = HashMap::new();

    if !lazy {
        let mut check_dep = |dep: &TopDecl| -> std::io::Result<()> {
            if dep.is_byte() {
                if let Some(local) = local_types.get("ByteReader") {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!(
                            "E_RUST_TYPE_COLLISION: local {local} shadows builtin byte reader `ByteReader`"
                        ),
                    ));
                }
                return Ok(());
            }
            if local_decls.contains(dep.name()) {
                return Ok(());
            }
            let refs = [
                ident_name(dep.name(), "").to_string(),
                ident_name(dep.name(), "Reader").to_string(),
            ];
            for generated_type in refs {
                if let Some(local) = local_types.get(&generated_type) {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!(
                            "E_RUST_TYPE_COLLISION: local {local} shadows imported {} as `{generated_type}`",
                            dep.name()
                        ),
                    ));
                }
                if let Some(previous) =
                    imported_refs.insert(generated_type.clone(), dep.name().to_owned())
                {
                    if previous != dep.name() {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!(
                                "E_RUST_TYPE_COLLISION: imported {} collides with imported {previous} as `{generated_type}`",
                                dep.name()
                            ),
                        ));
                    }
                }
                let ambiguous = direct_import_types
                    .get(&generated_type)
                    .is_some_and(|set| set.len() > 1);
                let transitive = ast
                    .decl_origin(dep.name())
                    .is_some_and(|(_, direct)| !direct);
                if ambiguous || transitive {
                    if let Some((owner_path, _)) = ast.decl_origin(dep.name()) {
                        explicit_set.insert((owner_path.clone(), generated_type));
                    } else {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!(
                                "E_RUST_TYPE_COLLISION: imported {} is ambiguous as `{generated_type}`",
                                dep.name()
                            ),
                        ));
                    }
                }
            }
            Ok(())
        };
        for decl in &major {
            match decl.as_ref() {
                TopDecl::Option_(o) => check_dep(o.item().typ())?,
                TopDecl::Union(u) => {
                    for item in u.items() {
                        check_dep(item.typ())?;
                    }
                }
                TopDecl::Array(a) => check_dep(a.item().typ())?,
                TopDecl::FixVec(v) => check_dep(v.item().typ())?,
                TopDecl::DynVec(v) => check_dep(v.item().typ())?,
                TopDecl::Struct(s) => {
                    for f in s.fields() {
                        check_dep(f.typ())?;
                    }
                }
                TopDecl::Table(t) => {
                    for f in t.fields() {
                        check_dep(f.typ())?;
                    }
                }
                TopDecl::Primitive(_) => {}
            }
        }
    } else {
        struct LazyCheck<'a> {
            ast: &'a crate::ast::Ast,
            local_decls: &'a HashSet<&'a str>,
            local_types: &'a HashMap<String, &'a str>,
            direct_import_types: &'a HashMap<String, HashSet<&'a str>>,
            imported_refs: HashMap<String, String>,
            qualified_lazy: HashMap<String, crate::ast::verified::ModulePath>,
        }
        impl LazyCheck<'_> {
            fn visit(&mut self, dep: &TopDecl, in_union: bool) -> std::io::Result<()> {
                match dep {
                    TopDecl::Primitive(_) => Ok(()),
                    TopDecl::Option_(o) => self.visit(o.item().typ(), in_union),
                    TopDecl::Array(a) if !in_union && a.item().typ().is_byte() => Ok(()),
                    TopDecl::FixVec(v) if v.item().typ().is_byte() => Ok(()),
                    _ => {
                        if self.local_decls.contains(dep.name()) {
                            return Ok(());
                        }
                        let generated_type = ident_name(dep.name(), "").to_string();
                        let shadowed = self.local_types.contains_key(&generated_type);
                        let ambiguous = self
                            .direct_import_types
                            .get(&generated_type)
                            .is_some_and(|set| set.len() > 1);
                        let transitive = self
                            .ast
                            .decl_origin(dep.name())
                            .is_some_and(|(_, direct)| !direct);
                        let reused_conflict = self
                            .imported_refs
                            .get(&generated_type)
                            .is_some_and(|prev| prev != dep.name());
                        if shadowed || ambiguous || transitive || reused_conflict {
                            if let Some((owner_path, _)) = self.ast.decl_origin(dep.name()) {
                                self.qualified_lazy
                                    .insert(dep.name().to_owned(), owner_path.clone());
                                Ok(())
                            } else {
                                Err(std::io::Error::new(
                                    std::io::ErrorKind::InvalidInput,
                                    format!(
                                        "E_RUST_TYPE_COLLISION: imported {} collides or is shadowed as `{generated_type}`",
                                        dep.name()
                                    ),
                                ))
                            }
                        } else {
                            self.imported_refs
                                .insert(generated_type, dep.name().to_owned());
                            Ok(())
                        }
                    }
                }
            }
        }
        let mut ctx = LazyCheck {
            ast,
            local_decls: &local_decls,
            local_types: &local_types,
            direct_import_types: &direct_import_types,
            imported_refs,
            qualified_lazy,
        };
        for decl in &major {
            match decl.as_ref() {
                TopDecl::Option_(_) | TopDecl::Primitive(_) => {}
                TopDecl::Union(u) => {
                    for item in u.items() {
                        ctx.visit(item.typ(), true)?;
                    }
                }
                TopDecl::Array(a) => ctx.visit(a.item().typ(), false)?,
                TopDecl::FixVec(v) => ctx.visit(v.item().typ(), false)?,
                TopDecl::DynVec(v) => ctx.visit(v.item().typ(), false)?,
                TopDecl::Struct(s) => {
                    for f in s.fields() {
                        ctx.visit(f.typ(), false)?;
                    }
                }
                TopDecl::Table(t) => {
                    for f in t.fields() {
                        ctx.visit(f.typ(), false)?;
                    }
                }
            }
        }
        qualified_lazy = ctx.qualified_lazy;
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
    Ok(RustNamePlan {
        explicit_imports: explicit_set.into_iter().collect(),
        qualified_lazy,
    })
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
        let inner = entity_name(typ.name());
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
        let inner = reader_name(typ.name());
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
        let inner = reader_name(typ.name());
        quote::quote!(<#inner<'_> as molecule::prelude::Reader<'_>>::verify(#slice, compatible))
    }
}
