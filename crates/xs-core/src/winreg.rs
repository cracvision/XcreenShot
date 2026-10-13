//! Minimal read-only Windows registry helpers.

use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::System::Registry::{
    RegDeleteKeyValueW, RegGetValueW, HKEY, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, RRF_RT_REG_BINARY,
    RRF_RT_REG_DWORD, RRF_RT_REG_SZ,
};

/// Registry hive. Only the predefined roots are allowed, so callers never pass raw handles.
#[derive(Debug, Clone, Copy)]
pub enum Root {
    LocalMachine,
    CurrentUser,
}

pub use Root::{CurrentUser as HKCU, LocalMachine as HKLM};

fn hkey(root: Root) -> HKEY {
    match root {
        Root::LocalMachine => HKEY_LOCAL_MACHINE,
        Root::CurrentUser => HKEY_CURRENT_USER,
    }
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn read_dword(root: Root, subkey: &str, value: &str) -> Option<u32> {
    let (k, v) = (wide(subkey), wide(value));
    let mut data = 0u32;
    let mut size = std::mem::size_of::<u32>() as u32;
    // SAFETY: buffers are valid for the sizes passed.
    let rc = unsafe {
        RegGetValueW(
            hkey(root),
            k.as_ptr(),
            v.as_ptr(),
            RRF_RT_REG_DWORD,
            std::ptr::null_mut(),
            (&mut data as *mut u32).cast(),
            &mut size,
        )
    };
    (rc == ERROR_SUCCESS).then_some(data)
}

pub fn read_string(root: Root, subkey: &str, value: &str) -> Option<String> {
    let (k, v) = (wide(subkey), wide(value));
    let mut buf = vec![0u16; 512];
    let mut size = (buf.len() * 2) as u32;
    // SAFETY: buffers are valid for the sizes passed.
    let rc = unsafe {
        RegGetValueW(
            hkey(root),
            k.as_ptr(),
            v.as_ptr(),
            RRF_RT_REG_SZ,
            std::ptr::null_mut(),
            buf.as_mut_ptr().cast(),
            &mut size,
        )
    };
    if rc != ERROR_SUCCESS {
        return None;
    }
    let len = (size as usize / 2).saturating_sub(1);
    Some(String::from_utf16_lossy(&buf[..len]))
}

pub fn read_binary(root: Root, subkey: &str, value: &str) -> Option<Vec<u8>> {
    let (k, v) = (wide(subkey), wide(value));
    let mut buf = vec![0u8; 256];
    let mut size = buf.len() as u32;
    // SAFETY: buffers are valid for the sizes passed.
    let rc = unsafe {
        RegGetValueW(
            hkey(root),
            k.as_ptr(),
            v.as_ptr(),
            RRF_RT_REG_BINARY,
            std::ptr::null_mut(),
            buf.as_mut_ptr().cast(),
            &mut size,
        )
    };
    (rc == ERROR_SUCCESS).then(|| {
        buf.truncate(size as usize);
        buf
    })
}

/// Deletes a value. Returns true if it existed and was removed.
pub fn delete_value(root: Root, subkey: &str, value: &str) -> bool {
    let (k, v) = (wide(subkey), wide(value));
    // SAFETY: null-terminated UTF-16 strings that outlive the call.
    unsafe { RegDeleteKeyValueW(hkey(root), k.as_ptr(), v.as_ptr()) == ERROR_SUCCESS }
}

const STARTUP_APPROVED: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";

/// Settings > Apps > Startup does not delete a login item when the user turns it off:
/// it records the choice under `StartupApproved\Run`. An odd first byte (0x03, 0x07)
/// means turned off; even (0x02, 0x06) or no entry means on.
pub fn startup_entry_turned_off(bytes: &[u8]) -> bool {
    bytes.first().is_some_and(|b| b & 1 == 1)
}

/// Whether the user turned off the login item `name` in Windows Settings (APP-003).
pub fn login_item_turned_off(name: &str) -> bool {
    read_binary(HKCU, STARTUP_APPROVED, name).is_some_and(|b| startup_entry_turned_off(&b))
}

/// Clears the user's "off" choice so that turning the option on in Preferences works.
pub fn clear_login_item_override(name: &str) {
    delete_value(HKCU, STARTUP_APPROVED, name);
}

/// Whether Windows sends Print Screen to its own capture tool (APP-019, APP-020).
/// Windows 11 turns this on by default when the value is missing.
pub fn print_screen_opens_system_capture() -> bool {
    match read_dword(
        HKCU,
        r"Control Panel\Keyboard",
        "PrintScreenKeyForSnippingEnabled",
    ) {
        Some(v) => v != 0,
        None => crate::core::windows_build().is_some_and(|b| b >= 22000),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn startup_approved_flag() {
        use super::startup_entry_turned_off;
        assert!(startup_entry_turned_off(&[3, 0, 0, 0, 0xd8, 0xf9]));
        assert!(startup_entry_turned_off(&[7, 0]));
        assert!(!startup_entry_turned_off(&[2, 0, 0, 0]));
        assert!(!startup_entry_turned_off(&[6]));
        assert!(!startup_entry_turned_off(&[]));
    }

    #[test]
    fn reads_build_number() {
        assert!(crate::core::windows_build().unwrap() >= 10240);
    }
}
