use super::{Ir, ToIntermediate, TopDecl};
use crate::{
    ast,
    integer::{self, LogicalInteger},
    schema::SchemaGraph,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

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

    pub fn recover(mut self) -> Result<ast::Ast, String> {
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
        validate_wire(&self.wire_ir, true)?;
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
        if !crate::utils::valid_namespace(&self.sources.root)
            || std::path::Path::new(&self.sources.root)
                .file_stem()
                .and_then(|name| name.to_str())
                != Some(self.wire_ir.namespace.as_str())
        {
            return Err("E_IR_SOURCE: namespace differs from root file name".into());
        }
        let origins = recover_decl_origins(&self.sources, &mut self.wire_ir)?;
        let annotations = self
            .logical_types
            .into_iter()
            .map(|typ| match typ {
                LogicalType::Integer(integer) => integer,
            })
            .collect();
        let mut ast = self.wire_ir.recover(true)?;
        integer::bind(&mut ast, self.sources, annotations)?;
        ast.set_decl_origins(origins);
        Ok(ast)
    }
}

fn recover_decl_origins(
    graph: &SchemaGraph,
    wire: &mut Ir,
) -> Result<std::collections::BTreeMap<String, ast::verified::ModulePath>, String> {
    use std::collections::{BTreeMap, VecDeque};
    let modules: BTreeMap<_, _> = graph.modules.iter().map(|m| (m.id.as_str(), m)).collect();
    let mut declarations: BTreeMap<_, _> = wire
        .decls
        .iter_mut()
        .map(|decl| (decl.name().to_owned(), decl))
        .collect();
    let mut origins = BTreeMap::new();
    let mut visited = BTreeSet::from([graph.root.as_str()]);
    let mut queue = VecDeque::from([(graph.root.as_str(), ast::verified::ModulePath::root(), 0)]);
    while let Some((id, mod_path, depth)) = queue.pop_front() {
        let module = modules
            .get(id)
            .ok_or("E_IR_SOURCE: missing source module")?;
        for source in &module.declarations {
            let decl = declarations
                .remove(&source.name)
                .ok_or("E_IR_SOURCE: declarations and source owners differ")?;
            let stored = decl.imported_depth_mut();
            if (*stored == 0) != (depth == 0) {
                return Err(format!(
                    "E_IR_SOURCE: ownership/depth mismatch for {}",
                    source.name
                ));
            }
            // Early v2 producers stored 1 for every import. The source graph
            // still identifies its shortest path when exporting v1 again.
            *stored = depth;
            if depth != 0 {
                origins.insert(source.name.clone(), mod_path.clone());
            }
        }
        for edge in &module.imports {
            if visited.insert(edge.module.as_str()) {
                let edge_path = ast::verified::ModulePath::from_import_str(&edge.path)
                    .ok_or_else(|| format!("E_IR_NAME: invalid import path {}", edge.path))?;
                queue.push_back((
                    edge.module.as_str(),
                    mod_path.resolve_edge(&edge_path),
                    depth + 1,
                ));
            }
        }
    }
    if declarations.is_empty() {
        Ok(origins)
    } else {
        Err("E_IR_SOURCE: declarations and source owners differ".into())
    }
}

pub(crate) fn identifier(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes.next().is_some_and(|byte| byte.is_ascii_alphabetic())
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

/// Check names, syntax version, fields, and union invariants on wire IR.
pub(crate) fn validate_wire(ir: &Ir, canonical: bool) -> Result<(), String> {
    if u32::try_from(ir.syntax_version.version()).is_err() {
        return Err("E_IR_NUMBER: syntax version exceeds u32".into());
    }
    if !crate::utils::valid_namespace(&ir.namespace) {
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
        let key = if canonical {
            decl.name().to_ascii_lowercase()
        } else {
            decl.name().to_owned()
        };
        if !identifier(decl.name())
            || decl.name().eq_ignore_ascii_case("byte")
            || !names.insert(key)
        {
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
            if !identifier(&field.name) || !field_names.insert(field.name.as_str()) {
                return Err(format!(
                    "E_IR_FIELD: duplicate or invalid field {}.{}",
                    decl.name(),
                    field.name
                ));
            }
        }
        if let TopDecl::Union(union) = decl {
            let mut item_ids = BTreeSet::new();
            let mut item_types = BTreeSet::new();
            if union.items.is_empty()
                || union
                    .items
                    .iter()
                    .any(|item| u32::try_from(item.id).is_err() || !item_ids.insert(item.id))
                || (canonical
                    && union
                        .items
                        .windows(2)
                        .any(|items| items[0].id >= items[1].id))
                || union.items.iter().any(|item| {
                    let item_key = if canonical {
                        item.typ.to_ascii_lowercase()
                    } else {
                        item.typ.clone()
                    };
                    !item_types.insert(item_key)
                })
            {
                return Err(format!(
                    "E_IR_UNION: invalid, duplicate or unordered union items in {}",
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
        ast::Ast::recover(self, canonical)
    }
}

impl TopDecl {
    fn imported_depth_mut(&mut self) -> &mut usize {
        match self {
            Self::Option_(value) => &mut value.imported_depth,
            Self::Union(value) => &mut value.imported_depth,
            Self::Array(value) => &mut value.imported_depth,
            Self::Struct(value) => &mut value.imported_depth,
            Self::FixVec(value) => &mut value.imported_depth,
            Self::DynVec(value) => &mut value.imported_depth,
            Self::Table(value) => &mut value.imported_depth,
        }
    }
}
