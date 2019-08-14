use std::{convert::TryInto, path::PathBuf, process};

use codegen::Language;

pub(crate) struct AppConfig {
    pub(crate) schema_file: PathBuf,
    pub(crate) language: Language,
}

pub(crate) fn build_commandline() -> AppConfig {
    let yaml = clap::load_yaml!("cli.yaml");
    let matches = clap::App::from_yaml(yaml).get_matches();
    AppConfig::from(&matches)
}

impl<'a> From<&'a clap::ArgMatches<'a>> for AppConfig {
    fn from(matches: &'a clap::ArgMatches) -> Self {
        let schema_file = value_t!(matches, "schema-file", PathBuf).unwrap_or_else(|e| e.exit());
        let language_string = value_t!(matches, "language", String).unwrap_or_else(|e| e.exit());
        if !schema_file.as_path().is_file() {
            eprintln!(
                "Error: schema-file [{}] should be a file",
                schema_file.to_str().unwrap()
            );
            process::exit(1);
        }
        let language: Language = language_string
            .as_str()
            .try_into()
            .unwrap_or_else(|_| unreachable!());
        Self {
            schema_file,
            language,
        }
    }
}
