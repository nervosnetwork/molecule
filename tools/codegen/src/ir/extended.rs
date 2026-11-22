use super::{Ir, ToIntermediate, TopDecl};
use crate::{
    ast,
    integer::{self, LogicalInteger},
    schema::SchemaGraph,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const INTEGER_FEATURE: &str = "logical-integer-v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum LogicalType {
    Integer(LogicalInteger),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Envelope {
    pub ir_version: u32,
    pub required_features: Vec<String>,
    pub wire_ir: Ir,
    pub sources: SchemaGraph,
    pub logical_types: Vec<LogicalType>,
}

impl Envelope {
    pub(crate) fn check_integer_contract(
        path: &std::path::Path,
        ast: &ast::Ast,
    ) -> Result<(), String> {
        let bytes = std::fs::read(path).map_err(|err| err.to_string())?;
        let published: Self = serde_json::from_slice(&bytes).map_err(|err| err.to_string())?;
        ast.check_integer_contract(&published.recover()?)
    }

    pub fn from_ast(ast: &ast::Ast) -> Result<Self, String> {
        let sources = ast
            .source_graph()
            .ok_or("E_IR_SOURCE: extended IR requires source provenance")?
            .clone();
        let mut logical_types = Vec::new();
        for (name, spec) in ast.integers() {
            let owner = ast.integer_owner(name)?;
            logical_types.push(LogicalType::Integer(LogicalInteger {
                schema: owner.to_owned(),
                name: name.clone(),
                bits: spec.bits,
                signed: spec.signed,
                byte_order: spec.byte_order,
            }));
        }
        let required_features = if logical_types.is_empty() {
            Vec::new()
        } else {
            vec![INTEGER_FEATURE.into()]
        };
        Ok(Self {
            ir_version: 2,
            required_features,
            wire_ir: ast.to_ir(),
            sources,
            logical_types,
        })
    }

    pub fn recover(self) -> Result<ast::Ast, String> {
        if self.ir_version != 2 {
            return Err(format!(
                "E_IR_VERSION: expected 2, found {}",
                self.ir_version
            ));
        }
        let features: BTreeSet<_> = self.required_features.iter().map(String::as_str).collect();
        if features.len() != self.required_features.len()
            || features.iter().any(|name| *name != INTEGER_FEATURE)
        {
            return Err(format!(
                "E_IR_FEATURE: unsupported or duplicate required features {:?}",
                self.required_features
            ));
        }
        if !self.logical_types.is_empty() && !features.contains(INTEGER_FEATURE) {
            return Err("E_IR_FEATURE: integer semantics require logical-integer-v1".into());
        }
        self.sources.validate()?;
        validate_wire(&self.wire_ir)?;
        let root = self
            .sources
            .modules
            .iter()
            .find(|module| module.id == self.sources.root)
            .ok_or("E_IR_SOURCE: missing root")?;
        let mut imported = BTreeSet::new();
        let expected: Vec<_> = root
            .imports
            .iter()
            .filter(|edge| imported.insert(&edge.module))
            .map(|edge| edge.path.as_str())
            .collect();
        let actual: Vec<_> = self
            .wire_ir
            .imports
            .iter()
            .map(|import| {
                let mut path = "../".repeat(import.path_supers);
                for part in &import.paths {
                    path.push_str(part);
                    path.push('/');
                }
                path.push_str(&import.name);
                path
            })
            .collect();
        if expected != actual {
            return Err("E_IR_SOURCE: wire imports differ from source graph".into());
        }
        if std::path::Path::new(&self.sources.root)
            .file_stem()
            .and_then(|name| name.to_str())
            != Some(self.wire_ir.namespace.as_str())
        {
            return Err("E_IR_SOURCE: namespace differs from root file name".into());
        }
        let source_names: BTreeSet<_> = self
            .sources
            .modules
            .iter()
            .flat_map(|module| module.declarations.iter().map(|decl| decl.name.as_str()))
            .collect();
        let wire_names: BTreeSet<_> = self.wire_ir.decls.iter().map(TopDecl::name).collect();
        if source_names != wire_names {
            return Err("E_IR_SOURCE: declarations and source owners differ".into());
        }
        for decl in &self.wire_ir.decls {
            let owner = self
                .sources
                .owner(decl.name())
                .ok_or("E_IR_SOURCE: missing declaration owner")?;
            if (owner.id == self.sources.root) != (decl.imported_depth() == 0) {
                return Err(format!(
                    "E_IR_SOURCE: ownership/depth mismatch for {}",
                    decl.name()
                ));
            }
        }
        let annotations = self
            .logical_types
            .into_iter()
            .map(|typ| match typ {
                LogicalType::Integer(integer) => integer,
            })
            .collect();
        let mut ast = self.wire_ir.recover(true)?;
        integer::bind(&mut ast, self.sources, annotations)?;
        Ok(ast)
    }
}

pub(crate) fn identifier(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes.next().is_some_and(|byte| byte.is_ascii_alphabetic())
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

/// Check the names and canonical union IDs published in v2 envelopes.
fn validate_wire(ir: &Ir) -> Result<(), String> {
    if ir.namespace.is_empty()
        || ir.namespace.contains(['/', '\\', '\0'])
        || ir.namespace == "."
        || ir.namespace == ".."
    {
        return Err("E_IR_NAME: invalid namespace".into());
    }
    for import in &ir.imports {
        if !identifier(&import.name)
            || import.paths.iter().any(|part| !identifier(part))
            || import.path_supers > 1024
        {
            return Err("E_IR_NAME: invalid import path".into());
        }
    }
    let mut names = BTreeSet::from(["byte".to_owned()]);
    for decl in &ir.decls {
        if !identifier(decl.name()) || !names.insert(decl.name().to_ascii_lowercase()) {
            return Err(format!(
                "E_IR_NAME: invalid, reserved or duplicate name {}",
                decl.name()
            ));
        }
        let fields = match decl {
            TopDecl::Struct(record) => record.fields.as_slice(),
            TopDecl::Table(table) => table.fields.as_slice(),
            _ => &[],
        };
        let mut field_names = BTreeSet::new();
        for field in fields {
            if !identifier(&field.name) || !field_names.insert(field.name.to_ascii_lowercase()) {
                return Err(format!(
                    "E_IR_FIELD: duplicate or invalid field {}.{}",
                    decl.name(),
                    field.name
                ));
            }
        }
        if let TopDecl::Union(union) = decl {
            if union.items.is_empty()
                || union
                    .items
                    .iter()
                    .any(|item| u32::try_from(item.id).is_err())
                || union
                    .items
                    .windows(2)
                    .any(|items| items[0].id >= items[1].id)
            {
                return Err(format!(
                    "E_IR_UNION: invalid or unordered union IDs in {}",
                    union.name
                ));
            }
        }
    }
    Ok(())
}

impl Ir {
    /// Legacy IR carries explicit vector kinds. Only v2 requires the canonical kind
    /// and requires every fixed layout to fit a Molecule word.
    pub(super) fn recover(self, canonical: bool) -> Result<ast::Ast, String> {
        let mut names = BTreeSet::new();
        for decl in &self.decls {
            if decl.name().eq_ignore_ascii_case("byte") || !names.insert(decl.name()) {
                return Err(format!(
                    "E_IR_NAME: reserved or duplicate name {}",
                    decl.name()
                ));
            }
            if matches!(decl, TopDecl::Union(union) if union.items.is_empty()) {
                return Err(format!("E_IR_UNION: empty union {}", decl.name()));
            }
        }
        validate_layout(&self, &names, canonical)?;
        Ok(ast::Ast::recover(self))
    }
}

fn validate_layout(ir: &Ir, names: &BTreeSet<&str>, canonical: bool) -> Result<(), String> {
    let mut fixed: BTreeMap<&str, Option<usize>> = BTreeMap::from([("byte", Some(1))]);
    let mut pending: Vec<_> = ir.decls.iter().collect();
    while !pending.is_empty() {
        let mut remaining = Vec::new();
        for decl in &pending {
            let dependencies: Vec<&str> = match decl {
                TopDecl::Option_(value) => vec![&value.item.typ],
                TopDecl::Array(value) => vec![&value.item.typ],
                TopDecl::FixVec(value) => vec![&value.item.typ],
                TopDecl::DynVec(value) => vec![&value.item.typ],
                TopDecl::Union(value) => value.items.iter().map(|item| item.typ.as_str()).collect(),
                TopDecl::Struct(value) => value
                    .fields
                    .iter()
                    .map(|field| field.typ.as_str())
                    .collect(),
                TopDecl::Table(value) => value
                    .fields
                    .iter()
                    .map(|field| field.typ.as_str())
                    .collect(),
            };
            if let Some(reference) = dependencies.iter().find(|name| !fixed.contains_key(*name)) {
                if !names.contains(reference) {
                    return Err(format!(
                        "E_IR_DEPENDENCY: {} references unknown type {reference}",
                        decl.name()
                    ));
                }
                remaining.push(*decl);
                continue;
            }
            let invalid = || format!("E_IR_LAYOUT: invalid fixed size in {}", decl.name());
            let size = match decl {
                TopDecl::Array(array) => {
                    let item = fixed[array.item.typ.as_str()]
                        .filter(|size| *size != 0)
                        .ok_or_else(invalid)?;
                    let size = item
                        .checked_mul(array.item_count)
                        .filter(|size| !canonical || (*size != 0 && u32::try_from(*size).is_ok()))
                        .ok_or_else(invalid)?;
                    Some(size)
                }
                TopDecl::Struct(_) => {
                    let size = dependencies.iter().try_fold(0usize, |size, name| {
                        size.checked_add(fixed[name].ok_or_else(invalid)?)
                            .ok_or_else(invalid)
                    })?;
                    if size == 0 || (canonical && u32::try_from(size).is_err()) {
                        return Err(invalid());
                    }
                    Some(size)
                }
                TopDecl::FixVec(vector) => {
                    if fixed[vector.item.typ.as_str()].is_none() {
                        return Err(invalid());
                    }
                    None
                }
                TopDecl::DynVec(vector) => {
                    if canonical && fixed[vector.item.typ.as_str()].is_some() {
                        return Err(format!(
                            "E_IR_LAYOUT: {} must be a fixed vector",
                            vector.name
                        ));
                    }
                    None
                }
                _ => None,
            };
            fixed.insert(decl.name(), size);
        }
        if remaining.len() == pending.len() {
            return Err(format!(
                "E_IR_DEPENDENCY: missing or cyclic references: {:?}",
                remaining.iter().map(|decl| decl.name()).collect::<Vec<_>>()
            ));
        }
        pending = remaining;
    }
    Ok(())
}

impl TopDecl {
    pub(crate) fn imported_depth(&self) -> usize {
        match self {
            Self::Option_(value) => value.imported_depth,
            Self::Union(value) => value.imported_depth,
            Self::Array(value) => value.imported_depth,
            Self::Struct(value) => value.imported_depth,
            Self::FixVec(value) => value.imported_depth,
            Self::DynVec(value) => value.imported_depth,
            Self::Table(value) => value.imported_depth,
        }
    }
}
