use std::{convert::TryFrom, process, str};

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
            .get_matches();
        AppConfig::from(matches)
    }

    impl From<clap::ArgMatches> for AppConfig {
        fn from(matches: clap::ArgMatches) -> Self {
            let schema_file = {
                let path = matches.get_one::<String>("schema-file").unwrap();
                PathBuf::from(path)
            };
            let language = matches.get_one::<String>("language").unwrap();
            if !schema_file.as_path().is_file() {
                eprintln!(
                    "Error: schema-file [{}] should be a file",
                    schema_file.to_str().unwrap()
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
            }
        }
    }
}

fn main() {
    let config = config::build_commandline();
    let default_format = IntermediateFormat::JSON;
    let mut compiler = Compiler::new();
    match config.output_config {
        config::OutputConfig::Plugin(ref plugin_file) => {
            let output = process::Command::new(plugin_file.as_path())
                .arg("--format")
                .output()
                .expect("Error: failed to execute plugin process");
            let format = if output.status.success() {
                str::from_utf8(&output.stdout[..])
                    .ok()
                    .and_then(|s| IntermediateFormat::try_from(s).ok())
                    .unwrap_or(default_format)
            } else {
                default_format
            };
            if let Ok(child) = process::Command::new(plugin_file.as_path())
                .stdin(process::Stdio::piped())
                .spawn()
            {
                compiler
                    .generate_intermediate(format)
                    .output_plugin_process(child);
            } else {
                eprintln!("Error: failed to spawn plugin process");
                process::exit(1);
            }
        }
        config::OutputConfig::Output(format) => {
            compiler.generate_intermediate(format);
        }
    };
    compiler
        .input_schema_file(config.schema_file.as_path())
        .run()
        .unwrap()
}
