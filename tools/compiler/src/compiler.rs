use std::{convert::TryFrom, io::Write, path::Path, process, str, time::Duration};

use molecule_codegen::{Compiler, IntermediateFormat};

pub(crate) mod config {
    use std::{convert::TryFrom, path::PathBuf, process};

    use molecule_codegen::IntermediateFormat;

    pub(crate) enum OutputConfig {
        Plugin(PathBuf),
        Output(IntermediateFormat),
    }

    pub(crate) struct AppConfig {
        pub(crate) schema_file: PathBuf,
        pub(crate) output_config: OutputConfig,
        pub(crate) type_metadata: Vec<PathBuf>,
        pub(crate) wire_only: bool,
        pub(crate) integer_contract: Option<PathBuf>,
        pub(crate) plugin_timeout: u64,
    }

    pub(crate) fn build_commandline() -> AppConfig {
        let matches = clap::Command::new("moleculec")
            .name("Moleculec")
            .about("Schema compiler for molecule.")
            .author("Nervos Core Dev <dev@nervos.org>")
            .version(clap::crate_version!())
            .arg(
                clap::Arg::new("schema-file")
                    .long("schema-file")
                    .help("Provide a schema file to compile.")
                    .required(true)
                    .value_parser(clap::value_parser!(PathBuf))
                    .action(clap::ArgAction::Set),
            )
            .arg(
                clap::Arg::new("language")
                    .long("language")
                    .help("Specify a language, then generate source code for the specified language and output the generated code to the stdout.\
                    \nThis parameter actually specifies a plugin to use. It should be a simple word, and the compiler will search for a plugin called \"moleculec-<language>\" in `$PATH`.\
                    \nIf \"<language>\" is \"-\", the compiler will dump the intermediate data of schema to standard output.")
                    .required(true)
                    .action(clap::ArgAction::Set),
            )
            .arg(
                clap::Arg::new("format")
                    .long("format")
                    .help(r#"If "<language>" is "-", this parameter is used to specify a format for the intermediate data."#)
                    .value_parser(["json", "yaml"])
                    .action(clap::ArgAction::Set),
            )
            .arg(
                clap::Arg::new("type-metadata")
                    .long("type-metadata")
                    .help("Explicit integer metadata sidecar; repeat for imported schemas.")
                    .value_parser(clap::value_parser!(PathBuf))
                    .action(clap::ArgAction::Append),
            )
            .arg(
                clap::Arg::new("check-integer-contract")
                    .long("check-integer-contract")
                    .value_name("PUBLISHED_V2_JSON")
                    .help("Require the published integer names, owners and numeric specifications before generation.")
                    .value_parser(clap::value_parser!(PathBuf)),
            )
            .arg(
                clap::Arg::new("wire-only")
                    .long("wire-only")
                    .help("Export only legacy wire IR, intentionally omitting semantic extensions.")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                clap::Arg::new("plugin-timeout")
                    .long("plugin-timeout")
                    .help("Timeout in seconds for plugin generation with semantic extensions.")
                    .default_value("60")
                    .value_parser(clap::value_parser!(u64).range(1..)),
            )
            .get_matches();
        AppConfig::from(matches)
    }

    impl From<clap::ArgMatches> for AppConfig {
        fn from(matches: clap::ArgMatches) -> Self {
            let schema_file = matches
                .get_one::<PathBuf>("schema-file")
                .unwrap()
                .to_owned();
            let language = matches.get_one::<String>("language").unwrap();
            let wire_only = matches.get_flag("wire-only");
            if wire_only && language != "-" {
                eprintln!("Error: --wire-only requires --language -");
                process::exit(1);
            }
            let type_metadata = matches
                .get_many::<PathBuf>("type-metadata")
                .into_iter()
                .flatten()
                .cloned()
                .collect();
            let plugin_timeout = *matches.get_one::<u64>("plugin-timeout").unwrap();
            if !schema_file.as_path().is_file() {
                eprintln!(
                    "Error: schema-file [{}] should be a file",
                    schema_file.display()
                );
                process::exit(1);
            }
            let output_config = if language == "-" {
                let format = matches.get_one::<String>("format").unwrap_or_else(|| {
                    eprintln!("Error: since language is \"-\", a format is required");
                    process::exit(1);
                });
                match IntermediateFormat::try_from(format.as_str()) {
                    Ok(format) => OutputConfig::Output(format),
                    Err(error) => {
                        eprintln!("Error: {}", error);
                        process::exit(1);
                    }
                }
            } else {
                if matches.get_one::<String>("format").is_some() {
                    eprintln!("Error: since language is not \"-\", don't specify format");
                    process::exit(1);
                }
                let plugin_name = format!("moleculec-{}", language);
                if let Ok(plugin) = which::which(&plugin_name) {
                    OutputConfig::Plugin(plugin)
                } else {
                    eprintln!("Error: no such plugin (should be '{}')", plugin_name);
                    process::exit(1);
                }
            };
            Self {
                schema_file,
                output_config,
                type_metadata,
                wire_only,
                integer_contract: matches
                    .get_one::<PathBuf>("check-integer-contract")
                    .cloned(),
                plugin_timeout,
            }
        }
    }
}

fn main() {
    if let Err(error) = compile(config::build_commandline()) {
        eprintln!("Error: {error}");
        process::exit(1);
    }
}

fn compile(config: config::AppConfig) -> Result<(), String> {
    let mut compiler = Compiler::new();
    compiler.input_schema_file(&config.schema_file);
    for path in &config.type_metadata {
        compiler.type_metadata_file(path);
    }
    if let Some(path) = config.integer_contract {
        compiler.integer_contract_file(path);
    }
    let ast = compiler.parse_ast()?;
    let bytes = match &config.output_config {
        config::OutputConfig::Output(format) => {
            if config.wire_only {
                eprintln!("Wire-only export: logical type semantics are intentionally omitted.");
                format.serialize_wire(&ast)?
            } else {
                format.serialize_ast(&ast)?
            }
        }
        config::OutputConfig::Plugin(plugin) if ast.source_graph().is_none() => {
            return legacy_plugin(plugin, &ast);
        }
        config::OutputConfig::Plugin(plugin) => {
            let format = moleculec::plugin::negotiate(
                plugin,
                &ast,
                Duration::from_secs(config.plugin_timeout.min(5)),
            )?;
            let input = format.serialize_ast(&ast)?;
            let (bytes, diagnostics) = moleculec::plugin::execute(
                plugin,
                input,
                Duration::from_secs(config.plugin_timeout),
            )?;
            std::io::stderr()
                .lock()
                .write_all(&diagnostics)
                .map_err(|error| format!("cannot write plugin diagnostics: {error}"))?;
            bytes
        }
    };
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    handle
        .write_all(&bytes)
        .and_then(|_| handle.flush())
        .map_err(|error| format!("cannot write compiler output: {error}"))
}

/// V1 plugins negotiate only a format and stream their output directly to the caller.
fn legacy_plugin(path: &Path, ast: &molecule_codegen::ast::Ast) -> Result<(), String> {
    let reply = process::Command::new(path)
        .arg("--format")
        .output()
        .map_err(|err| err.to_string())?;
    let format = str::from_utf8(&reply.stdout)
        .ok()
        .filter(|_| reply.status.success())
        .and_then(|value| IntermediateFormat::try_from(value.trim()).ok())
        .unwrap_or(IntermediateFormat::JSON);
    let input = format.serialize_ast(ast)?;
    let mut child = process::Command::new(path)
        .stdin(process::Stdio::piped())
        .spawn()
        .map_err(|err| err.to_string())?;
    let written = child
        .stdin
        .take()
        .expect("piped plugin stdin")
        .write_all(&input);
    let status = child.wait().map_err(|err| err.to_string())?;
    if !status.success() {
        return Err(format!("plugin exited with {status}"));
    }
    written.map_err(|err| err.to_string())?;
    Ok(())
}
