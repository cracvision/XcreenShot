//! The folders of the sync services installed on this computer (LIB-029): OneDrive
//! (personal and work or school), Dropbox, Google Drive for desktop, Box Drive and
//! iCloud Drive; one per account. The location picker offers them.

use std::path::{Path, PathBuf};

/// A sync service's folder.
#[derive(Debug, Clone, PartialEq)]
pub struct SyncFolder {
    /// onedrive, onedrive_business, dropbox, google_drive, box, icloud.
    pub service: &'static str,
    /// The account, when the service says it (a work's name).
    pub account: Option<String>,
    pub path: PathBuf,
}

fn existing(service: &'static str, account: Option<String>, path: PathBuf, out: &mut Vec<SyncFolder>) {
    if path.is_dir() && !out.iter().any(|f| f.path == path) {
        out.push(SyncFolder {
            service,
            account,
            path,
        });
    }
}

/// Dropbox writes its accounts' folders in `info.json`.
fn dropbox(base: &Path, out: &mut Vec<SyncFolder>) {
    let Ok(bytes) = std::fs::read(base.join("Dropbox").join("info.json")) else {
        return;
    };
    let Ok(info) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
        return;
    };
    for kind in ["personal", "business"] {
        if let Some(path) = info[kind]["path"].as_str() {
            let account = info[kind]["display_name"]
                .as_str()
                .or(info[kind]["team_name"].as_str())
                .map(str::to_string);
            existing("dropbox", account, PathBuf::from(path), out);
        }
    }
}

/// The sync folders of this computer, as the services left them.
pub fn folders() -> Vec<SyncFolder> {
    let mut out = Vec::new();
    let var = |k: &str| {
        std::env::var_os(k)
            .map(PathBuf::from)
            .filter(|p| !p.as_os_str().is_empty())
    };
    // OneDrive keeps one variable per account kind.
    if let Some(p) = var("OneDriveConsumer") {
        existing("onedrive", None, p, &mut out);
    }
    if let Some(p) = var("OneDriveCommercial") {
        let account = p
            .file_name()
            .map(|n| n.to_string_lossy().trim_start_matches("OneDrive - ").to_string());
        existing("onedrive_business", account, p, &mut out);
    }
    if let Some(p) = var("OneDrive") {
        existing("onedrive", None, p, &mut out);
    }
    let home = directories::BaseDirs::new().map(|d| d.home_dir().to_path_buf());
    for base in [var("APPDATA"), var("LOCALAPPDATA")].into_iter().flatten() {
        dropbox(&base, &mut out);
    }
    if let Some(home) = &home {
        dropbox(&home.join(".dropbox"), &mut out);
        // More OneDrive work accounts: "OneDrive - <organization>" in the home folder.
        if let Ok(entries) = std::fs::read_dir(home) {
            for e in entries.flatten() {
                let name = e.file_name().to_string_lossy().into_owned();
                if let Some(org) = name.strip_prefix("OneDrive - ") {
                    existing("onedrive_business", Some(org.to_string()), e.path(), &mut out);
                }
            }
        }
        existing("box", None, home.join("Box"), &mut out);
        existing("icloud", None, home.join("iCloudDrive"), &mut out);
        existing(
            "icloud",
            None,
            home.join("Library/Mobile Documents/com~apple~CloudDocs"),
            &mut out,
        );
        existing("google_drive", None, home.join("Google Drive"), &mut out);
        // Google Drive for desktop on macOS: ~/Library/CloudStorage/GoogleDrive-<account>.
        if let Ok(entries) = std::fs::read_dir(home.join("Library/CloudStorage")) {
            for e in entries.flatten() {
                let name = e.file_name().to_string_lossy().into_owned();
                if let Some(account) = name.strip_prefix("GoogleDrive-") {
                    existing(
                        "google_drive",
                        Some(account.to_string()),
                        e.path().join("My Drive"),
                        &mut out,
                    );
                }
            }
        }
    }
    // Google Drive for desktop on Windows: its drive, G: unless changed. Other letters
    // are not probed: a disconnected network drive can take seconds to answer.
    #[cfg(windows)]
    for name in ["My Drive", "Mi unidad"] {
        existing("google_drive", None, PathBuf::from("G:\\").join(name), &mut out);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dropbox_accounts_come_from_its_info_file() {
        let base = tempfile::tempdir().unwrap();
        let personal = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(base.path().join("Dropbox")).unwrap();
        let info = serde_json::json!({"personal": {"path": personal.path(), "display_name": "Carlos"}, "business": {"path": "Z:\nowhere"}});
        std::fs::write(base.path().join("Dropbox/info.json"), info.to_string()).unwrap();
        let mut out = Vec::new();
        dropbox(base.path(), &mut out);
        assert_eq!(out.len(), 1, "a folder that is not there is left out");
        assert_eq!(
            (out[0].service, out[0].account.as_deref()),
            ("dropbox", Some("Carlos"))
        );
        assert_eq!(out[0].path, personal.path());
    }
}
