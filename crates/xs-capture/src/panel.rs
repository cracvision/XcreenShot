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
    /// Destinations (CAP-052): `file` for now.
    pub share: Vec<String>,
    /// The File destination's options (CAP-053).
    pub file: crate::filedest::FileDest,
}

impl Default for UniversalTab {
    fn default() -> Self {
        UniversalTab {
            selection: "region".into(),
            copy: true,
            cursor: false,
            open: "editor".into(),
            delay5: false,
            share: Vec::new(),
            file: crate::filedest::FileDest::default(),
        }
    }
}

/// The properties of Selection = Menu (CAP-031).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MenuTab {
    pub submenus: bool,
    pub menubar: bool,
}

impl Default for MenuTab {
    fn default() -> Self {
        MenuTab {
            submenus: true,
            menubar: false,
        }
    }
}

/// The properties of Selection = Multiple areas (CAP-033).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AreasTab {
    /// `original`, `stacked` or `row`.
    pub composition: String,
    /// Space between stacked or side by side areas, in pixels.
    pub gap: u32,
}

impl Default for AreasTab {
    fn default() -> Self {
        AreasTab {
            composition: "original".into(),
            gap: 10,
        }
    }
}

/// The properties of Selection = Region (CAP-026): a fixed size, and a fixed place.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RegionTab {
    pub fixed: bool,
    pub width: u32,
    pub height: u32,
    /// Also a fixed place: the capture is taken at once, without selecting.
    pub positioned: bool,
    /// The monitor's name (CAP-026); x and y from its top left, physical pixels.
    pub monitor: String,
    pub x: i32,
    pub y: i32,
}

impl Default for RegionTab {
    fn default() -> Self {
        RegionTab {
            fixed: false,
            width: 800,
            height: 600,
            positioned: false,
            monitor: String::new(),
            x: 0,
            y: 0,
        }
    }
}

impl RegionTab {
    /// What `capture.region` takes for these properties: a `rect` on a monitor, a
    /// `size` to place, or nothing (a free selection).
    pub fn params(&self) -> Value {
        if !self.fixed {
            return json!({});
        }
        if self.positioned {
            json!({ "rect": { "monitor": self.monitor, "x": self.x, "y": self.y, "width": self.width, "height": self.height } })
        } else {
            json!({ "size": { "width": self.width, "height": self.height } })
        }
    }
}

/// The properties of Selection = Scrolling (CAP-034, CAP-035).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ScrollTab {
    /// `manual` or `auto`.
    #[serde(rename = "type")]
    pub kind: String,
    /// Automatic: `vertical`, `horizontal` or `both`.
    pub direction: String,
    /// Automatic: `start` (the beginning) or `current`.
    pub start_from: String,
    /// Automatic: seconds between steps, 0 to 5.
    pub wait: f64,
    /// Automatic: put the content back where it was.
    pub restore: bool,
}

impl Default for ScrollTab {
    fn default() -> Self {
        ScrollTab {
            kind: "manual".into(),
            direction: "vertical".into(),
            start_from: "start".into(),
            wait: 0.3,
            restore: true,
        }
    }
}

impl ScrollTab {
    /// The parameters of `capture.scrolling` these properties stand for.
    pub fn params(&self) -> Value {
        json!({
            "type": self.kind,
            "direction": self.direction,
            "startFrom": self.start_from,
            "wait": self.wait,
            "restore": self.restore,
        })
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
    /// Selection = Scrolling (CAP-034, CAP-035).
    pub scroll: ScrollTab,
    /// Selection = Region (CAP-026).
    pub region: RegionTab,
    /// Selection = Multiple areas (CAP-033).
    pub areas: AreasTab,
    /// Selection = Menu (CAP-031).
    pub menu: MenuTab,
    /// Destinations (CAP-052): `file` for now.
    pub share: Vec<String>,
    /// The File destination's options (CAP-053).
    pub file: crate::filedest::FileDest,
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
            scroll: ScrollTab::default(),
            region: RegionTab::default(),
            areas: AreasTab::default(),
            menu: MenuTab::default(),
            share: Vec::new(),
            file: crate::filedest::FileDest::default(),
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
/// The Image tab also has Scrolling (CAP-034, CAP-035) and Extract text (SMT-005).
const IMAGE_SELECTIONS: [&str; 8] = [
    "region",
    "window",
    "fullscreen",
    "picker",
    "scrolling",
    "menu",
    "multi",
    "text",
];

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
        one_of("image.selection", &self.image.selection, &IMAGE_SELECTIONS)?;
        one_of("image.open", &self.image.open, &["editor", "none"])?;
        one_of("image.timing", &self.image.timing, &["none", "delay"])?;
        if !(1..=60).contains(&self.image.delay) {
            return Err("image.delay: must be between 1 and 60".into());
        }
        let s = &self.image.scroll;
        one_of("image.scroll.type", &s.kind, &["manual", "auto"])?;
        one_of(
            "image.scroll.direction",
            &s.direction,
            &["vertical", "horizontal", "both"],
        )?;
        one_of("image.scroll.startFrom", &s.start_from, &["start", "current"])?;
        if !(0.0..=5.0).contains(&s.wait) {
            return Err("image.scroll.wait: must be between 0 and 5".into());
        }
        one_of(
            "image.areas.composition",
            &self.image.areas.composition,
            &["original", "stacked", "row"],
        )?;
        if self.image.areas.gap > 500 {
            return Err("image.areas.gap: must be between 0 and 500".into());
        }
        for (tab, share, file) in [
            ("universal", &self.universal.share, &self.universal.file),
            ("image", &self.image.share, &self.image.file),
        ] {
            if share.iter().any(|d| d != "file") {
                return Err(format!("{tab}.share: only file is available"));
            }
            file.validate().map_err(|e| format!("{tab}.{e}"))?;
        }
        let r = &self.image.region;
        if r.width == 0 || r.height == 0 || r.width > 100_000 || r.height > 100_000 {
            return Err("image.region: width and height must be between 1 and 100000".into());
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
            "share": i.share,
            "file": i.file,
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
                "share": u.share,
                "file": u.file,
            });
            let extra = json!({
                "windowsOnly": u.selection == "window",
                "startFullscreen": u.selection == "fullscreen",
            });
            ("capture.universal", params, extra)
        } else {
            let mut params = self.image_options();
            if self.image.selection == "text" {
                // A zone of the screen whose text is read (SMT-005): the delay and
                // the clipboard apply; the pointer and Open in do not.
                params["extract"] = json!(true);
                params["cursor"] = json!(false);
                return ("capture.region", params, json!({}));
            }
            if self.image.selection == "picker" {
                return ("capture.window_picker", params, json!({}));
            }
            if self.image.selection == "menu" {
                // From the panel there is always time to open the menu: the delay, or
                // 5 s without one (CAP-031). No pointer in a menu capture.
                params["delay"] = json!(if self.image.timing == "delay" {
                    self.image.delay
                } else {
                    5
                });
                params["cursor"] = json!(false);
                params["submenus"] = json!(self.image.menu.submenus);
                params["menubar"] = json!(self.image.menu.menubar);
                return ("capture.menu", params, json!({}));
            }
            if self.image.selection == "multi" {
                let a = &self.image.areas;
                params["multiple"] = json!({ "composition": a.composition, "gap": a.gap });
                return ("capture.region", params, json!({}));
            }
            if self.image.selection == "scrolling" {
                // No pointer in a scrolling capture (CAP-002).
                params["cursor"] = json!(false);
                return (
                    "capture.scrolling",
                    with_overrides(params, &self.image.scroll.params()),
                    json!({}),
                );
            }
            let command = match self.image.selection.as_str() {
                "window" => {
                    params["target"] = json!("interactive");
                    "capture.window"
                }
                "fullscreen" => "capture.fullscreen",
                _ => {
                    // A fixed region (CAP-026).
                    params = with_overrides(params, &self.image.region.params());
                    "capture.region"
                }
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
                delay5: false,
                // No destination by default (CAP-052).
                share: Vec::new(),
                file: crate::filedest::FileDest::default(),
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
        assert!(s.patched(&json!({"image": {"selection": "webpage"}})).is_err());
        let p = s
            .patched(&json!({"image": {"selection": "scrolling", "scroll": {"type": "auto"}}}))
            .unwrap();
        let (command, params, _) = p.plan("image");
        assert_eq!(command, "capture.scrolling");
        assert_eq!(params["type"], "auto");
        assert_eq!(params["direction"], "vertical");
        assert_eq!(params["startFrom"], "start");
        assert!(s.patched(&json!({"image": {"scroll": {"wait": 9}}})).is_err());
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
        let o = s.image_options();
        assert_eq!(
            (&o["copy"], &o["cursor"], &o["open"], &o["delay"]),
            (&json!(true), &json!(false), &json!("none"), &json!(7))
        );
        assert!(o.get("selection").is_none());
        // And the destinations with their options (CAP-052, CAP-053).
        assert_eq!(o["share"], json!([]));
        assert_eq!(o["file"]["pattern"], "{prefix}_{date}_{time}");
    }

    #[test]
    fn overrides_win() {
        let merged = with_overrides(json!({"copy": true, "open": "editor"}), &json!({"open": "none"}));
        assert_eq!(merged, json!({"copy": true, "open": "none"}));
    }

    /// CAP-026: a fixed region is a size to place, or a rectangle on a monitor.
    #[test]
    fn a_fixed_region_plans_its_size_or_place() {
        let s = PanelSettings::default();
        let p = s
            .patched(&json!({"image": {"region": {"fixed": true, "width": 1080, "height": 1080}}}))
            .unwrap();
        let (command, params, _) = p.plan("image");
        assert_eq!(command, "capture.region");
        assert_eq!(params["size"], json!({"width": 1080, "height": 1080}));
        assert!(params.get("rect").is_none());
        let p = p
            .patched(&json!({"image": {"region": {"positioned": true, "monitor": "M2", "x": 100, "y": 50, "width": 800, "height": 600}}}))
            .unwrap();
        let (_, params, _) = p.plan("image");
        assert_eq!(
            params["rect"],
            json!({"monitor": "M2", "x": 100, "y": 50, "width": 800, "height": 600})
        );
        assert!(p.patched(&json!({"image": {"region": {"width": 0}}})).is_err());
        // Off, a free selection again.
        let p = p
            .patched(&json!({"image": {"region": {"fixed": false}}}))
            .unwrap();
        let (_, params, _) = p.plan("image");
        assert!(params.get("rect").is_none() && params.get("size").is_none());
    }
}
