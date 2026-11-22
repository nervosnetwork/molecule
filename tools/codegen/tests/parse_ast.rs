use molecule_codegen::Compiler;
use std::fs;

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
}

#[test]
fn invalid_layouts_return_diagnostics_with_and_without_integers() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("root.mol");
    for prefix in ["", "integer Amount = u64le;"] {
        for (schema, names) in [
            ("table Order { price: Pric, }", &["Order", "Pric"][..]),
            ("option A (B); option B (A);", &["A", "B"]),
            (
                "vector Bytes <byte>; struct Record { data: Bytes, }",
                &["Record"],
            ),
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
}
