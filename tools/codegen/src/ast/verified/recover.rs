use crate::ast::verified;
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::ir;

use super::must_get_primitive_types;

trait RecoverFromIr {
    fn recover(&self, deps: &super::Deps) -> Option<super::TopDecl>;
}

impl RecoverFromIr for ir::Option_ {
    fn recover(&self, deps: &super::Deps) -> Option<super::TopDecl> {
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

impl RecoverFromIr for ir::Union {
    fn recover(&self, deps: &super::Deps) -> Option<super::TopDecl> {
        if self.items().is_empty() {
            panic!("the union ({}) is empty", self.name());
        }

        self.items()
            .iter()
            .map(|ir_item| {
                deps.get(ir_item.typ())
                    .map(|item| super::UnionItemDecl::new(item, ir_item.id()))
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

impl RecoverFromIr for ir::Array {
    fn recover(&self, deps: &super::Deps) -> Option<super::TopDecl> {
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

impl RecoverFromIr for ir::Struct {
    fn recover(&self, deps: &super::Deps) -> Option<super::TopDecl> {
        let mut fields = Vec::with_capacity(self.fields().len());
        let mut field_sizes = Vec::with_capacity(self.fields().len());
        for ir_field in self.fields() {
            let field_name = ir_field.name();
            if let Some(dep) = deps.get(ir_field.typ()) {
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

impl RecoverFromIr for ir::FixVec {
    fn recover(&self, deps: &super::Deps) -> Option<super::TopDecl> {
        deps.get(self.item().typ()).map(|dep| {
            let name = self.name().to_owned();
            let item = super::ItemDecl::new(dep);
            let item_size = dep.total_size().unwrap();
            super::FixVec {
                name,
                item,
                imported_depth: self.imported_depth(),
                item_size,
            }
            .into()
        })
    }
}

impl RecoverFromIr for ir::DynVec {
    fn recover(&self, deps: &super::Deps) -> Option<super::TopDecl> {
        deps.get(self.item().typ()).map(|dep| {
            let name = self.name().to_owned();
            let item = super::ItemDecl::new(dep);
            super::DynVec {
                name,
                item,
                imported_depth: self.imported_depth(),
            }
            .into()
        })
    }
}

impl RecoverFromIr for ir::Table {
    fn recover(&self, deps: &super::Deps) -> Option<super::TopDecl> {
        self.fields()
            .iter()
            .map(|ir_field| {
                let field_name = ir_field.name();
                deps.get(ir_field.typ())
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
    pub(crate) fn recover(ir: ir::Ir) -> Self {
        let expand_primitive = !ir.syntax_version().support_primitive_types();
        let mut decls_idx = HashMap::new();
        let mut decls_keys = HashSet::new();
        for decl in ir.decls() {
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
        let mut primitives = vec![verified::Primitive {
            name: "byte".to_string(),
            size: 1,
        }];

        if !expand_primitive {
            primitives.extend(must_get_primitive_types());
        }

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
            let unrecovered = decls_keys.len();
            decls_keys.retain(|&name| {
                let decl_ir = decls_idx.get(name).unwrap();
                if let Some(decl) = super::TopDecl::recover(decl_ir, &decls_result) {
                    decls_result.insert(name, Rc::new(decl));
                    false
                } else {
                    true
                }
            });
            if decls_keys.len() == unrecovered {
                panic!(
                    "there are {} types which are unable to be recovered: {:?}",
                    unrecovered, decls_keys
                );
            }
        }
        let namespace = ir.namespace().to_owned();
        let imports = ir
            .imports()
            .iter()
            .map(super::ImportStmt::recover)
            .collect();
        // remove the primitive types and keep the order
        let mut decls = Vec::with_capacity(ir.decls().len());
        for decl in ir.decls() {
            let result = decls_result.get(decl.name()).unwrap();
            decls.push(Rc::clone(result));
        }

        let syntax_version = ir.syntax_version().to_owned();
        Self {
            syntax_version,
            namespace,
            imports,
            decls,
        }
    }
}

impl super::ImportStmt {
    fn recover(ir: &ir::ImportStmt) -> Self {
        Self {
            name: ir.name().to_owned(),
            paths: ir.paths().to_owned(),
            path_supers: ir.path_supers(),
        }
    }
}

impl super::TopDecl {
    fn recover(ir: &ir::TopDecl, deps: &super::Deps) -> Option<Self> {
        match ir {
            ir::TopDecl::Option_(inner) => inner.recover(deps),
            ir::TopDecl::Union(inner) => inner.recover(deps),
            ir::TopDecl::Array(inner) => inner.recover(deps),
            ir::TopDecl::Struct(inner) => inner.recover(deps),
            ir::TopDecl::FixVec(inner) => inner.recover(deps),
            ir::TopDecl::DynVec(inner) => inner.recover(deps),
            ir::TopDecl::Table(inner) => inner.recover(deps),
        }
    }
}
