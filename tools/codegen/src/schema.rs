//! Source identity and direct dependency edges for explicit schema extensions.
//! Paths in the serializable graph are relative to the root schema's directory.

use crate::parser::{InnerParser, Rule};
use pest::Parser;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeclarationSource {
    pub name: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportSource {
    pub path: String,
    pub module: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaSource {
    pub id: String,
    pub sha256: String,
    pub imports: Vec<ImportSource>,
    pub declarations: Vec<DeclarationSource>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaGraph {
    pub root: String,
    pub modules: Vec<SchemaSource>,
}

pub struct LoadedGraph {
    pub(crate) graph: SchemaGraph,
    paths: FileIndex,
    layout: Result<crate::ast::raw::Ast, String>,
    with_provenance: bool,
}

#[derive(Default)]
struct FileIndex {
    by_path: BTreeMap<PathBuf, String>,
    #[cfg(unix)]
    by_inode: BTreeMap<(u64, u64), String>,
}

impl FileIndex {
    #[cfg(unix)]
    fn file_key(path: &Path) -> Result<(u64, u64), String> {
        use std::os::unix::fs::MetadataExt;
        let meta = fs::metadata(path)
            .map_err(|error| format!("E_SCHEMA_SOURCE: {}: {error}", path.display()))?;
        Ok((meta.dev(), meta.ino()))
    }

    fn insert(&mut self, canonical_path: PathBuf, id: String) -> Result<(), String> {
        #[cfg(unix)]
        {
            let key = Self::file_key(&canonical_path)?;
            self.by_inode.entry(key).or_insert_with(|| id.clone());
        }
        self.by_path.entry(canonical_path).or_insert(id);
        Ok(())
    }

    fn lookup(&self, canonical_path: &Path) -> Result<Option<&String>, String> {
        if let Some(id) = self.by_path.get(canonical_path) {
            return Ok(Some(id));
        }
        #[cfg(unix)]
        {
            let key = Self::file_key(canonical_path)?;
            Ok(self.by_inode.get(&key))
        }
        #[cfg(not(unix))]
        {
            // Compare new paths without retaining one open descriptor per module.
            for (known, id) in &self.by_path {
                if same_file::is_same_file(known, canonical_path).map_err(|error| {
                    format!("E_SCHEMA_SOURCE: {}: {error}", canonical_path.display())
                })? {
                    return Ok(Some(id));
                }
            }
            Ok(None)
        }
    }

    fn rename_all(&mut self, rename: &impl Fn(&mut String)) {
        self.by_path.values_mut().for_each(rename);
        #[cfg(unix)]
        self.by_inode.values_mut().for_each(rename);
    }
}

pub fn source_digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn canonical(path: &Path) -> Result<PathBuf, String> {
    path.canonicalize()
        .map_err(|error| format!("E_SCHEMA_SOURCE: {}: {error}", path.display()))
}

fn parent_dir(path: &Path) -> Result<PathBuf, String> {
    let parent = path.parent().ok_or_else(|| {
        format!(
            "E_SCHEMA_SOURCE: {} has no parent directory",
            path.display()
        )
    })?;
    canonical(parent)
}

fn relative_id(base: &Path, path: &Path) -> Result<String, String> {
    let base: Vec<_> = base.components().collect();
    let path: Vec<_> = path.components().collect();
    let shared = base.iter().zip(&path).take_while(|(a, b)| a == b).count();
    let mut result = PathBuf::new();
    for _ in shared..base.len() {
        result.push("..");
    }
    for part in &path[shared..] {
        result.push(part.as_os_str());
    }
    result
        .to_str()
        .map(|path| path.replace('\\', "/"))
        .filter(|id| valid_id(id))
        .ok_or_else(|| "E_SCHEMA_SOURCE: source has no relative UTF-8 path".into())
}

impl LoadedGraph {
    pub fn graph(&self) -> &SchemaGraph {
        &self.graph
    }
    pub fn load(root: &Path) -> Result<Self, String> {
        Self::read(root, true)
    }

    pub(crate) fn read(root: &Path, mut with_provenance: bool) -> Result<Self, String> {
        let root = if root.is_absolute() {
            root.to_owned()
        } else {
            std::env::current_dir()
                .map_err(|error| error.to_string())?
                .join(root)
        };
        let base = root
            .parent()
            .ok_or("E_SCHEMA_SOURCE: root has no parent directory")?;
        let root_id = relative_id(base, &root)?;
        let canonical_base = canonical(base)?;
        let mut pending = VecDeque::from([(
            root_id.clone(),
            root.clone(),
            crate::ast::verified::ModulePath::root(),
            0,
        )]);
        let mut paths = FileIndex::default();
        paths.insert(canonical(&root)?, root_id.clone())?;
        let mut parent_dirs = BTreeMap::from([(root_id.clone(), vec![canonical_base.clone()])]);
        let mut alias_queue = VecDeque::<(String, PathBuf)>::new();
        let mut modules = Vec::new();
        let mut hardlink_names = BTreeMap::new();
        let mut layout = crate::ast::raw::Ast::for_file(&root);
        while let Some((id, path, mod_path, depth)) = pending.pop_front() {
            let current_dir = parent_dir(&path)?;
            let source = fs::read_to_string(&path)
                .map_err(|error| format!("E_SCHEMA_SOURCE: {}: {error}", path.display()))?;
            let parsed = InnerParser::parse(Rule::grammar, &source)
                .map_err(|error| format!("E_SCHEMA_SYNTAX: {}: {error}", path.display()))?;
            let grammar = parsed.peek().ok_or("E_SCHEMA_SYNTAX: missing grammar")?;
            let mut module = SchemaSource {
                id: id.clone(),
                sha256: source_digest(source.as_bytes()),
                imports: Vec::new(),
                declarations: Vec::new(),
            };
            // Index newlines once; asking each Pest span for its line scans from byte zero.
            let lines: Vec<_> = std::iter::once(0)
                .chain(source.match_indices('\n').map(|(offset, _)| offset + 1))
                .collect();
            for pair in grammar.into_inner() {
                let offset = pair.as_span().start();
                let line = lines.partition_point(|start| *start <= offset);
                let column = 1 + source[lines[line - 1]..offset].chars().count();
                match pair.as_rule() {
                    Rule::import_stmt => {
                        let imported = pair
                            .into_inner()
                            .next()
                            .ok_or("E_SCHEMA_SYNTAX: missing import path")?;
                        let mut target =
                            crate::utils::join_relative(&current_dir, Path::new(imported.as_str()));
                        target.set_extension("mol");
                        let key = canonical(&target)?;
                        let target_parent = parent_dir(&target)?;
                        let name = relative_id(&canonical_base, &key)
                            .or_else(|_| relative_id(&canonical_base, &target))
                            .or_else(|_| relative_id(base, &target))?;
                        let target_id = if let Some(known) = paths.lookup(&key)? {
                            let dirs = parent_dirs.entry(known.clone()).or_default();
                            if !dirs.contains(&target_parent) {
                                dirs.push(target_parent.clone());
                                alias_queue.push_back((known.clone(), target_parent));
                            }
                            if known != &root_id && name < *known {
                                let preferred = hardlink_names
                                    .entry(known.clone())
                                    .or_insert_with(|| known.clone());
                                if name < *preferred {
                                    *preferred = name;
                                }
                            }
                            known.clone()
                        } else {
                            let edge_path = crate::ast::verified::ModulePath::from_import_str(
                                imported.as_str(),
                            )
                            .ok_or_else(|| {
                                format!(
                                    "E_SCHEMA_SYNTAX: invalid import path {}",
                                    imported.as_str()
                                )
                            })?;
                            let child_mod_path = mod_path.resolve_edge(&edge_path);
                            parent_dirs.insert(name.clone(), vec![target_parent]);
                            pending.push_back((name.clone(), target, child_mod_path, depth + 1));
                            name
                        };
                        paths.insert(key, target_id.clone())?;
                        module.imports.push(ImportSource {
                            path: imported.as_str().to_owned(),
                            module: target_id,
                            line,
                            column,
                        });
                    }
                    Rule::option_decl
                    | Rule::union_decl
                    | Rule::integer_decl
                    | Rule::array_decl
                    | Rule::struct_decl
                    | Rule::vector_decl
                    | Rule::table_decl => {
                        with_provenance |= pair.as_rule() == Rule::integer_decl;
                        let name = pair
                            .into_inner()
                            .next()
                            .ok_or("E_SCHEMA_SYNTAX: missing declaration name")?
                            .as_str()
                            .to_owned();
                        module
                            .declarations
                            .push(DeclarationSource { name, line, column });
                    }
                    _ => {}
                }
            }
            // Ownership and layout consume the same tokens. Keep layout errors for
            // parse_ast so callers can inspect source provenance independently.
            layout = layout.and_then(|mut raw| {
                let decl_start = raw.decls().len();
                crate::Parser::append_declarations(&mut raw, depth, parsed)
                    .map_err(|error| error.with_path(&path.display().to_string()).to_string())?;
                if id != root_id {
                    let added: Vec<String> = raw.decls()[decl_start..]
                        .iter()
                        .map(|d| d.name().to_owned())
                        .collect();
                    for name in added {
                        raw.decl_origins
                            .entry(name)
                            .or_insert_with(|| mod_path.clone());
                    }
                }
                if id == root_id {
                    raw.retain_root_imports(&module.imports);
                }
                Ok(raw)
            });
            modules.push(module);
        }
        // Hard links can have several canonical paths. Publish the smallest name
        // seen for each file, keeping import resolution in its original directory.
        if !hardlink_names.is_empty() {
            let rename = |id: &mut String| {
                if let Some(name) = hardlink_names.get(id) {
                    id.clone_from(name);
                }
            };
            for module in &mut modules {
                rename(&mut module.id);
                for edge in &mut module.imports {
                    rename(&mut edge.module);
                }
            }
            paths.rename_all(&rename);
            parent_dirs = parent_dirs
                .into_iter()
                .map(|(mut key, dirs)| {
                    rename(&mut key);
                    (key, dirs)
                })
                .collect();
            for (id, _) in &mut alias_queue {
                rename(id);
            }
        }
        modules.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        let graph = SchemaGraph {
            root: root_id,
            modules,
        };
        graph.validate()?;
        // Verify that any module reached via aliases in multiple directories resolves
        // every relative import to the exact same dependency (including transitively).
        while let Some((module_id, alt_dir)) = alias_queue.pop_front() {
            let Some(module) = graph.modules.iter().find(|module| module.id == module_id) else {
                continue;
            };
            for edge in &module.imports {
                let mut alt_target = crate::utils::join_relative(&alt_dir, Path::new(&edge.path));
                alt_target.set_extension("mol");
                let ambiguous = || {
                    format!(
                        "E_SCHEMA_SOURCE: {module_id} resolves import {} differently across directories",
                        edge.path
                    )
                };
                let alt_key = canonical(&alt_target).map_err(|_| ambiguous())?;
                let resolved_id = paths.lookup(&alt_key)?.ok_or_else(ambiguous)?;
                if resolved_id != &edge.module {
                    return Err(ambiguous());
                }
                let alt_parent = parent_dir(&alt_target)?;
                let dirs = parent_dirs.entry(edge.module.clone()).or_default();
                if !dirs.contains(&alt_parent) {
                    dirs.push(alt_parent.clone());
                    alias_queue.push_back((edge.module.clone(), alt_parent));
                }
            }
        }
        Ok(Self {
            with_provenance,
            graph,
            paths,
            layout,
        })
    }

    pub fn module_at(&self, path: &Path) -> Result<&SchemaSource, String> {
        let id = self.paths.lookup(&canonical(path)?)?.ok_or_else(|| {
            format!(
                "E_META_SCHEMA: {} is not reachable from the input schema",
                path.display()
            )
        })?;
        self.graph
            .modules
            .iter()
            .find(|module| &module.id == id)
            .ok_or_else(|| "E_META_SCHEMA: source has no graph entry".into())
    }

    pub fn parse_ast(&self) -> Result<crate::ast::Ast, String> {
        self.parse_with_metadata(&[])
    }

    pub(crate) fn parse_with_metadata(&self, paths: &[PathBuf]) -> Result<crate::ast::Ast, String> {
        let layout = self.layout.as_ref().map_err(Clone::clone)?;
        let mut ast = crate::ast::Ast::complete(layout)?;
        if self.with_provenance {
            let mut annotations = Vec::new();
            for path in paths {
                annotations.extend(crate::integer::MetadataFile::load(path, self)?);
            }
            crate::integer::bind(&mut ast, self.graph.clone(), annotations)?;
        }
        Ok(ast)
    }
}

fn valid_id(id: &str) -> bool {
    if id.is_empty() || id.contains(['\\', ':', '\0']) || id.bytes().any(|b| b.is_ascii_control()) {
        return false;
    }
    let mut saw_normal = false;
    for part in id.split('/') {
        match part {
            "" | "." => return false,
            ".." if saw_normal => return false,
            ".." => {}
            _ => saw_normal = true,
        }
    }
    saw_normal
}

impl SchemaGraph {
    pub fn owner(&self, name: &str) -> Option<&SchemaSource> {
        self.modules
            .iter()
            .find(|module| module.declarations.iter().any(|decl| decl.name == name))
    }

    pub fn validate(&self) -> Result<(), String> {
        let mut modules = BTreeMap::new();
        let mut declarations = BTreeMap::new();
        for module in &self.modules {
            if !valid_id(&module.id) || modules.insert(module.id.as_str(), module).is_some() {
                return Err(format!(
                    "E_MODULE_ID: invalid or duplicate module {}",
                    module.id
                ));
            }
            if module.sha256.len() != 64
                || !module.sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
            {
                return Err(format!("E_SCHEMA_SOURCE: invalid SHA256 for {}", module.id));
            }
            let mut import_paths = BTreeMap::new();
            for edge in &module.imports {
                let parsed = InnerParser::parse(Rule::path, &edge.path)
                    .map_err(|_| format!("E_IR_NAME: invalid import path {}", edge.path))?;
                if parsed.as_str() != edge.path {
                    return Err(format!("E_IR_NAME: invalid import path {}", edge.path));
                }
                if let Some(prev) = import_paths.insert(edge.path.as_str(), edge.module.as_str()) {
                    if prev != edge.module.as_str() {
                        return Err(format!(
                            "E_IR_SOURCE: {} maps import {} to both {} and {}",
                            module.id, edge.path, prev, edge.module
                        ));
                    }
                }
            }
            for decl in &module.declarations {
                if let Some(previous) =
                    declarations.insert(decl.name.to_ascii_lowercase(), &module.id)
                {
                    return Err(format!(
                        "E_SYMBOL_COLLISION: {}:{}:{}: {} conflicts with {previous}",
                        module.id, decl.line, decl.column, decl.name
                    ));
                }
            }
        }
        if !modules.contains_key(self.root.as_str()) {
            return Err("E_MODULE_ID: root is missing".into());
        }
        let mut done = BTreeSet::new();
        let mut active = BTreeSet::from([self.root.as_str()]);
        let mut stack = vec![(self.root.as_str(), 0)];
        while let Some((id, next)) = stack.last_mut() {
            let module = modules
                .get(id)
                .ok_or_else(|| format!("E_MODULE_ID: missing module {id}"))?;
            let Some(edge) = module.imports.get(*next) else {
                let id = *id;
                stack.pop();
                active.remove(id);
                done.insert(id);
                continue;
            };
            *next += 1;
            if active.contains(edge.module.as_str()) {
                let mut cycle: Vec<_> = stack.iter().map(|(id, _)| *id).collect();
                cycle.push(&edge.module);
                return Err(format!(
                    "E_MODULE_CYCLE: {}:{}:{}: {}",
                    module.id,
                    edge.line,
                    edge.column,
                    cycle.join(" -> ")
                ));
            }
            if !done.contains(edge.module.as_str()) {
                active.insert(edge.module.as_str());
                stack.push((&edge.module, 0));
            }
        }
        if done.len() != modules.len() {
            return Err("E_MODULE_ID: graph contains unreachable modules".into());
        }
        Ok(())
    }
}
