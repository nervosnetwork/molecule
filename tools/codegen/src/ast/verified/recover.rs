use std::{
    collections::{BTreeSet, VecDeque},
    rc::Rc,
};

use crate::ir;

enum RecoverError<'a> {
    Unresolved(&'a str),
    Message(String),
}

fn recover_decl<'a>(
    decl: &'a ir::TopDecl,
    deps: &super::Deps,
    canonical: bool,
) -> Result<super::TopDecl, RecoverError<'a>> {
    use RecoverError::{Message, Unresolved};
    let lookup = |name: &'a str| deps.get(name).ok_or(Unresolved(name));
    let invalid = || {
        Message(format!(
            "E_IR_LAYOUT: invalid fixed size in {}",
            decl.name()
        ))
    };
    match decl {
        ir::TopDecl::Option_(inner) => Ok(super::Option_ {
            name: inner.name().to_owned(),
            item: super::ItemDecl::new(lookup(inner.item().typ())?),
            imported_depth: inner.imported_depth(),
        }
        .into()),
        ir::TopDecl::Union(inner) => {
            if inner.items().is_empty() {
                return Err(Message(format!("E_IR_UNION: empty union {}", inner.name())));
            }
            let items = inner
                .items()
                .iter()
                .map(|item| Ok(super::UnionItemDecl::new(lookup(item.typ())?, item.id())))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(super::Union {
                name: inner.name().to_owned(),
                items,
                imported_depth: inner.imported_depth(),
            }
            .into())
        }
        ir::TopDecl::Array(inner) => {
            let dep = lookup(inner.item().typ())?;
            let item_size = dep.total_size().filter(|&s| s != 0).ok_or_else(invalid)?;
            item_size
                .checked_mul(inner.item_count())
                .filter(|&size| size != 0 && u32::try_from(size).is_ok())
                .ok_or_else(invalid)?;
            Ok(super::Array {
                name: inner.name().to_owned(),
                item: super::ItemDecl::new(dep),
                item_count: inner.item_count(),
                imported_depth: inner.imported_depth(),
                item_size,
            }
            .into())
        }
        ir::TopDecl::Struct(inner) => {
            let mut fields = Vec::with_capacity(inner.fields().len());
            let mut field_sizes = Vec::with_capacity(inner.fields().len());
            for ir_field in inner.fields() {
                let dep = lookup(ir_field.typ())?;
                let field_size = dep.total_size().ok_or_else(invalid)?;
                field_sizes.push(field_size);
                fields.push(super::FieldDecl::new(ir_field.name(), dep));
            }
            let total = field_sizes
                .iter()
                .copied()
                .try_fold(0usize, usize::checked_add)
                .ok_or_else(invalid)?;
            if total == 0 || u32::try_from(total).is_err() {
                return Err(invalid());
            }
            Ok(super::Struct {
                name: inner.name().to_owned(),
                fields,
                imported_depth: inner.imported_depth(),
                field_sizes,
            }
            .into())
        }
        ir::TopDecl::FixVec(inner) => {
            let dep = lookup(inner.item().typ())?;
            let item_size = dep.total_size().filter(|&s| s != 0).ok_or_else(invalid)?;
            Ok(super::FixVec {
                name: inner.name().to_owned(),
                item: super::ItemDecl::new(dep),
                imported_depth: inner.imported_depth(),
                item_size,
            }
            .into())
        }
        ir::TopDecl::DynVec(inner) => {
            let dep = lookup(inner.item().typ())?;
            if canonical && dep.total_size().is_some() {
                return Err(Message(format!(
                    "E_IR_LAYOUT: {} must be a fixed vector",
                    inner.name()
                )));
            }
            Ok(super::DynVec {
                name: inner.name().to_owned(),
                item: super::ItemDecl::new(dep),
                imported_depth: inner.imported_depth(),
            }
            .into())
        }
        ir::TopDecl::Table(inner) => {
            let fields = inner
                .fields()
                .iter()
                .map(|field| Ok(super::FieldDecl::new(field.name(), lookup(field.typ())?)))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(super::Table {
                name: inner.name().to_owned(),
                fields,
                imported_depth: inner.imported_depth(),
            }
            .into())
        }
    }
}

impl super::Ast {
    pub(crate) fn recover(ir: ir::Ir, canonical: bool) -> Result<Self, String> {
        ir::validate_wire(&ir, canonical)?;
        let names: BTreeSet<&str> = ir.decls().iter().map(ir::TopDecl::name).collect();

        let mut decls_result = super::Deps::from([(
            "byte",
            Rc::new(super::TopDecl::new_primitive("byte").unwrap()),
        )]);
        let mut pending: VecDeque<&ir::TopDecl> = ir.decls().iter().collect();
        let mut deferred = 0;
        while let Some(decl) = pending.pop_front() {
            match recover_decl(decl, &decls_result, canonical) {
                Ok(recovered) => {
                    decls_result.insert(decl.name(), Rc::new(recovered));
                    deferred = 0;
                }
                Err(RecoverError::Unresolved(reference)) => {
                    if !names.contains(reference) {
                        return Err(format!(
                            "E_IR_DEPENDENCY: {} references unknown type {reference}",
                            decl.name()
                        ));
                    }
                    pending.push_back(decl);
                    deferred += 1;
                    if deferred == pending.len() {
                        return Err(format!(
                            "E_IR_DEPENDENCY: missing or cyclic references: {:?}",
                            pending.iter().map(|d| d.name()).collect::<Vec<_>>()
                        ));
                    }
                }
                Err(RecoverError::Message(message)) => return Err(message),
            }
        }

        let namespace = ir.namespace().to_owned();
        let imports = ir
            .imports()
            .iter()
            .map(super::ImportStmt::recover)
            .collect();
        let decls = ir
            .decls()
            .iter()
            .map(|decl| Rc::clone(&decls_result[decl.name()]))
            .collect();
        let syntax_version = ir.syntax_version().to_owned();

        Ok(Self {
            syntax_version,
            namespace,
            imports,
            decls,
            integers: Default::default(),
            source_graph: None,
            decl_origins: None,
        })
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
