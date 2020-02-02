use std::path::PathBuf;

use property::Property;

mod utils;

#[derive(Debug, Default, Property)]
pub(crate) struct Ast {
    namespace: String,
    imports: Vec<ImportStmt>,
    decls: Vec<TopDecl>,
}

#[derive(Debug, Clone, Property)]
pub(crate) struct ImportStmt {
    name: String,
    paths: Vec<String>,
    path_supers: usize,
    imported_base: PathBuf,
    imported_depth: usize,
}

#[derive(Debug)]
pub(crate) enum TopDecl {
    Option_(OptionDecl),
    Union(UnionDecl),
    Array(ArrayDecl),
    Struct(StructDecl),
    Vector(VectorDecl),
    Table(TableDecl),
}

#[derive(Debug, Property)]
pub(crate) struct OptionDecl {
    name: String,
    item: ItemDecl,
    imported_depth: usize,
}

#[derive(Debug, Property)]
pub(crate) struct UnionDecl {
    name: String,
    items: Vec<ItemDecl>,
    imported_depth: usize,
}

#[derive(Debug, Property)]
pub(crate) struct ArrayDecl {
    name: String,
    item: ItemDecl,
    item_count: usize,
    imported_depth: usize,
}

#[derive(Debug, Property)]
pub(crate) struct StructDecl {
    name: String,
    fields: Vec<FieldDecl>,
    imported_depth: usize,
}

#[derive(Debug, Property)]
pub(crate) struct VectorDecl {
    name: String,
    item: ItemDecl,
    imported_depth: usize,
}

#[derive(Debug, Property)]
pub(crate) struct TableDecl {
    name: String,
    fields: Vec<FieldDecl>,
    imported_depth: usize,
}

#[derive(Debug, Property)]
pub(crate) struct ItemDecl {
    typ: String,
}

#[derive(Debug, Property)]
pub(crate) struct FieldDecl {
    name: String,
    typ: String,
}

impl Ast {
    pub(crate) fn add_import(&mut self, stmt: ImportStmt) {
        self.imports.push(stmt);
    }

    pub(crate) fn add_decl(&mut self, decl: impl Into<TopDecl>) {
        self.decls.push(decl.into());
    }
}

impl TopDecl {
    pub(crate) fn name(&self) -> &str {
        match self {
            TopDecl::Option_(inner) => inner.name(),
            TopDecl::Union(inner) => inner.name(),
            TopDecl::Array(inner) => inner.name(),
            TopDecl::Struct(inner) => inner.name(),
            TopDecl::Vector(inner) => inner.name(),
            TopDecl::Table(inner) => inner.name(),
        }
    }
}

macro_rules! impl_into_top_decl_for {
    ($item:ident, $decl:ident) => {
        impl From<$decl> for TopDecl {
            fn from(decl: $decl) -> Self {
                TopDecl::$item(decl)
            }
        }
    };
}

impl_into_top_decl_for!(Option_, OptionDecl);
impl_into_top_decl_for!(Union, UnionDecl);
impl_into_top_decl_for!(Array, ArrayDecl);
impl_into_top_decl_for!(Struct, StructDecl);
impl_into_top_decl_for!(Vector, VectorDecl);
impl_into_top_decl_for!(Table, TableDecl);
