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
    println!("cargo:rerun-if-changed={}", "../../test/vectors");
    compile_schema("../../test/schemas/types.mol");
}
