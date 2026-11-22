use std::{
    io::{self, Read},
    process,
};

use molecule_codegen::{Compiler, IntermediateFormat, Language};
pub mod plugin;

pub(crate) enum AppAction {
    DisplayFormat,
    DisplayCapabilities,
    ProcessIntermediate(Vec<u8>),
}

pub struct AppConfig {
    action: AppAction,
    lang: Language,
    format: IntermediateFormat,
}

type RawAppConfig = (Language, IntermediateFormat, clap::ArgMatches);

pub fn build_commandline(lang: Language, format: IntermediateFormat) -> AppConfig {
    let matches = clap::Command::new("moleculec-plugin")
        .name(format!("Moleculec {} Plugin", lang))
        .about("Compiler plugin for molecule to generate code.")
        .version(clap::crate_version!())
        .arg(
            clap::Arg::new("format")
                .long("format")
                .help("Output the supported format for the intermediate data.")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("capabilities")
                .long("capabilities")
                .help("Output versioned IR and exact logical-type capabilities.")
                .conflicts_with("format")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();
    AppConfig::from((lang, format, matches))
}

impl From<RawAppConfig> for AppConfig {
    fn from(input: RawAppConfig) -> Self {
        let (lang, format, matches) = input;
        let action = if matches.get_flag("capabilities") {
            AppAction::DisplayCapabilities
        } else if matches.get_flag("format") {
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
            lang,
            format,
        }
    }
}

impl AppConfig {
    pub fn execute(self) {
        match self.action {
            AppAction::DisplayCapabilities => {
                match molecule_codegen::ir::Capabilities::builtin(self.lang, self.format).to_json()
                {
                    Ok(json) => println!("{json}"),
                    Err(error) => {
                        eprintln!("{error}");
                        process::exit(1);
                    }
                }
            }
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
