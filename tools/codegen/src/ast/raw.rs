#[derive(Debug)]
pub(crate) struct FieldDecl {
    pub(crate) name: String,
    pub(crate) typ: String,
}

#[derive(Debug)]
pub(crate) struct OptionDecl {
    pub(crate) name: String,
    pub(crate) typ: String,
}

#[derive(Debug)]
pub(crate) struct ArrayDecl {
    pub(crate) name: String,
    pub(crate) typ: String,
    pub(crate) length: usize,
}

#[derive(Debug)]
pub(crate) struct StructDecl {
    pub(crate) name: String,
    pub(crate) inner: Vec<FieldDecl>,
}

#[derive(Debug)]
pub(crate) struct VectorDecl {
    pub(crate) name: String,
    pub(crate) typ: String,
}

#[derive(Debug)]
pub(crate) struct TableDecl {
    pub(crate) name: String,
    pub(crate) inner: Vec<FieldDecl>,
}

#[derive(Debug)]
pub(crate) enum TopDecl {
    Option_(OptionDecl),
    Array(ArrayDecl),
    Struct(StructDecl),
    Vector(VectorDecl),
    Table(TableDecl),
}

pub(crate) trait IsTopDecl: Into<TopDecl> {
    fn name(&self) -> &str;
}

#[derive(Debug)]
pub(crate) struct ImportStmt {
    pub(crate) name: String,
    pub(crate) path: Vec<String>,
    pub(crate) depth: usize,
}

#[derive(Debug, Default)]
pub(crate) struct Ast {
    pub(crate) imports: Vec<ImportStmt>,
    pub(crate) decls: Vec<TopDecl>,
}

macro_rules! impl_top_decl_for {
    ($item:ident, $decl:ident) => {
        impl IsTopDecl for $decl {
            fn name(&self) -> &str {
                &self.name
            }
        }

        impl From<$decl> for TopDecl {
            fn from(decl: $decl) -> Self {
                TopDecl::$item(decl)
            }
        }
    };
}

impl_top_decl_for!(Option_, OptionDecl);
impl_top_decl_for!(Array, ArrayDecl);
impl_top_decl_for!(Struct, StructDecl);
impl_top_decl_for!(Vector, VectorDecl);
impl_top_decl_for!(Table, TableDecl);

impl TopDecl {
    pub(crate) fn name(&self) -> &str {
        match self {
            TopDecl::Option_(inner) => inner.name(),
            TopDecl::Array(inner) => inner.name(),
            TopDecl::Struct(inner) => inner.name(),
            TopDecl::Vector(inner) => inner.name(),
            TopDecl::Table(inner) => inner.name(),
        }
    }
}

impl Ast {
    pub(crate) fn add_import(&mut self, stmt: ImportStmt) {
        self.imports.push(stmt);
    }

    pub(crate) fn add_decl(&mut self, decl: impl IsTopDecl) {
        self.decls.push(decl.into());
    }
}
