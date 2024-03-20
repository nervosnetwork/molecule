use codegen::{Compiler, Language};
use std::ffi::OsStr;
use std::process::Command;
use std::{
    env, fs,
    path::{self, PathBuf},
};

fn compile_schema_rust(schema: &str) {
    let mut compiler = Compiler::new();
    let out_dir = path::PathBuf::from(&env::var("OUT_DIR").unwrap_or_else(|_| ".".to_string()));
    let mut file_path = out_dir.clone();
    compiler
        .input_schema_file(schema)
        .generate_code(Language::Rust)
        .output_dir(out_dir)
        .run()
        .unwrap();
    file_path.push("types.rs");
    Command::new("rustfmt")
        .arg(<PathBuf as AsRef<OsStr>>::as_ref(&file_path))
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    println!("cargo:rerun-if-changed={}", schema);
}

fn compile_schema_rust_lazy_reader(schema: &str) {
    let mut compiler = Compiler::new();
    let mut out_dir = path::PathBuf::from(&env::var("OUT_DIR").unwrap_or_else(|_| ".".to_string()));
    out_dir.push("lazy_reader");
    drop(fs::create_dir(&out_dir));
    let mut file_path = out_dir.clone();
    compiler
        .input_schema_file(schema)
        .generate_code(Language::RustLazyReader)
        .output_dir(out_dir)
        .run()
        .unwrap();
    file_path.push("types.rs");
    Command::new("rustfmt")
        .arg(<PathBuf as AsRef<OsStr>>::as_ref(&file_path))
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    println!("cargo:rerun-if-changed={}", schema);
}

fn main() {
    compile_schema_rust("../../test/schemas/types.mol");
    compile_schema_rust_lazy_reader("../../test/schemas/types.mol");
}
