//! Metadata for a same-directory replacement. Only the staging inode is modified.
use std::{collections::BTreeMap, ffi::OsString, fs::File, io};
#[cfg(any(target_os = "freebsd", target_os = "netbsd"))]
use std::{os::fd::AsRawFd, ptr};
use xattr::FileExt;

type Attributes = BTreeMap<OsString, Vec<u8>>;

fn attributes(file: &File) -> io::Result<Attributes> {
    let mut attrs = BTreeMap::new();
    for name in names(file)? {
        // Some filesystems list protected names even when their values are hidden.
        #[cfg(any(target_os = "linux", target_os = "android"))]
        let protected = name.as_encoded_bytes().starts_with(b"trusted.");
        #[cfg(any(target_os = "freebsd", target_os = "netbsd"))]
        let protected = name.as_encoded_bytes().starts_with(b"system.");
        match file.get_xattr(&name) {
            Ok(Some(value)) => {
                attrs.insert(name, value);
            }
            Ok(None) if protected => continue,
            Err(err) if protected && err.kind() == io::ErrorKind::PermissionDenied => continue,
            Ok(None) => return Err(io::Error::other("extended attribute disappeared")),
            Err(err) => return Err(err),
        }
    }
    Ok(attrs)
}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn names(file: &File) -> io::Result<Vec<OsString>> {
    match file.list_xattr() {
        Ok(names) => Ok(names.collect()),
        Err(e) if e.kind() == io::ErrorKind::Unsupported => Ok(Vec::new()),
        Err(e) => Err(e),
    }
}

pub(super) fn copy(src: &File, dst: &File) -> io::Result<()> {
    #[cfg(any(target_os = "freebsd", target_os = "netbsd"))]
    copy_acl(src, dst)?;
    let wanted = attributes(src)?;
    // Replacing the contents invalidates integrity signatures and executable
    // capabilities, even if their attribute bytes could be copied.
    #[cfg(any(target_os = "linux", target_os = "android"))]
    if ["security.capability", "security.ima", "security.evm"]
        .iter()
        .any(|key| wanted.contains_key(std::ffi::OsStr::new(key)))
    {
        return Err(io::Error::other(
            "content-bound security metadata cannot be preserved on changed output",
        ));
    }
    let inherited = attributes(dst)?;
    for name in inherited.keys() {
        if !wanted.contains_key(name) {
            dst.remove_xattr(name)?;
        }
    }
    for (name, value) in &wanted {
        if inherited.get(name) != Some(value) {
            dst.set_xattr(name, value)?;
        }
    }
    Ok(())
}

pub(super) fn verify(src: &File, dst: &File) -> io::Result<()> {
    if attributes(src)? != attributes(dst)? {
        return Err(io::Error::other(
            "extended attributes changed during replacement",
        ));
    }
    Ok(())
}

#[cfg(any(target_os = "freebsd", target_os = "netbsd"))]
fn names(file: &File) -> io::Result<Vec<OsString>> {
    use std::os::unix::ffi::OsStringExt;
    let mut names = Vec::new();
    // extattr_list_fd returns length-prefixed names in each namespace.
    for (ns, prefix) in [
        (libc::EXTATTR_NAMESPACE_USER, b"user.".as_slice()),
        (libc::EXTATTR_NAMESPACE_SYSTEM, b"system.".as_slice()),
    ] {
        // SAFETY: the first call queries the size, the second writes into that buffer.
        let list = |ptr, len| unsafe { libc::extattr_list_fd(file.as_raw_fd(), ns, ptr, len) };
        let len = list(ptr::null_mut(), 0);
        if len < 0 {
            let err = io::Error::last_os_error();
            if err.kind() == io::ErrorKind::Unsupported
                || (ns == libc::EXTATTR_NAMESPACE_SYSTEM
                    && err.kind() == io::ErrorKind::PermissionDenied)
            {
                continue;
            }
            let namespace = if ns == libc::EXTATTR_NAMESPACE_SYSTEM {
                "system"
            } else {
                "user"
            };
            return Err(io::Error::new(
                err.kind(),
                format!("cannot list {namespace} extended attributes: {err}"),
            ));
        }
        let mut data = vec![0u8; len as usize];
        if len == 0 {
            continue;
        }
        let count = list(data.as_mut_ptr().cast(), data.len());
        if count < 0 {
            return Err(io::Error::last_os_error());
        }
        let invalid = || io::Error::other("extended-attribute list changed");
        if count != len {
            return Err(invalid());
        }
        let mut remaining = data.as_slice();
        while let Some((&len, rest)) = remaining.split_first() {
            let (name, rest) = rest
                .split_at_checked(usize::from(len))
                .ok_or_else(invalid)?;
            names.push(OsString::from_vec([prefix, name].concat()));
            remaining = rest;
        }
    }
    Ok(names)
}

#[cfg(any(target_os = "freebsd", target_os = "netbsd"))]
pub(super) fn copy_acl(src: &File, dst: &File) -> io::Result<()> {
    use std::ffi::c_void;
    #[cfg(target_os = "netbsd")]
    const PC_ACL_NFS4: i32 = 17; // <sys/unistd.h>
    #[cfg(target_os = "freebsd")]
    const PC_ACL_NFS4: i32 = libc::_PC_ACL_NFS4;
    extern "C" {
        fn acl_get_fd(fd: i32) -> *mut c_void;
        fn acl_set_fd(fd: i32, acl: *mut c_void) -> i32;
        fn acl_free(acl: *mut c_void) -> i32;
    }
    for kind in [PC_ACL_NFS4, libc::_PC_ACL_EXTENDED] {
        // SAFETY: the queried fd stays open. acl_set_fd copies the ACL into
        // the kernel. The allocation returned by acl_get_fd is freed once.
        unsafe {
            match libc::fpathconf(src.as_raw_fd(), kind) {
                -1 => {
                    let err = io::Error::last_os_error();
                    if err.raw_os_error() != Some(libc::EINVAL) {
                        return Err(err);
                    }
                }
                0 => (),
                _ => {
                    let acl = acl_get_fd(src.as_raw_fd());
                    if acl.is_null() {
                        return Err(io::Error::last_os_error());
                    }
                    let result = acl_set_fd(dst.as_raw_fd(), acl);
                    let err = (result != 0).then(io::Error::last_os_error);
                    acl_free(acl);
                    return err.map_or(Ok(()), Err);
                }
            }
        }
    }
    Ok(())
}
