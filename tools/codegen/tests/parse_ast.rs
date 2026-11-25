use molecule_codegen::Compiler;
use std::fs;

#[test]
fn detached_imports_use_the_shortest_distance_from_the_root() {
    use molecule_codegen::ast::TopDecl;
    let dir = tempfile::tempdir().unwrap();
    for (name, source) in [
        ("leaf", "struct fee_rate { n: byte, }"),
        ("mid", "import leaf; table Mid {}"),
        ("left", "struct FeeRate { n: byte, }"),
    ] {
        fs::write(dir.path().join(format!("{name}.mol")), source).unwrap();
    }
    let root = dir.path().join("root.mol");
    for (imports, field, depth) in [
        ("", "FeeRate", 2),
        ("", "fee_rate", 2),
        ("import leaf;", "fee_rate", 1),
    ] {
        fs::write(
            &root,
            format!("import mid; import left; {imports} table Root {{ value: {field}, }}"),
        )
        .unwrap();
        let mut compiler = Compiler::new();
        compiler.input_schema_file(&root);
        let ast = compiler.parse_ast().unwrap();
        let depth_of = |name| {
            ast.decls().iter().find_map(|d| match d.as_ref() {
                TopDecl::Struct(s) if s.name() == name => Some(s.imported_depth()),
                _ => None,
            })
        };
        assert_eq!(depth_of("FeeRate"), Some(1));
        assert_eq!(depth_of("fee_rate"), Some(depth));
        #[cfg(feature = "compiler-plugin")]
        {
            use molecule_codegen::{IntermediateFormat::JSON, Language};
            let wire = JSON.serialize_wire(&ast).unwrap();
            let sources = compiler.parse_ast_with_sources().unwrap();
            let mut old: serde_json::Value =
                serde_json::from_slice(&JSON.serialize_ast(&sources).unwrap()).unwrap();
            for decl in old["wire_ir"]["declarations"].as_array_mut().unwrap() {
                if decl.get("imported_depth").is_some() {
                    decl["imported_depth"] = 1.into();
                }
            }
            let recovered = JSON.recover(&serde_json::to_vec(&old).unwrap()).unwrap();
            assert_eq!(JSON.serialize_wire(&recovered).unwrap(), wire);
            for lang in [Language::Rust, Language::RustLazyReader] {
                let result = Compiler::new()
                    .input_intermediate(JSON, wire.clone())
                    .generate_code(lang)
                    .output_dir(dir.path())
                    .run();
                if field == "FeeRate" {
                    result.unwrap();
                } else {
                    let error = result.unwrap_err();
                    assert!(
                        error.contains("E_RUST_TYPE_COLLISION") && error.contains(field),
                        "{error}"
                    );
                }
            }
        }
    }
}

#[test]
fn missing_schema_and_import_report_the_missing_path() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("root.mol");
    let mut compiler = Compiler::new();
    compiler.input_schema_file(&root);
    let err = compiler.parse_ast().unwrap_err();
    assert!(err.contains("root.mol"), "{err}");

    fs::write(&root, "import child; table Root {}").unwrap();
    fs::write(
        dir.path().join("child.mol"),
        "import absent; table Child {}",
    )
    .unwrap();
    let err = compiler.parse_ast().unwrap_err();
    assert!(err.contains("absent.mol"), "{err}");

    fs::write(dir.path().join("absent.mol"), "table Present {}").unwrap();
    assert_eq!(compiler.parse_ast().unwrap().decls().len(), 3);

    // Non-root import cycles are rejected under both default and extended features.
    fs::write(
        dir.path().join("absent.mol"),
        "import child; table Present {}",
    )
    .unwrap();
    let err = compiler.parse_ast().unwrap_err();
    assert!(
        err.contains("CYCLE"),
        "expected cycle diagnostic, got: {err}"
    );
}

#[test]
fn invalid_layouts_return_diagnostics_with_and_without_integers() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("root.mol");
    for prefix in ["", "integer Amount = u64;"] {
        for (schema, names) in [
            ("table Order { price: Pric, }", &["Order", "Pric"][..]),
            ("option A (B); option B (A);", &["A", "B"]),
            (
                "vector Bytes <byte>; struct Record { data: Bytes, }",
                &["Record"],
            ),
            ("struct Dup { a: byte, a: byte, }", &["Dup", "a"]),
            ("table DupTable { x: byte, x: byte, }", &["DupTable", "x"]),
        ] {
            fs::write(&root, format!("{prefix} {schema}")).unwrap();
            let err = Compiler::new()
                .input_schema_file(&root)
                .parse_ast()
                .unwrap_err();
            for name in names {
                assert!(err.contains(name), "{schema}: {err}");
            }
        }
    }
}

#[cfg(feature = "compiler-plugin")]
#[test]
fn legacy_ir_keeps_explicit_vector_kinds_and_reports_unresolved_types() {
    use molecule_codegen::{ast::TopDecl, IntermediateFormat::JSON};
    let ir = br#"{"namespace":"sample","imports":[],"declarations":[{"type":"dynvec","name":"Items","item":"byte"}]}"#;
    let ast = Compiler::new()
        .input_intermediate(JSON, ir.to_vec())
        .parse_ast()
        .unwrap();
    assert!(matches!(ast.decls()[0].as_ref(), TopDecl::DynVec(_)));

    let ir = br#"{"namespace":"sample","imports":[],"declarations":[{"type":"option","name":"Order","item":"MissingAmount"}]}"#;
    let err = Compiler::new()
        .input_intermediate(JSON, ir.to_vec())
        .parse_ast()
        .unwrap_err();
    assert!(
        err.contains("Order") && err.contains("MissingAmount"),
        "{err}"
    );

    // Zero-length arrays in legacy v1 IR are rejected cleanly without panicking.
    let zero_arr = br#"{"namespace":"sample","imports":[],"declarations":[{"type":"array","name":"Zero","item":"byte","item_count":0}]}"#;
    let err = Compiler::new()
        .input_intermediate(JSON, zero_arr.to_vec())
        .parse_ast()
        .unwrap_err();
    assert!(err.contains("E_IR_LAYOUT"), "{err}");

    // Drive prefixes, control chars, and path separators in namespaces cannot escape output_dir.
    let out_dir = tempfile::tempdir().unwrap();
    for bad in [
        "C:evil",
        "d:out",
        "foo:stream",
        "../escape",
        "a/b",
        r"a\\b",
        r"bad\nname",
    ] {
        let ir = format!(r#"{{"namespace":"{bad}","imports":[],"declarations":[]}}"#);
        let err = Compiler::new()
            .generate_code(molecule_codegen::Language::Rust)
            .input_intermediate(JSON, ir.into_bytes())
            .output_dir(out_dir.path())
            .run()
            .unwrap_err();
        assert!(
            err.contains("E_IR_NAME") || err.contains("E_OUTPUT"),
            "{bad}: {err}"
        );
    }
    assert!(fs::read_dir(out_dir.path()).unwrap().next().is_none());

    // Legacy v1 IR with unique but unsorted explicit union IDs (0.9.2 compatible) is accepted.
    let unsorted_union = br#"{"namespace":"sample","imports":[],"declarations":[{"type":"array","name":"A","item":"byte","item_count":1},{"type":"array","name":"B","item":"byte","item_count":2},{"type":"union","name":"U","items":[{"typ":"A","id":8},{"typ":"B","id":3}]}]}"#;
    let ast = Compiler::new()
        .input_intermediate(JSON, unsorted_union.to_vec())
        .parse_ast()
        .unwrap();
    assert!(matches!(ast.decls()[2].as_ref(), TopDecl::Union(u) if u.items().len() == 2));

    // Legacy v1 IR and schemas with distinct case-variant field names (`feeRate` / `feerate`) are accepted.
    let case_fields_ir = br#"{"namespace":"sample","imports":[],"declarations":[{"type":"struct","name":"Rates","fields":[{"name":"feeRate","type":"byte"},{"name":"feerate","type":"byte"}]}]}"#;
    let ast = Compiler::new()
        .input_intermediate(JSON, case_fields_ir.to_vec())
        .parse_ast()
        .unwrap();
    assert_eq!(ast.decls().len(), 1);
    for lang in [
        molecule_codegen::Language::Rust,
        molecule_codegen::Language::RustLazyReader,
        molecule_codegen::Language::C,
    ] {
        Compiler::new()
            .generate_code(lang)
            .input_intermediate(JSON, case_fields_ir.to_vec())
            .output_dir(out_dir.path())
            .run()
            .unwrap();
    }

    // Crafted legacy v1 IR rejects invalid identifiers, duplicate/oversized union IDs, duplicate union items, and huge path_supers.
    for malformed in [
        r#"{"namespace":"s","imports":[],"declarations":[{"type":"table","name":"123bad","fields":[]}]}"#,
        r#"{"namespace":"s","imports":[{"name":"x\n#include","paths":[],"path_supers":0}],"declarations":[]}"#,
        r#"{"namespace":"s","imports":[{"name":"a","paths":[],"path_supers":2000}],"declarations":[]}"#,
        r#"{"namespace":"s","imports":[],"declarations":[{"type":"union","name":"U","items":[{"typ":"byte","id":4294967296}]}]}"#,
        r#"{"namespace":"s","imports":[],"declarations":[{"type":"union","name":"U","items":[{"typ":"byte","id":0},{"typ":"byte","id":1}]}]}"#,
        r#"{"namespace":"s","imports":[],"declarations":[{"type":"array","name":"A","item":"byte","item_count":1},{"type":"array","name":"B","item":"byte","item_count":2},{"type":"union","name":"U","items":[{"typ":"A","id":8},{"typ":"B","id":8}]}]}"#,
    ] {
        assert!(
            Compiler::new()
                .input_intermediate(JSON, malformed.as_bytes().to_vec())
                .parse_ast()
                .is_err(),
            "accepted malformed v1 IR: {malformed}"
        );
    }
}

#[test]
fn rust_and_lazy_generators_reject_colliding_snake_case_field_names() {
    use molecule_codegen::Language;
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("fields.mol");
    let out = tempfile::tempdir().unwrap();
    for (schema, code) in [
        (
            "table T { feeRate: byte, fee_rate: byte, }",
            "E_RUST_FIELD_COLLISION",
        ),
        (
            "struct S { feeRate: byte, fee_rate: byte, }",
            "E_RUST_FIELD_COLLISION",
        ),
        (
            "table Kw { type: byte, type_: byte, }",
            "E_RUST_FIELD_COLLISION",
        ),
        (
            "integer Rate = u64; table IntT { feeRate: Rate, fee_rate: Rate, }",
            "E_RUST_FIELD_COLLISION",
        ),
        (
            "table feeRate {} table fee_rate {}",
            "E_RUST_TYPE_COLLISION",
        ),
        ("array Self [byte; 4];", "E_RUST_TYPE_COLLISION"),
    ] {
        fs::write(&root, schema).unwrap();
        for lang in [Language::Rust, Language::RustLazyReader] {
            let err = Compiler::new()
                .generate_code(lang)
                .input_schema_file(&root)
                .output_dir(out.path())
                .run()
                .unwrap_err();
            assert!(err.contains(code), "{lang:?} {schema}: {err}");
        }
    }
    // Inherent method collisions per backend.
    fs::write(&root, "table T { verify: byte, }").unwrap();
    assert!(Compiler::new()
        .generate_code(Language::RustLazyReader)
        .input_schema_file(&root)
        .output_dir(out.path())
        .run()
        .unwrap_err()
        .contains("E_RUST_FIELD_COLLISION"));
    fs::write(&root, "table T { as_reader: byte, }").unwrap();
    assert!(Compiler::new()
        .generate_code(Language::Rust)
        .input_schema_file(&root)
        .output_dir(out.path())
        .run()
        .unwrap_err()
        .contains("E_RUST_FIELD_COLLISION"));

    // Generated auxiliary type collision (Amount + AmountReader) is rejected in ordinary Rust,
    // while byte FixVec (which does not emit ReaderIterator) does not falsely collide.
    fs::write(
        &root,
        "integer Amount = u64be; array AmountReader [byte; 8];",
    )
    .unwrap();
    assert!(Compiler::new()
        .generate_code(Language::Rust)
        .input_schema_file(&root)
        .output_dir(out.path())
        .run()
        .unwrap_err()
        .contains("E_RUST_TYPE_COLLISION"));
    fs::write(&root, "vector Bytes <byte>; table BytesReaderIterator {}").unwrap();
    Compiler::new()
        .generate_code(Language::Rust)
        .input_schema_file(&root)
        .output_dir(out.path())
        .run()
        .unwrap();

    // Separate imported modules may define distinct names mapping to the same CamelCase name.
    let mod_dir = tempfile::tempdir().unwrap();
    fs::write(
        mod_dir.path().join("mod_a.mol"),
        "integer feeRate = u32; table A { x: feeRate, }",
    )
    .unwrap();
    fs::write(
        mod_dir.path().join("mod_b.mol"),
        "integer fee_rate = u64; table B { y: fee_rate, }",
    )
    .unwrap();
    let combined = mod_dir.path().join("combined.mol");
    fs::write(
        &combined,
        "import mod_a; import mod_b; table Root { a: A, b: B, }",
    )
    .unwrap();
    for lang in [Language::Rust, Language::RustLazyReader] {
        Compiler::new()
            .generate_code(lang)
            .input_schema_file(&combined)
            .output_dir(out.path())
            .run()
            .unwrap();
    }

    // Custom `Option` type does not shadow std Option in generated ordinary Rust.
    let opt_schema = dir.path().join("custom_option.mol");
    fs::write(
        &opt_schema,
        "table Option {} integer Amount = u64; option AmountOpt (Amount);",
    )
    .unwrap();
    Compiler::new()
        .generate_code(Language::Rust)
        .input_schema_file(&opt_schema)
        .output_dir(out.path())
        .run()
        .unwrap();
    let generated_opt = fs::read_to_string(out.path().join("custom_option.rs")).unwrap();
    let stripped = generated_opt.replace(":: core :: option :: Option", "");
    assert!(!stripped.contains("Option <"), "{generated_opt}");
    assert!(!stripped.contains("Option<"), "{generated_opt}");
    assert!(generated_opt.contains(":: core :: option :: Option < self :: Amount >"));

    // Duplicate union item types and missing transitive type diagnostics.
    fs::write(&root, "union U { byte: 0, byte: 1, }").unwrap();
    assert!(Compiler::new()
        .input_schema_file(&root)
        .parse_ast()
        .unwrap_err()
        .contains("E_SCHEMA_UNION"));
    fs::write(&root, "table B { x: Missing, } table A { b: B, }").unwrap();
    let err = Compiler::new()
        .input_schema_file(&root)
        .parse_ast()
        .unwrap_err();
    assert!(err.contains("B") && err.contains("Missing"), "{err}");

    // Asymmetric syntax versions across imports are rejected in both directions.
    fs::write(dir.path().join("child_v1.mol"), "table Child {}").unwrap();
    fs::write(&root, "syntax = 2; import child_v1; table Root {}").unwrap();
    assert!(Compiler::new()
        .input_schema_file(&root)
        .parse_ast()
        .unwrap_err()
        .contains("syntax version"));

    // Distinct case-variant fields (`feeRate` / `feerate`) succeed in schema, Rust, lazy Rust, and C;
    // case-only duplicates (`x` / `X`) fail in Rust/lazy Rust while remaining distinct in C.
    fs::write(&root, "struct Rates { feeRate: byte, feerate: byte, }").unwrap();
    for lang in [Language::Rust, Language::RustLazyReader, Language::C] {
        Compiler::new()
            .generate_code(lang)
            .input_schema_file(&root)
            .output_dir(out.path())
            .run()
            .unwrap();
    }
    fs::write(&root, "table CaseOnly { x: byte, X: byte, }").unwrap();
    for lang in [Language::Rust, Language::RustLazyReader] {
        assert!(Compiler::new()
            .generate_code(lang)
            .input_schema_file(&root)
            .output_dir(out.path())
            .run()
            .unwrap_err()
            .contains("E_RUST_FIELD_COLLISION"));
    }
    Compiler::new()
        .generate_code(Language::C)
        .input_schema_file(&root)
        .output_dir(out.path())
        .run()
        .unwrap();

    // C include guards sanitize punctuation, include a content hash so `a/base.h` and `b/base.h`
    // get distinct guards, and detect macro collisions.
    let dashed = dir.path().join("types-v2.mol");
    fs::write(&dashed, "integer Amount = u64be;").unwrap();
    Compiler::new()
        .generate_code(Language::C)
        .input_schema_file(&dashed)
        .output_dir(out.path())
        .run()
        .unwrap();
    let header = fs::read_to_string(out.path().join("types-v2.h")).unwrap();
    assert!(header.contains("#ifndef TYPES_V2_"), "{header}");
    assert!(!header.contains("#ifndef TYPES-V2_"), "{header}");

    let c_a = dir.path().join("ca");
    let c_b = dir.path().join("cb");
    fs::create_dir_all(&c_a).unwrap();
    fs::create_dir_all(&c_b).unwrap();
    fs::write(c_a.join("base.mol"), "integer AInt = u32;").unwrap();
    fs::write(c_b.join("base.mol"), "integer BInt = u64be;").unwrap();
    let out_a = out.path().join("ca");
    let out_b = out.path().join("cb");
    fs::create_dir_all(&out_a).unwrap();
    fs::create_dir_all(&out_b).unwrap();
    Compiler::new()
        .generate_code(Language::C)
        .input_schema_file(c_a.join("base.mol"))
        .output_dir(&out_a)
        .run()
        .unwrap();
    Compiler::new()
        .generate_code(Language::C)
        .input_schema_file(c_b.join("base.mol"))
        .output_dir(&out_b)
        .run()
        .unwrap();
    let guard_line = |s: &str| {
        s.lines()
            .find(|l| l.starts_with("#ifndef BASE_"))
            .unwrap()
            .to_owned()
    };
    let h_a = fs::read_to_string(out_a.join("base.h")).unwrap();
    let h_b = fs::read_to_string(out_b.join("base.h")).unwrap();
    assert_ne!(guard_line(&h_a), guard_line(&h_b));

    fs::write(
        &root,
        "integer Foo_get_bar = u64be; struct Foo { bar_read_value: byte, }",
    )
    .unwrap();
    assert!(Compiler::new()
        .generate_code(Language::C)
        .input_schema_file(&root)
        .output_dir(out.path())
        .run()
        .unwrap_err()
        .contains("E_C_SYMBOL_COLLISION"));

    // Local C field/union macro colliding with an imported integer accessor is also rejected.
    fs::write(
        dir.path().join("imp_int.mol"),
        "integer Foo_get_bar = u64be; integer U_set_Item = u64be;",
    )
    .unwrap();
    fs::write(
        &root,
        "import imp_int; struct Foo { bar_read_value: byte, }",
    )
    .unwrap();
    assert!(Compiler::new()
        .generate_code(Language::C)
        .input_schema_file(&root)
        .output_dir(out.path())
        .run()
        .unwrap_err()
        .contains("E_C_SYMBOL_COLLISION"));
    fs::write(
        &root,
        "import imp_int; array Item_write_value [byte; 1]; union U { Item_write_value, }",
    )
    .unwrap();
    assert!(Compiler::new()
        .generate_code(Language::C)
        .input_schema_file(&root)
        .output_dir(out.path())
        .run()
        .unwrap_err()
        .contains("E_C_SYMBOL_COLLISION"));
}

#[cfg(unix)]
#[test]
fn cross_directory_aliases_allow_unambiguous_imports_and_reject_conflicting_ones() {
    use std::os::unix::fs::symlink;
    for use_symlink in [false, true] {
        // 1. Conflicting targets (a/word.mol 2B vs b/word.mol 4B) must fail in both orders.
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a");
        let b = dir.path().join("b");
        fs::create_dir(&a).unwrap();
        fs::create_dir(&b).unwrap();
        fs::write(a.join("word.mol"), "array Word [byte; 2];").unwrap();
        fs::write(b.join("word.mol"), "array Word [byte; 4];").unwrap();
        if use_symlink {
            let real = dir.path().join("real");
            fs::create_dir(&real).unwrap();
            fs::write(real.join("shared.mol"), "import word; table S { w: Word, }").unwrap();
            symlink("../real/shared.mol", a.join("shared.mol")).unwrap();
            symlink("../real/shared.mol", b.join("shared.mol")).unwrap();
        } else {
            fs::write(a.join("shared.mol"), "import word; table S { w: Word, }").unwrap();
            fs::hard_link(a.join("shared.mol"), b.join("shared.mol")).unwrap();
        }
        let root = dir.path().join("root.mol");
        for imports in [
            "import a/shared; import b/shared;",
            "import b/shared; import a/shared;",
        ] {
            fs::write(&root, format!("{imports} table Root {{ s: S, }}")).unwrap();
            let err = Compiler::new()
                .input_schema_file(&root)
                .parse_ast()
                .unwrap_err();
            assert!(
                err.contains("E_SCHEMA_SOURCE"),
                "symlink={use_symlink} imports={imports}: {err}"
            );
        }

        // 2. Unambiguous targets (both a/shared and b/shared resolve `import ../word;`
        //    to the exact same root/word.mol) must succeed identically in both orders.
        fs::write(dir.path().join("word.mol"), "array Word [byte; 2];").unwrap();
        let shared_src = "import ../word; table S { w: Word, }";
        if use_symlink {
            fs::write(dir.path().join("real/shared.mol"), shared_src).unwrap();
        } else {
            fs::write(a.join("shared.mol"), shared_src).unwrap();
        }
        for imports in [
            "import a/shared; import b/shared;",
            "import b/shared; import a/shared;",
        ] {
            fs::write(&root, format!("{imports} table Root {{ s: S, }}")).unwrap();
            let ast = Compiler::new()
                .input_schema_file(&root)
                .parse_ast()
                .unwrap();
            use molecule_codegen::ast::{HasName as _, TopDecl};
            let word = ast
                .decls()
                .iter()
                .find(|d| d.name() == "Word")
                .expect("Word declaration");
            assert!(matches!(word.as_ref(), TopDecl::Array(a) if a.item_count() == 2));
        }
    }
}

#[cfg(all(unix, feature = "compiler-plugin"))]
#[test]
fn compiler_run_returns_err_instead_of_exiting_when_plugin_fails() {
    use molecule_codegen::IntermediateFormat::JSON;
    use std::process::{Command, Stdio};
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("sample.mol");
    fs::write(&root, "array Word [byte; 2];").unwrap();
    let child = Command::new("/bin/sh")
        .args(["-c", "exit 7"])
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();
    let err = Compiler::new()
        .generate_intermediate(JSON)
        .input_schema_file(&root)
        .output_plugin_process(child)
        .run()
        .unwrap_err();
    assert!(err.contains("E_OUTPUT"), "{err}");
}
