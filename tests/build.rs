fn compile_schema_0_7_3(schema: &str) {
    let out_dir = std::path::PathBuf::from(&std::env::var("OUT_DIR").unwrap()).join("0_7_3");
    std::fs::create_dir_all(&out_dir).unwrap();

    let mut compiler = codegen_0_7_3::Compiler::new();
    compiler
        .input_schema_file(schema)
        .generate_code(codegen_0_7_3::Language::Rust)
        .output_dir(out_dir)
        .run()
        .unwrap();
    println!("cargo:rerun-if-changed={}", schema);
}

fn compile_schema_dev(schema: &str) {
    let out_dir = std::path::PathBuf::from(&std::env::var("OUT_DIR").unwrap()).join("dev");
    std::fs::create_dir_all(&out_dir).unwrap();

    let mut compiler = codegen_dev::Compiler::new();
    compiler
        .input_schema_file(schema)
        .generate_code(codegen_dev::Language::Rust)
        .output_dir(out_dir)
        .run()
        .unwrap();
    println!("cargo:rerun-if-changed={}", schema);
}

fn compile_intermediate_0_7_3(schema: &str) {
    let out_dir = std::path::PathBuf::from(&std::env::var("OUT_DIR").unwrap()).join("0_7_3");
    std::fs::create_dir_all(&out_dir).unwrap();

    let mut compiler = codegen_0_7_3::Compiler::new();
    compiler
        .input_schema_file(schema)
        .generate_intermediate(codegen_0_7_3::IntermediateFormat::JSON)
        .output_dir(out_dir)
        .run()
        .unwrap();
    println!("cargo:rerun-if-changed={}", schema);
}

fn compile_intermediate_dev(schema: &str) {
    let out_dir = std::path::PathBuf::from(&std::env::var("OUT_DIR").unwrap()).join("dev");
    std::fs::create_dir_all(&out_dir).unwrap();

    let mut compiler = codegen_dev::Compiler::new();
    compiler
        .input_schema_file(schema)
        .generate_intermediate(codegen_dev::IntermediateFormat::JSON)
        .output_dir(out_dir)
        .run()
        .unwrap();
    println!("cargo:rerun-if-changed={}", schema);
}

fn main() {
    println!("cargo:rerun-if-changed=./union_foo_0_7_3.mol");
    println!("cargo:rerun-if-changed=./union_foo_with_custom_id.mol");

    compile_intermediate_0_7_3("./union_foo_0_7_3.mol");
    compile_intermediate_dev("./union_foo_with_custom_id.mol");

    compile_schema_0_7_3("./union_foo_0_7_3.mol");
    compile_schema_dev("./union_foo_with_custom_id.mol");
}
