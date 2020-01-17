use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

pub(crate) use super::raw::ImportStmt;
use super::raw::{Ast as RawAst, TopDecl as RawTopDecl};

mod complete;
mod default_content;
mod has_name;

use complete::CompleteRawDecl;
pub(crate) use default_content::DefaultContent;
pub use has_name::HasName;

pub(crate) const ATOM_NAME: &str = "byte";
pub(crate) const ATOM_SIZE: usize = 1;
pub(crate) const ATOM_PRIMITIVE_NAME: &str = "Byte";

#[derive(Debug)]
pub struct Ast {
    pub(crate) namespace: String,
    pub(self) imports: Vec<Rc<ImportStmt>>,
    pub(self) decls: Vec<Rc<TopDecl>>,
}

#[derive(Debug)]
pub enum TopDecl {
    Atom(Atom),
    Option_(Option_),
    Union(Union),
    Array(Array),
    Struct(Struct),
    FixVec(FixVec),
    DynVec(DynVec),
    Table(Table),
}

#[derive(Debug)]
pub struct Atom {
    pub(crate) name: String,
    pub(crate) size: usize,
}

#[derive(Debug)]
pub struct Option_ {
    pub(crate) name: String,
    pub typ: Rc<TopDecl>,
    pub(crate) imported_depth: usize,
}

#[derive(Debug)]
pub struct Union {
    pub(crate) name: String,
    pub(crate) inner: Vec<ItemDecl>,
    pub(crate) imported_depth: usize,
}

#[derive(Debug)]
pub struct Array {
    pub(crate) name: String,
    pub(crate) item_size: usize,
    pub(crate) item_count: usize,
    pub typ: Rc<TopDecl>,
    pub(crate) imported_depth: usize,
}

#[derive(Debug)]
pub struct Struct {
    pub(crate) name: String,
    pub(crate) field_size: Vec<usize>,
    pub inner: Vec<FieldDecl>,
    pub(crate) imported_depth: usize,
}

#[derive(Debug)]
pub struct FixVec {
    pub(crate) name: String,
    pub(crate) item_size: usize,
    pub typ: Rc<TopDecl>,
    pub(crate) imported_depth: usize,
}

#[derive(Debug)]
pub struct DynVec {
    pub(crate) name: String,
    pub typ: Rc<TopDecl>,
    pub(crate) imported_depth: usize,
}

#[derive(Debug)]
pub struct Table {
    pub(crate) name: String,
    pub inner: Vec<FieldDecl>,
    pub(crate) imported_depth: usize,
}

#[derive(Debug)]
pub(crate) struct ItemDecl {
    pub(crate) typ: Rc<TopDecl>,
}

#[derive(Debug)]
pub struct FieldDecl {
    pub name: String,
    pub typ: Rc<TopDecl>,
}

impl Array {
    pub(crate) fn total_size(&self) -> usize {
        self.item_size * self.item_count
    }
}

impl Struct {
    pub(crate) fn total_size(&self) -> usize {
        self.field_size.iter().sum::<usize>()
    }
}

impl TopDecl {
    fn atom() -> Self {
        let atom = Atom {
            name: ATOM_NAME.to_owned(),
            size: ATOM_SIZE,
        };
        TopDecl::Atom(atom)
    }

    pub(crate) fn is_atom(&self) -> bool {
        match self {
            TopDecl::Atom(_) => true,
            _ => false,
        }
    }

    fn imported_depth(&self) -> usize {
        match self {
            TopDecl::Atom(_) => unreachable!(),
            TopDecl::Option_(ref typ) => typ.imported_depth,
            TopDecl::Union(ref typ) => typ.imported_depth,
            TopDecl::Array(ref typ) => typ.imported_depth,
            TopDecl::Struct(ref typ) => typ.imported_depth,
            TopDecl::FixVec(ref typ) => typ.imported_depth,
            TopDecl::DynVec(ref typ) => typ.imported_depth,
            TopDecl::Table(ref typ) => typ.imported_depth,
        }
    }

    fn total_size(&self) -> Option<usize> {
        match self {
            TopDecl::Atom(ref typ) => Some(typ.size),
            TopDecl::Option_(_) => None,
            TopDecl::Union(_) => None,
            TopDecl::Array(ref typ) => Some(typ.total_size()),
            TopDecl::Struct(ref typ) => Some(typ.total_size()),
            TopDecl::FixVec(_) => None,
            TopDecl::DynVec(_) => None,
            TopDecl::Table(_) => None,
        }
    }

    fn complete(raw: &RawTopDecl, deps: &HashMap<&str, Rc<Self>>) -> Option<Self> {
        match raw {
            RawTopDecl::Option_(raw_decl) => raw_decl.complete(deps),
            RawTopDecl::Union(raw_decl) => raw_decl.complete(deps),
            RawTopDecl::Array(raw_decl) => raw_decl.complete(deps),
            RawTopDecl::Struct(raw_decl) => raw_decl.complete(deps),
            RawTopDecl::Vector(raw_decl) => raw_decl.complete(deps),
            RawTopDecl::Table(raw_decl) => raw_decl.complete(deps),
        }
    }
}

impl Ast {
    pub(crate) fn new(raw: RawAst) -> Self {
        let mut decls_idx = HashMap::new();
        let mut decls_keys = HashSet::new();
        for decl in &raw.decls[..] {
            let name = decl.name();
            if name == ATOM_NAME || name == ATOM_PRIMITIVE_NAME {
                panic!("the name `{}` is reserved", name);
            }
            if decls_idx.insert(name, decl).is_some() || !decls_keys.insert(name) {
                panic!("the name `{}` is used more than once", name);
            };
        }
        let mut decls_result = HashMap::new();
        decls_result.insert(ATOM_NAME, Rc::new(TopDecl::atom()));
        loop {
            if decls_keys.is_empty() {
                break;
            }
            let incompleted = decls_keys.len();
            decls_keys.retain(|&name| {
                let decl_raw = decls_idx.get(name).unwrap();
                if let Some(decl) = TopDecl::complete(decl_raw, &decls_result) {
                    decls_result.insert(name, Rc::new(decl));
                    false
                } else {
                    true
                }
            });
            if decls_keys.len() == incompleted {
                panic!(
                    "there are {} types which are unable to be completed: {:?}",
                    incompleted, decls_keys
                );
            }
        }
        let mut decls = Vec::with_capacity(raw.decls.len());
        for decl in &raw.decls[..] {
            let result = decls_result.get(decl.name()).unwrap();
            decls.push(Rc::clone(result));
        }
        Self {
            namespace: raw.namespace,
            imports: raw.imports,
            decls,
        }
    }

    pub(crate) fn major_decls(&self) -> Vec<Rc<TopDecl>> {
        self.decls
            .iter()
            .filter(|x| x.imported_depth() == 0)
            .map(Rc::clone)
            .collect()
    }

    pub fn decls(&self) -> Vec<Rc<TopDecl>> {
        self.decls.clone()
    }

    pub(crate) fn major_imports(&self) -> Vec<Rc<ImportStmt>> {
        self.imports
            .iter()
            .filter(|x| x.imported_depth == 0)
            .map(Rc::clone)
            .collect()
    }
}
