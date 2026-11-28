#[cfg(windows)]
use std::path::PathBuf;
use std::path::{Component, Path};

use pest::error::Error as PestError;

use crate::{ast::raw as ast, parser};

pub(crate) trait PairsUtils {
    fn next_string(&mut self) -> String;
    fn next_usize(&mut self) -> usize;
    fn next_item(&mut self) -> ast::ItemDecl;
    #[allow(dead_code)]
    fn next_items(&mut self) -> Vec<ast::ItemDecl>;
    fn next_custom_union_items(&mut self) -> Vec<ast::CustomUnionItemDecl>;
    fn next_fields(&mut self) -> Vec<ast::FieldDecl>;
    fn next_import<P: AsRef<Path>>(
        &mut self,
        imported_base: &P,
        imported_depth: usize,
    ) -> ast::ImportStmt;
    fn next_should_be_none(self);
}

pub(crate) trait ParserUtils {
    fn preprocess<P: AsRef<Path>>(path: &P) -> Result<ast::Ast, Box<PestError<parser::Rule>>>;
}

pub(crate) fn valid_namespace(name: &str) -> bool {
    if name.is_empty()
        || name.contains(['/', '\\', ':', '\0'])
        || name.bytes().any(|b| b.is_ascii_control())
        || name == "."
        || name == ".."
    {
        return false;
    }
    let mut components = Path::new(name).components();
    matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none()
}

/// Join a relative path onto an already-canonicalized base directory, resolving
/// symlinks before `..` so both symlinked segments and Windows `\\?\` paths stay valid.
#[cfg(windows)]
pub(crate) fn join_relative(base_dir: &Path, rel: &Path) -> PathBuf {
    let mut out = base_dir.to_path_buf();
    for part in rel.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                if let Ok(resolved) = out.canonicalize() {
                    out = resolved;
                }
                out.pop();
            }
            Component::Normal(seg) => out.push(seg),
            Component::RootDir => out.push(part.as_os_str()),
            Component::Prefix(_) => return rel.to_path_buf(),
        }
    }
    out
}
