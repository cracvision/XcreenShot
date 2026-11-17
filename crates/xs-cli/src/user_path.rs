//! The installer's `xcreenshot setup` (APP-037, APP-042): adds the install folder to the
//! user's PATH, or takes it out, so `xcreenshot` works in a new terminal. Done here and
//! not in the installer's script, whose strings are cut at 1024 characters: a long
//! PATH would come out damaged.

/// The folders of a PATH value with `dir` added once, or removed (same folder written
/// with another case or a final backslash counts as the same).
pub fn edited(path: &str, dir: &str, add: bool) -> String {
    let norm = |s: &str| s.trim().trim_end_matches(['\\', '/']).to_lowercase();
    let target = norm(dir);
    let mut parts: Vec<&str> = path
        .split(';')
        .filter(|p| !p.trim().is_empty())
        .filter(|p| norm(p) != target)
        .collect();
    if add {
        parts.push(dir);
    }
    parts.join(";")
}

#[cfg(windows)]
pub fn change(dir: &str, add: bool) -> Result<(), String> {
    use windows_sys::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
    use windows_sys::Win32::System::Registry::{
        RegGetValueW, RegSetKeyValueW, HKEY_CURRENT_USER, REG_EXPAND_SZ, RRF_NOEXPAND, RRF_RT_REG_EXPAND_SZ,
        RRF_RT_REG_SZ,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
    };
    let wide = |s: &str| -> Vec<u16> { s.encode_utf16().chain(std::iter::once(0)).collect() };
    let (key, name) = (wide("Environment"), wide("Path"));
    let mut size = 0u32;
    // SAFETY: a size query with no buffer.
    let rc = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            key.as_ptr(),
            name.as_ptr(),
            RRF_RT_REG_SZ | RRF_RT_REG_EXPAND_SZ | RRF_NOEXPAND,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut size,
        )
    };
    let current = if rc == ERROR_FILE_NOT_FOUND {
        String::new()
    } else if rc != ERROR_SUCCESS {
        return Err(format!("reading the PATH failed ({rc})"));
    } else {
        let mut buf = vec![0u16; (size as usize).div_ceil(2) + 1];
        let mut size = (buf.len() * 2) as u32;
        // SAFETY: the buffer is as large as `size` says.
        let rc = unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                key.as_ptr(),
                name.as_ptr(),
                RRF_RT_REG_SZ | RRF_RT_REG_EXPAND_SZ | RRF_NOEXPAND,
                std::ptr::null_mut(),
                buf.as_mut_ptr().cast(),
                &mut size,
            )
        };
        if rc != ERROR_SUCCESS {
            return Err(format!("reading the PATH failed ({rc})"));
        }
        let len = buf.iter().position(|c| *c == 0).unwrap_or(buf.len());
        String::from_utf16_lossy(&buf[..len])
    };
    let next = edited(&current, dir, add);
    if next == current {
        return Ok(());
    }
    let value = wide(&next);
    // SAFETY: the value is a NUL-terminated UTF-16 string of the size given.
    let rc = unsafe {
        RegSetKeyValueW(
            HKEY_CURRENT_USER,
            key.as_ptr(),
            name.as_ptr(),
            REG_EXPAND_SZ,
            value.as_ptr().cast(),
            (value.len() * 2) as u32,
        )
    };
    if rc != ERROR_SUCCESS {
        return Err(format!("writing the PATH failed ({rc})"));
    }
    // New terminals see it without signing out.
    let env = wide("Environment");
    // SAFETY: a broadcast with a valid string, bounded by a timeout.
    unsafe {
        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            0,
            env.as_ptr() as isize,
            SMTO_ABORTIFHUNG,
            2000,
            std::ptr::null_mut(),
        );
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn change(_dir: &str, _add: bool) -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn added_once_and_removed_cleanly() {
        let p = r"C:\Windows;%USERPROFILE%\bin";
        let dir = r"C:\Users\a\AppData\Local\XcreenShot";
        let added = edited(p, dir, true);
        assert_eq!(added, format!("{p};{dir}"));
        // The same folder in another spelling is not added twice.
        let again = edited(&added, r"c:\users\a\appdata\local\xcreenshot\", true);
        assert_eq!(again.matches(';').count(), 2);
        assert_eq!(edited(&added, dir, false), p);
        assert_eq!(edited("", r"C:\X", true), r"C:\X");
        assert_eq!(edited(r"C:\A;;C:\X;", r"C:\X", false), r"C:\A");
    }
}
