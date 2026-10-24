//! Library files (LIB-). Build 4 of M1 brings the location (LIB-027) and where each
//! capture is saved as `.xshot` (LIB-020). The index (LIB-022) and the Library view
//! arrive in build 5.

use std::path::{Path, PathBuf};

/// Library folder: `Documents\XcreenShot` on Windows (the real Documents folder, also
/// when OneDrive redirects it), `~/Pictures/XcreenShot` on macOS (LIB-027).
/// `XCREENSHOT_LIBRARY_DIR` overrides it for isolated test instances.
pub fn location() -> PathBuf {
    if let Some(dir) = std::env::var_os("XCREENSHOT_LIBRARY_DIR") {
        return PathBuf::from(dir);
    }
    let dirs = directories::UserDirs::new();
    let base = if cfg!(target_os = "macos") {
        dirs.as_ref().and_then(|d| d.picture_dir().map(Path::to_path_buf))
    } else {
        dirs.as_ref()
            .and_then(|d| d.document_dir().map(Path::to_path_buf))
    };
    base.unwrap_or_else(std::env::temp_dir).join("XcreenShot")
}

/// Whether `path` is inside the library folder `root`. Library documents save
/// themselves; files from elsewhere ask (EDT-012).
pub fn contains(root: &Path, path: &Path) -> bool {
    let canon = |p: &Path| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    let file_dir = path.parent().map(canon).unwrap_or_default();
    file_dir.starts_with(canon(root))
}

/// Characters that no file name may contain on Windows or macOS.
const INVALID: &[char] = &['\\', '/', ':', '*', '?', '"', '<', '>', '|'];

/// A source app name that is safe as part of a file name, or `XcreenShot`.
pub fn safe_name(name: Option<&str>) -> String {
    let cleaned: String = name
        .unwrap_or_default()
        .chars()
        .filter(|c| !c.is_control() && !INVALID.contains(c))
        .take(60)
        .collect();
    let cleaned = cleaned.trim().trim_end_matches('.').trim().to_string();
    if cleaned.is_empty() {
        "XcreenShot".to_string()
    } else {
        cleaned
    }
}

/// Where a new library item goes (LIB-020): a `AAAA-MM` folder per month and a name
/// like `Chrome 2026-10-12 14.03.22.xshot`, with ` (2)`, ` (3)`… if it exists.
/// Provisional pattern until SHR-006 (file names) is written.
pub fn new_item_path(
    root: &Path,
    app: Option<&str>,
    now: chrono::DateTime<chrono::Local>,
    extension: &str,
) -> PathBuf {
    let dir = root.join(now.format("%Y-%m").to_string());
    let stem = format!("{} {}", safe_name(app), now.format("%Y-%m-%d %H.%M.%S"));
    unique(&dir, &stem, extension)
}

/// `dir/stem.ext`, or `dir/stem (2).ext`… if taken (also by a half-written file).
pub fn unique(dir: &Path, stem: &str, extension: &str) -> PathBuf {
    let taken = |p: &Path| {
        let mut partial = p.as_os_str().to_os_string();
        partial.push(".partial");
        p.exists() || Path::new(&partial).exists()
    };
    let mut path = dir.join(format!("{stem}.{extension}"));
    let mut n = 2;
    while taken(&path) {
        path = dir.join(format!("{stem} ({n}).{extension}"));
        n += 1;
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at() -> chrono::DateTime<chrono::Local> {
        chrono::Local.with_ymd_and_hms(2026, 10, 12, 14, 3, 22).unwrap()
    }

    #[test]
    fn names_by_app_and_time_in_a_month_folder() {
        let root = tempfile::tempdir().unwrap();
        let p = new_item_path(root.path(), Some("Chrome"), at(), "xshot");
        assert_eq!(
            p,
            root.path()
                .join("2026-10")
                .join("Chrome 2026-10-12 14.03.22.xshot")
        );
    }

    #[test]
    fn two_items_in_the_same_second_get_distinct_names() {
        let root = tempfile::tempdir().unwrap();
        let first = new_item_path(root.path(), Some("Excel"), at(), "xshot");
        std::fs::create_dir_all(first.parent().unwrap()).unwrap();
        std::fs::write(&first, b"x").unwrap();
        let second = new_item_path(root.path(), Some("Excel"), at(), "xshot");
        assert_ne!(first, second);
        assert!(second.ends_with("Excel 2026-10-12 14.03.22 (2).xshot"));
    }

    #[test]
    fn unsafe_app_names_are_cleaned() {
        assert_eq!(safe_name(Some("a/b:c*?.")), "abc");
        assert_eq!(safe_name(Some("  ")), "XcreenShot");
        assert_eq!(safe_name(None), "XcreenShot");
    }

    #[test]
    fn knows_what_is_inside_the_library() {
        let root = tempfile::tempdir().unwrap();
        let inside = root.path().join("2026-10").join("a.xshot");
        std::fs::create_dir_all(inside.parent().unwrap()).unwrap();
        assert!(contains(root.path(), &inside));
        let other = tempfile::tempdir().unwrap();
        assert!(!contains(root.path(), &other.path().join("b.png")));
    }
}
