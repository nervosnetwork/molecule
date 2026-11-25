use molecule_codegen::{Compiler, Language};
use std::fs;
#[cfg(unix)]
use std::{os::unix::fs::PermissionsExt, process::Command};

#[test]
fn read_only_output_survives_a_failed_replacement_without_staging_files() {
    let dir = tempfile::tempdir().unwrap();
    let schema = dir.path().join("types.mol");
    let output = dir.path().join("types.rs");
    fs::write(&schema, "integer Word = u16;").unwrap();
    fs::write(&output, "original").unwrap();
    let original = fs::metadata(&output).unwrap().permissions();
    let mut readonly = original.clone();
    readonly.set_readonly(true);
    fs::set_permissions(&output, readonly).unwrap();
    let writable = fs::OpenOptions::new().write(true).open(&output).is_ok();
    let result = Compiler::new()
        .input_schema_file(schema)
        .generate_code(Language::Rust)
        .output_dir(dir.path())
        .run();
    let still_readonly = fs::metadata(&output).unwrap().permissions().readonly();
    fs::set_permissions(&output, original).unwrap();
    if cfg!(windows) || !writable {
        assert!(result.unwrap_err().contains("E_OUTPUT"));
        assert!(still_readonly);
        assert_eq!(fs::read(output).unwrap(), b"original");
    }
    assert_eq!(dir.path().read_dir().unwrap().count(), 2);
}

#[cfg(target_os = "macos")]
fn metadata_command(program: &str, args: &[&str], file: &std::path::Path) -> String {
    let result = Command::new(program).args(args).arg(file).output().unwrap();
    assert!(result.status.success(), "{program} {args:?}: {result:?}");
    String::from_utf8(result.stdout).unwrap()
}

#[cfg(unix)]
#[test]
fn overwriting_preserves_output_permissions() {
    let dir = tempfile::tempdir().unwrap();
    let schema = dir.path().join("types.mol");
    let output = dir.path().join("types.rs");
    for (source, mode, sidecar) in [
        ("array Word [byte; 2];", 0o200, false),
        ("integer Word = u16;", 0o600, false),
        #[cfg(feature = "schema-extensions")]
        ("array Word [byte; 2];", 0o600, true),
    ] {
        fs::write(&schema, source).unwrap();
        fs::write(&output, "old").unwrap();
        #[cfg(target_os = "macos")]
        {
            metadata_command("xattr", &["-w", "user.molecule", "kept"], &output);
            metadata_command("chmod", &["+a", "everyone allow readattr"], &output);
            if mode == 0o200 {
                metadata_command("chmod", &["+a", "owner allow readextattr"], &output);
            }
        }
        fs::set_permissions(&output, fs::Permissions::from_mode(mode)).unwrap();
        let mut compiler = Compiler::new();
        compiler
            .input_schema_file(&schema)
            .generate_code(Language::Rust)
            .output_dir(dir.path());
        #[cfg(feature = "schema-extensions")]
        if sidecar {
            let metadata = dir.path().join("types.json");
            let digest = molecule_codegen::schema::source_digest(source.as_bytes());
            fs::write(&metadata, format!(
                r#"{{"metadata_version":1,"schema":"types.mol","schema_sha256":"{digest}","integers":[{{"type":"Word","bits":16,"signed":false,"byte_order":"little"}}]}}"#
            )).unwrap();
            compiler.type_metadata_file(metadata);
        }
        #[cfg(not(feature = "schema-extensions"))]
        let _ = sidecar;
        // BSD may deny enumeration of user attributes on a write-only file.
        // Only accept that failure when the filesystem itself denies the read.
        #[cfg(any(target_os = "freebsd", target_os = "netbsd"))]
        let cannot_list = {
            use std::{io, os::fd::AsRawFd};
            let fd = fs::OpenOptions::new().write(true).open(&output).unwrap();
            // SAFETY: a null buffer with size zero only queries the list length.
            unsafe {
                libc::extattr_list_fd(
                    fd.as_raw_fd(),
                    libc::EXTATTR_NAMESPACE_USER,
                    std::ptr::null_mut(),
                    0,
                ) < 0
                    && io::Error::last_os_error().kind() == io::ErrorKind::PermissionDenied
            }
        };
        #[cfg(not(any(target_os = "freebsd", target_os = "netbsd")))]
        let cannot_list = false;
        let result = compiler.run();
        if cannot_list {
            assert_eq!(mode, 0o200);
            assert!(result
                .unwrap_err()
                .contains("cannot list user extended attributes"));
        } else {
            result.unwrap();
        }
        assert_eq!(
            fs::metadata(&output).unwrap().permissions().mode() & 0o777,
            mode
        );
        fs::set_permissions(&output, fs::Permissions::from_mode(0o600)).unwrap();
        if cannot_list {
            assert_eq!(fs::read(&output).unwrap(), b"old");
        } else {
            assert!(fs::read_to_string(&output).unwrap().contains("Word"));
        }
        #[cfg(target_os = "macos")]
        {
            assert_eq!(
                metadata_command("xattr", &["-p", "user.molecule"], &output).trim(),
                "kept"
            );
            assert!(metadata_command("ls", &["-le"], &output).contains("everyone allow readattr"));
        }
        fs::remove_file(&output).unwrap();
    }
}

#[cfg(unix)]
#[test]
fn output_links_preserve_the_target_and_reject_conflicting_hard_links() {
    use std::os::unix::fs::{symlink, MetadataExt};
    let dir = tempfile::tempdir().unwrap();
    let schema = dir.path().join("linked.mol");
    let output = dir.path().join("linked.rs");
    let target = dir.path().join("target");
    let alias = dir.path().join("alias");
    fs::write(&schema, "array Word [byte; 2];").unwrap();
    symlink("symlink", &output).unwrap();
    symlink("target", dir.path().join("symlink")).unwrap();
    let mut compiler = Compiler::new();
    compiler
        .input_schema_file(&schema)
        .generate_code(Language::Rust)
        .output_dir(dir.path());
    compiler.run().unwrap();
    assert!(output.is_symlink());
    assert!(dir.path().join("symlink").is_symlink());
    assert!(target.is_file());
    let generated = fs::read(&target).unwrap();
    fs::write(&target, "replace me").unwrap();
    compiler.run().unwrap();
    assert_eq!(fs::read(&output).unwrap(), generated);
    fs::hard_link(&target, &alias).unwrap();
    let inode = fs::metadata(&target).unwrap().ino();
    fs::set_permissions(&target, fs::Permissions::from_mode(0o444)).unwrap();
    compiler.run().unwrap();
    fs::write(&schema, "array Word [byte; 3];").unwrap();
    let err = compiler.run().unwrap_err();
    assert!(
        err.contains("E_OUTPUT") && err.contains("hard links"),
        "{err}"
    );
    for path in [&output, &target, &alias] {
        let metadata = fs::metadata(path).unwrap();
        assert_eq!((metadata.ino(), metadata.nlink()), (inode, 2));
        assert_eq!(fs::read(path).unwrap(), generated);
    }
}

#[cfg(target_os = "macos")]
#[test]
fn unreadable_extended_attributes_leave_the_original_intact() {
    use std::os::unix::fs::MetadataExt;
    let dir = tempfile::tempdir().unwrap();
    let schema = dir.path().join("types.mol");
    let output = dir.path().join("types.rs");
    fs::write(&schema, "array Word [byte; 2];").unwrap();
    fs::write(&output, "old").unwrap();
    metadata_command("xattr", &["-w", "user.molecule", "kept"], &output);
    fs::set_permissions(&output, fs::Permissions::from_mode(0o200)).unwrap();
    // A privileged runner may still have read access to the attributes.
    if Command::new("xattr")
        .arg(&output)
        .output()
        .unwrap()
        .status
        .success()
    {
        return;
    }
    let inode = fs::metadata(&output).unwrap().ino();
    let err = Compiler::new()
        .input_schema_file(schema)
        .generate_code(Language::Rust)
        .output_dir(dir.path())
        .run()
        .unwrap_err();
    assert!(err.contains("extended attributes"), "{err}");
    assert_eq!(fs::metadata(&output).unwrap().ino(), inode);
    fs::set_permissions(&output, fs::Permissions::from_mode(0o600)).unwrap();
    assert_eq!(fs::read(&output).unwrap(), b"old");
    assert_eq!(
        metadata_command("xattr", &["-p", "user.molecule"], &output).trim(),
        "kept"
    );
}

#[cfg(target_os = "macos")]
#[test]
fn replacing_acl_free_output_preserves_its_group_and_empty_acl() {
    use std::os::unix::fs::{fchown, MetadataExt};
    let dir = tempfile::tempdir().unwrap();
    let schema = dir.path().join("types.mol");
    let output = dir.path().join("types.rs");
    fs::write(&schema, "array Word [byte; 2];").unwrap();
    fs::write(&output, "old").unwrap();
    metadata_command(
        "chmod",
        &["+a", "everyone allow readattr,file_inherit"],
        dir.path(),
    );
    let groups = Command::new("id").arg("-G").output().unwrap();
    assert!(groups.status.success());
    let parent_gid = fs::metadata(dir.path()).unwrap().gid();
    if let Some(other) = String::from_utf8(groups.stdout)
        .unwrap()
        .split_whitespace()
        .map(|group| group.parse::<u32>().unwrap())
        .find(|gid| *gid != parent_gid)
    {
        fchown(fs::File::open(&output).unwrap(), None, Some(other)).unwrap();
    }
    let gid = fs::metadata(&output).unwrap().gid();
    Compiler::new()
        .input_schema_file(&schema)
        .generate_code(Language::Rust)
        .output_dir(dir.path())
        .run()
        .unwrap();
    assert!(!metadata_command("ls", &["-le"], &output).contains("allow readattr"));
    assert_eq!(fs::metadata(output).unwrap().gid(), gid);
}

#[cfg(unix)]
#[test]
fn plain_overwrite_preserves_existing_file_on_write_failure() {
    if std::env::var_os("MOLECULE_TEST_RLIMIT_FSIZE_CHILD").is_some() {
        let dir = std::path::PathBuf::from(std::env::var_os("MOLECULE_TEST_RLIMIT_DIR").unwrap());
        let schema_path = dir.join("plain.mol");
        let out_file = dir.join("plain.rs");
        let err = Compiler::new()
            .generate_code(Language::Rust)
            .input_schema_file(&schema_path)
            .output_dir(&dir)
            .run()
            .unwrap_err();
        assert!(err.contains("E_OUTPUT"), "{err}");
        assert_eq!(fs::read(&out_file).unwrap(), b"// original intact content");
        let entries: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert!(
            !entries.iter().any(|n| n.starts_with(".molecule-")),
            "leaked temporary files: {entries:?}"
        );
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("plain.mol"), "array Word [byte; 2];").unwrap();
    fs::write(dir.path().join("plain.rs"), b"// original intact content").unwrap();
    let exe = std::env::current_exe().unwrap();
    let output = Command::new("python3")
        .args([
            "-c",
            "import os, resource, signal, sys; signal.signal(signal.SIGXFSZ, signal.SIG_IGN); resource.setrlimit(resource.RLIMIT_FSIZE, (1024, 1024)); os.execv(sys.argv[1], sys.argv[1:])",
        ])
        .arg(&exe)
        .args([
            "--exact",
            "plain_overwrite_preserves_existing_file_on_write_failure",
            "--nocapture",
        ])
        .env("MOLECULE_TEST_RLIMIT_FSIZE_CHILD", "1")
        .env("MOLECULE_TEST_RLIMIT_DIR", dir.path())
        .output()
        .unwrap();
    assert!(output.status.success(), "rlimit child failed: {output:?}");
    assert_eq!(
        fs::read(dir.path().join("plain.rs")).unwrap(),
        b"// original intact content"
    );
}

#[cfg(windows)]
#[test]
fn windows_replacement_preserves_hard_links_and_cleans_up_after_a_sharing_violation() {
    use std::os::windows::fs::OpenOptionsExt;
    use winapi_util::file::information;

    let dir = tempfile::tempdir().unwrap();
    let schema = dir.path().join("types.mol");
    let output = dir.path().join("types.rs");
    let alias = dir.path().join("alias");
    fs::write(&schema, "array Word [byte; 2];").unwrap();
    let mut compiler = Compiler::new();
    compiler
        .input_schema_file(&schema)
        .generate_code(Language::Rust)
        .output_dir(dir.path());
    compiler.run().unwrap();
    let original = fs::read(&output).unwrap();
    let id = information(&fs::File::open(&output).unwrap())
        .unwrap()
        .file_index();

    fs::hard_link(&output, &alias).unwrap();
    compiler.run().unwrap();
    fs::write(&schema, "array Word [byte; 3];").unwrap();
    let err = compiler.run().unwrap_err();
    assert!(err.contains("hard links"), "{err}");
    for path in [&alias, &output] {
        let info = information(&fs::File::open(path).unwrap()).unwrap();
        assert_eq!((info.file_index(), info.number_of_links()), (id, 2));
        assert_eq!(fs::read(path).unwrap(), original);
    }
    fs::remove_file(&alias).unwrap();

    // Permit the write-access check but deny the final replacement.
    let handle = fs::OpenOptions::new()
        .read(true)
        .share_mode(0x1 | 0x2)
        .open(&output)
        .unwrap();
    let err = compiler.run().unwrap_err();
    assert!(err.contains("E_OUTPUT"), "{err}");
    assert_eq!(information(&handle).unwrap().file_index(), id);
    assert_eq!(fs::read(&output).unwrap(), original);
    assert_eq!(dir.path().read_dir().unwrap().count(), 2);
    drop(handle);
    compiler.run().unwrap();
    assert_ne!(fs::read(output).unwrap(), original);
}

#[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "netbsd"))]
#[test]
fn replacing_output_preserves_visible_attributes_and_acl() {
    use std::os::unix::fs::MetadataExt;
    use xattr::FileExt;
    let dir = tempfile::tempdir().unwrap();
    let schema = dir.path().join("types.mol");
    let output = dir.path().join("types.rs");
    fs::write(&schema, "integer Word = u16;").unwrap();
    let mut compiler = Compiler::new();
    compiler
        .input_schema_file(&schema)
        .generate_code(Language::Rust)
        .output_dir(dir.path());
    compiler.run().unwrap();
    let first = fs::read(&output).unwrap();
    let src = fs::File::open(&output).unwrap();
    let attr = b"\0value\xff";
    if let Err(err) = src.set_xattr("user.molecule", attr) {
        assert_eq!(err.kind(), std::io::ErrorKind::Unsupported, "{err}");
        return;
    }
    // A privileged run must exercise preservation of the protected namespace.
    let protected = if cfg!(target_os = "linux") {
        "trusted.molecule"
    } else {
        "system.molecule"
    };
    let privileged = src.set_xattr(protected, b"private").is_ok();
    #[cfg(target_os = "linux")]
    let acl = {
        // Linux xattr ACL v2: owner, named user, group, mask, other.
        let mut bytes = 2u32.to_le_bytes().to_vec();
        for (tag, perm, id) in [
            (1u16, 6u16, u32::MAX),
            (2, 4, 65534),
            (4, 0, u32::MAX),
            (16, 4, u32::MAX),
            (32, 0, u32::MAX),
        ] {
            bytes.extend(tag.to_le_bytes());
            bytes.extend(perm.to_le_bytes());
            bytes.extend(id.to_le_bytes());
        }
        match src.set_xattr("system.posix_acl_access", &bytes) {
            Ok(()) => Some(bytes),
            Err(err) if err.kind() == std::io::ErrorKind::Unsupported => None,
            Err(err) => panic!("{err}"),
        }
    };
    #[cfg(any(target_os = "freebsd", target_os = "netbsd"))]
    let acl = {
        let native = |name: &str| {
            let result = Command::new("getconf")
                .arg(name)
                .arg(&output)
                .output()
                .unwrap();
            result.status.success() && result.stdout.starts_with(b"1")
        };
        let entry = if native("ACL_NFS4") {
            Some("user:65534:r::allow")
        } else if native("ACL_EXTENDED") {
            Some("user:65534:r--")
        } else {
            None
        };
        if let Some(entry) = entry {
            let set = Command::new("setfacl")
                .args(["-m", entry])
                .arg(&output)
                .output()
                .unwrap();
            assert!(set.status.success(), "{set:?}");
        }
        let get = Command::new("getfacl").arg(&output).output().unwrap();
        assert!(get.status.success(), "{get:?}");
        get.stdout
    };

    let before = src.metadata().unwrap();
    fs::write(&schema, "integer Word = u32;").unwrap();
    compiler.run().unwrap();
    let replaced = fs::File::open(&output).unwrap();
    let after = replaced.metadata().unwrap();
    assert_ne!(after.ino(), before.ino());
    assert_ne!(fs::read(&output).unwrap(), first);
    assert_eq!(after.mode(), before.mode());
    assert_eq!(
        replaced.get_xattr("user.molecule").unwrap().as_deref(),
        Some(attr.as_slice())
    );
    if privileged {
        assert_eq!(
            replaced.get_xattr(protected).unwrap().as_deref(),
            Some(b"private".as_slice())
        );
    }
    #[cfg(target_os = "linux")]
    if let Some(acl) = acl {
        assert_eq!(
            replaced.get_xattr("system.posix_acl_access").unwrap(),
            Some(acl)
        );
    }
    #[cfg(any(target_os = "freebsd", target_os = "netbsd"))]
    assert_eq!(
        Command::new("getfacl")
            .arg(&output)
            .output()
            .unwrap()
            .stdout,
        acl
    );
    assert_eq!(dir.path().read_dir().unwrap().count(), 2);
}

#[cfg(target_os = "linux")]
#[test]
fn changed_output_with_file_capabilities_keeps_the_original() {
    use std::{io::ErrorKind, os::unix::fs::MetadataExt};
    use xattr::FileExt;

    let dir = tempfile::tempdir().unwrap();
    let schema = dir.path().join("types.mol");
    let output = dir.path().join("types.rs");
    fs::write(&schema, "array Word [byte; 2];").unwrap();
    let mut compiler = Compiler::new();
    compiler
        .input_schema_file(&schema)
        .generate_code(Language::Rust)
        .output_dir(dir.path());
    compiler.run().unwrap();
    let src = fs::File::open(&output).unwrap();
    // Linux vfs_cap_data revision 2, with no capability bits set.
    let mut caps = [0u8; 20];
    caps[..4].copy_from_slice(&0x0200_0000u32.to_le_bytes());
    if let Err(err) = src.set_xattr("security.capability", &caps) {
        assert!(
            matches!(
                err.kind(),
                ErrorKind::PermissionDenied | ErrorKind::Unsupported
            ),
            "{err}"
        );
        return;
    }
    let original = fs::read(&output).unwrap();
    compiler.run().unwrap();
    fs::write(&schema, "array Word [byte; 3];").unwrap();
    let err = compiler.run().unwrap_err();
    assert!(
        err.contains("E_OUTPUT") && err.contains("content-bound"),
        "{err}"
    );
    let kept = fs::File::open(&output).unwrap();
    assert_eq!(
        kept.metadata().unwrap().ino(),
        src.metadata().unwrap().ino()
    );
    assert_eq!(fs::read(&output).unwrap(), original);
    assert_eq!(
        kept.get_xattr("security.capability").unwrap().as_deref(),
        Some(caps.as_slice())
    );
    assert_eq!(dir.path().read_dir().unwrap().count(), 2);
}
