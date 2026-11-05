//! The API key lives only in the system keychain (SMT-039, SMT-042): the Windows
//! Credential Manager or the macOS Keychain. Never in a file or a log.

use std::sync::Mutex;

/// Where the key is kept. Tests use [`MemoryVault`].
pub trait Vault: Send + Sync {
    fn get(&self) -> Result<Option<String>, String>;
    fn set(&self, key: &str) -> Result<(), String>;
    fn delete(&self) -> Result<(), String>;
}

/// The system keychain entry of this instance. An isolated instance
/// (`XCREENSHOT_DATA_DIR`) gets its own entry, so tests never touch the real key.
pub struct SystemVault {
    target: String,
}

impl SystemVault {
    pub fn new(instance: Option<&str>) -> SystemVault {
        let target = match instance {
            Some(scope) => format!("XcreenShot/anthropic-api-key/{:016x}", fnv(scope)),
            None => "XcreenShot/anthropic-api-key".to_string(),
        };
        SystemVault { target }
    }
}

fn fnv(s: &str) -> u64 {
    s.bytes().fold(0xcbf29ce484222325, |h, b| {
        (h ^ b as u64).wrapping_mul(0x100000001b3)
    })
}

impl Vault for SystemVault {
    fn get(&self) -> Result<Option<String>, String> {
        platform::get(&self.target)
    }
    fn set(&self, key: &str) -> Result<(), String> {
        platform::set(&self.target, key)
    }
    fn delete(&self) -> Result<(), String> {
        platform::delete(&self.target)
    }
}

/// A key held in memory only (tests).
#[derive(Default)]
pub struct MemoryVault(Mutex<Option<String>>);

impl Vault for MemoryVault {
    fn get(&self) -> Result<Option<String>, String> {
        Ok(self.0.lock().unwrap().clone())
    }
    fn set(&self, key: &str) -> Result<(), String> {
        *self.0.lock().unwrap() = Some(key.to_string());
        Ok(())
    }
    fn delete(&self) -> Result<(), String> {
        *self.0.lock().unwrap() = None;
        Ok(())
    }
}

#[cfg(windows)]
mod platform {
    use std::ptr::null_mut;

    use windows_sys::Win32::Foundation::{GetLastError, ERROR_NOT_FOUND, FILETIME};
    use windows_sys::Win32::Security::Credentials::{
        CredDeleteW, CredFree, CredReadW, CredWriteW, CREDENTIALW, CRED_PERSIST_LOCAL_MACHINE,
        CRED_TYPE_GENERIC,
    };

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(Some(0)).collect()
    }

    pub fn get(target: &str) -> Result<Option<String>, String> {
        let name = wide(target);
        let mut cred: *mut CREDENTIALW = null_mut();
        // SAFETY: `name` is NUL-terminated and outlives the call; `cred` receives a
        // buffer the system allocates, freed below with CredFree.
        let ok = unsafe { CredReadW(name.as_ptr(), CRED_TYPE_GENERIC, 0, &mut cred) };
        if ok == 0 {
            // SAFETY: plain thread-local error read.
            let err = unsafe { GetLastError() };
            return if err == ERROR_NOT_FOUND {
                Ok(None)
            } else {
                Err(format!("CredReadW failed ({err})"))
            };
        }
        // SAFETY: on success `cred` points to a valid CREDENTIALW whose blob holds
        // `CredentialBlobSize` bytes; both stay valid until CredFree.
        let secret = unsafe {
            let c = &*cred;
            let bytes = std::slice::from_raw_parts(c.CredentialBlob, c.CredentialBlobSize as usize);
            let secret = String::from_utf8(bytes.to_vec());
            CredFree(cred as *const _);
            secret
        };
        secret
            .map(Some)
            .map_err(|_| "the saved key is not text".to_string())
    }

    pub fn set(target: &str, key: &str) -> Result<(), String> {
        let name = wide(target);
        let user = wide("XcreenShot");
        let blob = key.as_bytes();
        let cred = CREDENTIALW {
            Flags: 0,
            Type: CRED_TYPE_GENERIC,
            TargetName: name.as_ptr() as *mut u16,
            Comment: null_mut(),
            LastWritten: FILETIME {
                dwLowDateTime: 0,
                dwHighDateTime: 0,
            },
            CredentialBlobSize: blob.len() as u32,
            CredentialBlob: blob.as_ptr() as *mut u8,
            Persist: CRED_PERSIST_LOCAL_MACHINE,
            AttributeCount: 0,
            Attributes: null_mut(),
            TargetAlias: null_mut(),
            UserName: user.as_ptr() as *mut u16,
        };
        // SAFETY: every pointer in `cred` refers to buffers alive for the call; the
        // system copies them.
        if unsafe { CredWriteW(&cred, 0) } == 0 {
            // SAFETY: plain thread-local error read.
            return Err(format!("CredWriteW failed ({})", unsafe { GetLastError() }));
        }
        Ok(())
    }

    pub fn delete(target: &str) -> Result<(), String> {
        let name = wide(target);
        // SAFETY: `name` is NUL-terminated and outlives the call.
        if unsafe { CredDeleteW(name.as_ptr(), CRED_TYPE_GENERIC, 0) } == 0 {
            // SAFETY: plain thread-local error read.
            let err = unsafe { GetLastError() };
            if err != ERROR_NOT_FOUND {
                return Err(format!("CredDeleteW failed ({err})"));
            }
        }
        Ok(())
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use security_framework::passwords::{
        delete_generic_password, get_generic_password, set_generic_password,
    };

    const ACCOUNT: &str = "api-key";
    /// `errSecItemNotFound`.
    const NOT_FOUND: i32 = -25300;

    pub fn get(target: &str) -> Result<Option<String>, String> {
        match get_generic_password(target, ACCOUNT) {
            Ok(bytes) => String::from_utf8(bytes)
                .map(Some)
                .map_err(|_| "the saved key is not text".to_string()),
            Err(e) if e.code() == NOT_FOUND => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn set(target: &str, key: &str) -> Result<(), String> {
        set_generic_password(target, ACCOUNT, key.as_bytes()).map_err(|e| e.to_string())
    }

    pub fn delete(target: &str) -> Result<(), String> {
        match delete_generic_password(target, ACCOUNT) {
            Err(e) if e.code() != NOT_FOUND => Err(e.to_string()),
            _ => Ok(()),
        }
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
mod platform {
    pub fn get(_target: &str) -> Result<Option<String>, String> {
        Ok(None)
    }
    pub fn set(_target: &str, _key: &str) -> Result<(), String> {
        Err("no system keychain on this platform".to_string())
    }
    pub fn delete(_target: &str) -> Result<(), String> {
        Ok(())
    }
}
