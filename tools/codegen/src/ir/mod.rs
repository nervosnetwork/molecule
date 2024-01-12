mod format;
mod from_ast;

use serde::{Deserialize, Deserializer, Serialize};

use property::Property;

pub use format::Format;
pub(crate) use from_ast::ToIntermediate;

pub use crate::ast::SyntaxVersion;

/// Intermediate file.
#[derive(Debug, Property, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Ir {
    #[serde(default)]
    pub syntax_version: SyntaxVersion,
    pub namespace: String,
    pub imports: Vec<ImportStmt>,
    #[serde(rename = "declarations")]
    pub decls: Vec<TopDecl>,
}

#[derive(Debug, Clone, Property, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ImportStmt {
    pub name: String,
    pub paths: Vec<String>,
    pub path_supers: usize,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, tag = "type", rename_all = "lowercase")]
pub enum TopDecl {
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
pub struct Option_ {
    pub name: String,
    pub item: ItemDecl,
    #[serde(default = "zero", skip_serializing_if = "is_zero")]
    pub imported_depth: usize,
}

#[derive(Debug, Property, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Union {
    pub name: String,

    #[serde(deserialize_with = "deserialize_union_items")]
    pub items: Vec<UnionItemDecl>,
    #[serde(default = "zero", skip_serializing_if = "is_zero")]
    pub imported_depth: usize,
}

#[derive(Debug, Property, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Array {
    pub name: String,
    pub item: ItemDecl,
    pub item_count: usize,
    #[serde(default = "zero", skip_serializing_if = "is_zero")]
    pub imported_depth: usize,
}

#[derive(Debug, Property, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<FieldDecl>,
    #[serde(default = "zero", skip_serializing_if = "is_zero")]
    pub imported_depth: usize,
}

#[derive(Debug, Property, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FixVec {
    pub name: String,
    pub item: ItemDecl,
    #[serde(default = "zero", skip_serializing_if = "is_zero")]
    pub imported_depth: usize,
}

#[derive(Debug, Property, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DynVec {
    pub name: String,
    pub item: ItemDecl,
    #[serde(default = "zero", skip_serializing_if = "is_zero")]
    pub imported_depth: usize,
}

#[derive(Debug, Property, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Table {
    pub name: String,
    pub fields: Vec<FieldDecl>,
    #[serde(default = "zero", skip_serializing_if = "is_zero")]
    pub imported_depth: usize,
}

#[derive(Debug, Property, Deserialize, Serialize)]
#[serde(deny_unknown_fields, transparent)]
pub struct ItemDecl {
    pub typ: String,
}

#[derive(Debug, Property, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnionItemDecl {
    pub typ: String,
    pub id: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UnionItemsForCompatibility {
    ItemsForCompatibility(Vec<ItemDecl>),
    Items(Vec<UnionItemDecl>),
}

fn deserialize_union_items<'de, D>(d: D) -> Result<Vec<UnionItemDecl>, D::Error>
where
    D: Deserializer<'de>,
{
    let union_com = UnionItemsForCompatibility::deserialize(d)?;
    Ok(match union_com {
        UnionItemsForCompatibility::ItemsForCompatibility(items) => items
            .iter()
            .enumerate()
            .map(|(id, item)| UnionItemDecl {
                typ: item.typ.clone(),
                id,
            })
            .collect(),
        UnionItemsForCompatibility::Items(items) => items,
    })
}

#[derive(Debug, Property, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FieldDecl {
    pub name: String,
    #[serde(rename = "type")]
    pub typ: String,
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
