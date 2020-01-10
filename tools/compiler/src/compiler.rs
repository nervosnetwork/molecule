#[macro_use]
extern crate clap;

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
        let yaml = clap::load_yaml!("cli/compiler.yaml");
        let matches = clap::App::from_yaml(yaml)
            .version(clap::crate_version!())
            .get_matches();
        AppConfig::from(&matches)
    }

    impl<'a> From<&'a clap::ArgMatches<'a>> for AppConfig {
        fn from(matches: &'a clap::ArgMatches) -> Self {
            let schema_file = value_t_or_exit!(matches, "schema-file", PathBuf);
            let language = value_t_or_exit!(matches, "language", String);
            if !schema_file.as_path().is_file() {
                eprintln!(
                    "Error: schema-file [{}] should be a file",
                    schema_file.to_str().unwrap()
                );
                process::exit(1);
            }
            let output_config = if language == "-" {
                let format = value_t!(matches, "format", String).unwrap_or_else(|_| {
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
                if value_t!(matches, "format", String).is_ok() {
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
