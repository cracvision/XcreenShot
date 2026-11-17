//! Per-user app folders. Shared by the app and the CLI.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Paths {
    /// Preferences, shortcuts and small state files.
    pub config_dir: PathBuf,
    /// Logs and other machine-local data.
    pub local_dir: PathBuf,
}

/// An instance with its own data folder (`XCREENSHOT_DATA_DIR`): a test copy. It never
/// touches what belongs to the installed app, such as starting with the system.
pub fn isolated() -> bool {
    std::env::var_os("XCREENSHOT_DATA_DIR").is_some()
}

/// Portable mode (APP-038): a `portable.flag` next to the executable. Its folder.
pub fn portable_dir() -> Option<PathBuf> {
    static DIR: std::sync::OnceLock<Option<PathBuf>> = std::sync::OnceLock::new();
    DIR.get_or_init(|| {
        let exe = std::env::current_exe().ok()?;
        let dir = exe.parent()?.to_path_buf();
        dir.join("portable.flag").is_file().then_some(dir)
    })
    .clone()
}

/// Portable mode on a folder that cannot be written (a read-only medium): the data
/// live in a temporary folder and nothing is kept after closing (APP-038).
pub fn portable_read_only() -> bool {
    portable_dir().is_some_and(|d| !writable(&d.join("data")))
}

fn writable(dir: &Path) -> bool {
    if std::fs::create_dir_all(dir).is_err() {
        return false;
    }
    let probe = dir.join(".xs-write-test");
    let ok = std::fs::write(&probe, b"").is_ok();
    let _ = std::fs::remove_file(&probe);
    ok
}

impl Paths {
    /// Standard locations: `%APPDATA%\XcreenShot` / `%LOCALAPPDATA%\XcreenShot` on
    /// Windows, `~/Library/Application Support/XcreenShot` on macOS.
    /// `XCREENSHOT_DATA_DIR` overrides both (tests); in portable mode, a `data` folder
    /// next to the executable (APP-038).
    pub fn standard() -> Paths {
        if let Some(dir) = std::env::var_os("XCREENSHOT_DATA_DIR") {
            return Paths::rooted(Path::new(&dir));
        }
        if let Some(dir) = portable_dir() {
            let data = dir.join("data");
            if writable(&data) {
                return Paths::rooted(&data);
            }
            return Paths::rooted(
                &std::env::temp_dir().join(format!("XcreenShot-portable-{}", std::process::id())),
            );
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
