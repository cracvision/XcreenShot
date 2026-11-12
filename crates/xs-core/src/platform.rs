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
