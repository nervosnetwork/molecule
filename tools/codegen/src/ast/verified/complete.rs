use crate::ast::SyntaxVersion;
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use super::must_get_primitive_types;

use super::{super::raw, Primitive};

trait CompleteRawDecl {
    fn complete(&self, deps: &super::Deps) -> Option<super::TopDecl>;
}

impl CompleteRawDecl for raw::OptionDecl {
    fn complete(&self, deps: &super::Deps) -> Option<super::TopDecl> {
        deps.get(self.item().typ()).map(|dep| {
            let name = self.name().to_owned();
            let item = super::ItemDecl::new(dep);
            super::Option_ {
                name,
                item,
                imported_depth: self.imported_depth(),
            }
            .into()
        })
    }
}

impl CompleteRawDecl for raw::UnionDecl {
    fn complete(&self, deps: &super::Deps) -> Option<super::TopDecl> {
        if self.items().is_empty() {
            panic!("the union ({}) is empty", self.name());
        }
        self.items()
            .iter()
            .map(|raw_item| {
                deps.get(raw_item.typ())
                    .map(|typ| super::UnionItemDecl::new(typ, raw_item.id()))
            })
            .collect::<Option<Vec<_>>>()
            .map(|items| {
                let name = self.name().to_owned();
                super::Union {
                    name,
                    items,
                    imported_depth: self.imported_depth(),
                }
                .into()
            })
    }
}

impl CompleteRawDecl for raw::ArrayDecl {
    fn complete(&self, deps: &super::Deps) -> Option<super::TopDecl> {
        deps.get(self.item().typ()).map(|dep| {
            let item_size = dep.total_size().unwrap_or_else(|| {
                panic!(
                    "the item type ({}) of array ({}) doesn't have fixed size",
                    self.item().typ(),
                    self.name(),
                );
            });
            if item_size == 0 {
                panic!("the array ({}) has no size", self.name());
            }
            let name = self.name().to_owned();
            let item = super::ItemDecl::new(dep);
            let item_count = self.item_count();
            super::Array {
                name,
                item,
                item_count,
                imported_depth: self.imported_depth(),
                item_size,
            }
            .into()
        })
    }
}

impl CompleteRawDecl for raw::StructDecl {
    fn complete(&self, deps: &super::Deps) -> Option<super::TopDecl> {
        let mut fields = Vec::with_capacity(self.fields().len());
        let mut field_sizes = Vec::with_capacity(self.fields().len());
        for raw_field in self.fields() {
            let field_name = raw_field.name();
            if let Some(dep) = deps.get(raw_field.typ()) {
                if let Some(field_size) = dep.total_size() {
                    field_sizes.push(field_size);
                } else {
                    panic!(
                        "the filed type ({}) in struct ({}) doesn't have fixed size",
                        field_name,
                        self.name(),
                    );
                }
                let field = super::FieldDecl::new(field_name, dep);
                fields.push(field);
            } else {
                break;
            }
        }
        if fields.len() != self.fields().len() {
            return None;
        }
        if field_sizes.iter().sum::<usize>() == 0 {
            panic!("the struct ({}) has no size", self.name());
        }
        let name = self.name().to_owned();
        Some(
            super::Struct {
                name,
                fields,
                imported_depth: self.imported_depth(),
                field_sizes,
            }
            .into(),
        )
    }
}

impl CompleteRawDecl for raw::VectorDecl {
    fn complete(&self, deps: &super::Deps) -> Option<super::TopDecl> {
        deps.get(self.item().typ()).map(|dep| {
            let name = self.name().to_owned();
            let item = super::ItemDecl::new(dep);
            if let Some(item_size) = dep.total_size() {
                super::FixVec {
                    name,
                    item,
                    imported_depth: self.imported_depth(),
                    item_size,
                }
                .into()
            } else {
                super::DynVec {
                    name,
                    item,
                    imported_depth: self.imported_depth(),
                }
                .into()
            }
        })
    }
}

impl CompleteRawDecl for raw::TableDecl {
    fn complete(&self, deps: &super::Deps) -> Option<super::TopDecl> {
        self.fields()
            .iter()
            .map(|raw_field| {
                let field_name = raw_field.name();
                deps.get(raw_field.typ())
                    .map(|dep| super::FieldDecl::new(field_name, dep))
            })
            .collect::<Option<Vec<_>>>()
            .map(|fields| {
                let name = self.name().to_owned();
                super::Table {
                    name,
                    fields,
                    imported_depth: self.imported_depth(),
                }
                .into()
            })
    }
}

impl super::Ast {
    pub(crate) fn complete(raw: raw::Ast) -> Self {
        let mut decls_idx = HashMap::new();
        let mut decls_keys = HashSet::new();
        let expand_primitive = !raw
            .syntax_version()
            .unwrap_or(&SyntaxVersion::default())
            .support_primitive_types();
        for decl in raw.decls() {
            let name = decl.name();
            if super::TopDecl::new_primitive(name.to_lowercase().as_str(), !expand_primitive)
                .is_some()
            {
                panic!("the name `{}` is reserved", name);
            }
            if decls_idx.insert(name, decl).is_some() || !decls_keys.insert(name) {
                panic!("the name `{}` is used more than once", name);
            };
        }

        let mut primitives = vec![Primitive {
            name: "byte".to_string(),
            size: 1,
        }];

        if !expand_primitive {
            primitives.extend(must_get_primitive_types());
        }
        dbg!(expand_primitive);

        let mut decls_result = HashMap::new();
        primitives.iter().for_each(|primitive_type| {
            decls_result.insert(
                primitive_type.name(),
                Rc::new(super::TopDecl::new_primitive(primitive_type.name(), true).unwrap()),
            );
        });

        loop {
            if decls_keys.is_empty() {
                break;
            }
            let incompleted = decls_keys.len();
            decls_keys.retain(|&name| {
                let decl_raw = decls_idx.get(name).unwrap();
                if let Some(decl) = super::TopDecl::complete(decl_raw, &decls_result) {
                    decls_result.insert(name, Rc::new(decl));
                    false
                } else {
                    true
                }
            });
            if decls_keys.len() == incompleted {
                panic!(
                    "there are {} types which are unable to be completed: {:?}",
                    incompleted, decls_keys
                );
            }
        }
        let namespace = raw.namespace().to_owned();
        let imports = raw
            .imports()
            .iter()
            .filter(|r| r.imported_depth() == 0)
            .map(super::ImportStmt::complete)
            .collect();
        // remove the primitive types and keep the order
        let mut decls = Vec::with_capacity(raw.decls().len());
        for decl in raw.decls() {
            let result = decls_result.get(decl.name()).unwrap();
            decls.push(Rc::clone(result));
        }

        let syntax_version = raw.syntax_version().unwrap().to_owned();

        Self {
            syntax_version,
            namespace,
            imports,
            decls,
        }
    }
}

impl super::ImportStmt {
    fn complete(raw: &raw::ImportStmt) -> Self {
        Self {
            name: raw.name().to_owned(),
            paths: raw.paths().to_owned(),
            path_supers: raw.path_supers(),
        }
    }
}

impl super::TopDecl {
    fn complete(raw: &raw::TopDecl, deps: &super::Deps) -> Option<Self> {
        match raw {
            raw::TopDecl::Option_(inner) => inner.complete(deps),
            raw::TopDecl::Union(inner) => inner.complete(deps),
            raw::TopDecl::Array(inner) => inner.complete(deps),
            raw::TopDecl::Struct(inner) => inner.complete(deps),
            raw::TopDecl::Vector(inner) => inner.complete(deps),
            raw::TopDecl::Table(inner) => inner.complete(deps),
        }
    }
}
