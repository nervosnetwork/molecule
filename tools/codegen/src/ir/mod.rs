mod format;
mod from_ast;

use serde::{Deserialize, Serialize};

use property::Property;

pub use format::Format;
pub(crate) use from_ast::ToIntermediate;

/// Intermediate file.
#[derive(Debug, Property, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Ir {
    namespace: String,
    imports: Vec<ImportStmt>,
    #[serde(rename = "declarations")]
    decls: Vec<TopDecl>,
}

#[derive(Debug, Clone, Property, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ImportStmt {
    name: String,
    paths: Vec<String>,
    path_supers: usize,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, tag = "type", rename_all = "lowercase")]
pub(crate) enum TopDecl {
    #[serde(rename = "option")]
    Option_(Option_),
    Union(Union),
    Array(Array),
    Struct(Struct),
    FixVec(FixVec),
    DynVec(DynVec),
    Table(Table),
}

#[derive(Debug, Property, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Option_ {
    name: String,
    item: ItemDecl,
    #[serde(default = "zero", skip_serializing_if = "is_zero")]
    imported_depth: usize,
}

#[derive(Debug, Property, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Union {
    name: String,
    items: Vec<ItemDecl>,
    #[serde(default = "zero", skip_serializing_if = "is_zero")]
    imported_depth: usize,
}

#[derive(Debug, Property, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Array {
    name: String,
    item: ItemDecl,
    item_count: usize,
    #[serde(default = "zero", skip_serializing_if = "is_zero")]
    imported_depth: usize,
}

#[derive(Debug, Property, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Struct {
    name: String,
    fields: Vec<FieldDecl>,
    #[serde(default = "zero", skip_serializing_if = "is_zero")]
    imported_depth: usize,
}

#[derive(Debug, Property, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FixVec {
    name: String,
    item: ItemDecl,
    #[serde(default = "zero", skip_serializing_if = "is_zero")]
    imported_depth: usize,
}

#[derive(Debug, Property, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DynVec {
    name: String,
    item: ItemDecl,
    #[serde(default = "zero", skip_serializing_if = "is_zero")]
    imported_depth: usize,
}

#[derive(Debug, Property, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Table {
    name: String,
    fields: Vec<FieldDecl>,
    #[serde(default = "zero", skip_serializing_if = "is_zero")]
    imported_depth: usize,
}

#[derive(Debug, Property, Deserialize, Serialize)]
#[serde(deny_unknown_fields, transparent)]
pub(crate) struct ItemDecl {
    typ: String,
}

#[derive(Debug, Property, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FieldDecl {
    name: String,
    #[serde(rename = "type")]
    typ: String,
}

const fn zero() -> usize {
    0
}

#[allow(clippy::trivially_copy_pass_by_ref)]
const fn is_zero(value: &usize) -> bool {
    *value == 0
}

impl TopDecl {
    pub(crate) fn name(&self) -> &str {
        match self {
            Self::Option_(inner) => inner.name(),
            Self::Union(inner) => inner.name(),
            Self::Array(inner) => inner.name(),
            Self::Struct(inner) => inner.name(),
            Self::FixVec(inner) => inner.name(),
            Self::DynVec(inner) => inner.name(),
            Self::Table(inner) => inner.name(),
        }
    }
}
