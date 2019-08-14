#[macro_use]
extern crate clap;

pub(crate) mod config;

use codegen::Compiler;

fn main() {
    let config = config::build_commandline();
    let mut compiler = Compiler::new();
    compiler
        .language(config.language)
        .file_path(&config.schema_file)
        .run();
}
