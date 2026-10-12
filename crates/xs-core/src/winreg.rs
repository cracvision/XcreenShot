//! Minimal read-only Windows registry helpers.

use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::System::Registry::{
    RegGetValueW, HKEY, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, RRF_RT_REG_DWORD, RRF_RT_REG_SZ,
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
    fn reads_build_number() {
        assert!(crate::core::windows_build().unwrap() >= 10240);
    }
}
