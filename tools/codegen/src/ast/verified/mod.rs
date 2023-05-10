use std::{collections::HashMap, rc::Rc};

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

pub fn must_get_primitive_types() -> Vec<Primitive> {
    use crate::utils::ParserUtils;
    crate::parser::Parser::preprocess(&std::path::PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/primitive_types.mol"
    )))
    .unwrap()
    .decls()
    .iter()
    .map(|decl| match decl {
        crate::ast::raw::TopDecl::Array(array) => Primitive {
            name: array.name().to_string(),
            size: array.item_count(),
        },
        _ => panic!("primitive types is not array"),
    })
    .collect::<Vec<Primitive>>()
}

#[derive(Debug, Property)]
#[property(get(public))]
pub struct Ast {
    syntax_version: SyntaxVersion,
    namespace: String,
    imports: Vec<ImportStmt>,
    decls: Vec<Rc<TopDecl>>,
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

#[derive(Debug, Property, Clone)]
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
    pub fn major_decls(&self) -> Vec<Rc<TopDecl>> {
        self.decls
            .iter()
            .filter(|d| d.imported_depth() == 0)
            .map(Rc::clone)
            .collect()
    }
}

impl TopDecl {
    fn new_primitive(name: &str, with_primitive_ext: bool) -> Option<Self> {
        if name.eq("byte") {
            return Some(Primitive {
                name: name.to_owned(),
                size: 1,
            })
            .map(Self::Primitive);
        }
        if !with_primitive_ext {
            return None;
        }
        must_get_primitive_types()
            .iter()
            .find(|primitive_decl| primitive_decl.name().eq(name))
            .map(|v| Self::Primitive(v.clone()))
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
            Self::Primitive(_) => usize::max_value(),
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
