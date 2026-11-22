#![cfg(feature = "schema-extensions")]
use molecule_codegen::{
    integer::{ByteOrder, IntegerSpec},
    schema::{source_digest, LoadedGraph},
    Compiler, Language,
};
use serde_json::{json, Value};
use std::{fs, path::Path};
use tempfile::TempDir;

fn write_metadata(dir: &Path, schema: &str, name: &str, integers: Value) -> std::path::PathBuf {
    let bytes = fs::read(dir.join(schema)).unwrap();
    let path = dir.join(name);
    fs::write(&path, serde_json::to_vec(&json!({"metadata_version":1,"schema":schema,"schema_sha256":source_digest(&bytes),"integers":integers})).unwrap()).unwrap();
    path
}

fn annotation(name: &str, bits: u16, signed: bool, byte_order: &str) -> Value {
    json!({"type":name,"bits":bits,"signed":signed,"byte_order":byte_order})
}

fn fixture() -> (TempDir, Compiler) {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("types.mol"), "array Amount [byte; 8]; array Byte32 [byte; 32]; array Nested [Amount; 1]; vector Values <Amount>;").unwrap();
    let metadata = write_metadata(
        dir.path(),
        "types.mol",
        "types.json",
        json!([annotation("Amount", 64, false, "big")]),
    );
    let mut compiler = Compiler::new();
    compiler
        .input_schema_file(dir.path().join("types.mol"))
        .type_metadata_file(metadata);
    (dir, compiler)
}

#[test]
fn metadata_is_explicit_and_opaque_arrays_stay_opaque() {
    let (dir, compiler) = fixture();
    let ast = compiler.parse_ast().unwrap();
    assert_eq!(
        ast.integer("Amount"),
        Some(&IntegerSpec {
            bits: 64,
            signed: false,
            byte_order: ByteOrder::Big
        })
    );
    assert!(ast.integer("Byte32").is_none());
    let plain = Compiler::new()
        .input_schema_file(dir.path().join("types.mol"))
        .parse_ast()
        .unwrap();
    assert!(plain.integers().is_empty());
    assert!(plain.source_graph().is_none());
}

#[test]
fn invalid_metadata_never_replaces_existing_output() {
    for (entry, expected) in [
        (annotation("Missing", 64, false, "little"), "E_INT_TARGET"),
        (annotation("Values", 64, false, "little"), "E_INT_TARGET"),
        (annotation("Nested", 64, false, "little"), "E_INT_TARGET"),
        (annotation("Amount", 32, false, "little"), "E_INT_WIDTH"),
        (annotation("Byte32", 256, true, "little"), "E_INT_WIDTH"),
        (annotation("Amount", 63, false, "little"), "E_INT_WIDTH"),
        (annotation("Amount", 64, false, "native"), "E_META_JSON"),
    ] {
        let (dir, mut compiler) = fixture();
        write_metadata(dir.path(), "types.mol", "types.json", json!([entry]));
        fs::write(dir.path().join("types.rs"), "existing output").unwrap();
        let error = compiler
            .generate_code(Language::Rust)
            .output_dir(dir.path())
            .run()
            .unwrap_err();
        assert!(error.contains(expected), "{error}");
        assert_eq!(
            fs::read_to_string(dir.path().join("types.rs")).unwrap(),
            "existing output"
        );
    }
}

#[test]
fn unknown_duplicate_missing_and_stale_metadata_are_rejected() {
    let (dir, compiler) = fixture();
    let path = dir.path().join("types.json");
    let valid = fs::read_to_string(&path).unwrap();
    let inputs = [
        valid.replacen(
            "\"metadata_version\":1",
            "\"metadata_version\":1,\"metadata_version\":1",
            1,
        ),
        valid.replacen("\"bits\":64", "\"bits\":64,\"bits\":64", 1),
        valid.replacen("\"bits\":64", "\"bits\":64,\"unknown\":true", 1),
        valid.replacen("\"signed\":false,", "", 1),
        valid.replacen("\"metadata_version\":1", "\"metadata_version\":2", 1),
        valid.replacen("\"schema_sha256\":\"", "\"schema_sha256\":\"0", 1),
    ];
    for input in inputs {
        fs::write(&path, input).unwrap();
        assert!(compiler.parse_ast().is_err());
    }
    write_metadata(
        dir.path(),
        "types.mol",
        "types.json",
        json!([
            annotation("Amount", 64, false, "big"),
            annotation("Amount", 64, false, "big")
        ]),
    );
    assert!(compiler
        .parse_ast()
        .unwrap_err()
        .contains("E_INT_DUPLICATE"));
}

#[test]
fn imported_semantics_belong_to_the_declaring_file() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("base.mol"), "array Amount [byte; 8];").unwrap();
    fs::write(
        dir.path().join("root.mol"),
        "import base; table Invoice { amount: Amount, }",
    )
    .unwrap();
    let wrong = write_metadata(
        dir.path(),
        "root.mol",
        "wrong.json",
        json!([annotation("Amount", 64, false, "big")]),
    );
    let mut compiler = Compiler::new();
    compiler
        .input_schema_file(dir.path().join("root.mol"))
        .type_metadata_file(wrong);
    assert!(compiler
        .parse_ast()
        .unwrap_err()
        .contains("not declared by"));
    let valid = write_metadata(
        dir.path(),
        "base.mol",
        "base.json",
        json!([annotation("Amount", 64, false, "big")]),
    );
    let ast = Compiler::new()
        .input_schema_file(dir.path().join("root.mol"))
        .type_metadata_file(valid)
        .parse_ast()
        .unwrap();
    assert_eq!(
        ast.source_graph().unwrap().owner("Amount").unwrap().id,
        "base.mol"
    );
}

#[test]
fn source_graph_distinguishes_cycles_from_shared_dependencies() {
    let dir = TempDir::new().unwrap();
    for (name, source) in [
        ("root", "import b; import c; table Root { b: B, c: C, }"),
        ("b", "import d; table B { d: D, }"),
        ("c", "import d; table C { d: D, }"),
        ("d", "array D [byte; 8];"),
    ] {
        fs::write(dir.path().join(format!("{name}.mol")), source).unwrap();
    }
    let root = dir.path().join("root.mol");
    let graph = LoadedGraph::load(&root).unwrap().graph().clone();
    assert_eq!(graph.modules.len(), 4);
    assert_eq!(
        graph
            .modules
            .iter()
            .find(|module| module.id == "b.mol")
            .unwrap()
            .imports[0]
            .module,
        "d.mol"
    );
    fs::write(dir.path().join("d.mol"), "import b; array D [byte; 8];").unwrap();
    let error = LoadedGraph::load(&root).err().unwrap();
    assert!(error.contains("E_MODULE_CYCLE"));
    assert!(error.contains("b.mol -> d.mol -> b.mol"));
}

#[cfg(feature = "compiler-plugin")]
#[test]
fn v2_roundtrips_without_changing_the_legacy_wire_ir() {
    use molecule_codegen::IntermediateFormat as Format;
    let (dir, compiler) = fixture();
    let ast = compiler.parse_ast().unwrap();
    let plain = Compiler::new()
        .input_schema_file(dir.path().join("types.mol"))
        .parse_ast()
        .unwrap();
    for format in [Format::JSON, Format::YAML] {
        let encoded = format.serialize_ast(&ast).unwrap();
        let recovered = format.recover(&encoded).unwrap();
        assert_eq!(recovered.integers(), ast.integers());
        assert_eq!(recovered.source_graph(), ast.source_graph());
        assert_eq!(format.serialize_ast(&recovered).unwrap(), encoded);
        assert_eq!(
            format.serialize_wire(&ast).unwrap(),
            format.serialize_ast(&plain).unwrap()
        );
    }
}

#[cfg(feature = "compiler-plugin")]
#[test]
fn v2_recovery_revalidates_layout_features_and_ownership() {
    use molecule_codegen::IntermediateFormat as Format;
    let (_dir, compiler) = fixture();
    let ast = compiler.parse_ast().unwrap();
    let valid: Value = serde_json::from_slice(&Format::JSON.serialize_ast(&ast).unwrap()).unwrap();
    let replacements = [
        ("/ir_version", json!(3)),
        ("/required_features", json!([])),
        ("/required_features", json!(["unknown"])),
        ("/logical_types/0/bits", json!(32)),
        ("/logical_types/0/schema", json!("elsewhere.mol")),
        ("/logical_types/0/name", json!("Values")),
        ("/wire_ir/declarations/0/item_count", json!(0)),
        ("/wire_ir/declarations/0/item_count", json!(u64::MAX)),
        ("/wire_ir/declarations/0/item", json!("Missing")),
        ("/sources/modules/0/declarations/0/name", json!("Fake")),
    ];
    for (path, replacement) in replacements {
        let mut malformed = valid.clone();
        *malformed.pointer_mut(path).unwrap() = replacement;
        let encoded = serde_json::to_vec(&malformed).unwrap();
        assert!(Format::JSON.recover(&encoded).is_err(), "accepted {path}");
    }
    let json = String::from_utf8(Format::JSON.serialize_ast(&ast).unwrap()).unwrap();
    assert!(Format::JSON
        .recover(
            json.replacen("\"bits\": 64", "\"bits\": 64, \"bits\": 32", 1)
                .as_bytes()
        )
        .is_err());
}

#[cfg(feature = "compiler-plugin")]
#[test]
fn capabilities_reject_inexact_plugins() {
    use molecule_codegen::{ir::Capabilities, IntermediateFormat};
    let (_dir, compiler) = fixture();
    let ast = compiler.parse_ast().unwrap();
    let mut capabilities = Capabilities::builtin(Language::Rust, IntermediateFormat::JSON);
    assert!(capabilities.require(&ast).is_ok());
    capabilities
        .integers
        .retain(|spec| spec.byte_order != ByteOrder::Big);
    assert!(capabilities.require(&ast).unwrap_err().contains("Amount"));
    capabilities.ir_versions = vec![1];
    assert!(capabilities.require(&ast).is_err());
}

#[test]
fn source_snapshot_is_immutable_and_rejects_invalid_layouts_without_panics() {
    let (dir, _) = fixture();
    let path = dir.path().join("types.mol");
    let loaded = LoadedGraph::load(&path).unwrap();
    fs::write(&path, "array Changed [byte; 1];").unwrap();
    let snapshot = loaded.parse_ast().unwrap();
    use molecule_codegen::ast::HasName;
    assert!(snapshot
        .major_decls()
        .iter()
        .any(|decl| decl.name() == "Amount"));
    assert!(!snapshot
        .major_decls()
        .iter()
        .any(|decl| decl.name() == "Changed"));
    for source in [
        "array A [byte; 18446744073709551616];",
        "array A [Missing; 1];",
        "array A [byte; 4294967295]; array B [A; 2];",
        "union A {byte: 4294967295, byte,}",
        "struct A { a: byte, a: byte, }",
    ] {
        fs::write(&path, source).unwrap();
        assert!(LoadedGraph::load(&path).unwrap().parse_ast().is_err());
    }
}

#[cfg(unix)]
#[test]
fn symlink_imports_use_the_original_spelling_and_hardlinks_share_identity() {
    use std::os::unix::fs::symlink;
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join("alias")).unwrap();
    fs::create_dir(dir.path().join("real")).unwrap();
    fs::write(
        dir.path().join("real/root.mol"),
        "import base; table Root { value: Amount, }",
    )
    .unwrap();
    fs::write(dir.path().join("real/base.mol"), "array Wrong [byte; 4];").unwrap();
    fs::write(dir.path().join("alias/base.mol"), "array Amount [byte; 8];").unwrap();
    symlink("../real/root.mol", dir.path().join("alias/input.mol")).unwrap();
    let loaded = LoadedGraph::load(&dir.path().join("alias/input.mol")).unwrap();
    assert!(loaded.graph().owner("Amount").is_some());
    assert!(loaded.graph().owner("Wrong").is_none());
    assert_eq!(loaded.parse_ast().unwrap().namespace(), "input");
    fs::hard_link(
        dir.path().join("alias/base.mol"),
        dir.path().join("alias/other.mol"),
    )
    .unwrap();
    fs::write(
        dir.path().join("alias/root.mol"),
        "import base; import other; table Root { value: Amount, }",
    )
    .unwrap();
    let loaded = LoadedGraph::load(&dir.path().join("alias/root.mol")).unwrap();
    assert_eq!(loaded.graph().modules.len(), 2);
    assert_eq!(
        loaded
            .module_at(&dir.path().join("alias/other.mol"))
            .unwrap()
            .id,
        "base.mol"
    );
    assert_eq!(loaded.parse_ast().unwrap().imports().len(), 1);
    fs::write(
        dir.path().join("alias/root.mol"),
        "import other; import base; table Root { value: Amount, }",
    )
    .unwrap();
    let reordered = LoadedGraph::load(&dir.path().join("alias/root.mol")).unwrap();
    assert_eq!(reordered.graph().owner("Amount").unwrap().id, "base.mol");
}

#[test]
fn reordering_imports_keeps_the_published_integer_owner() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("root.mol");
    fs::write(dir.path().join("shared.mol"), "integer Amount = u64le;").unwrap();
    for (name, ty) in [("left", "Left"), ("right", "Right")] {
        let folder = dir.path().join(name);
        fs::create_dir(&folder).unwrap();
        fs::write(
            folder.join(format!("{name}.mol")),
            format!("import ../shared; struct {ty} {{ amount: Amount, }}"),
        )
        .unwrap();
    }
    let parse = |imports: &str| {
        fs::write(
            &root,
            format!("{imports} table Invoice {{ left: Left, right: Right, }}"),
        )
        .unwrap();
        Compiler::new()
            .input_schema_file(&root)
            .parse_ast()
            .unwrap()
    };
    let published = parse("import left/left; import right/right;");
    let reordered = parse("import right/right; import left/left;");
    reordered.check_integer_contract(&published).unwrap();
    assert_eq!(
        reordered
            .source_graph()
            .unwrap()
            .owner("Amount")
            .unwrap()
            .id,
        "shared.mol"
    );
}

#[cfg(feature = "compiler-plugin")]
#[test]
fn v2_preserves_filename_and_revalidates_import_edges() {
    use molecule_codegen::IntermediateFormat as Format;
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("types-v2.mol"),
        "import base; table Root { value: Amount, }",
    )
    .unwrap();
    fs::write(dir.path().join("base.mol"), "array Amount [byte; 8];").unwrap();
    let meta = write_metadata(
        dir.path(),
        "base.mol",
        "base.json",
        json!([annotation("Amount", 64, false, "little")]),
    );
    let ast = Compiler::new()
        .input_schema_file(dir.path().join("types-v2.mol"))
        .type_metadata_file(meta)
        .parse_ast()
        .unwrap();
    for format in [Format::JSON, Format::YAML] {
        let data = format.serialize_ast(&ast).unwrap();
        assert_eq!(format.recover(&data).unwrap().namespace(), "types-v2");
    }
    let mut data: Value =
        serde_json::from_slice(&Format::JSON.serialize_ast(&ast).unwrap()).unwrap();
    data["wire_ir"]["imports"] = json!([]);
    assert!(Format::JSON
        .recover(&serde_json::to_vec(&data).unwrap())
        .unwrap_err()
        .contains("wire imports"));
}

#[test]
fn output_names_preserve_the_complete_schema_stem() {
    let dir = TempDir::new().unwrap();
    for (name, language, extension) in [
        ("types.v2", Language::Rust, "rs"),
        ("molecule_integer", Language::C, "h"),
    ] {
        let path = dir.path().join(format!("{name}.mol"));
        fs::write(&path, "integer Amount = u64le;").unwrap();
        Compiler::new()
            .input_schema_file(path)
            .generate_code(language)
            .output_dir(dir.path())
            .run()
            .unwrap();
        assert!(dir.path().join(format!("{name}.{extension}")).is_file());
    }
}

#[test]
fn lazy_numeric_names_preserve_their_existing_return_types() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("types.mol");
    let output = dir.path().join("types.rs");
    for declaration in ["integer Uint64 = i64be;", "integer Int32 = i16le;"] {
        fs::write(&path, declaration).unwrap();
        fs::write(&output, "existing").unwrap();
        let error = Compiler::new()
            .input_schema_file(&path)
            .generate_code(Language::RustLazyReader)
            .output_dir(dir.path())
            .run()
            .unwrap_err();
        assert!(error.contains("E_INT_LAZY_TYPE"), "{error}");
        assert_eq!(fs::read_to_string(&output).unwrap(), "existing");
    }
}

#[test]
fn native_types_share_the_metadata_width_rule_and_keep_contextual_names() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("types.mol");
    let mut compiler = Compiler::new();
    compiler.input_schema_file(&path);
    fs::write(
        &path,
        "integer integer = i8; array u32le [byte; 4]; struct Word { integer: integer, }",
    )
    .unwrap();
    let ast = compiler.parse_ast().unwrap();
    assert_eq!(ast.integer("integer").unwrap().bits, 8);
    assert!(ast.integer("u32le").is_none());
    for (literal, diagnostic) in [
        ("u32", "E_INT_TYPE"),
        ("i256be", "E_INT_WIDTH"),
        ("u24le", "E_INT_WIDTH"),
        ("u032le", "E_INT_TYPE"),
        ("word", "E_INT_TYPE"),
    ] {
        fs::write(&path, format!("integer Amount = {literal};")).unwrap();
        let error = compiler.parse_ast().unwrap_err();
        assert!(
            error.contains(diagnostic) && error.contains("types.mol"),
            "{error}"
        );
    }
    fs::write(&path, "integer Amount = u64be;").unwrap();
    let metadata = write_metadata(
        dir.path(),
        "types.mol",
        "types.json",
        json!([annotation("Amount", 64, false, "big")]),
    );
    assert!(compiler
        .type_metadata_file(metadata)
        .parse_ast()
        .unwrap_err()
        .contains("E_INT_DUPLICATE"));
}

#[cfg(feature = "compiler-plugin")]
#[test]
fn published_contract_accepts_migration_and_rejects_drift_before_writing() {
    use molecule_codegen::IntermediateFormat as Format;
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("root.mol");
    let owner = dir.path().join("keys.mol");
    let fields = "import keys; array Byte32 [byte; 32];";
    fs::write(&root, fields).unwrap();
    fs::write(&owner, "array Key [byte; 4];").unwrap();
    let metadata = write_metadata(
        dir.path(),
        "keys.mol",
        "keys.json",
        json!([annotation("Key", 32, false, "big")]),
    );
    let old = Compiler::new()
        .input_schema_file(&root)
        .type_metadata_file(&metadata)
        .parse_ast()
        .unwrap();
    let path = dir.path().join("published.json");
    fs::write(&path, Format::JSON.serialize_ast(&old).unwrap()).unwrap();
    let mut compiler = Compiler::new();
    compiler
        .input_schema_file(&root)
        .integer_contract_file(&path)
        .generate_code(Language::Rust)
        .output_dir(dir.path());
    fs::write(&owner, "integer Key = u32be;").unwrap();
    let migrated = compiler.parse_ast().unwrap();
    assert_eq!(
        Format::JSON.serialize_wire(&migrated).unwrap(),
        Format::JSON.serialize_wire(&old).unwrap()
    );
    compiler.run().unwrap();
    let output = dir.path().join("root.rs");
    let before = fs::read(&output).unwrap();
    for (source, error) in [
        ("array Key [byte; 4];", "missing integer Key"),
        ("integer Key = u32le;", "changed integer Key"),
        ("integer Key = i32be;", "changed integer Key"),
        ("integer Key = u64be;", "changed integer Key"),
        (
            "integer Key = u32be; integer Fee = u64le;",
            "added integer Fee",
        ),
    ] {
        fs::write(&owner, source).unwrap();
        let message = compiler.run().unwrap_err();
        assert!(
            message.contains("E_INT_CONTRACT") && message.contains(error),
            "{message}"
        );
        assert_eq!(fs::read(&output).unwrap(), before);
    }
    fs::write(&owner, "array Opaque [byte; 4];").unwrap();
    fs::write(&root, format!("{fields} integer Key = u32be;")).unwrap();
    assert!(compiler.run().unwrap_err().contains("changed owner of Key"));
}

#[test]
fn native_and_sidecar_declarations_can_share_an_import_graph() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("root.mol");
    fs::write(dir.path().join("keys.mol"), "integer Index = u32be;").unwrap();
    fs::write(&root, "import keys; array Height [byte; 8];").unwrap();
    let metadata = write_metadata(
        dir.path(),
        "root.mol",
        "root.json",
        json!([annotation("Height", 64, false, "little")]),
    );
    let ast = Compiler::new()
        .input_schema_file(root)
        .type_metadata_file(metadata)
        .parse_ast()
        .unwrap();
    assert_eq!(ast.integers().len(), 2);
    assert_eq!(ast.integer("Index").unwrap().byte_order, ByteOrder::Big);
    assert_eq!(ast.integer("Height").unwrap().byte_order, ByteOrder::Little);
}

#[test]
fn provenance_uses_character_columns_across_crlf_and_unicode_comments() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("types.mol");
    fs::write(
        &path,
        "// 第一行\r\n/* π */ integer Key = u32be;\r\n  array Raw [byte; 4];",
    )
    .unwrap();
    let graph = LoadedGraph::load(&path).unwrap();
    let declarations = &graph.graph().modules[0].declarations;
    assert_eq!((declarations[0].line, declarations[0].column), (2, 9));
    assert_eq!((declarations[1].line, declarations[1].column), (3, 3));
}
