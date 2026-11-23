use std::collections::HashSet;
#[cfg(any(test, not(feature = "schema-extensions")))]
use std::{fs, path::Path};

#[cfg(any(test, not(feature = "schema-extensions")))]
use pest::Parser as _;
use pest::{error::Error as PestError, iterators::Pairs};
#[cfg(any(test, not(feature = "schema-extensions")))]
use same_file::is_same_file;

use crate::{
    ast::raw as ast,
    ast::raw::CustomUnionItemDecl,
    ast::raw::SyntaxVersion,
    parser::{self, ParseError},
    utils::{self, PairsUtils as _},
};

fn invalid(span: pest::Span<'_>, message: impl Into<String>) -> ParseError {
    Box::new(PestError::new_from_span(
        pest::error::ErrorVariant::CustomError {
            message: message.into(),
        },
        span,
    ))
}

impl utils::PairsUtils for Pairs<'_, parser::Rule> {
    fn next_string(&mut self) -> String {
        self.next().unwrap().as_str().to_owned()
    }

    fn next_usize(&mut self) -> Result<usize, ParseError> {
        let number = self.next().unwrap();
        number
            .as_str()
            .parse()
            .map_err(|_| invalid(number.as_span(), "E_SCHEMA_NUMBER: value exceeds usize"))
    }

    fn next_item(&mut self) -> ast::ItemDecl {
        ast::ItemDecl {
            typ: self.next_string(),
        }
    }

    fn next_items(&mut self) -> Vec<ast::ItemDecl> {
        let mut ret = Vec::new();
        for item in self {
            if item.as_rule() != parser::Rule::item_decl {
                unreachable!()
            }
            let mut pair = item.into_inner();
            let node = ast::ItemDecl {
                typ: pair.next_string(),
            };
            pair.next_should_be_none();
            ret.push(node);
        }
        ret
    }

    fn next_custom_union_items(&mut self) -> Result<Vec<CustomUnionItemDecl>, ParseError> {
        let mut next = Some(0usize);
        let mut items = Vec::new();
        let mut ids = HashSet::new();
        for item in self {
            let span = item.as_span();
            let mut parts = item.into_inner();
            let typ = parts.next_string();
            let id = if parts.peek().is_some() {
                parts.next_usize()?
            } else {
                next.ok_or_else(|| invalid(span, "E_SCHEMA_UNION: implicit ID exceeds usize"))?
            };
            if !ids.insert(id) {
                return Err(invalid(
                    span,
                    format!("E_SCHEMA_UNION: duplicate union ID {id}"),
                ));
            }
            next = id.checked_add(1);
            items.push(CustomUnionItemDecl { typ, id });
        }
        items.sort_by_key(|item| item.id);
        Ok(items)
    }

    fn next_fields(&mut self) -> Vec<ast::FieldDecl> {
        let mut ret = Vec::new();
        for field in self {
            if field.as_rule() != parser::Rule::field_decl {
                unreachable!()
            }
            let mut pair = field.into_inner();
            let node = ast::FieldDecl {
                name: pair.next_string(),
                typ: pair.next_string(),
            };
            pair.next_should_be_none();
            ret.push(node);
        }
        ret
    }

    fn next_import(&mut self, imported_depth: usize) -> ast::ImportStmt {
        let mut paths = Vec::new();
        let mut path_supers = 0;
        if let Some(inner) = self.next() {
            if inner.as_rule() != parser::Rule::path {
                unreachable!()
            }
            let mut pair = inner.into_inner();
            loop {
                if let Some(inner) = pair.peek() {
                    if inner.as_rule() == parser::Rule::path_super {
                        pair.next();
                        path_supers += 1;
                        continue;
                    }
                }
                break;
            }
            for inner in pair {
                paths.push(inner.as_str().to_owned())
            }
        }
        ast::ImportStmt {
            name: paths.pop().unwrap(),
            paths,
            path_supers,
            imported_depth,
        }
    }

    fn next_should_be_none(mut self) {
        if self.next().is_some() {
            unreachable!()
        }
    }
}

#[cfg(any(test, not(feature = "schema-extensions")))]
impl parser::Parser {
    pub(crate) fn preprocess(path: &Path) -> Result<ast::Ast, String> {
        use std::collections::VecDeque;
        fn canonical_parent(path: &Path) -> Result<std::path::PathBuf, String> {
            let parent = match path.parent() {
                Some(p) if !p.as_os_str().is_empty() => p,
                _ => Path::new("."),
            };
            parent
                .canonicalize()
                .map_err(|error| format!("E_SCHEMA_SOURCE: {}: {error}", path.display()))
        }
        fn import_rel_path(stmt: &ast::ImportStmt) -> std::path::PathBuf {
            let mut rel = std::path::PathBuf::new();
            for _ in 0..stmt.path_supers() {
                rel.push("..");
            }
            rel.extend(stmt.paths());
            rel.push(stmt.name());
            rel.set_extension("mol");
            rel
        }
        let mut ast = ast::Ast::for_file(path)?;
        let mut nodes = vec![path.to_path_buf()];
        let mut node_paths = vec![crate::ast::verified::ModulePath::root()];
        let mut node_depths = vec![0usize];
        let mut parent_dirs = vec![vec![canonical_parent(path)?]];
        let mut alias_queue = VecDeque::<(usize, std::path::PathBuf)>::new();
        let mut edges = Vec::<Vec<(std::path::PathBuf, usize)>>::new();
        let mut root_imports = Vec::new();
        let mut idx = 0;
        while idx < nodes.len() {
            let current = nodes[idx].clone();
            let current_dir = canonical_parent(&current)?;
            let start = ast.imports.len();
            let decl_start = ast.decls.len();
            Self::preprocess_single(&mut ast, &current, usize::from(idx != 0))?;
            if idx > 0 {
                let origin = (node_paths[idx].clone(), node_depths[idx] == 1);
                for decl in &ast.decls[decl_start..] {
                    ast.decl_origins
                        .entry(decl.name().to_owned())
                        .or_insert_with(|| origin.clone());
                }
            }
            let stmts: Vec<_> = ast.imports.drain(start..).collect();
            let mut outgoing = Vec::new();
            for stmt in stmts {
                let rel = import_rel_path(&stmt);
                let target = crate::utils::join_relative(&current_dir, &rel);
                let mut target_idx = None;
                for (i, existing) in nodes.iter().enumerate() {
                    if is_same_file(existing, &target).map_err(|error| {
                        format!("E_SCHEMA_SOURCE: {}: {error}", target.display())
                    })? {
                        let target_parent = canonical_parent(&target)?;
                        if !parent_dirs[i].contains(&target_parent) {
                            parent_dirs[i].push(target_parent.clone());
                            alias_queue.push_back((i, target_parent));
                        }
                        target_idx = Some(i);
                        break;
                    }
                }
                let target_idx = match target_idx {
                    Some(i) => i,
                    None => {
                        let target_parent = canonical_parent(&target)?;
                        let i = nodes.len();
                        let edge = crate::ast::verified::ModulePath::new(
                            stmt.name().to_owned(),
                            stmt.paths().to_owned(),
                            stmt.path_supers(),
                        );
                        nodes.push(target);
                        node_paths.push(node_paths[idx].resolve_edge(&edge));
                        node_depths.push(node_depths[idx] + 1);
                        parent_dirs.push(vec![target_parent]);
                        i
                    }
                };
                if idx == 0 && !outgoing.iter().any(|(_, id)| *id == target_idx) {
                    root_imports.push(stmt);
                }
                outgoing.push((rel, target_idx));
            }
            edges.push(outgoing);
            idx += 1;
        }
        fn check_cycle(
            node: usize,
            edges: &[Vec<(std::path::PathBuf, usize)>],
            nodes: &[std::path::PathBuf],
            state: &mut [u8],
        ) -> Result<(), String> {
            state[node] = 1;
            for &(_, next) in &edges[node] {
                if state[next] == 1 {
                    return Err(format!(
                        "E_SCHEMA_CYCLE: cyclic import {}",
                        nodes[next].display()
                    ));
                }
                if state[next] == 0 {
                    check_cycle(next, edges, nodes, state)?;
                }
            }
            state[node] = 2;
            Ok(())
        }
        check_cycle(0, &edges, &nodes, &mut vec![0; nodes.len()])?;
        while let Some((node, alt_dir)) = alias_queue.pop_front() {
            for (rel, expected_idx) in &edges[node] {
                let alt_target = crate::utils::join_relative(&alt_dir, rel);
                if !is_same_file(&alt_target, &nodes[*expected_idx]).unwrap_or(false) {
                    return Err(format!(
                        "E_SCHEMA_SOURCE: {} resolves import {} differently across directories",
                        nodes[node].display(),
                        rel.display()
                    ));
                }
                let alt_parent = canonical_parent(&alt_target)?;
                if !parent_dirs[*expected_idx].contains(&alt_parent) {
                    parent_dirs[*expected_idx].push(alt_parent.clone());
                    alias_queue.push_back((*expected_idx, alt_parent));
                }
            }
        }
        ast.imports = root_imports;
        Ok(ast)
    }

    fn preprocess_single(ast: &mut ast::Ast, path: &Path, depth: usize) -> Result<(), String> {
        let source = fs::read_to_string(path)
            .map_err(|error| format!("E_SCHEMA_SOURCE: {}: {error}", path.display()))?;
        let pairs = parser::InnerParser::parse(parser::Rule::grammar, &source)
            .map_err(|error| error.with_path(&path.display().to_string()).to_string())?;
        Self::append_declarations(ast, depth, pairs)
            .map_err(|error| error.with_path(&path.display().to_string()).to_string())
    }
}

impl parser::Parser {
    pub(crate) fn append_declarations(
        ast: &mut ast::Ast,
        imported_depth: usize,
        mut file_content: Pairs<'_, parser::Rule>,
    ) -> Result<(), Box<PestError<parser::Rule>>> {
        let grammar = file_content
            .next()
            .unwrap_or_else(|| panic!("grammar should only have one pair"));
        if file_content.peek().is_some() {
            panic!("grammar should only have only one pair");
        }
        let grammar_span = grammar.as_span();
        let mut file_version = None;
        let mut eoi = false;
        for pair in grammar.into_inner() {
            if eoi {
                panic!("grammar should have only one EOI");
            }
            match pair.as_rule() {
                parser::Rule::syntax_version_stmt => {
                    let span = pair.as_span();
                    let version = SyntaxVersion {
                        version: pair.into_inner().next_usize()?,
                    };
                    file_version = Some((version, span));
                }
                parser::Rule::import_stmt => {
                    let mut pair = pair.into_inner();
                    let node = pair.next_import(imported_depth);
                    pair.next_should_be_none();
                    ast.add_import(node);
                }
                parser::Rule::option_decl => {
                    let mut pair = pair.into_inner();
                    let node = ast::OptionDecl {
                        name: pair.next_string(),
                        item: pair.next_item(),
                        imported_depth,
                    };
                    pair.next_should_be_none();
                    ast.add_decl(node);
                }
                parser::Rule::union_decl => {
                    let mut pair = pair.into_inner();
                    let node = ast::UnionDecl {
                        name: pair.next_string(),
                        items: pair.next_custom_union_items()?,
                        imported_depth,
                    };
                    pair.next_should_be_none();
                    ast.add_decl(node);
                }
                parser::Rule::integer_decl => {
                    let mut parts = pair.into_inner();
                    let name = parts.next_string();
                    let literal = parts.next().unwrap();
                    let spec = literal
                        .as_str()
                        .parse::<crate::integer::IntegerSpec>()
                        .map_err(|message| invalid(literal.as_span(), message))?;
                    ast.add_decl(ast::ArrayDecl {
                        name: name.clone(),
                        item: ast::ItemDecl { typ: "byte".into() },
                        item_count: spec.byte_width(),
                        imported_depth,
                    });
                    ast.integers.insert(name, spec);
                }
                parser::Rule::array_decl => {
                    let mut pair = pair.into_inner();
                    let node = ast::ArrayDecl {
                        name: pair.next_string(),
                        item: pair.next_item(),
                        item_count: pair.next_usize()?,
                        imported_depth,
                    };
                    pair.next_should_be_none();
                    ast.add_decl(node);
                }
                parser::Rule::struct_decl => {
                    let mut pair = pair.into_inner();
                    let node = ast::StructDecl {
                        name: pair.next_string(),
                        fields: pair.next_fields(),
                        imported_depth,
                    };
                    pair.next_should_be_none();
                    ast.add_decl(node);
                }
                parser::Rule::vector_decl => {
                    let mut pair = pair.into_inner();
                    let node = ast::VectorDecl {
                        name: pair.next_string(),
                        item: pair.next_item(),
                        imported_depth,
                    };
                    pair.next_should_be_none();
                    ast.add_decl(node);
                }
                parser::Rule::table_decl => {
                    let mut pair = pair.into_inner();
                    let node = ast::TableDecl {
                        name: pair.next_string(),
                        fields: pair.next_fields(),
                        imported_depth,
                    };
                    pair.next_should_be_none();
                    ast.add_decl(node);
                }
                parser::Rule::EOI => {
                    if eoi {
                        panic!("grammar could not have more than one EOI");
                    }
                    eoi = true;
                }
                _ => {
                    unreachable!();
                }
            }
        }
        if !eoi {
            panic!("grammar should have only one EOI");
        }

        let (version, span) = file_version.unwrap_or((SyntaxVersion::default(), grammar_span));
        if ast
            .syntax_version
            .as_ref()
            .is_some_and(|previous| *previous != version)
        {
            return Err(invalid(
                span,
                "E_SCHEMA_SYNTAX: all schema files must have the same syntax version",
            ));
        }
        ast.syntax_version = Some(version);
        Ok(())
    }
}

#[cfg(feature = "schema-extensions")]
impl ast::Ast {
    pub(crate) fn retain_root_imports(&mut self, edges: &[crate::schema::ImportSource]) {
        let mut seen = HashSet::new();
        let mut keep = edges.iter().map(|edge| seen.insert(&edge.module));
        self.imports
            .retain(|import| import.imported_depth() == 0 && keep.next() == Some(true));
    }
}

#[cfg(test)]
mod tests {
    use super::{parser, SyntaxVersion};
    use std::io::Write;

    #[test]
    fn test_default_syntax_version_should_be_1_0() {
        // get path of  file
        let mut schema_file = tempfile::NamedTempFile::new().unwrap();
        let _ = schema_file.write(b"array uint32 [byte; 4];").unwrap();
        schema_file.flush().unwrap();

        let file = schema_file.into_temp_path();

        let ast = parser::Parser::preprocess(&file).unwrap();
        assert_eq!(ast.syntax_version, Some(SyntaxVersion { version: 1 }));
    }

    #[test]
    fn test_parse_syntax_version() {
        // get path of  file
        let mut schema_file = tempfile::NamedTempFile::new().unwrap();
        let test_version = SyntaxVersion { version: 7 };
        schema_file
            .write_fmt(format_args!("syntax = {};", test_version.version))
            .unwrap();
        let _ = schema_file.write(b"array uint32 [byte; 4];").unwrap();
        schema_file.flush().unwrap();

        let file = schema_file.into_temp_path();

        let ast = parser::Parser::preprocess(&file).unwrap();
        assert_eq!(ast.syntax_version, Some(test_version));
    }

    #[test]
    #[should_panic]
    // if A `syntax = 1` schema file imports a `syntax = 2` schema file, it should panic
    fn test_different_syntax_version_should_panic() {
        let mut child_schema_file = tempfile::NamedTempFile::new().unwrap();
        child_schema_file
            .write_fmt(format_args!("syntax = 2;"))
            .unwrap();
        let _ = child_schema_file.write(b"array uint64 [byte; 8];").unwrap();
        child_schema_file.flush().unwrap();

        let child_file = child_schema_file.into_temp_path();
        let child_file_path = child_file.to_str().unwrap();

        let mut root_schema_file = tempfile::NamedTempFile::new().unwrap();
        root_schema_file
            .write_fmt(format_args!("syntax = 1;",))
            .unwrap();
        root_schema_file
            .write_fmt(format_args!("import {:?}", child_file_path))
            .unwrap();
        let _ = root_schema_file.write(b"array uint32 [byte; 4];").unwrap();
        root_schema_file.flush().unwrap();

        let file = root_schema_file.into_temp_path();

        parser::Parser::preprocess(&file).unwrap();
    }
}
