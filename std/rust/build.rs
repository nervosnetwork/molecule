use codegen::{Compiler, Language};

fn compile_schema(schema: &str) {
    let mut compiler = Compiler::new();
    compiler
        .input_schema_file(schema)
        .generate_code(Language::Rust)
        .output_dir_set_default()
        .run()
        .unwrap();
    println!("cargo:rerun-if-changed={}", schema);
}

fn main() {
    println!("cargo:rerun-if-changed=primitive_types.mol");
    compile_schema("primitive_types.mol");
    let out_dir = ::std::env::var("OUT_DIR").unwrap();
    println!("{}", out_dir);
}
