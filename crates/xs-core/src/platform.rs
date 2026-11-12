//! Platform identification.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Windows,
    Macos,
}

impl Platform {
    pub fn current() -> Platform {
        if cfg!(target_os = "macos") {
            Platform::Macos
        } else {
            Platform::Windows
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Platform::Windows => "windows",
            Platform::Macos => "macos",
        }
    }
}

pub fn arch() -> &'static str {
    std::env::consts::ARCH
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Short commit of this build, with "+" if it had uncommitted changes.
pub const BUILD: &str = env!("XCREENSHOT_BUILD");

/// "0.2.0 (670038c)": what the user sees in the app, diagnostics and `--version`.
/// Day of the build's commit (APP-015).
pub const BUILD_DATE: &str = env!("XCREENSHOT_BUILD_DATE");
pub const VERSION_FULL: &str = concat!(env!("CARGO_PKG_VERSION"), " (", env!("XCREENSHOT_BUILD"), ")");

/// The account's visible name (EDT-097: the footer's author): the full name the
/// system keeps for it, or its sign-in name.
pub fn account_name() -> String {
    let login = std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_default();
    full_name(&login)
        .filter(|n| !n.trim().is_empty())
        .unwrap_or(login)
}

#[cfg(windows)]
fn full_name(login: &str) -> Option<String> {
    use windows_sys::Win32::NetworkManagement::NetManagement::{
        NetApiBufferFree, NetUserGetInfo, USER_INFO_2,
    };
    let user = widestring::U16CString::from_str(login).ok()?;
    let mut buf: *mut u8 = std::ptr::null_mut();
    // SAFETY: a local query; the buffer is the system's, read once and freed.
    unsafe {
        if NetUserGetInfo(std::ptr::null(), user.as_ptr(), 2, &mut buf) != 0 || buf.is_null() {
            return None;
        }
        let info = &*(buf as *const USER_INFO_2);
        let name = (!info.usri2_full_name.is_null())
            .then(|| {
                widestring::U16CStr::from_ptr_str(info.usri2_full_name)
                    .to_string()
                    .ok()
            })
            .flatten();
        NetApiBufferFree(buf as *const _);
        name
    }
}

#[cfg(target_os = "macos")]
fn full_name(_login: &str) -> Option<String> {
    // SAFETY: the record is the system's and read at once; the name before the first
    // comma of its comment field is the full name.
    unsafe {
        let pw = libc::getpwuid(libc::getuid());
        if pw.is_null() || (*pw).pw_gecos.is_null() {
            return None;
        }
        let gecos = std::ffi::CStr::from_ptr((*pw).pw_gecos)
            .to_string_lossy()
            .into_owned();
        gecos.split(',').next().map(str::to_string)
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
fn full_name(_login: &str) -> Option<String> {
    None
}

/// The nearest folder of `path` that exists (itself, or a parent).
fn existing(path: &std::path::Path) -> Option<std::path::PathBuf> {
    path.ancestors()
        .find(|p| p.exists())
        .map(std::path::Path::to_path_buf)
}

/// Whether `path` is on a network share (LIB-030): on Windows a UNC path or a mapped
/// network drive (also while it is disconnected); on macOS an SMB, AFP, NFS or WebDAV
/// volume.
pub fn is_network(path: &std::path::Path) -> bool {
    let text = path.to_string_lossy();
    #[cfg(windows)]
    {
        if text.starts_with(r"\\?\UNC\") {
            return true;
        }
        let local = text.strip_prefix(r"\\?\");
        if local.is_none() && (text.starts_with(r"\\") || text.starts_with("//")) {
            return true;
        }
        let mut chars = local.unwrap_or(&text).chars();
        let (Some(letter), Some(':')) = (chars.next(), chars.next()) else {
            return false;
        };
        if !letter.is_ascii_alphabetic() {
            return false;
        }
        use windows_sys::Win32::Storage::FileSystem::GetDriveTypeW;
        const DRIVE_REMOTE: u32 = 4;
        let root: Vec<u16> = format!("{letter}:\\")
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        // SAFETY: a NUL-terminated drive root.
        unsafe { GetDriveTypeW(root.as_ptr()) == DRIVE_REMOTE }
    }
    #[cfg(target_os = "macos")]
    {
        use std::os::unix::ffi::OsStrExt;
        let _ = &text;
        let Some(dir) = existing(path) else {
            return false;
        };
        let Ok(c) = std::ffi::CString::new(dir.as_os_str().as_bytes()) else {
            return false;
        };
        let mut st: libc::statfs = unsafe { std::mem::zeroed() };
        // SAFETY: a NUL-terminated path and a valid out pointer.
        if unsafe { libc::statfs(c.as_ptr(), &mut st) } != 0 {
            return false;
        }
        // SAFETY: the kernel fills f_fstypename with a NUL-terminated name.
        let kind = unsafe { std::ffi::CStr::from_ptr(st.f_fstypename.as_ptr()) };
        matches!(
            kind.to_bytes(),
            b"smbfs" | b"afpfs" | b"nfs" | b"webdav" | b"cifs"
        )
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        text.starts_with("//")
    }
}

/// Bytes free for the user on the disk of `path` (LIB-028), or `None` if unknown.
pub fn free_space(path: &std::path::Path) -> Option<u64> {
    let dir = existing(path)?;
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
        let wide: Vec<u16> = dir.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
        let (mut avail, mut total, mut free) = (0u64, 0u64, 0u64);
        // SAFETY: a NUL-terminated path and three valid out pointers.
        let ok = unsafe { GetDiskFreeSpaceExW(wide.as_ptr(), &mut avail, &mut total, &mut free) };
        (ok != 0).then_some(avail)
    }
    #[cfg(not(windows))]
    {
        use std::os::unix::ffi::OsStrExt;
        let c = std::ffi::CString::new(dir.as_os_str().as_bytes()).ok()?;
        let mut st: libc::statvfs = unsafe { std::mem::zeroed() };
        // SAFETY: a NUL-terminated path and a valid out pointer.
        let ok = unsafe { libc::statvfs(c.as_ptr(), &mut st) } == 0;
        ok.then(|| st.f_bavail as u64 * st.f_frsize as u64)
    }
}

/// Whether two paths are on the same disk: a move between them is a rename that needs
/// no free space (LIB-028).
pub fn same_volume(a: &std::path::Path, b: &std::path::Path) -> bool {
    let (Some(a), Some(b)) = (existing(a), existing(b)) else {
        return false;
    };
    #[cfg(windows)]
    {
        let root = |p: &std::path::Path| {
            std::fs::canonicalize(p)
                .unwrap_or_else(|_| p.to_path_buf())
                .components()
                .next()
                .map(|c| c.as_os_str().to_string_lossy().to_lowercase())
        };
        root(&a).is_some() && root(&a) == root(&b)
    }
    #[cfg(not(windows))]
    {
        use std::os::unix::fs::MetadataExt;
        match (std::fs::metadata(&a), std::fs::metadata(&b)) {
            (Ok(x), Ok(y)) => x.dev() == y.dev(),
            _ => false,
        }
    }
}

#[cfg(test)]
mod space_tests {
    use super::*;

    #[test]
    fn free_space_and_volumes() {
        let dir = std::env::temp_dir();
        assert!(free_space(&dir).is_some_and(|b| b > 0));
        assert!(
            free_space(&dir.join("not-there-yet").join("deeper")).is_some(),
            "from the nearest folder"
        );
        assert!(same_volume(&dir, &dir.join("x")));
    }

    /// LIB-030: network shares are known by their path; a local folder is not one.
    #[test]
    fn network_paths() {
        assert!(!is_network(&std::env::temp_dir()));
        #[cfg(windows)]
        {
            use std::path::Path;
            assert!(is_network(Path::new(r"\\servidor\equipo\XcreenShot")));
            assert!(is_network(Path::new("//servidor/equipo")));
            assert!(is_network(Path::new(r"\\?\UNC\servidor\equipo")));
            assert!(!is_network(Path::new(r"\\?\C:\Users")));
            assert!(!is_network(Path::new(r"C:\Users")));
        }
    }
}
