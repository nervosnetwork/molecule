use codegen::{schema::source_digest, Compiler, IntermediateFormat, Language};
use serde_json::json;
use std::{
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
};

fn metadata(schema: &Path, integers: serde_json::Value, path: &Path) -> Result<(), Box<dyn Error>> {
    fs::write(
        path,
        serde_json::to_vec(&json!({
            "metadata_version":1,"schema":schema,"schema_sha256":source_digest(&fs::read(schema)?),"integers":integers,
        }))?,
    )?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let out = PathBuf::from(env::var_os("OUT_DIR").ok_or("OUT_DIR missing")?);
    for dir in ["packed", "lazy", "c", "legacy", "recovered"] {
        fs::create_dir_all(out.join(dir))?;
    }
    let mut schema = String::new();
    let mut native_schema = String::new();
    let mut entries = Vec::new();
    let mut cases = String::from(
        "fn check(case: &Case) { match (case.bits, case.signed, case.order.as_str()) {\n",
    );
    let mut switches = String::new();
    let mut tag = 0;
    for bits in [8u16, 16, 32, 64, 128, 256] {
        for signed in [false, true] {
            if signed && bits == 256 {
                continue;
            }
            for (order, suffix) in [("little", "Le"), ("big", "Be")] {
                let native = format!("{}{bits}", if signed { "i" } else { "u" });
                let name = format!("{}{bits}{suffix}", if signed { "I" } else { "U" });
                let width = bits / 8;
                schema.push_str(&format!("array {name} [byte; {width}];\n"));
                native_schema.push_str(&format!(
                    "integer {name} = {native}{};\n",
                    suffix.to_ascii_lowercase()
                ));
                entries.push(json!({"type":name,"bits":bits,"signed":signed,"byte_order":order}));
                let vector = format!("vector {name}List <{name}>;\n");
                schema.push_str(&vector);
                native_schema.push_str(&vector);
                let value = if bits == 256 {
                    "decimal_u256(&case.value)".to_owned()
                } else {
                    format!("case.value.parse::<{native}>().unwrap()")
                };
                cases.push_str(&format!("({bits}, {signed}, {order:?}) => {{ let value = {value}; check_scalar!(case, {name}, {name}Reader, {name}List, from_{native}, try_to_{native}, value, {tag}, {width}); }},\n"));
                let (ctyp, read_compare) = if bits > 64 {
                    let typ = format!("mol_{}int{bits}_t", if signed { "" } else { "u" });
                    (typ, format!("for (size_t i=0;i<{};i++) if (value.limbs[i] != expected[i]) return -2;", bits/64))
                } else {
                    let typ = format!("{}int{bits}_t", if signed { "" } else { "u" });
                    let cmp = if signed {
                        "if ((uint64_t)(int64_t)value != expected[0]) return -2;"
                    } else {
                        "if ((uint64_t)value != expected[0]) return -2;"
                    };
                    (typ, cmp.to_owned())
                };
                switches.push_str(&format!(
                    r#"case {tag}: {{
    {ctyp} value;
    mol_errno error = MolReader_{name}_read_value(&input, &value);
    if (error != MOL_OK) return error;
    {read_compare}
    return MolBuilder_{name}_write_value(value, &output);
}}
"#
                ));
                tag += 1;
            }
        }
    }
    // A schema name can shadow the Rust prelude in the generated numeric vector.
    let containers = "array FnMut [byte; 1]; array Byte32 [byte; 32]; struct Pair { little: U64Le, big: I64Be, } table Invoice { amount: U64Be, opaque: Byte32, } vector Amounts <U64Be>;\n";
    schema.push_str(containers);
    native_schema.push_str(containers);
    let native_path = out.join("native/numbers.mol");
    fs::create_dir_all(native_path.parent().unwrap())?;
    fs::write(&native_path, native_schema)?;
    let schema_file = out.join("numbers.mol");
    let metadata_file = out.join("numbers.json");
    fs::write(&schema_file, schema)?;
    metadata(&schema_file, json!(entries), &metadata_file)?;
    cases.push_str("_ => panic!(\"unsupported test case\"), } }\n");
    fs::write(out.join("cases.rs"), cases)?;
    let ast = Compiler::new()
        .input_schema_file(&schema_file)
        .type_metadata_file(&metadata_file)
        .parse_ast()?;
    let native = Compiler::new()
        .input_schema_file(&native_path)
        .parse_ast()?;
    native.check_integer_contract(&ast)?;
    assert_eq!(
        IntermediateFormat::JSON.serialize_wire(&native)?,
        IntermediateFormat::JSON.serialize_wire(&ast)?
    );
    for (dir, language) in [
        ("packed", Language::Rust),
        ("lazy", Language::RustLazyReader),
        ("c", Language::C),
    ] {
        let mut compiler = Compiler::new();
        compiler
            .input_schema_file(&schema_file)
            .type_metadata_file(&metadata_file)
            .generate_code(language)
            .output_dir(out.join(dir))
            .run()?;
        let file = if dir == "c" {
            "numbers.h"
        } else {
            "numbers.rs"
        };
        Compiler::new()
            .input_schema_file(&native_path)
            .generate_code(language)
            .output_dir(out.join("recovered"))
            .run()?;
        assert_eq!(
            fs::read(out.join(dir).join(file))?,
            fs::read(out.join("recovered").join(file))?
        );
        for format in [IntermediateFormat::JSON, IntermediateFormat::YAML] {
            Compiler::new()
                .input_intermediate(format, format.serialize_ast(&ast)?)
                .generate_code(language)
                .output_dir(out.join("recovered"))
                .run()?;
            assert_eq!(
                fs::read(out.join(dir).join(file))?,
                fs::read(out.join("recovered").join(file))?
            );
        }
    }
    legacy_codegen::Compiler::new()
        .input_schema_file(&schema_file)
        .generate_code(legacy_codegen::Language::Rust)
        .output_dir(out.join("legacy"))
        .run()?;

    Compiler::new()
        .input_schema_file("fixtures/lazy_names.mol")
        .generate_code(Language::RustLazyReader)
        .output_dir(out.join("lazy"))
        .run()?;

    println!("cargo:rerun-if-changed=fixtures");
    let blockchain = Path::new("fixtures/blockchain.mol").canonicalize()?;
    println!("cargo:rerun-if-changed=schemas");
    for name in ["blockchain", "extensions", "protocols"] {
        Compiler::new()
            .input_schema_file(format!("schemas/{name}.mol"))
            .generate_code(Language::Rust)
            .output_dir(out.join("packed"))
            .run()?;
    }
    legacy_codegen::Compiler::new()
        .input_schema_file(&blockchain)
        .generate_code(legacy_codegen::Language::Rust)
        .output_dir(out.join("legacy"))
        .run()?;

    for (name, source) in [
        ("molecule_integer", "array Tag [byte; 1];"),
        (
            "invoice",
            "import molecule_integer; integer Uint64 = u64be; struct Charge { tag: Tag, amount: Uint64, }",
        ),
    ] {
        let schema = out.join(format!("{name}.mol"));
        fs::write(&schema, source)?;
        Compiler::new()
            .input_schema_file(schema)
            .generate_code(Language::C)
            .output_dir(out.join("c"))
            .run()?;
    }

    let bridge = out.join("bridge.c");
    fs::write(
        &bridge,
        format!(
            r#"#define MOLECULE_API_DECORATOR static
#include "c/numbers.h"
#include "c/invoice.h"
int check_c(unsigned int tag, const uint8_t *data, unsigned int len,
            const uint64_t *expected, uint8_t *out, unsigned int capacity) {{
    mol_seg_t input = {{(uint8_t *)data, len}}, output = {{out, capacity}};
    switch (tag) {{
        {switches}
        default: return -1;
    }}
}}
int narrow_c(int64_t signed_value, uint64_t unsigned_value, uint8_t *out) {{
    mol_seg_t output = {{out, 1}};
    return MolBuilder_I8Be_write_value(signed_value, &output)
        | MolBuilder_U8Le_write_value(unsigned_value, &output);
}}
"#
        ),
    )?;
    if env::var("HOST")? == env::var("TARGET")? {
        println!("cargo:rerun-if-changed=../../bindings/c/include");
        cc::Build::new()
            .file(&bridge)
            .include(&out)
            .include("../../bindings/c/include")
            .warnings(false)
            .compile("integer_bridge");
    }
    Ok(())
}
