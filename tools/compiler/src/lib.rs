use std::{
    io::{self, Read},
    process,
};

use molecule_codegen::{Compiler, IntermediateFormat, Language};

pub(crate) enum AppAction {
    DisplayFormat,
    ProcessIntermediate(Vec<u8>),
}

pub struct AppConfig {
    action: AppAction,
    lang: Language,
    format: IntermediateFormat,
}

type RawAppConfig<'a> = (Language, IntermediateFormat, &'a clap::ArgMatches<'a>);

pub fn build_commandline(lang: Language, format: IntermediateFormat) -> AppConfig {
    let yaml = clap::load_yaml!("cli/compiler-plugin.yaml");
    let matches = clap::App::from_yaml(yaml)
        .name(format!("Moleculec {} Plugin", lang))
        .about("Compiler plugin for molecule to generate code.")
        .version(clap::crate_version!())
        .get_matches();
    AppConfig::from(&(lang, format, &matches))
}

impl<'a> From<&'a RawAppConfig<'a>> for AppConfig {
    fn from(input: &'a RawAppConfig<'a>) -> Self {
        let (lang, format, matches) = input;
        let action = if matches.is_present("format") {
            AppAction::DisplayFormat
        } else {
            let mut input = Vec::new();
            if io::stdin().read_to_end(&mut input).is_err() {
                eprintln!("Error: failed to read data from stdin)");
                process::exit(1);
            };
            AppAction::ProcessIntermediate(input)
        };
        Self {
            action,
            lang: *lang,
            format: *format,
        }
    }
}

impl AppConfig {
    pub fn execute(self) {
        match self.action {
            AppAction::DisplayFormat => {
                println!("{}", self.format);
            }
            AppAction::ProcessIntermediate(ref input) => {
                Compiler::new()
                    .generate_code(self.lang)
                    .input_intermediate(self.format, input.to_owned())
                    .run()
                    .unwrap();
            }
        }
    }
}
