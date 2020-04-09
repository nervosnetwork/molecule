use codegen::{Compiler, Language};

fn compile_schema(schema: &str) {
    let mut compiler = Compiler::new();
    compiler
        .input_schema_file(schema)
        .generate_code(Language::Rust)
        .output_dir_set_default()
        .run()
        .unwrap();
    compiler
        .input_schema_file(schema)
        .generate_code(Language::C)
        .output_dir_set_default()
        .run()
        .unwrap();
    println!("cargo:rerun-if-changed={}", schema);
}

fn main() {
    println!("cargo:rerun-if-changed=../../test/vectors");
    println!("cargo:rerun-if-changed=src/capi.c");
    compile_schema("../../test/schemas/types.mol");
    let out_dir = ::std::env::var("OUT_DIR").unwrap();
    cc::Build::new()
        .file("src/capi.c")
        .include(&out_dir)
        .include("../../bindings/c/include")
        .warnings(false)
        .compile("c-api.o");
}
