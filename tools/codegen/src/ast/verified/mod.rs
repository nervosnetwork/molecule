use std::{
    collections::{BTreeMap, HashMap},
    rc::Rc,
};

use property::Property;

mod complete;
mod default_content;
mod has_name;

#[cfg(feature = "compiler-plugin")]
mod recover;

pub use default_content::DefaultContent;
pub use has_name::HasName;

use crate::ast::SyntaxVersion;

type Deps<'a> = HashMap<&'a str, Rc<super::TopDecl>>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ModulePath {
    pub(crate) name: String,
    pub(crate) paths: Vec<String>,
    pub(crate) path_supers: usize,
}

impl ModulePath {
    pub(crate) fn root() -> Self {
        Self {
            name: String::new(),
            paths: Vec::new(),
            path_supers: 0,
        }
    }

    pub(crate) fn new(name: String, paths: Vec<String>, path_supers: usize) -> Self {
        Self {
            name,
            paths,
            path_supers,
        }
    }

    /// Parse a Molecule `import` path string (e.g. `../common/defs`).
    #[cfg(feature = "schema-extensions")]
    pub(crate) fn from_import_str(raw: &str) -> Option<Self> {
        let mut path_supers = 0;
        let mut rest = raw;
        while let Some(stripped) = rest.strip_prefix("../") {
            path_supers += 1;
            rest = stripped;
        }
        let mut parts: Vec<String> = rest.split('/').map(str::to_owned).collect();
        let name = parts.pop()?;
        if name.is_empty() || parts.iter().any(String::is_empty) {
            return None;
        }
        Some(Self {
            name,
            paths: parts,
            path_supers,
        })
    }

    /// Resolve an import edge written inside the module at `self` into a path relative to root.
    pub(crate) fn resolve_edge(&self, edge: &Self) -> Self {
        let mut path_supers = self.path_supers;
        let mut paths = self.paths.clone();
        for _ in 0..edge.path_supers {
            if paths.pop().is_none() {
                path_supers += 1;
            }
        }
        paths.extend(edge.paths.iter().cloned());
        Self {
            name: edge.name.clone(),
            paths,
            path_supers,
        }
    }
}

#[derive(Debug, Property)]
#[property(get(public))]
pub struct Ast {
    syntax_version: SyntaxVersion,
    namespace: String,
    imports: Vec<ImportStmt>,
    decls: Vec<Rc<TopDecl>>,
    integers: BTreeMap<String, crate::integer::IntegerSpec>,
    #[cfg(feature = "schema-extensions")]
    source_graph: Option<crate::schema::SchemaGraph>,
    #[property(skip)]
    decl_origins: Option<BTreeMap<String, (ModulePath, bool)>>,
}

#[derive(Debug, Clone, Property)]
#[property(get(public))]
pub struct ImportStmt {
    name: String,
    paths: Vec<String>,
    path_supers: usize,
}

#[derive(Debug)]
pub enum TopDecl {
    Primitive(Primitive),
    Option_(Option_),
    Union(Union),
    Array(Array),
    Struct(Struct),
    FixVec(FixVec),
    DynVec(DynVec),
    Table(Table),
}

#[derive(Debug, Property)]
#[property(get(public))]
pub struct Primitive {
    name: String,
    size: usize,
}

#[derive(Debug, Property)]
#[property(get(public))]
pub struct Option_ {
    name: String,
    item: ItemDecl,
    imported_depth: usize,
}

#[derive(Debug, Property)]
#[property(get(public))]
pub struct Union {
    name: String,
    items: Vec<UnionItemDecl>,
    imported_depth: usize,
}

#[derive(Debug, Property)]
#[property(get(public))]
pub struct Array {
    name: String,
    item: ItemDecl,
    item_count: usize,
    imported_depth: usize,
    item_size: usize,
}

#[derive(Debug, Property)]
#[property(get(public))]
pub struct Struct {
    name: String,
    fields: Vec<FieldDecl>,
    imported_depth: usize,
    field_sizes: Vec<usize>,
}

#[derive(Debug, Property)]
#[property(get(public))]
pub struct FixVec {
    name: String,
    item: ItemDecl,
    imported_depth: usize,
    item_size: usize,
}

#[derive(Debug, Property)]
#[property(get(public))]
pub struct DynVec {
    name: String,
    item: ItemDecl,
    imported_depth: usize,
}

#[derive(Debug, Property)]
#[property(get(public))]
pub struct Table {
    name: String,
    fields: Vec<FieldDecl>,
    imported_depth: usize,
}

#[derive(Debug, Property)]
#[property(get(public))]
pub struct ItemDecl {
    typ: Rc<TopDecl>,
}

#[derive(Debug, Property)]
#[property(get(public))]
pub struct UnionItemDecl {
    typ: Rc<TopDecl>,
    id: usize,
}

#[derive(Debug, Property)]
#[property(get(public))]
pub struct FieldDecl {
    name: String,
    typ: Rc<TopDecl>,
}

impl Ast {
    pub fn integer(&self, name: &str) -> Option<&crate::integer::IntegerSpec> {
        self.integers.get(name)
    }

    #[cfg(feature = "schema-extensions")]
    pub(crate) fn set_extensions(
        &mut self,
        graph: crate::schema::SchemaGraph,
        integers: BTreeMap<String, crate::integer::IntegerSpec>,
    ) {
        self.source_graph = Some(graph);
        self.integers = integers;
    }

    pub fn major_decls(&self) -> Vec<Rc<TopDecl>> {
        self.decls
            .iter()
            .filter(|d| d.imported_depth() == 0)
            .map(Rc::clone)
            .collect()
    }

    pub(crate) fn has_decl_origins(&self) -> bool {
        self.decl_origins.is_some()
    }

    pub(crate) fn decl_origin(&self, name: &str) -> Option<(&ModulePath, bool)> {
        self.decl_origins
            .as_ref()
            .and_then(|map| map.get(name))
            .map(|(path, direct)| (path, *direct))
    }

    #[cfg(feature = "compiler-plugin")]
    pub(crate) fn set_decl_origins(&mut self, origins: BTreeMap<String, (ModulePath, bool)>) {
        self.decl_origins = Some(origins);
    }
}

impl TopDecl {
    fn new_primitive(name: &str) -> Option<Self> {
        match name {
            "byte" => Some(Primitive {
                name: name.to_owned(),
                size: 1,
            }),
            _ => None,
        }
        .map(Self::Primitive)
    }

    pub fn is_byte(&self) -> bool {
        if let Self::Primitive(inner) = self {
            inner.size == 1
        } else {
            false
        }
    }

    fn imported_depth(&self) -> usize {
        match self {
            Self::Primitive(_) => usize::MAX,
            Self::Option_(inner) => inner.imported_depth,
            Self::Union(inner) => inner.imported_depth,
            Self::Array(inner) => inner.imported_depth,
            Self::Struct(inner) => inner.imported_depth,
            Self::FixVec(inner) => inner.imported_depth,
            Self::DynVec(inner) => inner.imported_depth,
            Self::Table(inner) => inner.imported_depth,
        }
    }

    fn total_size(&self) -> Option<usize> {
        match self {
            Self::Primitive(inner) => Some(inner.size),
            Self::Option_(_) => None,
            Self::Union(_) => None,
            Self::Array(inner) => Some(inner.total_size()),
            Self::Struct(inner) => Some(inner.total_size()),
            Self::FixVec(_) => None,
            Self::DynVec(_) => None,
            Self::Table(_) => None,
        }
    }
}

impl Array {
    pub fn total_size(&self) -> usize {
        self.item_size() * self.item_count()
    }
}

impl Struct {
    pub fn total_size(&self) -> usize {
        self.field_sizes().iter().sum::<usize>()
    }
}

macro_rules! impl_into_top_decl_for {
    ($type:ident) => {
        impl From<$type> for TopDecl {
            fn from(typ: $type) -> Self {
                TopDecl::$type(typ)
            }
        }
    };
}

impl_into_top_decl_for!(Primitive);
impl_into_top_decl_for!(Option_);
impl_into_top_decl_for!(Union);
impl_into_top_decl_for!(Array);
impl_into_top_decl_for!(Struct);
impl_into_top_decl_for!(FixVec);
impl_into_top_decl_for!(DynVec);
impl_into_top_decl_for!(Table);

impl ItemDecl {
    fn new(top_decl: &Rc<TopDecl>) -> Self {
        Self {
            typ: Rc::clone(top_decl),
        }
    }
}

impl UnionItemDecl {
    fn new(top_decl: &Rc<TopDecl>, customize_id: usize) -> Self {
        Self {
            typ: Rc::clone(top_decl),
            id: customize_id,
        }
    }
}

impl FieldDecl {
    fn new(name: &str, top_decl: &Rc<TopDecl>) -> Self {
        Self {
            name: name.to_owned(),
            typ: Rc::clone(top_decl),
        }
    }
}
