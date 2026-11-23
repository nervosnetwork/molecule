use codegen::{Compiler, Language};
use std::{env, fs, path::PathBuf};

fn main() {
    let schema = "../../test/schemas/types.mol";
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let lazy_dir = out_dir.join("lazy_reader");
    fs::create_dir_all(&lazy_dir).unwrap();
    for (language, dir) in [
        (Language::Rust, out_dir),
        (Language::RustLazyReader, lazy_dir),
    ] {
        Compiler::new()
            .input_schema_file(schema)
            .generate_code(language)
            .output_dir(dir)
            .run()
            .unwrap();
    }
    println!("cargo:rerun-if-changed={schema}");
}
