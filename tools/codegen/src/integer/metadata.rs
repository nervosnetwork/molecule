use super::{ByteOrder, IntegerSpec};
use crate::{
    ast::{Ast, HasName, TopDecl},
    schema::{LoadedGraph, SchemaGraph},
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LogicalInteger {
    pub schema: String,
    pub name: String,
    pub bits: u16,
    pub signed: bool,
    pub byte_order: ByteOrder,
}

impl LogicalInteger {
    pub fn spec(&self) -> IntegerSpec {
        IntegerSpec {
            bits: self.bits,
            signed: self.signed,
            byte_order: self.byte_order,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetadataFile {
    metadata_version: u32,
    schema: PathBuf,
    schema_sha256: String,
    integers: Vec<Annotation>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Annotation {
    #[serde(rename = "type")]
    name: String,
    bits: u16,
    signed: bool,
    byte_order: ByteOrder,
}

impl MetadataFile {
    /// Read explicit metadata, rejecting unknown and duplicate object fields.
    pub fn load(path: &Path, sources: &LoadedGraph) -> Result<Vec<LogicalInteger>, String> {
        let bytes =
            fs::read(path).map_err(|error| format!("E_META_READ: {}: {error}", path.display()))?;
        let metadata: Self = serde_json::from_slice(&bytes)
            .map_err(|error| format!("E_META_JSON: {}: {error}", path.display()))?;
        if metadata.metadata_version != 1 {
            return Err(format!(
                "E_META_VERSION: {}: expected 1, found {}",
                path.display(),
                metadata.metadata_version
            ));
        }
        let base = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        #[cfg(not(windows))]
        let schema = base.join(&metadata.schema);
        #[cfg(windows)]
        let schema = if metadata.schema.is_absolute() {
            metadata.schema.clone()
        } else {
            let canonical_base = base
                .canonicalize()
                .map_err(|error| format!("E_META_SCHEMA: {}: {error}", base.display()))?;
            crate::utils::join_relative(&canonical_base, &metadata.schema)
        };
        let module = sources.module_at(&schema)?;
        if !metadata.schema_sha256.eq_ignore_ascii_case(&module.sha256) {
            return Err(format!(
                "E_META_SCHEMA: {}: source pin for {} is {}, actual {}",
                path.display(),
                schema.display(),
                metadata.schema_sha256,
                module.sha256
            ));
        }
        metadata
            .integers
            .into_iter()
            .enumerate()
            .map(|(index, entry)| {
                if !module
                    .declarations
                    .iter()
                    .any(|decl| decl.name == entry.name)
                {
                    return Err(format!(
                        "E_INT_TARGET: {}: integers[{index}]: {} is not declared by {}",
                        path.display(),
                        entry.name,
                        module.id
                    ));
                }
                Ok(LogicalInteger {
                    schema: module.id.clone(),
                    name: entry.name,
                    bits: entry.bits,
                    signed: entry.signed,
                    byte_order: entry.byte_order,
                })
            })
            .collect()
    }
}

impl Ast {
    pub(crate) fn integer_owner(&self, name: &str) -> Result<&str, String> {
        self.source_graph()
            .and_then(|graph| graph.owner(name))
            .map(|module| module.id.as_str())
            .ok_or_else(|| format!("E_IR_SOURCE: no owner for {name}"))
    }

    /// Compare the public scalar names, defining schemas and numeric interpretation.
    pub fn check_integer_contract(&self, published: &Ast) -> Result<(), String> {
        if self.namespace() != published.namespace() {
            return Err(format!(
                "expected schema {}, found {}",
                published.namespace(),
                self.namespace()
            ));
        }
        for (name, expected) in published.integers() {
            let actual = self
                .integer(name)
                .ok_or_else(|| format!("missing integer {name}"))?;
            if !expected.is_equivalent_to(*actual) {
                return Err(format!(
                    "changed integer {name}: expected {expected:?}, found {actual:?}"
                ));
            }
            let owner = self.integer_owner(name)?;
            let previous_owner = published.integer_owner(name)?;
            if owner != previous_owner {
                return Err(format!(
                    "changed owner of {name}: expected {previous_owner}, found {owner}"
                ));
            }
        }
        if let Some(name) = self
            .integers()
            .keys()
            .find(|name| published.integer(name).is_none())
        {
            return Err(format!("added integer {name}"));
        }
        Ok(())
    }
}

pub(crate) fn bind(
    ast: &mut Ast,
    graph: SchemaGraph,
    declarations: Vec<LogicalInteger>,
) -> Result<(), String> {
    let mut integers = ast.integers().clone();
    for decl in declarations {
        let owner = graph
            .owner(&decl.name)
            .ok_or_else(|| format!("E_INT_TARGET: {} has no source owner", decl.name))?;
        if owner.id != decl.schema {
            return Err(format!(
                "E_INT_TARGET: {} belongs to {}, not {}",
                decl.name, owner.id, decl.schema
            ));
        }
        let location = owner
            .declarations
            .iter()
            .find(|item| item.name == decl.name)
            .map(|item| format!("{}:{}:{}", owner.id, item.line, item.column))
            .unwrap_or_else(|| owner.id.clone());
        let slot = match integers.entry(decl.name.clone()) {
            std::collections::btree_map::Entry::Vacant(slot) => slot,
            std::collections::btree_map::Entry::Occupied(_) => {
                return Err(format!("E_INT_DUPLICATE: {location}: {}", decl.name));
            }
        };
        let spec = decl.spec();
        spec.validate()
            .map_err(|error| format!("{error}: {location}: {}", decl.name))?;
        let typ = ast
            .decls()
            .iter()
            .find(|typ| typ.name() == decl.name)
            .ok_or_else(|| {
                format!(
                    "E_INT_TARGET: {location}: missing declaration {}",
                    decl.name
                )
            })?;
        match typ.as_ref() {
            TopDecl::Array(array) if array.item().typ().is_byte() => {
                if array.item_count() != spec.byte_width() {
                    return Err(format!(
                        "E_INT_WIDTH: {location}: {} is {} bytes; metadata requires {}",
                        decl.name,
                        array.item_count(),
                        spec.byte_width()
                    ));
                }
            }
            _ => {
                return Err(format!(
                    "E_INT_TARGET: {location}: {} must be a direct byte array",
                    decl.name
                ))
            }
        }
        slot.insert(spec);
    }
    ast.set_extensions(graph, integers);
    Ok(())
}
