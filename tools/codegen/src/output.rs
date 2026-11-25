use std::{
    fs::{self, OpenOptions},
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "netbsd"
))]
mod attributes;

// Rename at the target of a symbolic link, including when the target is still absent.
fn destination(path: &Path) -> io::Result<PathBuf> {
    let mut path = path.to_owned();
    for _ in 0..=40 {
        match fs::symlink_metadata(&path) {
            Ok(meta) if meta.is_symlink() => {
                path = path.with_file_name(fs::read_link(&path)?);
            }
            Ok(meta) if meta.is_file() => return Ok(path),
            Ok(_) => return Err(io::Error::other("output is not a regular file")),
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(path),
            Err(err) => return Err(err),
        }
    }
    Err(io::Error::other("too many symbolic links in output path"))
}

// Keep one handle for link count, contents and metadata. A read-only handle
// can still prove that a multiply linked output needs no replacement.
fn open_existing(path: &Path) -> io::Result<(fs::File, Option<io::Error>)> {
    let mut options = OpenOptions::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(0x0020_0000); // FILE_FLAG_OPEN_REPARSE_POINT
    }
    match options.read(true).write(true).open(path) {
        Ok(file) => return Ok((file, None)),
        Err(err) if err.kind() == io::ErrorKind::PermissionDenied => (),
        Err(err) => return Err(err),
    }
    match options.read(false).open(path) {
        Ok(file) => Ok((file, None)),
        Err(err) if err.kind() == io::ErrorKind::PermissionDenied => {
            Ok((options.read(true).write(false).open(path)?, Some(err)))
        }
        Err(err) => Err(err),
    }
}

#[cfg(unix)]
fn prepare_replacement(
    _src: &fs::File,
    original: &fs::Metadata,
    file: &fs::File,
) -> io::Result<()> {
    use std::os::unix::fs::{fchown, MetadataExt};
    let staged = file.metadata()?;
    let uid = (original.uid() != staged.uid()).then(|| original.uid());
    let gid = (original.gid() != staged.gid()).then(|| original.gid());
    if uid.is_some() || gid.is_some() {
        fchown(file, uid, gid)?;
    }
    file.set_permissions(original.permissions())?;
    #[cfg(target_os = "macos")]
    copy_macos_metadata(_src, file)?;
    #[cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "freebsd",
        target_os = "netbsd"
    ))]
    attributes::copy(_src, file).map_err(|err| {
        io::Error::new(
            err.kind(),
            format!("cannot preserve ACLs and extended attributes: {err}"),
        )
    })?;
    #[cfg(any(target_os = "openbsd", target_os = "dragonfly"))]
    verify_bsd_filesystem(_src)?;
    Ok(())
}

#[cfg(any(target_os = "openbsd", target_os = "dragonfly"))]
fn verify_bsd_filesystem(src: &fs::File) -> io::Result<()> {
    use std::{ffi::CStr, mem::MaybeUninit, os::fd::AsRawFd};
    let mut stat = MaybeUninit::<libc::statfs>::uninit();
    // SAFETY: on success fstatfs initializes the structure and terminates the filesystem name.
    let stat = unsafe {
        if libc::fstatfs(src.as_raw_fd(), stat.as_mut_ptr()) != 0 {
            return Err(io::Error::last_os_error());
        }
        stat.assume_init()
    };
    let kind = unsafe { CStr::from_ptr(stat.f_fstypename.as_ptr()) }.to_bytes();
    // These local filesystems expose only Unix permissions on these systems.
    #[cfg(target_os = "openbsd")]
    if matches!(kind, b"ffs" | b"mfs" | b"tmpfs") {
        return Ok(());
    }
    #[cfg(target_os = "dragonfly")]
    if matches!(kind, b"hammer" | b"hammer2" | b"tmpfs") {
        return Ok(());
    }
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "cannot verify ACLs and extended attributes on this filesystem",
    ))
}

#[cfg(target_os = "macos")]
fn copy_macos_metadata(src: &fs::File, dst: &fs::File) -> io::Result<()> {
    use std::{ffi::c_void, os::fd::AsRawFd, ptr};
    const COPYFILE_XATTR: u32 = 1 << 2;
    extern "C" {
        fn flistxattr(fd: i32, names: *mut std::ffi::c_char, size: usize, options: i32) -> isize;
        fn fcopyfile(from: i32, to: i32, state: *mut c_void, flags: u32) -> i32;
        fn acl_get_fd(fd: i32) -> *mut c_void;
        fn acl_set_fd(fd: i32, acl: *mut c_void) -> i32;
        fn acl_init(count: i32) -> *mut c_void;
        fn acl_free(acl: *mut c_void) -> i32;
    }
    // Darwin distinguishes ENOTSUP (45) from EOPNOTSUPP on some targets.
    fn unsupported(err: &io::Error) -> bool {
        err.raw_os_error() == Some(45) || err.kind() == io::ErrorKind::Unsupported
    }
    // SAFETY: Both descriptors remain open. The ACL comes from libSystem,
    // is copied by acl_set_fd and is freed exactly once.
    unsafe {
        if fcopyfile(
            src.as_raw_fd(),
            dst.as_raw_fd(),
            ptr::null_mut(),
            COPYFILE_XATTR,
        ) != 0
        {
            let err = io::Error::last_os_error();
            // On a volume without xattrs, absence on the source is sufficient.
            if !unsupported(&err) || {
                let attributes = flistxattr(src.as_raw_fd(), ptr::null_mut(), 0, 0);
                attributes > 0 || (attributes < 0 && !unsupported(&io::Error::last_os_error()))
            } {
                return Err(io::Error::new(
                    err.kind(),
                    format!("cannot preserve extended attributes: {err}"),
                ));
            }
        }
        let mut acl = acl_get_fd(src.as_raw_fd());
        let empty = acl.is_null();
        if empty {
            let err = io::Error::last_os_error();
            if unsupported(&err) {
                return Ok(());
            }
            if err.kind() != io::ErrorKind::NotFound {
                return Err(err);
            }
            // fcopyfile(COPYFILE_ACL) merges inherited entries. A source
            // without an ACL must explicitly replace them with an empty ACL.
            acl = acl_init(0);
            if acl.is_null() {
                return Err(io::Error::last_os_error());
            }
        }
        let result = acl_set_fd(dst.as_raw_fd(), acl);
        let error = (result != 0).then(io::Error::last_os_error);
        acl_free(acl);
        match error {
            Some(err) if !(empty && unsupported(&err)) => Err(err),
            _ => Ok(()),
        }
    }
}

fn same_contents(file: &fs::File, mut bytes: &[u8]) -> io::Result<bool> {
    let mut reader = io::BufReader::new(file);
    loop {
        let chunk = reader.fill_buf()?;
        if chunk.is_empty() {
            return Ok(bytes.is_empty());
        }
        let Some(rest) = bytes.strip_prefix(chunk) else {
            return Ok(false);
        };
        bytes = rest;
        let count = chunk.len();
        reader.consume(count);
    }
}

struct Temporary {
    path: PathBuf,
    pending: bool,
}

impl Drop for Temporary {
    fn drop(&mut self) {
        if self.pending {
            let _ = fs::remove_file(&self.path);
        }
    }
}

pub(crate) fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let path = destination(path)?;
    let original = match open_existing(&path) {
        Ok((file, write_denied)) => {
            let meta = file.metadata()?;
            if !meta.is_file() {
                return Err(io::Error::other("output is not a regular file"));
            }
            #[cfg(unix)]
            let linked = std::os::unix::fs::MetadataExt::nlink(&meta) > 1;
            #[cfg(windows)]
            let linked = winapi_util::file::information(&file)?.number_of_links() > 1;
            #[cfg(not(any(unix, windows)))]
            let linked = false;
            if !linked {
                if let Some(err) = write_denied {
                    return Err(err);
                }
                #[cfg(windows)]
                if meta.permissions().readonly() {
                    return Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "output is read-only",
                    ));
                }
            }
            if meta.len() == bytes.len() as u64 {
                match same_contents(&file, bytes) {
                    Ok(true) => return Ok(()),
                    Err(err) if linked => {
                        return Err(io::Error::new(
                            err.kind(),
                            format!("cannot compare an output with multiple hard links: {err}"),
                        ))
                    }
                    _ => (),
                }
            }
            if linked {
                return Err(io::Error::other(
                    "cannot replace an output with multiple hard links",
                ));
            }
            Some((file, meta))
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => None,
        Err(err) => return Err(err),
    };
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    if original.is_some() {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    loop {
        let suffix = NEXT.fetch_add(1, Ordering::Relaxed);
        let tmp = path.with_file_name(format!(".molecule-{}-{suffix}.tmp", std::process::id()));
        let mut file = match options.open(&tmp) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        };
        let mut tmp = Temporary {
            path: tmp,
            pending: true,
        };
        // Install the original ACL before any content appears in a staging file
        // that may have inherited broader access from its directory.
        #[cfg(unix)]
        if let Some((src, meta)) = &original {
            prepare_replacement(src, meta, &file)?;
        }
        file.write_all(bytes)?;
        file.flush()?;
        if let Some((_src, meta)) = &original {
            if file.metadata()?.permissions() != meta.permissions() {
                file.set_permissions(meta.permissions())?;
                #[cfg(any(target_os = "freebsd", target_os = "netbsd"))]
                attributes::copy_acl(_src, &file)?;
                if file.metadata()?.permissions() != meta.permissions() {
                    return Err(io::Error::other("cannot preserve output permissions"));
                }
            }
        }
        #[cfg(any(
            target_os = "linux",
            target_os = "android",
            target_os = "freebsd",
            target_os = "netbsd"
        ))]
        if let Some((src, _)) = &original {
            attributes::verify(src, &file).map_err(|err| {
                io::Error::new(
                    err.kind(),
                    format!("cannot verify output extended attributes: {err}"),
                )
            })?;
        }
        drop(file);
        drop(original);
        fs::rename(&tmp.path, path)?;
        tmp.pending = false;
        return Ok(());
    }
}
