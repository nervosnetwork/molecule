#![cfg(unix)]
use molecule_codegen::{
    ir::Capabilities, schema::source_digest, Compiler, IntermediateFormat, Language,
};
use std::{
    fs,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};
use tempfile::TempDir;

fn script(directory: &Path, contents: &str) -> PathBuf {
    let path = directory.join("moleculec-fixture");
    // A concurrent fork can keep a freshly written executable open on Linux.
    if !path.exists() {
        symlink(
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/plugin"),
            &path,
        )
        .unwrap();
    }
    fs::write(path.with_extension("sh"), contents).unwrap();
    path
}

fn inputs(directory: &Path) -> Compiler {
    let schema = "array Amount [byte; 8];";
    let metadata = format!(
        r#"{{"metadata_version":1,"schema":"types.mol","schema_sha256":"{}","integers":[{{"type":"Amount","bits":64,"signed":false,"byte_order":"big"}}]}}"#,
        source_digest(schema.as_bytes())
    );
    fs::write(directory.join("types.mol"), schema).unwrap();
    fs::write(directory.join("types.json"), metadata).unwrap();
    let mut compiler = Compiler::new();
    compiler
        .input_schema_file(directory.join("types.mol"))
        .type_metadata_file(directory.join("types.json"));
    compiler
}

fn fixture_cli(directory: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_moleculec"));
    let mut paths = vec![directory.to_owned()];
    paths.extend(std::env::split_paths(&std::env::var_os("PATH").unwrap()));
    command
        .env("PATH", std::env::join_paths(paths).unwrap())
        .current_dir(directory)
        .args([
            "--schema-file",
            "types.mol",
            "--language",
            "fixture",
            "--plugin-timeout",
            "1",
        ]);
    command
}

fn cli(directory: &Path) -> Command {
    let mut command = fixture_cli(directory);
    command.args(["--type-metadata", "types.json"]);
    command
}

#[test]
fn builtin_plugins_advertise_exact_capabilities_and_accept_v2() {
    let directory = TempDir::new().unwrap();
    let compiler = inputs(directory.path());
    let ast = compiler.parse_ast().unwrap();
    for plugin in [
        env!("CARGO_BIN_EXE_moleculec-rust"),
        env!("CARGO_BIN_EXE_moleculec-c"),
        env!("CARGO_BIN_EXE_moleculec-rust-lazy-reader"),
    ] {
        let path = Path::new(plugin);
        let format = moleculec::plugin::negotiate(path, &ast, Duration::from_secs(5)).unwrap();
        let (output, _) = moleculec::plugin::execute(
            path,
            format.serialize_ast(&ast).unwrap(),
            Duration::from_secs(5),
        )
        .unwrap();
        assert!(!output.is_empty());
    }
}

#[test]
fn failed_negotiation_never_sends_ir_or_runs_generation() {
    let directory = TempDir::new().unwrap();
    inputs(directory.path());
    let mut capabilities = Capabilities::builtin(Language::Rust, IntermediateFormat::JSON);
    capabilities.ir_versions = vec![1];
    let inexact = capabilities.to_json().unwrap();
    for response in ["{}", "json", &inexact] {
        script(directory.path(), &format!("if [ \"$1\" = --capabilities ]; then cat <<'CAPS'\n{response}\nCAPS\nelse touch invoked; cat; fi"));
        let output = cli(directory.path()).output().unwrap();
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(!directory.path().join("invoked").exists());
        let diagnostic = String::from_utf8_lossy(&output.stderr);
        assert!(
            diagnostic.contains("moleculec-fixture: E_PLUGIN_CAPABILITY"),
            "{diagnostic}"
        );
    }
}

#[test]
fn plain_format_probe_offers_source_ir_only_for_imports() {
    let dir = TempDir::new().unwrap();
    let dir = dir.path();
    let schema = dir.join("types.mol");
    let run = || {
        fixture_cli(dir)
            .env("MOLECULEC_IR_VERSION", "2")
            .output()
            .unwrap()
    };
    script(
        dir,
        r#"if [ "$1" = --format ]; then echo "${MOLECULEC_IR_VERSION-unset}" > offer; echo json; else cat; fi"#,
    );
    fs::write(dir.join("leaf.mol"), "array Word [byte; 2];").unwrap();
    for (source, expected) in [
        ("table Root {}", "unset"),
        ("import leaf; table Root {}", "2"),
    ] {
        fs::write(&schema, source).unwrap();
        let result = run();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(
            fs::read_to_string(dir.join("offer")).unwrap().trim(),
            expected
        );
    }
    for reply in ["{", "{}"] {
        script(dir, &format!("echo '{reply}'"));
        let result = run();
        let diagnostic = String::from_utf8_lossy(&result.stderr);
        assert!(!result.status.success() && result.stdout.is_empty());
        assert!(
            diagnostic.contains("moleculec-fixture: E_PLUGIN_CAPABILITY"),
            "{diagnostic}"
        );
    }
}

#[test]
fn cli_buffers_failure_and_timeout_but_emits_success() {
    let directory = TempDir::new().unwrap();
    inputs(directory.path());
    let caps = Capabilities::builtin(Language::Rust, IntermediateFormat::JSON)
        .to_json()
        .unwrap();
    for (body, success) in [
        ("cat > received; printf complete", true),
        (
            "cat > received; printf partial; printf failure >&2; exit 7",
            false,
        ),
        ("cat > received; printf partial; exec sleep 20", false),
    ] {
        script(
            directory.path(),
            &format!(
                "if [ \"$1\" = --capabilities ]; then cat <<'CAPS'\n{caps}\nCAPS\nelse {body}; fi"
            ),
        );
        let start = Instant::now();
        let output = cli(directory.path()).output().unwrap();
        assert_eq!(output.status.success(), success);
        assert!(start.elapsed() < Duration::from_secs(5));
        assert_eq!(
            output.stdout,
            if success { b"complete".as_slice() } else { b"" }
        );
        let ir = fs::read(directory.path().join("received")).unwrap();
        assert!(IntermediateFormat::JSON
            .recover(&ir)
            .unwrap()
            .integer("Amount")
            .is_some());
    }
}

#[test]
fn capability_timeout_and_output_limits_are_enforced() {
    let directory = TempDir::new().unwrap();
    let ast = inputs(directory.path()).parse_ast().unwrap();
    let plugin = script(directory.path(), "exec sleep 20");
    let start = Instant::now();
    let error = moleculec::plugin::negotiate(&plugin, &ast, Duration::from_millis(50)).unwrap_err();
    assert!(error.contains("timed out"), "{error}");
    assert!(start.elapsed() < Duration::from_secs(2));
    let plugin = script(directory.path(), "exec head -c 65537 /dev/zero");
    assert!(
        moleculec::plugin::negotiate(&plugin, &ast, Duration::from_secs(2))
            .unwrap_err()
            .contains("exceeds")
    );
    let plugin = script(
        directory.path(),
        "head -c 65537 /dev/zero >&2; cat >/dev/null",
    );
    assert!(
        moleculec::plugin::execute(&plugin, vec![1; 256 * 1024], Duration::from_secs(2))
            .unwrap_err()
            .contains("exceeds")
    );
}

#[test]
fn communication_drains_stdout_while_writing_large_input() {
    let directory = TempDir::new().unwrap();
    let plugin = script(directory.path(), "head -c 262144 /dev/zero; cat");
    let (out, errors) =
        moleculec::plugin::execute(&plugin, vec![42; 262144], Duration::from_secs(5)).unwrap();
    assert_eq!(out[..262144], vec![0; 262144]);
    assert_eq!(out[262144..], vec![42; 262144]);
    assert!(errors.is_empty());
}

#[test]
fn timeout_terminates_background_descendants_holding_pipes() {
    let directory = TempDir::new().unwrap();
    let plugin = script(
        directory.path(),
        "(sleep 2; printf survived > \"$0.survived\") &\nprintf '%s' \"$!\" > \"$0.pid\"\nexit 0",
    );
    let start = Instant::now();
    assert!(
        moleculec::plugin::execute(&plugin, Vec::new(), Duration::from_secs(1))
            .unwrap_err()
            .contains("timed out")
    );
    assert!(start.elapsed() < Duration::from_secs(2));
    assert!(!fs::read_to_string(plugin.with_extension("pid"))
        .unwrap()
        .is_empty());
    std::thread::sleep(Duration::from_secs(3));
    assert!(
        !plugin.with_extension("survived").exists(),
        "background descendant survived cancellation"
    );
}

#[test]
fn loading_many_modules_does_not_keep_all_files_open() {
    let directory = TempDir::new().unwrap();
    let mut root = String::new();
    for index in 0..80 {
        root.push_str(&format!("import b{index};\n"));
        fs::write(
            directory.path().join(format!("b{index}.mol")),
            format!("array B{index} [byte; 1];"),
        )
        .unwrap();
    }
    root.push_str("table Root {}\n");
    fs::write(directory.path().join("root.mol"), &root).unwrap();
    fs::write(
        directory.path().join("meta.json"),
        format!(
            r#"{{"metadata_version":1,"schema":"root.mol","schema_sha256":"{}","integers":[]}}"#,
            source_digest(root.as_bytes())
        ),
    )
    .unwrap();
    let output=Command::new("/bin/sh").args(["-c","ulimit -n 48; exec \"$1\" --schema-file root.mol --type-metadata meta.json --language - --format json","fixture",env!("CARGO_BIN_EXE_moleculec")]).current_dir(directory.path()).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn imported_native_integers_require_semantic_plugins_and_allow_explicit_wire_export() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("root.mol");
    fs::write(&root, "import keys; struct Entry { key: Key, }").unwrap();
    fs::write(dir.path().join("keys.mol"), "integer Key = u32be;").unwrap();
    script(
        dir.path(),
        "if [ \"$1\" = --capabilities ]; then echo '{}'; else touch invoked; cat; fi",
    );
    let run = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_moleculec"))
            .env(
                "PATH",
                std::env::join_paths(
                    std::iter::once(dir.path().to_owned())
                        .chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
                )
                .unwrap(),
            )
            .current_dir(dir.path())
            .arg("--schema-file")
            .arg(&root)
            .args(args)
            .output()
            .unwrap()
    };
    let plugin = run(&["--language", "fixture"]);
    assert!(!plugin.status.success());
    assert!(plugin.stdout.is_empty());
    assert!(String::from_utf8_lossy(&plugin.stderr).contains("E_PLUGIN_CAPABILITY"));
    assert!(!dir.path().join("invoked").exists());
    let dump = ["--language", "-", "--format", "json"];
    let extended = run(&dump);
    assert!(extended.status.success());
    let published = dir.path().join("published.json");
    fs::write(&published, extended.stdout).unwrap();
    let wire = run(&["--language", "-", "--format", "json", "--wire-only"]);
    assert!(wire.status.success());
    assert!(String::from_utf8_lossy(&wire.stderr).contains("Wire-only"));
    assert!(IntermediateFormat::JSON
        .recover(&wire.stdout)
        .unwrap()
        .integer("Key")
        .is_none());
    fs::write(dir.path().join("keys.mol"), "array Key [byte; 4];").unwrap();
    let failed = run(&[
        "--language",
        "fixture",
        "--check-integer-contract",
        "published.json",
    ]);
    assert!(!failed.status.success());
    assert!(failed.stdout.is_empty());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("missing integer Key"));
    assert!(!dir.path().join("invoked").exists());
    script(
        dir.path(),
        "if [ \"$1\" = --format ]; then printf JSON; else cat; fi",
    );
    let legacy = run(&["--language", "fixture"]);
    assert!(legacy.status.success());
    assert_eq!(legacy.stdout, wire.stdout);

    // Legacy v1 plugins printing `YAML\n` with trailing newline still negotiate YAML.
    let wire_yaml = run(&["--language", "-", "--format", "yaml"]);
    assert!(wire_yaml.status.success());
    script(
        dir.path(),
        "if [ \"$1\" = --format ]; then printf 'YAML\\n'; else cat; fi",
    );
    let legacy_yaml = run(&["--language", "fixture"]);
    assert!(legacy_yaml.status.success());
    assert_eq!(legacy_yaml.stdout, wire_yaml.stdout);
}

#[test]
fn early_plugin_failure_preserves_stderr_over_broken_pipe() {
    let directory = TempDir::new().unwrap();
    // Exit immediately with an explicit diagnostic without reading stdin.
    let plugin = script(
        directory.path(),
        "printf 'E_CUSTOM_DIAGNOSTIC: rejected before reading stdin' >&2; exit 2",
    );
    // Input exceeds typical OS pipe buffers (64 KiB) so the writer hits EPIPE.
    let error = moleculec::plugin::execute(&plugin, vec![b'x'; 512 * 1024], Duration::from_secs(5))
        .unwrap_err();
    assert!(
        error.contains("E_CUSTOM_DIAGNOSTIC"),
        "expected plugin stderr diagnostic, got: {error}"
    );
}
