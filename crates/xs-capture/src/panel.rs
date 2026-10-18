//! Capture panel settings (CAP-001, CAP-002, CAP-003, CAP-010): the Universal and Image
//! tabs, kept across sessions, and what "Capture" does with them (CAP-006).

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use xs_core::Core;

const MEMORY_KEY: &str = "capture.panel";

/// Universal tab (CAP-010). Defaults from CAP-003.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct UniversalTab {
    /// `region` (crosshair with detection), `window` (windows only) or `fullscreen`.
    pub selection: String,
    pub copy: bool,
    pub cursor: bool,
    /// `editor` or `none` (CAP-045).
    pub open: String,
    /// Fixed 5 s countdown (CAP-010, CAP-048).
    pub delay5: bool,
}

impl Default for UniversalTab {
    fn default() -> Self {
        UniversalTab {
            selection: "region".into(),
            copy: true,
            cursor: false,
            open: "editor".into(),
            delay5: false,
        }
    }
}

/// Image tab (CAP-002). Defaults from CAP-003.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ImageTab {
    /// Only modes that exist are offered (CAP-002): `region`, `window`, `fullscreen`.
    pub selection: String,
    pub copy: bool,
    pub cursor: bool,
    pub open: String,
    /// `none` or `delay` (CAP-048; interval and scheduled arrive in M3).
    pub timing: String,
    /// Delay seconds, 1–60 (default 5).
    pub delay: u32,
}

impl Default for ImageTab {
    fn default() -> Self {
        ImageTab {
            selection: "region".into(),
            copy: true,
            cursor: false,
            open: "editor".into(),
            timing: "none".into(),
            delay: 5,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PanelSettings {
    /// Active tab: `universal` or `image` (`video` arrives in M4).
    pub tab: String,
    /// Options shown (expanded) or only the Capture button (compact).
    pub expanded: bool,
    pub universal: UniversalTab,
    pub image: ImageTab,
}

impl Default for PanelSettings {
    fn default() -> Self {
        PanelSettings {
            tab: "image".into(),
            expanded: true,
            universal: UniversalTab::default(),
            image: ImageTab::default(),
        }
    }
}

const SELECTIONS: [&str; 3] = ["region", "window", "fullscreen"];

impl PanelSettings {
    /// Checks every value; returns the offending field.
    pub fn validate(&self) -> Result<(), String> {
        let one_of = |field: &str, value: &str, options: &[&str]| {
            if options.contains(&value) {
                Ok(())
            } else {
                Err(format!("{field}: expected one of {options:?}"))
            }
        };
        one_of("tab", &self.tab, &["universal", "image"])?;
        one_of("universal.selection", &self.universal.selection, &SELECTIONS)?;
        one_of("universal.open", &self.universal.open, &["editor", "none"])?;
        one_of("image.selection", &self.image.selection, &SELECTIONS)?;
        one_of("image.open", &self.image.open, &["editor", "none"])?;
        one_of("image.timing", &self.image.timing, &["none", "delay"])?;
        if !(1..=60).contains(&self.image.delay) {
            return Err("image.delay: must be between 1 and 60".into());
        }
        Ok(())
    }

    /// Applies a partial update (`{"image": {"selection": "window"}}`).
    pub fn patched(&self, patch: &Value) -> Result<PanelSettings, String> {
        let mut current = serde_json::to_value(self).map_err(|e| e.to_string())?;
        merge(&mut current, patch);
        let next: PanelSettings = serde_json::from_value(current).map_err(|e| e.to_string())?;
        next.validate()?;
        Ok(next)
    }

    /// Everything but the mode from the Image tab: what the single-mode actions use
    /// (CAP-065).
    pub fn image_options(&self) -> Value {
        let i = &self.image;
        json!({
            "copy": i.copy,
            "cursor": i.cursor,
            "open": i.open,
            "delay": if i.timing == "delay" { i.delay } else { 0 },
        })
    }

    /// Command and parameters that "Capture" runs with `tab` (CAP-006, CAP-065).
    pub fn plan(&self, tab: &str) -> (&'static str, Value, Value) {
        if tab == "universal" {
            let u = &self.universal;
            let params = json!({
                "copy": u.copy,
                "cursor": u.cursor,
                "open": u.open,
                "delay": if u.delay5 { 5 } else { 0 },
            });
            let extra = json!({
                "windowsOnly": u.selection == "window",
                "startFullscreen": u.selection == "fullscreen",
            });
            ("capture.universal", params, extra)
        } else {
            let mut params = self.image_options();
            let command = match self.image.selection.as_str() {
                "window" => {
                    params["target"] = json!("interactive");
                    "capture.window"
                }
                "fullscreen" => "capture.fullscreen",
                _ => "capture.region",
            };
            (command, params, json!({}))
        }
    }
}

fn merge(base: &mut Value, patch: &Value) {
    match (base, patch) {
        (Value::Object(b), Value::Object(p)) => {
            for (k, v) in p {
                merge(b.entry(k.clone()).or_insert(Value::Null), v);
            }
        }
        (b, p) => *b = p.clone(),
    }
}

pub fn load(core: &Core) -> PanelSettings {
    core.recall(MEMORY_KEY)
        .and_then(|v| serde_json::from_value::<PanelSettings>(v).ok())
        .filter(|s| s.validate().is_ok())
        .unwrap_or_default()
}

pub fn save(core: &Core, settings: &PanelSettings) {
    core.remember(MEMORY_KEY, serde_json::to_value(settings).unwrap_or_default());
}

/// Explicit parameters win over the panel's (presets and commands, CAP-008).
pub fn with_overrides(mut base: Value, overrides: &Value) -> Value {
    if let (Some(b), Some(o)) = (base.as_object_mut(), overrides.as_object()) {
        for (k, v) in o {
            b.insert(k.clone(), v.clone());
        }
    }
    base
}

#[cfg(test)]
mod tests {
    use super::*;

    /// CAP-003: the documented defaults.
    #[test]
    fn defaults_match_cap_003() {
        let s = PanelSettings::default();
        assert_eq!(
            s.universal,
            UniversalTab {
                selection: "region".into(),
                copy: true,
                cursor: false,
                open: "editor".into(),
                delay5: false
            }
        );
        assert_eq!(s.image.selection, "region");
        assert_eq!(s.image.timing, "none");
        assert_eq!(s.image.delay, 5);
        assert!(s.image.copy && !s.image.cursor && s.image.open == "editor");
        s.validate().unwrap();
    }

    #[test]
    fn patches_keep_the_rest_and_reject_bad_values() {
        let s = PanelSettings::default();
        let p = s.patched(&json!({"image": {"selection": "window"}})).unwrap();
        assert_eq!(p.image.selection, "window");
        assert_eq!(p.image.delay, 5, "untouched fields stay");
        assert!(s.patched(&json!({"image": {"selection": "scrolling"}})).is_err());
        assert!(s.patched(&json!({"image": {"delay": 0}})).is_err());
        assert!(s.patched(&json!({"tab": "video"})).is_err());
    }

    /// CAP-006, CAP-001: Capture follows the tab's settings.
    #[test]
    fn plan_follows_the_tab() {
        let mut s = PanelSettings::default();
        s.image.selection = "fullscreen".into();
        s.image.timing = "delay".into();
        s.image.delay = 3;
        let (command, params, _) = s.plan("image");
        assert_eq!(command, "capture.fullscreen");
        assert_eq!(params["delay"], 3);
        s.image.selection = "window".into();
        assert_eq!(s.plan("image").1["target"], "interactive");
        s.universal.selection = "fullscreen".into();
        s.universal.delay5 = true;
        let (command, params, extra) = s.plan("universal");
        assert_eq!(command, "capture.universal");
        assert_eq!(params["delay"], 5);
        assert_eq!(extra["startFullscreen"], true);
    }

    /// CAP-065: single-mode actions take the rest from the Image tab.
    #[test]
    fn image_options_leave_out_the_mode() {
        let mut s = PanelSettings::default();
        s.image.selection = "fullscreen".into();
        s.image.open = "none".into();
        s.image.timing = "delay".into();
        s.image.delay = 7;
        assert_eq!(
            s.image_options(),
            json!({"copy": true, "cursor": false, "open": "none", "delay": 7})
        );
    }

    #[test]
    fn overrides_win() {
        let merged = with_overrides(json!({"copy": true, "open": "editor"}), &json!({"open": "none"}));
        assert_eq!(merged, json!({"copy": true, "open": "none"}));
    }
}
