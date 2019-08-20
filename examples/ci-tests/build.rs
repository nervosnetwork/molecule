use codegen::{Compiler, Language};

fn main() {
    let schema = "schemas/ci_tests.mol";
    let mut compiler = Compiler::new();
    compiler
        .language(Language::Rust)
        .default_out_dir()
        .file_path(schema)
        .run();
    compiler
        .language(Language::C)
        .default_out_dir()
        .file_path(schema)
        .run();
    println!("cargo:rerun-if-changed={}", schema);
}
