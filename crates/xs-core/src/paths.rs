//! Per-user app folders. Shared by the app and the CLI.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Paths {
    /// Preferences, shortcuts and small state files.
    pub config_dir: PathBuf,
    /// Logs and other machine-local data.
    pub local_dir: PathBuf,
}

impl Paths {
    /// Standard locations: `%APPDATA%\XcreenShot` / `%LOCALAPPDATA%\XcreenShot` on
    /// Windows, `~/Library/Application Support/XcreenShot` on macOS.
    /// `XCREENSHOT_DATA_DIR` overrides both (tests, portable mode later, APP-038).
    pub fn standard() -> Paths {
        if let Some(dir) = std::env::var_os("XCREENSHOT_DATA_DIR") {
            return Paths::rooted(Path::new(&dir));
        }
        match directories::ProjectDirs::from("", "", "XcreenShot") {
            Some(d) => Paths {
                config_dir: d.config_dir().to_path_buf(),
                local_dir: d.data_local_dir().to_path_buf(),
            },
            None => Paths::rooted(&std::env::temp_dir().join("XcreenShot")),
        }
    }

    pub fn rooted(root: &Path) -> Paths {
        Paths {
            config_dir: root.join("config"),
            local_dir: root.join("local"),
        }
    }

    pub fn settings_file(&self) -> PathBuf {
        self.config_dir.join("settings.json")
    }

    pub fn shortcuts_file(&self) -> PathBuf {
        self.config_dir.join("shortcuts.json")
    }

    pub fn state_file(&self) -> PathBuf {
        self.config_dir.join("state.json")
    }

    pub fn logs_dir(&self) -> PathBuf {
        self.local_dir.join("logs")
    }
}

/// Writes `contents` atomically (temp file + rename) and creates parent folders.
pub fn write_atomic(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, contents)?;
    std::fs::rename(&tmp, path)
}

/// Shortens paths inside the user's home folder to `~/…/<file name>` (APP-046).
pub fn abbreviate(path: &Path) -> String {
    let home = directories::UserDirs::new().map(|u| u.home_dir().to_path_buf());
    match home {
        Some(home) if path.starts_with(&home) => {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            format!("~/…/{name}")
        }
        _ => path.display().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abbreviates_home_paths() {
        let home = directories::UserDirs::new().unwrap().home_dir().to_path_buf();
        let p = home.join("Pictures").join("secret-project").join("capture.png");
        assert_eq!(abbreviate(&p), "~/…/capture.png");
    }
}
