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
    for dir in [
        "packed",
        "lazy",
        "c",
        "legacy",
        "recovered",
        "reg_mol/common",
        "reg_packed/common",
        "reg_lazy/common",
        "reg_mol/pkg",
        "reg_packed/pkg",
        "reg_lazy/pkg",
        "c/a",
        "c/b",
    ] {
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
    for bits in [8u16, 16, 32, 64, 128] {
        let orders: &[(&str, &str)] = if bits == 8 {
            &[("little", "")]
        } else {
            &[("little", "Le"), ("big", "Be")]
        };
        for signed in [false, true] {
            for &(order, suffix) in orders {
                let native = format!("{}{bits}", if signed { "i" } else { "u" });
                let name = format!("{}{bits}{suffix}", if signed { "I" } else { "U" });
                let literal = if order == "big" {
                    format!("{native}be")
                } else {
                    native.clone()
                };
                let width = bits / 8;
                schema.push_str(&format!("array {name} [byte; {width}];\n"));
                native_schema.push_str(&format!("integer {name} = {literal};\n"));
                entries.push(json!({"type":name,"bits":bits,"signed":signed,"byte_order":order}));
                let vector = format!("vector {name}List <{name}>;\n");
                schema.push_str(&vector);
                native_schema.push_str(&vector);
                let value = format!("case.value.parse::<{native}>().unwrap()");
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

    for (dir, language) in [
        ("packed", Language::Rust),
        ("lazy", Language::RustLazyReader),
    ] {
        Compiler::new()
            .input_schema_file("fixtures/lazy_names.mol")
            .generate_code(language)
            .output_dir(out.join(dir))
            .run()?;
    }

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
    Compiler::new()
        .input_schema_file("schemas/blockchain.mol")
        .generate_code(Language::RustLazyReader)
        .output_dir(out.join("lazy"))
        .run()?;
    legacy_codegen::Compiler::new()
        .input_schema_file(&blockchain)
        .generate_code(legacy_codegen::Language::Rust)
        .output_dir(out.join("legacy"))
        .run()?;

    let reg_schemas: &[(&str, &str)] = &[
        ("common/defs", "integer DefVal = u32;"),
        ("leaf", "integer LeafAmt = u64;"),
        ("mid", "import leaf; table Mid { a: LeafAmt, }"),
        ("mod_a", "integer FeeRate = u32;"),
        ("mod_b", "integer fee_rate = u32be;"),
        (
            "imp_row",
            "integer RowLe = u16; struct FeeRow { v: RowLe, } option OptFeeRow (FeeRow);",
        ),
        (
            "consumer",
            concat!(
                "import common/defs;\n",
                "import mid;\n",
                "import mod_a;\n",
                "import mod_b;\n",
                "import imp_row;\n",
                "integer RowBe = u16be;\n",
                "struct fee_row { v: RowBe, }\n",
                "struct CaseFields { feeRate: byte, feerate: byte, }\n",
                "table Ok {}\n",
                "table Err {}\n",
                "table Option {}\n",
                "table Result {}\n",
                "table Cursor {}\n",
                "table Error {}\n",
                "table Vec {}\n",
                "table From {}\n",
                "struct ShadowS { as_slice: byte, build: byte, }\n",
                "table Reader {} table Entity {} table Builder {}\n",
                "struct Value { as_builder: byte, into: byte, } vector Values <Value>; option MaybeValue (Value);\n",
                "integer T = u16; struct Row { default: T, } array Rows [Row; 2]; vector Amounts <T>;\n",
                "struct Rates { little: FeeRate, big: fee_rate, }\n",
                "vector Bytes <byte>; option OptBytes (Bytes); option OptByte (byte);\n",
                "option OptRate (FeeRate); option NestedRate (OptRate);\n",
                "union Inner { Error, byte, } option OptInner (Inner);\n",
                "union Payload { OptByte, OptBytes, Inner, OptInner, NestedRate, Value, }\n",
                "table ShadowT { as_slice: byte, ok: Ok, err: Err, cur: Cursor, }\n",
                "table Combined { def: DefVal, leaf: LeafAmt, rate: FeeRate, opt_row: OptFeeRow, local_row: fee_row, cases: CaseFields, }\n",
                "union MixedUnion { LeafAmt, FeeRate, OptFeeRow, }\n",
            ),
        ),
        (
            "plain_leaf",
            "vector PlainBytes <byte>; table PlainLeaf { tag: byte, data: PlainBytes, }",
        ),
        (
            "plain_child",
            concat!(
                "import plain_leaf;\n",
                "struct PlainPair { a: byte, b: byte, }\n",
                "vector PlainPairVec <PlainPair>;\n",
                "option OptPlainPairVec (PlainPairVec);\n",
                "vector PlainLeafVec <PlainLeaf>;\n",
                "option OptPlainLeaf (PlainLeaf);\n",
                "union PlainUnion { PlainLeaf, byte, }\n",
                "option OptPlainUnion (PlainUnion);\n",
                "table PlainChild { a: byte, leaf: PlainLeaf, list: PlainLeafVec, opt: OptPlainLeaf, u: PlainUnion, }\n",
                "table PlainUnused { x: byte, }\n",
            ),
        ),
        (
            "plain_root",
            concat!(
                "import plain_child;\n",
                "integer RootAmt = u64;\n",
                "option UnusedOpt (PlainChild);\n",
                "vector PlainUnionVec <PlainUnion>;\n",
                "union LocalWrap { PlainUnion, PlainPairVec, }\n",
                "table PlainRoot { amount: RootAmt, child: PlainChild, list: PlainLeafVec, u: PlainUnion, opt: OptPlainLeaf, wrap: LocalWrap, }\n",
                "union PlainRootUnion { RootAmt, PlainChild, OptPlainLeaf, PlainLeafVec, PlainPairVec, OptPlainPairVec, PlainUnion, OptPlainUnion, }\n",
            ),
        ),
        ("pkg/internal", "table PrivateInner { a: byte, }"),
        (
            "pkg/outer",
            "import internal;\nunion PublicOuter { PrivateInner, byte, }\n",
        ),
        (
            "pkg_root",
            "import pkg/outer;\ninteger RootAmt = u64;\ntable PublicRoot { amount: RootAmt, outer: PublicOuter, }\n",
        ),
    ];
    for &(rel, src) in reg_schemas {
        fs::write(out.join("reg_mol").join(format!("{rel}.mol")), src)?;
    }
    for &(rel, _) in reg_schemas {
        let schema_path = out.join("reg_mol").join(format!("{rel}.mol"));
        let subdir = Path::new(rel).parent().unwrap_or_else(|| Path::new(""));
        for (root_dir, language) in [
            ("reg_packed", Language::Rust),
            ("reg_lazy", Language::RustLazyReader),
        ] {
            Compiler::new()
                .input_schema_file(&schema_path)
                .generate_code(language)
                .output_dir(out.join(root_dir).join(subdir))
                .run()?;
        }
    }

    for (name, source) in [
        ("molecule_integer", "array Tag [byte; 1];"),
        (
            "invoice",
            "import molecule_integer; integer Uint64 = u64be; struct Charge { tag: Tag, amount: Uint64, }",
        ),
        ("a/base", "integer ABaseVal = u32;"),
        ("b/base", "integer BBaseVal = u64be;"),
        ("b/bridge", "import base; struct BDetail { amount: BBaseVal, }"),
        (
            "base_root",
            "import a/base; import b/bridge; struct CaseFieldsC { feeRate: byte, feerate: byte, } table BasePair { a: ABaseVal, b: BBaseVal, } table NoFields {}",
        ),
    ] {
        let schema = out.join(format!("{name}.mol"));
        if let Some(parent) = schema.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&schema, source)?;
        let out_subdir = Path::new(name).parent().unwrap_or_else(|| Path::new(""));
        Compiler::new()
            .input_schema_file(schema)
            .generate_code(Language::C)
            .output_dir(out.join("c").join(out_subdir))
            .run()?;
    }

    let bridge = out.join("bridge.c");
    fs::write(
        &bridge,
        format!(
            r#"#define MOLECULE_API_DECORATOR static
#include "c/numbers.h"
#include "c/invoice.h"
#include "c/base_root.h"
int check_c(unsigned int tag, const uint8_t *data, unsigned int len,
            const uint64_t *expected, uint8_t *out, unsigned int capacity) {{
    mol_seg_t input = {{(uint8_t *)data, len}}, output = {{out, capacity}};
    switch (tag) {{
        {switches}
        default: return -1;
    }}
}}
int narrow_i8_c(int64_t value, uint8_t *out) {{
    mol_seg_t output = {{out, 1}};
    return MolBuilder_I8_write_value(value, &output);
}}
int narrow_u8_c(uint64_t value, uint8_t *out) {{
    mol_seg_t output = {{out, 1}};
    return MolBuilder_U8_write_value(value, &output);
}}
int check_c_imports_and_case_fields(void) {{
    uint8_t a_bytes[4] = {{0x01, 0x02, 0x03, 0x04}};
    uint8_t b_bytes[8] = {{0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x02}};
    mol_seg_t a_seg = {{a_bytes, 4}}, b_seg = {{b_bytes, 8}};
    uint32_t a_val = 0;
    uint64_t b_val = 0;
    if (MolReader_ABaseVal_read_value(&a_seg, &a_val) != MOL_OK || a_val != 0x04030201u) return -1;
    if (MolReader_BBaseVal_read_value(&b_seg, &b_val) != MOL_OK || b_val != 0x0102ull) return -2;
    uint8_t cf_bytes[2] = {{4, 9}};
    mol_seg_t cf_seg = {{cf_bytes, 2}};
    if (*MolReader_CaseFieldsC_get_feeRate(&cf_seg).ptr != 4) return -3;
    if (*MolReader_CaseFieldsC_get_feerate(&cf_seg).ptr != 9) return -4;
    uint8_t table[34] = {{34,0,0,0, 20,0,0,0, 24,0,0,0, 32,0,0,0, 32,0,0,0}};
    mol_seg_t row = {{table, sizeof(table)}};
    if (MolReader_BasePair_verify(&row, true) != MOL_OK || MolReader_NoFields_verify(&row, true) != MOL_OK) return -5;
    if (MolReader_BasePair_verify(&row, false) != MOL_ERR_FIELD_COUNT) return -6;
    table[16] = 31;
    if (MolReader_BasePair_verify(&row, true) != MOL_ERR_OFFSET || MolReader_NoFields_verify(&row, true) != MOL_ERR_OFFSET) return -7;
    table[12] = 31;
    if (MolReader_BasePair_verify(&row, true) != MOL_ERR_DATA) return -8;
    return 0;
}}
"#
        ),
    )?;
    if env::var("HOST")? == env::var("TARGET")? {
        println!("cargo:rerun-if-changed=../../bindings/c/include");
        let plain = out.join("shared.mol");
        fs::write(&plain, "array Amount [byte; 2];")?;
        legacy_codegen::Compiler::new()
            .input_schema_file(&plain)
            .generate_code(legacy_codegen::Language::C)
            .output_dir(out.join("legacy"))
            .run()?;
        fs::write(
            &plain,
            "array Amount [byte; 2]; struct Pair { amount: Amount, }",
        )?;
        Compiler::new()
            .input_schema_file(&plain)
            .generate_code(Language::C)
            .output_dir(out.join("packed"))
            .run()?;
        let sidecar = out.join("shared.json");
        metadata(
            &plain,
            json!([{"type":"Amount","bits":16,"signed":false,"byte_order":"big"}]),
            &sidecar,
        )?;
        Compiler::new()
            .input_schema_file(&plain)
            .type_metadata_file(sidecar)
            .generate_code(Language::C)
            .output_dir(out.join("c"))
            .run()?;
        let additional = out.join("additional");
        fs::create_dir_all(&additional)?;
        fs::write(
            &plain,
            "array Amount [byte; 2]; struct Extra { value: Amount, }",
        )?;
        Compiler::new()
            .input_schema_file(&plain)
            .generate_code(Language::C)
            .output_dir(additional)
            .run()?;
        let wide = out.join("wide");
        fs::create_dir_all(&wide)?;
        fs::write(&plain, "integer Amount = u32;")?;
        Compiler::new()
            .input_schema_file(&plain)
            .generate_code(Language::C)
            .output_dir(wide)
            .run()?;

        for (dir, source) in [
            ("swapped", "struct Amount { nth1: byte, nth0: byte, }"),
            (
                "wrapped",
                "struct Item { inner: byte, } array Amount [Item; 2];",
            ),
            ("aggregate", "struct Pair { other: byte, amount: byte, }"),
        ] {
            fs::create_dir_all(out.join(dir))?;
            fs::write(&plain, source)?;
            legacy_codegen::Compiler::new()
                .input_schema_file(&plain)
                .generate_code(legacy_codegen::Language::C)
                .output_dir(out.join(dir))
                .run()?;
        }

        let mut build = cc::Build::new();
        // The 0.9.2 bridge includes b/base.h after the current a/base.h.
        let mixed = out.join("mixed");
        for dir in ["a", "b"] {
            fs::create_dir_all(mixed.join(dir))?;
        }
        fs::copy(out.join("c/a/base.h"), mixed.join("a/base.h"))?;
        fs::copy(out.join("c/base_root.h"), mixed.join("base_root.h"))?;
        for (name, schema) in [
            ("base", "array BBaseVal [byte; 8];"),
            (
                "bridge",
                "import base; struct BDetail { amount: BBaseVal, }",
            ),
        ] {
            let input = mixed.join(format!("b/{name}.mol"));
            fs::write(&input, schema)?;
            legacy_codegen::Compiler::new()
                .input_schema_file(input)
                .generate_code(legacy_codegen::Language::C)
                .output_dir(mixed.join("b"))
                .run()?;
        }
        let source = mixed.join("check.c");
        fs::write(&source, "#define MOLECULE_API_DECORATOR static\n#include \"base_root.h\"\nsize_t mixed_b_size(void) { return sizeof(MolDefault_BBaseVal); }\n")?;
        build.file(source);

        for (i, headers) in [
            ["legacy", "packed", "c"],
            ["c", "packed", "c"],
            ["packed", "c", "packed"],
            ["legacy", "c", "packed"],
        ]
        .iter()
        .enumerate()
        {
            let source = out.join(format!("include_order_{i}.c"));
            let includes: String = headers
                .iter()
                .map(|dir| format!("#include \"{dir}/shared.h\"\n"))
                .collect();
            fs::write(&source, format!("#define MOLECULE_API_DECORATOR static\n{includes}mol_errno included_{i}(mol_seg_t *s, uint16_t *v) {{ return sizeof(MolDefault_Pair) == 2 ? MolReader_Amount_read_value(s, v) : MOL_ERR; }}\n"))?;
            build.file(source);
        }
        build
            .file(&bridge)
            .include(&out)
            .include("../../bindings/c/include")
            .warnings(false)
            .compile("integer_bridge");
    }
    Ok(())
}
