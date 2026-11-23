use std::{env, fs, io, io::Write as _, path};

#[cfg(feature = "compiler-plugin")]
use std::process;

use crate::{generator, parser};

#[cfg(feature = "compiler-plugin")]
use crate::ir;

pub struct Compiler {
    target: Option<generator::Target>,
    input: Option<Input>,
    output: Output,
    #[cfg(feature = "schema-extensions")]
    type_metadata: Vec<path::PathBuf>,
    #[cfg(feature = "compiler-plugin")]
    integer_contract: Option<path::PathBuf>,
}

pub(crate) enum Input {
    SchemaFile(path::PathBuf),
    #[cfg(feature = "compiler-plugin")]
    Intermediate(ir::Format, Vec<u8>),
}

#[cfg(feature = "compiler-plugin")]
pub(crate) struct PluginChild(Option<process::Child>);

#[cfg(feature = "compiler-plugin")]
impl Drop for PluginChild {
    fn drop(&mut self) {
        if let Some(mut child) = self.0.take() {
            drop(child.stdin.take());
            if matches!(child.try_wait(), Ok(None)) {
                let _ = child.kill();
            }
            let _ = child.wait();
        }
    }
}

pub(crate) enum Output {
    Directory(path::PathBuf),
    Stdout,
    #[cfg(feature = "compiler-plugin")]
    PluginProcess(PluginChild),
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            target: None,
            input: None,
            output: Output::Stdout,
            #[cfg(feature = "schema-extensions")]
            type_metadata: Vec::new(),
            #[cfg(feature = "compiler-plugin")]
            integer_contract: None,
        }
    }

    pub fn generate_code(&mut self, lang: generator::Language) -> &mut Self {
        self.target.replace(generator::Target::Language(lang));
        self
    }

    #[cfg(feature = "compiler-plugin")]
    pub fn generate_intermediate(&mut self, format: ir::Format) -> &mut Self {
        self.target.replace(generator::Target::Intermediate(format));
        self
    }

    #[cfg(feature = "compiler-plugin")]
    pub fn generate_wire_intermediate(&mut self, format: ir::Format) -> &mut Self {
        self.target
            .replace(generator::Target::WireIntermediate(format));
        self
    }

    pub fn input_schema_file<P: AsRef<path::Path>>(&mut self, path: P) -> &mut Self {
        self.input
            .replace(Input::SchemaFile(path.as_ref().to_path_buf()));
        self
    }

    /// Add metadata explicitly. Relative schema paths are resolved from this file.
    #[cfg(feature = "schema-extensions")]
    pub fn type_metadata_file<P: AsRef<path::Path>>(&mut self, path: P) -> &mut Self {
        self.type_metadata.push(path.as_ref().to_owned());
        self
    }

    /// Require the named integers of a published IR v2 JSON file before generating output.
    #[cfg(feature = "compiler-plugin")]
    pub fn integer_contract_file(&mut self, path: impl AsRef<path::Path>) -> &mut Self {
        self.integer_contract = Some(path.as_ref().to_owned());
        self
    }

    /// Parse and validate all configured schema inputs without producing output.
    pub fn parse_ast(&self) -> Result<crate::ast::Ast, String> {
        let input = self
            .input
            .as_ref()
            .ok_or("input is not set: schema file or intermediate data")?;
        let ast = match input {
            #[cfg(feature = "schema-extensions")]
            Input::SchemaFile(path) if !self.type_metadata.is_empty() => {
                crate::schema::LoadedGraph::load(path)?.parse_with_metadata(&self.type_metadata)
            }
            Input::SchemaFile(path) => parser::Parser::parse_file(path),
            #[cfg(feature = "compiler-plugin")]
            Input::Intermediate(format, data) => {
                if !self.type_metadata.is_empty() {
                    return Err("E_META_SCHEMA: metadata sidecars require a schema input; IR carries its own semantics".into());
                }
                format.recover(data)
            }
        }?;
        #[cfg(feature = "compiler-plugin")]
        if let Some(path) = &self.integer_contract {
            ir::Envelope::check_integer_contract(path, &ast)
                .map_err(|err| format!("E_INT_CONTRACT: {}: {err}", path.display()))?;
        }
        Ok(ast)
    }

    #[cfg(feature = "compiler-plugin")]
    pub fn input_intermediate(&mut self, format: ir::Format, data: Vec<u8>) -> &mut Self {
        self.input.replace(Input::Intermediate(format, data));
        self
    }

    pub fn output_dir_set_default(&mut self) -> &mut Self {
        let out_dir = path::PathBuf::from(&env::var("OUT_DIR").unwrap_or_else(|_| ".".to_string()));
        self.output_dir(out_dir)
    }

    pub fn output_dir<P: AsRef<path::Path>>(&mut self, path: P) -> &mut Self {
        self.output = Output::Directory(path.as_ref().to_path_buf());
        self
    }

    #[cfg(feature = "compiler-plugin")]
    pub fn output_plugin_process(&mut self, child: process::Child) -> &mut Self {
        self.output = Output::PluginProcess(PluginChild(Some(child)));
        self
    }

    pub fn run(&mut self) -> Result<(), String> {
        let target = self
            .target
            .ok_or("target is not set: generate code or intermediate data")?;
        let ast = self.parse_ast()?;
        let namespace = ast.namespace().to_owned();
        #[cfg(feature = "schema-extensions")]
        let extended = ast.source_graph().is_some();
        let generator = generator::Generator::new(ast);

        let mut output_data = Vec::<u8>::new();
        generator
            .generate(target, &mut output_data)
            .map_err(|err| format!("failed to write data by generator: {}", err))?;

        match &mut self.output {
            Output::Directory(ref out_dir) => {
                if !crate::utils::valid_namespace(&namespace) {
                    return Err(format!("E_OUTPUT: invalid output namespace {namespace:?}"));
                }
                let out_file = out_dir.join(format!("{namespace}.{}", target.extension()));
                #[cfg(feature = "schema-extensions")]
                if extended {
                    write_atomic(&out_file, &output_data)?;
                    return Ok(());
                }
                write_in_place(&out_file, &output_data)?;
            }
            Output::Stdout => {
                let stdout = io::stdout();
                let mut stdout_handle = stdout.lock();
                stdout_handle
                    .write_all(&output_data)
                    .and_then(|_| stdout_handle.flush())
                    .map_err(|err| format!("E_OUTPUT: {err}"))?;
            }
            #[cfg(feature = "compiler-plugin")]
            Output::PluginProcess(ref mut plugin) => {
                let process = plugin
                    .0
                    .as_mut()
                    .ok_or("E_OUTPUT: plugin process already consumed")?;
                let mut child_stdin = process
                    .stdin
                    .take()
                    .ok_or("E_OUTPUT: plugin process stdin is not piped")?;
                let written = child_stdin
                    .write_all(&output_data)
                    .and_then(|_| child_stdin.flush());
                drop(child_stdin);
                let status = process
                    .wait()
                    .map_err(|err| format!("E_OUTPUT: failed to wait for plugin: {err}"))?;
                plugin.0 = None;
                if !status.success() {
                    return Err(format!("E_OUTPUT: plugin exited with {status}"));
                }
                written.map_err(|err| format!("E_OUTPUT: {err}"))?;
            }
        }

        Ok(())
    }
}

fn write_in_place(destination: &path::Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(destination)
        .map_err(|error| format!("E_OUTPUT: {}: {error}", destination.display()))?;
    file.write_all(bytes)
        .and_then(|_| file.flush())
        .map_err(|error| format!("E_OUTPUT: {}: {error}", destination.display()))
}

#[cfg(feature = "schema-extensions")]
fn write_atomic(destination: &path::Path, bytes: &[u8]) -> Result<(), String> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Temporary(Option<path::PathBuf>);
    impl Drop for Temporary {
        fn drop(&mut self) {
            if let Some(path) = self.0.take() {
                let _ = fs::remove_file(path);
            }
        }
    }
    let directory = destination
        .parent()
        .ok_or("E_OUTPUT: destination has no parent")?;
    #[cfg(unix)]
    let existing_perms = fs::metadata(destination).ok().map(|m| m.permissions());
    loop {
        let path = directory.join(format!(
            ".molecule-{}-{}.tmp",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        if existing_perms.is_some() {
            use std::os::unix::fs::OpenOptionsExt;
            // Create strictly private before applying the destination's mode so
            // overwriting a restricted file (e.g. 0600) never widens permissions.
            options.mode(0o600);
        }
        let mut file = match options.open(&path) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(format!("E_OUTPUT: {}: {error}", destination.display())),
        };
        let mut temporary = Temporary(Some(path));
        file.write_all(bytes)
            .and_then(|_| file.flush())
            .map_err(|error| format!("E_OUTPUT: {}: {error}", destination.display()))?;
        #[cfg(unix)]
        if let Some(perms) = existing_perms.as_ref() {
            file.set_permissions(perms.clone())
                .map_err(|error| format!("E_OUTPUT: {}: {error}", destination.display()))?;
        }
        drop(file);
        let temp_path = temporary.0.as_ref().expect("active temporary path");
        fs::rename(temp_path, destination)
            .map_err(|error| format!("E_OUTPUT: {}: {error}", destination.display()))?;
        temporary.0 = None;
        return Ok(());
    }
}
