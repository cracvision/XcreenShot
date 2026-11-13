//! The File destination at capture time (CAP-052, CAP-053, SHR-018): each capture is
//! also written to a folder, named by a pattern ([`crate::filename`]), in PNG, JPG or
//! `.xshot`.
//!
//! - The counter of `{counter}` starts at 1 and goes up with each file; there is one for
//!   each folder, pattern and format. A name already taken skips to the next free number.
//! - Without a counter, a name already taken gets " (2)", " (3)"… or is overwritten, as
//!   the destination says.
//! - The folder is the user's Pictures folder, in an `XcreenShot` subfolder, unless one
//!   is chosen. Isolated test instances use their data folder instead.

use std::path::{Path, PathBuf};

use image::RgbaImage;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use xs_core::Core;

use crate::filename::{self, Values};

const COUNTERS_KEY: &str = "share.file.counters";

/// The options of a File destination (CAP-053).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct FileDest {
    /// Empty: the default folder.
    pub folder: String,
    pub prefix: String,
    pub pattern: String,
    /// `png`, `jpg` or `xshot`.
    pub format: String,
    /// JPG quality, 1 to 100.
    pub quality: u8,
    /// `number` (add " (2)") or `overwrite`.
    pub on_exists: String,
}

impl Default for FileDest {
    fn default() -> Self {
        FileDest {
            folder: String::new(),
            prefix: "XcreenShot".into(),
            pattern: filename::DEFAULT_PATTERN.into(),
            format: "png".into(),
            quality: 90,
            on_exists: "number".into(),
        }
    }
}

impl FileDest {
    pub fn validate(&self) -> Result<(), String> {
        if !["png", "jpg", "xshot"].contains(&self.format.as_str()) {
            return Err("file.format: expected png, jpg or xshot".into());
        }
        if !(1..=100).contains(&self.quality) {
            return Err("file.quality: must be between 1 and 100".into());
        }
        if !["number", "overwrite"].contains(&self.on_exists.as_str()) {
            return Err("file.onExists: expected number or overwrite".into());
        }
        Ok(())
    }

    pub fn folder(&self) -> PathBuf {
        if self.folder.trim().is_empty() {
            default_folder()
        } else {
            PathBuf::from(self.folder.trim())
        }
    }

    fn counter_key(&self) -> String {
        format!("{}|{}|{}", self.folder().display(), self.pattern, self.format)
    }
}

/// `Pictures\XcreenShot` (CAP-053).
pub fn default_folder() -> PathBuf {
    if xs_core::paths::isolated() {
        if let Some(dir) = std::env::var_os("XCREENSHOT_DATA_DIR") {
            return PathBuf::from(dir).join("Pictures").join("XcreenShot");
        }
    }
    directories::UserDirs::new()
        .and_then(|d| d.picture_dir().map(Path::to_path_buf))
        .unwrap_or_else(std::env::temp_dir)
        .join("XcreenShot")
}

/// The next number of a destination's counter.
pub fn next_counter(core: &Core, dest: &FileDest) -> u32 {
    core.recall(COUNTERS_KEY)
        .and_then(|v| v.get(dest.counter_key()).and_then(Value::as_u64))
        .unwrap_or(1)
        .max(1) as u32
}

/// Sets the next number of a destination's counter ("Next number", CAP-053).
pub fn set_counter(core: &Core, dest: &FileDest, next: u32) {
    let mut all = core.recall(COUNTERS_KEY).unwrap_or_else(|| json!({}));
    if !all.is_object() {
        all = json!({});
    }
    all[dest.counter_key()] = json!(next.max(1));
    core.remember(COUNTERS_KEY, all);
}

/// The name the destination gives a capture now, for the preview under the pattern.
pub fn preview(core: &Core, dest: &FileDest, values: &Values) -> String {
    let (dirs, name) = filename::render(
        &dest.pattern,
        values,
        chrono::Local::now(),
        next_counter(core, dest),
    );
    let mut parts = dirs;
    parts.push(format!("{name}.{}", dest.format));
    parts.join("/")
}

/// Writes the capture to the destination and returns the file written.
pub fn write(
    core: &Core,
    dest: &FileDest,
    image: &RgbaImage,
    xshot: &Path,
    values: &Values,
    dpi: f64,
) -> Result<PathBuf, String> {
    let now = chrono::Local::now();
    let root = dest.folder();
    let ext = dest.format.as_str();
    let numbered = filename::uses_counter(&dest.pattern);
    let mut counter = next_counter(core, dest);
    let path = loop {
        let (dirs, name) = filename::render(&dest.pattern, values, now, counter);
        let folder = dirs.iter().fold(root.clone(), |p, d| p.join(d));
        let first = folder.join(format!("{name}.{ext}"));
        if numbered {
            // A name already taken skips to the next free number.
            if first.exists() && counter < u32::MAX - 1 {
                counter += 1;
                continue;
            }
            break first;
        }
        if dest.on_exists == "overwrite" || !first.exists() {
            break first;
        }
        break (2..)
            .map(|n| folder.join(format!("{name} ({n}).{ext}")))
            .find(|p| !p.exists())
            .unwrap_or(first);
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    let bytes = match ext {
        "jpg" => xs_image::encode::jpg(image, dpi, dest.quality, xs_image::color::Rgba::WHITE)?,
        "xshot" => std::fs::read(xshot).map_err(|e| e.to_string())?,
        _ => xs_image::encode::png(image, dpi, false)?,
    };
    std::fs::write(&path, bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    if numbered {
        set_counter(core, dest, counter + 1);
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use xs_core::platform::Platform;

    fn values() -> Values {
        Values {
            prefix: "Informe".into(),
            mode: "region".into(),
            width: 4,
            height: 3,
            ..Values::default()
        }
    }

    /// CAP-053: two captures with `{prefix}_{date}_{counter:3}` get 001 and 002.
    #[test]
    fn the_counter_goes_up_and_skips_names_taken() {
        let core = Core::for_tests(Platform::Windows);
        let dir = tempfile::tempdir().unwrap();
        let dest = FileDest {
            folder: dir.path().to_string_lossy().into_owned(),
            pattern: "{prefix}_{counter:3}".into(),
            ..FileDest::default()
        };
        let img = RgbaImage::new(4, 3);
        let a = write(&core, &dest, &img, Path::new("x"), &values(), 96.0).unwrap();
        let b = write(&core, &dest, &img, Path::new("x"), &values(), 96.0).unwrap();
        assert!(a.ends_with("Informe_001.png") && b.ends_with("Informe_002.png"));
        // A file already there: the next free number.
        std::fs::write(dir.path().join("Informe_003.png"), b"x").unwrap();
        let c = write(&core, &dest, &img, Path::new("x"), &values(), 96.0).unwrap();
        assert!(c.ends_with("Informe_004.png"));
        // Next number, set by hand.
        set_counter(&core, &dest, 1);
        assert_eq!(next_counter(&core, &dest), 1);
    }

    /// CAP-053, SHR-018: the same name again gets a number; the first stays.
    #[test]
    fn a_name_taken_gets_a_number_or_is_overwritten() {
        let core = Core::for_tests(Platform::Windows);
        let dir = tempfile::tempdir().unwrap();
        let dest = FileDest {
            folder: dir.path().to_string_lossy().into_owned(),
            pattern: "{prefix}".into(),
            ..FileDest::default()
        };
        let img = RgbaImage::new(4, 3);
        let a = write(&core, &dest, &img, Path::new("x"), &values(), 96.0).unwrap();
        let b = write(&core, &dest, &img, Path::new("x"), &values(), 96.0).unwrap();
        assert!(a.ends_with("Informe.png") && b.ends_with("Informe (2).png"));
        let over = FileDest {
            on_exists: "overwrite".into(),
            ..dest
        };
        let c = write(&core, &over, &img, Path::new("x"), &values(), 96.0).unwrap();
        assert_eq!(c, a);
    }

    /// CAP-053: `{date}/{time}` makes a dated subfolder; JPG is a real JPG.
    #[test]
    fn subfolders_and_jpg() {
        let core = Core::for_tests(Platform::Windows);
        let dir = tempfile::tempdir().unwrap();
        let dest = FileDest {
            folder: dir.path().to_string_lossy().into_owned(),
            pattern: "{date}/{prefix}".into(),
            format: "jpg".into(),
            ..FileDest::default()
        };
        let img = RgbaImage::new(4, 3);
        let p = write(&core, &dest, &img, Path::new("x"), &values(), 96.0).unwrap();
        assert_eq!(p.parent().unwrap().parent().unwrap(), dir.path());
        let bytes = std::fs::read(&p).unwrap();
        assert_eq!(&bytes[..2], &[0xFF, 0xD8]);
    }

    #[test]
    fn a_folder_that_cannot_be_made_is_an_error() {
        let core = Core::for_tests(Platform::Windows);
        let dir = tempfile::tempdir().unwrap();
        let blocker = dir.path().join("file");
        std::fs::write(&blocker, b"x").unwrap();
        let dest = FileDest {
            folder: blocker.join("sub").to_string_lossy().into_owned(),
            ..FileDest::default()
        };
        assert!(write(
            &core,
            &dest,
            &RgbaImage::new(2, 2),
            Path::new("x"),
            &values(),
            96.0
        )
        .is_err());
    }
}
