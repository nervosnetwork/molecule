use std::collections::BTreeMap;

use property::Property;
use serde::Deserialize;

use crate::bytes::Bytes;

pub(crate) type All = Vec<Any>;

#[derive(Deserialize)]
#[serde(deny_unknown_fields, untagged)]
pub(crate) enum Any {
    Option_(Option_),
    Union(Union),
    Array(Array),
    StructOrTable(StructOrTable),
    Vector(Vector),
}

#[derive(Deserialize, Property)]
#[serde(deny_unknown_fields)]
pub(crate) struct Option_ {
    name: String,
    item: Option<Bytes>,
    expected: Bytes,
}

#[derive(Deserialize, Property)]
#[serde(deny_unknown_fields)]
pub(crate) struct Union {
    name: String,
    item: Option<Item>,
    expected: Bytes,
}

#[derive(Deserialize, Property)]
#[serde(deny_unknown_fields)]
pub(crate) struct Array {
    name: String,
    #[serde(default)]
    data: BTreeMap<usize, Bytes>,
    expected: Bytes,
}

#[derive(Deserialize, Property)]
#[serde(deny_unknown_fields)]
pub(crate) struct StructOrTable {
    name: String,
    #[serde(default)]
    data: BTreeMap<String, Bytes>,
    expected: Bytes,
}

#[derive(Deserialize, Property)]
#[serde(deny_unknown_fields)]
pub(crate) struct Vector {
    name: String,
    #[serde(default)]
    data: Vec<Bytes>,
    expected: Bytes,
}

#[derive(Deserialize, Property)]
#[serde(deny_unknown_fields)]
pub(crate) struct Item {
    #[serde(rename = "type")]
    typ: String,
    data: Bytes,
}
