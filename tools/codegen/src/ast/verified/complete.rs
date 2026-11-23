use super::{
    super::raw, Array, Ast, Deps, DynVec, FieldDecl, FixVec, HasName, ImportStmt, ItemDecl,
    Option_, Struct, Table, TopDecl, Union, UnionItemDecl,
};
use std::{
    collections::{HashSet, VecDeque},
    rc::Rc,
};

impl Ast {
    pub(crate) fn complete(raw: &raw::Ast) -> Result<Self, String> {
        let mut names = HashSet::from(["byte".to_owned()]);
        let mut declared = HashSet::from(["byte"]);
        for decl in raw.decls() {
            if !names.insert(decl.name().to_ascii_lowercase()) {
                return Err(format!("E_SYMBOL_COLLISION: {}", decl.name()));
            }
            declared.insert(decl.name());
        }

        let mut deps = Deps::from([("byte", Rc::new(TopDecl::new_primitive("byte").unwrap()))]);
        let mut pending: VecDeque<_> = raw.decls().iter().collect();
        let mut deferred = 0;
        while let Some(decl) = pending.pop_front() {
            match TopDecl::complete(decl, &deps) {
                Ok(completed) => {
                    deps.insert(decl.name(), Rc::new(completed));
                    deferred = 0;
                }
                Err(LayoutError::Unresolved(reference)) => {
                    if !declared.contains(reference) {
                        return Err(format!(
                            "E_SCHEMA_DEPENDENCY: {} cannot resolve {reference} (missing type)",
                            decl.name()
                        ));
                    }
                    pending.push_back(decl);
                    deferred += 1;
                    // One full pass without resolving a declaration cannot make progress.
                    if deferred == pending.len() {
                        return Err(format!(
                            "E_SCHEMA_DEPENDENCY: {} cannot resolve {reference} (missing or cyclic type)",
                            decl.name()
                        ));
                    }
                }
                Err(LayoutError::InvalidSize) => {
                    return Err(format!(
                        "E_SCHEMA_LAYOUT: invalid fixed size in {}",
                        decl.name()
                    ));
                }
            }
        }

        let ast = Self {
            syntax_version: raw.syntax_version().cloned().unwrap_or_default(),
            namespace: raw.namespace().to_owned(),
            imports: raw
                .imports()
                .iter()
                .filter(|i| i.imported_depth() == 0)
                .map(|i| ImportStmt {
                    name: i.name().to_owned(),
                    paths: i.paths().to_owned(),
                    path_supers: i.path_supers(),
                })
                .collect(),
            decls: raw
                .decls()
                .iter()
                .map(|decl| Rc::clone(&deps[decl.name()]))
                .collect(),
            integers: raw.integers().clone(),
            #[cfg(feature = "schema-extensions")]
            source_graph: None,
        };
        ast.validate_layout_limits()?;
        Ok(ast)
    }

    fn validate_layout_limits(&self) -> Result<(), String> {
        if u32::try_from(self.syntax_version.version()).is_err() {
            return Err("E_SCHEMA_NUMBER: syntax version exceeds u32".into());
        }
        for decl in &self.decls {
            let fields = match decl.as_ref() {
                TopDecl::Union(union) => {
                    if union.items.iter().any(|i| u32::try_from(i.id).is_err()) {
                        return Err(format!("E_SCHEMA_UNION: ID exceeds u32 in {}", union.name));
                    }
                    let mut item_types = HashSet::new();
                    for item in &union.items {
                        if !item_types.insert(item.typ().name().to_ascii_lowercase()) {
                            return Err(format!(
                                "E_SCHEMA_UNION: duplicate union item {} in {}",
                                item.typ().name(),
                                union.name
                            ));
                        }
                    }
                    continue;
                }
                TopDecl::Array(array) if u32::try_from(array.item_count).is_err() => {
                    return Err(format!(
                        "E_SCHEMA_NUMBER: item count exceeds u32 in {}",
                        array.name
                    ));
                }
                _ if decl
                    .total_size()
                    .is_some_and(|size| u32::try_from(size).is_err()) =>
                {
                    return Err(format!(
                        "E_SCHEMA_LAYOUT: invalid fixed size in {}",
                        decl.name()
                    ));
                }
                TopDecl::Struct(record) => &record.fields,
                TopDecl::Table(table) => &table.fields,
                _ => continue,
            };
            let mut names = HashSet::new();
            for f in fields {
                if !names.insert(f.name.to_ascii_lowercase()) {
                    return Err(format!(
                        "E_SCHEMA_FIELD: duplicate {}.{}",
                        decl.name(),
                        f.name
                    ));
                }
            }
        }
        Ok(())
    }
}

enum LayoutError<'a> {
    Unresolved(&'a str),
    InvalidSize,
}

impl TopDecl {
    fn complete<'a>(decl: &'a raw::TopDecl, deps: &Deps) -> Result<Self, LayoutError<'a>> {
        use LayoutError::{InvalidSize, Unresolved};
        let lookup = |name: &'a str| deps.get(name).ok_or(Unresolved(name));
        let field = |f: &'a raw::FieldDecl| Ok(FieldDecl::new(f.name(), lookup(f.typ())?));
        Ok(match decl {
            raw::TopDecl::Option_(value) => Self::Option_(Option_ {
                name: value.name().to_owned(),
                item: ItemDecl::new(lookup(value.item().typ())?),
                imported_depth: value.imported_depth(),
            }),
            raw::TopDecl::Union(value) => Self::Union(Union {
                name: value.name().to_owned(),
                items: value
                    .items()
                    .iter()
                    .map(|i| Ok(UnionItemDecl::new(lookup(i.typ())?, i.id())))
                    .collect::<Result<_, _>>()?,
                imported_depth: value.imported_depth(),
            }),
            raw::TopDecl::Array(value) => {
                let typ = lookup(value.item().typ())?;
                let item_size = typ.total_size().ok_or(InvalidSize)?;
                item_size
                    .checked_mul(value.item_count())
                    .ok_or(InvalidSize)?;
                Self::Array(Array {
                    name: value.name().to_owned(),
                    item: ItemDecl::new(typ),
                    item_count: value.item_count(),
                    imported_depth: value.imported_depth(),
                    item_size,
                })
            }
            raw::TopDecl::Struct(value) => {
                let fields: Vec<_> = value.fields().iter().map(field).collect::<Result<_, _>>()?;
                let field_sizes: Vec<_> = fields
                    .iter()
                    .map(|f| f.typ().total_size().ok_or(InvalidSize))
                    .collect::<Result<_, _>>()?;
                field_sizes
                    .iter()
                    .copied()
                    .try_fold(0usize, usize::checked_add)
                    .ok_or(InvalidSize)?;
                Self::Struct(Struct {
                    name: value.name().to_owned(),
                    fields,
                    imported_depth: value.imported_depth(),
                    field_sizes,
                })
            }
            raw::TopDecl::Vector(value) => {
                let typ = lookup(value.item().typ())?;
                let name = value.name().to_owned();
                let item = ItemDecl::new(typ);
                let imported_depth = value.imported_depth();
                match typ.total_size() {
                    Some(item_size) => Self::FixVec(FixVec {
                        name,
                        item,
                        imported_depth,
                        item_size,
                    }),
                    None => Self::DynVec(DynVec {
                        name,
                        item,
                        imported_depth,
                    }),
                }
            }
            raw::TopDecl::Table(value) => Self::Table(Table {
                name: value.name().to_owned(),
                fields: value.fields().iter().map(field).collect::<Result<_, _>>()?,
                imported_depth: value.imported_depth(),
            }),
        })
    }
}
