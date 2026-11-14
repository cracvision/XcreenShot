//! Keyboard shortcuts: scopes and rules (APP-017), conflicts (APP-019),
//! taking over occupied shortcuts (APP-020) and profiles (APP-021).
//!
//! Shortcuts are stored by physical key position using W3C `code` names
//! (`KeyK`, `Digit5`, `PrintScreen`), so changing the keyboard layout never breaks them.
//! Registration with the OS is done by the shell, which reports back a [`Status`].

use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::RwLock;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::{CommandError, ErrorCode};
use crate::i18n::{tr, Lang};
use crate::paths::write_atomic;
use crate::platform::Platform;

// ---------------------------------------------------------------------------
// Accelerator

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Accelerator {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    /// Windows key / Command key.
    pub meta: bool,
    /// W3C `KeyboardEvent.code` of the main key.
    pub code: String,
}

const MODIFIER_CODES: &[&str] = &[
    "ControlLeft",
    "ControlRight",
    "AltLeft",
    "AltRight",
    "ShiftLeft",
    "ShiftRight",
    "MetaLeft",
    "MetaRight",
    "OSLeft",
    "OSRight",
    "CapsLock",
    "Fn",
];

/// Keys that are never accepted as a shortcut (APP-017).
const FORBIDDEN_CODES: &[&str] = &["Escape", "Tab", "Enter", "NumpadEnter"];

fn normalize_code(token: &str) -> Option<String> {
    let t = token.trim();
    if t.is_empty() {
        return None;
    }
    let upper = t.to_ascii_uppercase();
    if t.len() == 1 {
        let c = t.chars().next()?;
        if c.is_ascii_alphabetic() {
            return Some(format!("Key{}", c.to_ascii_uppercase()));
        }
        if c.is_ascii_digit() {
            return Some(format!("Digit{c}"));
        }
        return Some(
            match c {
                ',' => "Comma",
                '.' => "Period",
                '/' => "Slash",
                ';' => "Semicolon",
                '\'' => "Quote",
                '[' => "BracketLeft",
                ']' => "BracketRight",
                '\\' => "Backslash",
                '-' => "Minus",
                '=' => "Equal",
                '`' => "Backquote",
                _ => return None,
            }
            .to_string(),
        );
    }
    let aliases: &[(&str, &str)] = &[
        ("PRTSCN", "PrintScreen"),
        ("PRTSC", "PrintScreen"),
        ("PRINTSCREEN", "PrintScreen"),
        ("PRINT", "PrintScreen"),
        ("ESC", "Escape"),
        ("ESCAPE", "Escape"),
        ("RETURN", "Enter"),
        ("ENTER", "Enter"),
        ("SPACE", "Space"),
        ("TAB", "Tab"),
        ("DEL", "Delete"),
        ("DELETE", "Delete"),
        ("BACKSPACE", "Backspace"),
        ("INSERT", "Insert"),
        ("HOME", "Home"),
        ("END", "End"),
        ("PAGEUP", "PageUp"),
        ("PAGEDOWN", "PageDown"),
        ("UP", "ArrowUp"),
        ("DOWN", "ArrowDown"),
        ("LEFT", "ArrowLeft"),
        ("RIGHT", "ArrowRight"),
        ("COMMA", "Comma"),
        ("PERIOD", "Period"),
        ("SLASH", "Slash"),
        ("MINUS", "Minus"),
        ("EQUAL", "Equal"),
    ];
    if let Some((_, code)) = aliases.iter().find(|(a, _)| *a == upper) {
        return Some(code.to_string());
    }
    if let Some(n) = upper.strip_prefix('F').and_then(|n| n.parse::<u8>().ok()) {
        if (1..=24).contains(&n) {
            return Some(format!("F{n}"));
        }
    }
    if let Some(rest) = upper.strip_prefix("KEY") {
        if rest.len() == 1 && rest.chars().all(|c| c.is_ascii_alphabetic()) {
            return Some(format!("Key{rest}"));
        }
    }
    if let Some(rest) = upper.strip_prefix("DIGIT") {
        if rest.len() == 1 && rest.chars().all(|c| c.is_ascii_digit()) {
            return Some(format!("Digit{rest}"));
        }
    }
    // Already a W3C code with the right casing (e.g. "ArrowUp", "Numpad1", "ControlLeft").
    if t.chars().next()?.is_ascii_uppercase() && t.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Some(t.to_string());
    }
    None
}

impl FromStr for Accelerator {
    type Err = String;

    /// Parses `Ctrl+Alt+Shift+K`, `Cmd+Shift+5`, `PrtScn`, `Control+Option+KeyC`…
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut acc = Accelerator {
            ctrl: false,
            alt: false,
            shift: false,
            meta: false,
            code: String::new(),
        };
        let tokens: Vec<&str> = s.split('+').map(str::trim).collect();
        // A trailing "+" means the plus key itself, which we do not support.
        if tokens.iter().any(|t| t.is_empty()) {
            return Err(format!("cannot parse shortcut \"{s}\""));
        }
        for (i, token) in tokens.iter().enumerate() {
            let upper = token.to_ascii_uppercase();
            let last = i == tokens.len() - 1;
            match upper.as_str() {
                "CTRL" | "CONTROL" | "⌃" => acc.ctrl = true,
                "ALT" | "OPTION" | "OPT" | "⌥" => acc.alt = true,
                "SHIFT" | "⇧" => acc.shift = true,
                "WIN" | "WINDOWS" | "SUPER" | "META" | "CMD" | "COMMAND" | "⌘" => acc.meta = true,
                _ if last => {
                    acc.code =
                        normalize_code(token).ok_or_else(|| format!("unknown key \"{token}\" in \"{s}\""))?;
                }
                _ => return Err(format!("unknown modifier \"{token}\" in \"{s}\"")),
            }
        }
        if acc.code.is_empty() {
            return Err(format!("\"{s}\" has no main key"));
        }
        Ok(acc)
    }
}

impl fmt::Display for Accelerator {
    /// Canonical storage form, also accepted by the OS shortcut layer:
    /// `Ctrl+Alt+Shift+Super+KeyK`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("Ctrl");
        }
        if self.alt {
            parts.push("Alt");
        }
        if self.shift {
            parts.push("Shift");
        }
        if self.meta {
            parts.push("Super");
        }
        parts.push(&self.code);
        write!(f, "{}", parts.join("+"))
    }
}

impl Serialize for Accelerator {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Accelerator {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

fn key_label(code: &str) -> String {
    if let Some(c) = code.strip_prefix("Key") {
        return c.to_string();
    }
    if let Some(c) = code.strip_prefix("Digit") {
        return c.to_string();
    }
    match code {
        "PrintScreen" => "PrtScn",
        "Comma" => ",",
        "Period" => ".",
        "Slash" => "/",
        "Semicolon" => ";",
        "Quote" => "'",
        "BracketLeft" => "[",
        "BracketRight" => "]",
        "Backslash" => "\\",
        "Minus" => "-",
        "Equal" => "=",
        "Backquote" => "`",
        "ArrowUp" => "↑",
        "ArrowDown" => "↓",
        "ArrowLeft" => "←",
        "ArrowRight" => "→",
        other => other,
    }
    .to_string()
}

impl Accelerator {
    pub fn has_modifier(&self) -> bool {
        self.ctrl || self.alt || self.shift || self.meta
    }

    /// Human label using the US layout for the key. The UI replaces the key with the
    /// active layout's character where the platform exposes it (APP-017).
    pub fn display(&self, platform: Platform) -> String {
        let key = key_label(&self.code);
        match platform {
            Platform::Macos => {
                let mut s = String::new();
                if self.ctrl {
                    s.push('⌃');
                }
                if self.alt {
                    s.push('⌥');
                }
                if self.shift {
                    s.push('⇧');
                }
                if self.meta {
                    s.push('⌘');
                }
                s + &key
            }
            Platform::Windows => {
                let mut parts: Vec<String> = Vec::new();
                if self.ctrl {
                    parts.push("Ctrl".into());
                }
                if self.alt {
                    parts.push("Alt".into());
                }
                if self.shift {
                    parts.push("Shift".into());
                }
                if self.meta {
                    parts.push("Win".into());
                }
                parts.push(key);
                parts.join("+")
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Slots and defaults

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShortcutScope {
    /// Works system-wide (APP-017).
    Global,
    /// Works while a XcreenShot window has focus.
    App,
    /// Works in the editor window, where it takes precedence over an app shortcut with
    /// the same combination (EDT-115, EDT-116; `Ctrl+W` closes the document there).
    Editor,
}

impl ShortcutScope {
    pub fn parse(s: &str) -> Option<ShortcutScope> {
        match s {
            "global" => Some(ShortcutScope::Global),
            "app" => Some(ShortcutScope::App),
            "editor" => Some(ShortcutScope::Editor),
            _ => None,
        }
    }

    /// Whether two shortcuts of these scopes can be pressed in the same place: a global
    /// one everywhere, app ones in every window, editor ones in the editor, where they
    /// win over app ones.
    fn collides_with(self, other: ShortcutScope) -> bool {
        use ShortcutScope::*;
        !matches!((self, other), (App, Editor) | (Editor, App))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Profile {
    Coexistence,
    Compatibility,
}

impl Profile {
    pub fn parse(s: &str) -> Option<Profile> {
        match s {
            "coexistence" => Some(Profile::Coexistence),
            "compatibility" => Some(Profile::Compatibility),
            _ => None,
        }
    }
}

/// One row of Preferences > Shortcuts (APP-018). `id` is stable and names the row; the
/// shortcut runs `command` with `params` (JSON object text, empty for none), so one
/// command can have several rows, e.g. "Window capture" and "Active window" (CAP-065).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Slot {
    pub id: &'static str,
    pub command: &'static str,
    pub params: &'static str,
    pub scope: ShortcutScope,
    /// Group in the Shortcuts table (APP-018).
    pub group: &'static str,
}

impl Slot {
    pub fn key(&self) -> String {
        let scope = match self.scope {
            ShortcutScope::Global => "global",
            ShortcutScope::App => "app",
            ShortcutScope::Editor => "editor",
        };
        format!("{}@{scope}", self.id)
    }

    /// Parameters the shortcut passes to its command.
    pub fn params_value(&self) -> Value {
        if self.params.is_empty() {
            json!({})
        } else {
            serde_json::from_str(self.params).unwrap_or_else(|_| json!({}))
        }
    }

    /// The row has its own title (`shortcuts.slot.<id>`) instead of its command's.
    pub fn has_own_title(&self) -> bool {
        self.id != self.command || !self.params.is_empty()
    }
}

/// The key of a preset's row (`preset.<id>@global`, CAP-062).
pub fn is_preset_key(key: &str) -> bool {
    key.starts_with("preset.")
}

/// Assignable slots that exist in this build. Owners add theirs as features land.
pub fn slots() -> Vec<Slot> {
    use ShortcutScope::*;
    let capture = |id, command, params| Slot {
        id,
        command,
        params,
        scope: Global,
        group: "capture",
    };
    let s = |command, scope| Slot {
        id: command,
        command,
        params: "",
        scope,
        group: "app",
    };
    // Editor actions run in the editor view (`editor.action`), tools and zoom by
    // their own commands. Only what exists in this build (EDT-115, EDT-116).
    let editor = |id, command, params| Slot {
        id,
        command,
        params,
        scope: Editor,
        group: "editor",
    };
    let tool = |id, params| Slot {
        id,
        command: "editor.select_tool",
        params,
        scope: Editor,
        group: "tools",
    };
    vec![
        // Capture actions (CAP-065), in the spec's order.
        capture("capture.capture", "capture.capture", ""),
        capture("capture.universal", "capture.universal", ""),
        capture("capture.image", "capture.image", ""),
        // Extract text (SMT-005).
        capture(
            "smart.extract_text",
            "smart.extract_text",
            r#"{"source":"screen"}"#,
        ),
        capture("capture.window", "capture.window", r#"{"target":"interactive"}"#),
        capture(
            "capture.active_window",
            "capture.window",
            r#"{"target":"active"}"#,
        ),
        capture("capture.repeat_last", "capture.repeat_last", ""),
        // No default in any profile (CAP-065); assignable in Preferences > Shortcuts.
        capture("capture.repeat_last_area", "capture.repeat_last_area", ""),
        capture("capture.panel", "capture.panel", ""),
        // No default (CAP-067); assignable in Preferences > Shortcuts.
        capture("capture.multi_start", "capture.multi_start", ""),
        // The window picker (CAP-030).
        capture("capture.window_picker", "capture.window_picker", ""),
        // Video (VID-075): Video capture (stops during a recording), Record / Pause /
        // Resume and Stop recording.
        capture("record.video", "record.video", ""),
        capture("record.toggle", "record.toggle", ""),
        capture("record.stop", "record.stop", ""),
        // Live drawing on and off (VID-048, VID-075).
        capture("record.draw", "record.draw", ""),
        // No default: losing a recording by accident is worse (VID-075).
        capture("record.restart", "record.restart", ""),
        capture("record.cancel", "record.cancel", ""),
        // The clipboard as a source (CAP-039). No default.
        capture("capture.clipboard", "capture.clipboard", ""),
        // Quick annotation (CAP-057): only while its trial feature is on (APP-034).
        capture("capture.quick_annotate", "capture.quick_annotate", ""),
        // The open menu, without a countdown (CAP-031). No default yet: the spec's
        // Ctrl+Alt+Shift+E is the editor's Export (see spec/QUESTIONS.md).
        capture("capture.menu", "capture.menu", ""),
        s("app.ping", Global),
        s("app.show", Global),
        s("settings.open", Global),
        s("settings.open", App),
        s("app.help", App),
        s("shortcuts.pause", Global),
        s("app.close_window", App),
        s("app.quit", Global),
        s("app.quit", App),
        // Editor: file, edit and view (EDT-116), in the spec's order.
        editor("editor.open", "editor.action", r#"{"action":"open"}"#),
        editor("editor.save", "editor.action", r#"{"action":"save"}"#),
        editor("editor.save_as", "editor.action", r#"{"action":"save_as"}"#),
        editor("editor.export", "editor.action", r#"{"action":"export"}"#),
        editor("editor.close", "editor.action", r#"{"action":"close"}"#),
        editor("editor.close_all", "editor.action", r#"{"action":"close_all"}"#),
        editor("editor.undo", "editor.action", r#"{"action":"undo"}"#),
        editor("editor.redo", "editor.action", r#"{"action":"redo"}"#),
        editor("editor.redo_alt", "editor.action", r#"{"action":"redo"}"#),
        editor("editor.cut", "editor.action", r#"{"action":"cut"}"#),
        editor("editor.copy", "editor.action", r#"{"action":"copy"}"#),
        editor("editor.paste", "editor.action", r#"{"action":"paste"}"#),
        editor(
            "editor.paste_in_place",
            "editor.action",
            r#"{"action":"paste_in_place"}"#,
        ),
        editor("editor.copy_all", "editor.action", r#"{"action":"copy_all"}"#),
        editor("editor.delete", "editor.action", r#"{"action":"delete"}"#),
        editor("editor.delete_alt", "editor.action", r#"{"action":"delete"}"#),
        editor("editor.duplicate", "editor.action", r#"{"action":"duplicate"}"#),
        editor(
            "editor.bring_front",
            "editor.action",
            r#"{"action":"bring_front"}"#,
        ),
        editor(
            "editor.bring_forward",
            "editor.action",
            r#"{"action":"bring_forward"}"#,
        ),
        editor(
            "editor.send_backward",
            "editor.action",
            r#"{"action":"send_backward"}"#,
        ),
        editor("editor.send_back", "editor.action", r#"{"action":"send_back"}"#),
        editor(
            "editor.equal_width",
            "editor.action",
            r#"{"action":"equal_width"}"#,
        ),
        editor(
            "editor.equal_height",
            "editor.action",
            r#"{"action":"equal_height"}"#,
        ),
        editor("editor.flatten", "editor.action", r#"{"action":"flatten"}"#),
        editor(
            "editor.flatten_all",
            "editor.action",
            r#"{"action":"flatten_all"}"#,
        ),
        editor(
            "editor.unflatten_all",
            "editor.action",
            r#"{"action":"unflatten_all"}"#,
        ),
        editor("editor.group", "editor.action", r#"{"action":"group"}"#),
        editor("editor.ungroup", "editor.action", r#"{"action":"ungroup"}"#),
        editor("editor.lock", "editor.action", r#"{"action":"lock"}"#),
        // No default (EDT-116): assignable in Preferences > Shortcuts.
        editor("editor.unlock_all", "editor.action", r#"{"action":"unlock_all"}"#),
        editor("editor.align_left", "editor.action", r#"{"action":"align_left"}"#),
        editor(
            "editor.align_hcenter",
            "editor.action",
            r#"{"action":"align_hcenter"}"#,
        ),
        editor(
            "editor.align_right",
            "editor.action",
            r#"{"action":"align_right"}"#,
        ),
        editor("editor.align_top", "editor.action", r#"{"action":"align_top"}"#),
        editor(
            "editor.align_vcenter",
            "editor.action",
            r#"{"action":"align_vcenter"}"#,
        ),
        editor(
            "editor.align_bottom",
            "editor.action",
            r#"{"action":"align_bottom"}"#,
        ),
        editor(
            "editor.distribute_h",
            "editor.action",
            r#"{"action":"distribute_h"}"#,
        ),
        editor(
            "editor.distribute_v",
            "editor.action",
            r#"{"action":"distribute_v"}"#,
        ),
        editor("editor.select_all", "editor.action", r#"{"action":"select_all"}"#),
        editor("editor.deselect", "editor.action", r#"{"action":"deselect"}"#),
        editor(
            "editor.import_theme",
            "editor.action",
            r#"{"action":"import_theme"}"#,
        ),
        // Pixel selection (EDT-044).
        editor(
            "editor.invert_selection",
            "editor.action",
            r#"{"action":"invert_selection"}"#,
        ),
        editor(
            "editor.float_selection",
            "editor.action",
            r#"{"action":"float_selection"}"#,
        ),
        editor(
            "editor.crop_selection",
            "editor.action",
            r#"{"action":"crop_selection"}"#,
        ),
        // The Image menu (EDT-069 to EDT-077) and Object > Flip (EDT-073).
        editor(
            "editor.resize_image",
            "editor.action",
            r#"{"action":"resize_image"}"#,
        ),
        editor(
            "editor.canvas_size",
            "editor.action",
            r#"{"action":"canvas_size"}"#,
        ),
        editor(
            "editor.canvas_color",
            "editor.action",
            r#"{"action":"canvas_color"}"#,
        ),
        editor(
            "editor.rotate_right",
            "editor.action",
            r#"{"action":"rotate_right"}"#,
        ),
        editor(
            "editor.rotate_left",
            "editor.action",
            r#"{"action":"rotate_left"}"#,
        ),
        editor("editor.rotate_180", "editor.action", r#"{"action":"rotate_180"}"#),
        editor(
            "editor.rotate_custom",
            "editor.action",
            r#"{"action":"rotate_custom"}"#,
        ),
        editor(
            "editor.flip_horizontal",
            "editor.action",
            r#"{"action":"flip_horizontal"}"#,
        ),
        editor(
            "editor.flip_vertical",
            "editor.action",
            r#"{"action":"flip_vertical"}"#,
        ),
        editor("editor.corners", "editor.action", r#"{"action":"corners"}"#),
        editor("editor.resolution", "editor.action", r#"{"action":"resolution"}"#),
        editor("editor.revert", "editor.action", r#"{"action":"revert"}"#),
        editor(
            "editor.flip_objects_horizontal",
            "editor.action",
            r#"{"action":"flip_objects_horizontal"}"#,
        ),
        editor(
            "editor.flip_objects_vertical",
            "editor.action",
            r#"{"action":"flip_objects_vertical"}"#,
        ),
        editor(
            "editor.crop_to_canvas",
            "editor.action",
            r#"{"action":"crop_to_canvas"}"#,
        ),
        editor("editor.fit_canvas", "editor.action", r#"{"action":"fit_canvas"}"#),
        editor("editor.trim", "editor.action", r#"{"action":"trim"}"#),
        editor("editor.snap", "editor.action", r#"{"action":"snap"}"#),
        editor("editor.new_image", "editor.action", r#"{"action":"new_image"}"#),
        editor(
            "editor.show_in_library",
            "editor.action",
            r#"{"action":"show_in_library"}"#,
        ),
        editor(
            "editor.new_from_template",
            "editor.action",
            r#"{"action":"new_from_template"}"#,
        ),
        editor(
            "editor.apply_template",
            "editor.action",
            r#"{"action":"apply_template"}"#,
        ),
        editor(
            "editor.save_as_template",
            "editor.action",
            r#"{"action":"save_as_template"}"#,
        ),
        editor(
            "editor.import_template",
            "editor.action",
            r#"{"action":"import_template"}"#,
        ),
        editor(
            "editor.batch_export",
            "editor.action",
            r#"{"action":"batch_export"}"#,
        ),
        editor("editor.print", "editor.action", r#"{"action":"print"}"#),
        editor("editor.pixel_grid", "editor.action", r#"{"action":"pixel_grid"}"#),
        editor("editor.fullscreen", "editor.action", r#"{"action":"fullscreen"}"#),
        editor(
            "editor.new_from_clipboard",
            "editor.action",
            r#"{"action":"new_from_clipboard"}"#,
        ),
        editor("editor.zoom_in", "editor.zoom", r#"{"mode":"in"}"#),
        editor("editor.zoom_out", "editor.zoom", r#"{"mode":"out"}"#),
        editor(
            "editor.zoom_actual",
            "editor.zoom",
            r#"{"mode":"percent","value":100}"#,
        ),
        editor("editor.zoom_fit", "editor.zoom", r#"{"mode":"fit"}"#),
        editor("editor.zoom_original", "editor.zoom", r#"{"mode":"original"}"#),
        editor("editor.side_panel", "editor.action", r#"{"action":"side_panel"}"#),
        editor("editor.ai_ask", "editor.action", r#"{"action":"ai_ask"}"#),
        editor("editor.ai_name", "editor.action", r#"{"action":"ai_name"}"#),
        editor(
            "editor.view_editor",
            "editor.action",
            r#"{"action":"view_editor"}"#,
        ),
        editor(
            "editor.view_library",
            "editor.action",
            r#"{"action":"view_library"}"#,
        ),
        editor("editor.tray", "editor.action", r#"{"action":"tray"}"#),
        editor("editor.prev_item", "editor.action", r#"{"action":"prev_item"}"#),
        editor("editor.next_item", "editor.action", r#"{"action":"next_item"}"#),
        editor(
            "editor.close_others",
            "editor.action",
            r#"{"action":"close_others"}"#,
        ),
        editor(
            "editor.rename_item",
            "editor.action",
            r#"{"action":"rename_item"}"#,
        ),
        editor("editor.pin_item", "editor.action", r#"{"action":"pin_item"}"#),
        editor(
            "editor.duplicate_item",
            "editor.action",
            r#"{"action":"duplicate_item"}"#,
        ),
        editor(
            "editor.shortcut_sheet",
            "editor.action",
            r#"{"action":"shortcut_sheet"}"#,
        ),
        editor(
            "editor.move_computer",
            "editor.action",
            r#"{"action":"move_computer"}"#,
        ),
        editor(
            "editor.show_in_folder",
            "editor.action",
            r#"{"action":"show_in_folder"}"#,
        ),
        editor(
            "editor.delete_item",
            "editor.action",
            r#"{"action":"delete_item"}"#,
        ),
        // Tools (EDT-115): a single key.
        tool("editor.tool_favorites", r#"{"tool":"favorites"}"#),
        tool("editor.tool_move", r#"{"tool":"move"}"#),
        tool("editor.tool_arrow", r#"{"tool":"arrow"}"#),
        tool("editor.tool_line", r#"{"tool":"line"}"#),
        tool("editor.tool_text", r#"{"tool":"text"}"#),
        tool("editor.tool_shape", r#"{"tool":"shape"}"#),
        tool("editor.tool_callout", r#"{"tool":"callout"}"#),
        tool("editor.tool_stamp", r#"{"tool":"stamp"}"#),
        tool("editor.tool_step", r#"{"tool":"step"}"#),
        tool("editor.tool_highlight", r#"{"tool":"highlight"}"#),
        tool("editor.tool_pen", r#"{"tool":"pen"}"#),
        tool("editor.tool_magnify", r#"{"tool":"magnify"}"#),
        tool("editor.tool_spotlight", r#"{"tool":"spotlight"}"#),
        tool("editor.tool_blur", r#"{"tool":"blur"}"#),
        tool("editor.tool_simplify", r#"{"tool":"simplify"}"#),
        tool("editor.tool_select", r#"{"tool":"select"}"#),
        tool("editor.tool_wand", r#"{"tool":"wand"}"#),
        tool("editor.tool_cutout", r#"{"tool":"cutout"}"#),
        tool("editor.tool_fill", r#"{"tool":"fill"}"#),
        tool("editor.tool_eraser", r#"{"tool":"eraser"}"#),
        tool("editor.tool_crop", r#"{"tool":"crop"}"#),
    ]
}

/// Default shortcut of a slot in a profile (APP-021; capture actions: CAP-065, CAP-066).
pub fn default_for(slot: &Slot, profile: Profile, platform: Platform) -> Option<Accelerator> {
    use Platform::*;
    use Profile::*;
    use ShortcutScope::*;
    let s = match (slot.id, slot.scope, profile, platform) {
        // CAP-065. Coexistence: Ctrl+Alt+Shift (Windows) or Control+Option+Shift (macOS)
        // plus a letter; Capture on Windows is Ctrl+Shift+PrtScn (D-08). Compatibility
        // (CAP-066): Capture = PrtScn on Windows; Control+Shift + letter on macOS for the
        // actions that have one there. The rest keep their Coexistence value.
        ("capture.capture", Global, Coexistence, Windows) => "Ctrl+Shift+PrintScreen",
        ("capture.capture", Global, Compatibility, Windows) => "PrintScreen",
        ("capture.capture", Global, Coexistence, Macos) => "Ctrl+Alt+Shift+KeyC",
        ("capture.capture", Global, Compatibility, Macos) => "Ctrl+Shift+KeyC",
        ("capture.universal", Global, Compatibility, Macos) => "Ctrl+Shift+KeyO",
        ("capture.universal", Global, _, _) => "Ctrl+Alt+Shift+KeyO",
        ("capture.image", Global, Compatibility, Macos) => "Ctrl+Shift+KeyS",
        ("capture.image", Global, _, _) => "Ctrl+Alt+Shift+KeyS",
        ("smart.extract_text", Global, _, _) => "Ctrl+Alt+Shift+KeyT",
        ("capture.window", Global, Compatibility, Macos) => "Ctrl+Shift+KeyW",
        ("capture.window", Global, _, _) => "Ctrl+Alt+Shift+KeyW",
        ("capture.active_window", Global, _, _) => "Ctrl+Alt+Shift+KeyA",
        ("capture.window_picker", Global, Compatibility, Macos) => "Ctrl+Shift+KeyM",
        ("capture.window_picker", Global, _, _) => "Ctrl+Alt+Shift+KeyM",
        ("capture.quick_annotate", Global, _, _) => "Ctrl+Alt+Shift+KeyQ",
        // VID-075.
        ("record.video", Global, Compatibility, Windows) => return None,
        ("record.video", Global, Compatibility, Macos) => "Ctrl+Shift+KeyV",
        ("record.video", Global, _, _) => "Ctrl+Alt+Shift+KeyV",
        ("record.toggle", Global, Coexistence, Windows) => "Ctrl+Shift+F9",
        ("record.toggle", Global, Compatibility, Windows) => "Shift+F9",
        ("record.toggle", Global, Coexistence, Macos) => "Ctrl+Alt+Shift+Space",
        ("record.toggle", Global, Compatibility, Macos) => "Ctrl+Shift+Space",
        ("record.stop", Global, Coexistence, Windows) => "Ctrl+Shift+F10",
        ("record.stop", Global, Compatibility, Windows) => "Shift+F10",
        // Ctrl+Shift+D is Duplicate of the editor (Compatibility, Windows).
        ("record.draw", Global, Compatibility, Macos) => "Ctrl+Shift+KeyD",
        ("record.draw", Global, _, _) => "Ctrl+Alt+Shift+KeyD",
        ("capture.repeat_last", Global, Compatibility, Macos) => "Ctrl+Shift+KeyR",
        ("capture.repeat_last", Global, _, _) => "Ctrl+Alt+Shift+KeyR",
        ("capture.panel", Global, _, _) => "Ctrl+Alt+Shift+KeyP",
        // M0 test shortcut for the exit criterion. Provisional: Coexistence family
        // with a letter CAP-065 does not use (see spec/QUESTIONS.md).
        ("app.ping", Global, _, _) => "Ctrl+Alt+Shift+KeyK",
        ("app.quit", Global, Compatibility, Windows) => "Ctrl+Shift+KeyX",
        ("app.quit", App, _, Windows) => "Ctrl+KeyQ",
        ("app.quit", App, _, Macos) => "Super+KeyQ",
        ("app.close_window", App, _, Windows) => "Ctrl+KeyW",
        ("app.close_window", App, _, Macos) => "Super+KeyW",
        ("settings.open", App, _, Windows) => "Ctrl+Comma",
        ("settings.open", App, _, Macos) => "Super+Comma",
        ("app.help", App, _, Windows) => "F1",
        ("app.help", App, _, Macos) => "Super+Shift+Slash",
        (_, Editor, _, _) => return editor_default(slot.id, profile, platform),
        _ => return None,
    };
    Some(s.parse().expect("default shortcuts parse"))
}

/// Editor defaults (EDT-115, EDT-116). Written for Windows; on macOS `Ctrl` becomes
/// `Cmd`, except where the table names `Control`.
fn editor_default(id: &str, profile: Profile, platform: Platform) -> Option<Accelerator> {
    use Platform::*;
    use Profile::*;
    let s = match (id, profile, platform) {
        ("editor.close", Compatibility, Macos) => "Super+Shift+KeyW",
        ("editor.copy_all", Compatibility, Macos) => "Alt+Super+KeyC",
        ("editor.side_panel", Compatibility, Macos) => "Super+Digit5",
        ("editor.redo", _, Macos) => "Super+Shift+KeyZ",
        // The table's Ctrl+Alt+Shift+S is Image capture on Windows (CAP-065), a global
        // shortcut that would never let it through (spec/QUESTIONS.md).
        ("editor.export", _, Windows) => "Ctrl+Alt+Shift+KeyE",
        ("editor.redo_alt", _, Macos) => return None,
        // macOS: Control+Cmd+F (EDT-009).
        ("editor.fullscreen", _, Macos) => "Ctrl+Super+KeyF",
        // Compatibility on Windows: Ctrl+Shift+X quits the app (a global shortcut).
        ("editor.trim", Compatibility, Windows) => return None,
        // Compatibility on macOS: Cmd+R asks for the angle (EDT-116).
        ("editor.rotate_custom", Compatibility, Macos) => "Super+KeyR",
        ("editor.rotate_right", Compatibility, Macos) => return None,
        // Compatibility on macOS: Control and a letter per tool (EDT-115).
        ("editor.tool_favorites", Compatibility, Macos) => "Ctrl+KeyR",
        ("editor.tool_move", Compatibility, Macos) => "Ctrl+KeyV",
        ("editor.tool_arrow", Compatibility, Macos) => "Ctrl+KeyA",
        ("editor.tool_text", Compatibility, Macos) => "Ctrl+KeyT",
        ("editor.tool_shape", Compatibility, Macos) => "Ctrl+KeyU",
        ("editor.tool_blur", Compatibility, Macos) => "Ctrl+KeyB",
        ("editor.tool_simplify", Compatibility, Macos) => "Ctrl+KeyY",
        ("editor.tool_select", Compatibility, Macos) => "Ctrl+KeyL",
        ("editor.tool_wand", Compatibility, Macos) => "Ctrl+KeyW",
        ("editor.tool_cutout", Compatibility, Macos) => "Ctrl+KeyX",
        ("editor.tool_fill", Compatibility, Macos) => "Ctrl+KeyF",
        ("editor.tool_eraser", Compatibility, Macos) => "Ctrl+KeyE",
        ("editor.tool_step", Compatibility, Macos) => "Ctrl+KeyS",
        ("editor.tool_stamp", Compatibility, Macos) => "Ctrl+KeyI",
        ("editor.tool_highlight", Compatibility, Macos) => "Ctrl+KeyH",
        ("editor.tool_pen", Compatibility, Macos) => "Ctrl+KeyP",
        ("editor.tool_magnify", Compatibility, Macos) => "Ctrl+KeyM",
        ("editor.tool_callout", Compatibility, Macos) => "Ctrl+KeyD",
        ("editor.tool_crop", Compatibility, Macos) => "Ctrl+KeyC",
        ("editor.tool_favorites", ..) => "KeyF",
        ("editor.tool_move", ..) => "KeyV",
        ("editor.tool_arrow", ..) => "KeyA",
        ("editor.tool_text", ..) => "KeyT",
        ("editor.tool_line", ..) => "KeyL",
        ("editor.tool_shape", ..) => "KeyU",
        ("editor.tool_blur", ..) => "KeyB",
        ("editor.tool_simplify", ..) => "KeyY",
        ("editor.tool_select", ..) => "KeyQ",
        ("editor.tool_wand", ..) => "KeyW",
        ("editor.tool_cutout", ..) => "KeyX",
        ("editor.tool_fill", ..) => "KeyG",
        ("editor.tool_eraser", ..) => "KeyE",
        ("editor.tool_step", ..) => "KeyN",
        ("editor.tool_stamp", ..) => "KeyS",
        ("editor.tool_highlight", ..) => "KeyH",
        ("editor.tool_pen", ..) => "KeyP",
        ("editor.tool_magnify", ..) => "KeyM",
        ("editor.tool_spotlight", ..) => "KeyO",
        ("editor.tool_callout", ..) => "KeyC",
        ("editor.tool_crop", ..) => "KeyR",
        ("editor.delete", ..) => "Delete",
        // LIB-001: Cmd+1 / Cmd+2 on macOS; Ctrl+Shift on Windows, where Ctrl+1…9 pick
        // tools. TRY-001, TRY-009, TRY-013 as the spec writes them.
        ("editor.view_editor", _, Macos) => "Super+Digit1",
        ("editor.view_library", _, Macos) => "Super+Digit2",
        ("editor.tray", _, Macos) => "Alt+Super+KeyT",
        ("editor.prev_item", _, Macos) => "Super+BracketLeft",
        ("editor.next_item", _, Macos) => "Super+BracketRight",
        ("editor.delete_item", _, Macos) => "Super+Backspace",
        ("editor.rename_item", _, Macos) => return None,
        // TRY-004: Control+Shift+P on macOS too, as the spec writes it.
        ("editor.pin_item", _, Macos) => "Ctrl+Shift+KeyP",
        // LIB-014: none on Windows, where Ctrl+Shift+L is the share link.
        ("editor.show_in_library", _, Macos) => "Shift+Super+KeyL",
        ("editor.show_in_library", ..) => return None,
        ("editor.delete_alt", ..) => "Backspace",
        _ => {
            let windows = match id {
                "editor.open" => "Ctrl+KeyO",
                "editor.new_image" => "Ctrl+KeyN",
                "editor.new_from_template" => "Ctrl+Shift+KeyT",
                "editor.print" => "Ctrl+KeyP",
                "editor.pixel_grid" => "Ctrl+Quote",
                "editor.fullscreen" => "F11",
                "editor.new_from_clipboard" => "Ctrl+Shift+KeyN",
                "editor.save" => "Ctrl+KeyS",
                "editor.save_as" => "Ctrl+Shift+KeyS",
                "editor.export" => "Ctrl+Alt+Shift+KeyS",
                "editor.close" => "Ctrl+KeyW",
                "editor.close_all" => "Ctrl+Alt+KeyW",
                "editor.undo" => "Ctrl+KeyZ",
                "editor.redo" => "Ctrl+KeyY",
                "editor.redo_alt" => "Ctrl+Shift+KeyZ",
                "editor.cut" => "Ctrl+KeyX",
                "editor.copy" => "Ctrl+KeyC",
                "editor.paste" => "Ctrl+KeyV",
                "editor.paste_in_place" => "Ctrl+Shift+KeyV",
                "editor.copy_all" => "Ctrl+Shift+KeyC",
                "editor.select_all" => "Ctrl+KeyA",
                "editor.duplicate" => "Ctrl+KeyJ",
                "editor.lock" => "Ctrl+KeyL",
                "editor.group" => "Ctrl+KeyG",
                "editor.flatten" => "Ctrl+KeyE",
                "editor.flatten_all" => "Ctrl+Shift+KeyE",
                "editor.ungroup" => "Ctrl+Shift+KeyG",
                "editor.bring_forward" => "Ctrl+ArrowUp",
                "editor.send_backward" => "Ctrl+ArrowDown",
                "editor.bring_front" => "Ctrl+Shift+ArrowUp",
                "editor.send_back" => "Ctrl+Shift+ArrowDown",
                "editor.deselect" => "Ctrl+KeyD",
                "editor.invert_selection" => "Ctrl+Shift+KeyI",
                "editor.float_selection" => "Ctrl+Shift+KeyJ",
                "editor.crop_selection" => "Ctrl+Shift+KeyK",
                "editor.resize_image" => "Ctrl+Shift+KeyR",
                "editor.canvas_size" => "Ctrl+Alt+KeyK",
                "editor.rotate_right" => "Ctrl+KeyR",
                "editor.crop_to_canvas" => "Ctrl+KeyK",
                "editor.trim" => "Ctrl+Shift+KeyX",
                "editor.snap" => "Ctrl+Semicolon",
                "editor.zoom_in" => "Ctrl+Equal",
                "editor.zoom_out" => "Ctrl+Minus",
                "editor.zoom_actual" => "Ctrl+Digit0",
                "editor.zoom_fit" => "Ctrl+Shift+Digit0",
                "editor.zoom_original" => "Ctrl+Alt+Digit0",
                "editor.side_panel" => "Ctrl+Alt+KeyP",
                "editor.view_editor" => "Ctrl+Shift+Digit1",
                "editor.view_library" => "Ctrl+Shift+Digit2",
                "editor.tray" => "Ctrl+Alt+KeyR",
                "editor.prev_item" => "Ctrl+PageUp",
                "editor.next_item" => "Ctrl+PageDown",
                "editor.delete_item" => "Ctrl+Delete",
                "editor.rename_item" => "F2",
                "editor.pin_item" => "Ctrl+Shift+KeyP",
                // TRY-010: Shift+Cmd+D on macOS.
                "editor.duplicate_item" => "Ctrl+Shift+KeyD",
                // APP-023: Cmd+/ on macOS.
                "editor.shortcut_sheet" => "Ctrl+Slash",
                _ => return None,
            };
            let mut acc: Accelerator = windows.parse().expect("editor defaults parse");
            if platform == Macos && acc.ctrl {
                acc.ctrl = false;
                acc.meta = true;
            }
            return Some(acc);
        }
    };
    Some(s.parse().expect("editor defaults parse"))
}

// ---------------------------------------------------------------------------
// Validation and system conflicts

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SystemConflict {
    /// The OS never lets apps use it (APP-019).
    Reserved,
    /// The OS assigns it to one of its own features; the user can change that.
    SystemAssigned {
        #[serde(skip_serializing_if = "Option::is_none")]
        settings_uri: Option<&'static str>,
    },
}

pub const WINDOWS_KEYBOARD_SETTINGS: &str = "ms-settings:easeofaccess-keyboard";
pub const MACOS_KEYBOARD_SETTINGS: &str = "x-apple.systempreferences:com.apple.preference.keyboard?Shortcuts";

/// Checks an accelerator against the OS lists. `print_screen_to_system` tells whether
/// the Windows "Print Screen opens screen capture" setting is on.
pub fn system_conflict(
    acc: &Accelerator,
    platform: Platform,
    print_screen_to_system: bool,
) -> Option<SystemConflict> {
    let canonical = acc.to_string();
    match platform {
        Platform::Windows => {
            const RESERVED: &[&str] = &[
                "Ctrl+Alt+Delete",
                "Super+KeyL",
                "Alt+Tab",
                "Ctrl+Shift+Escape",
                "Super+Tab",
            ];
            const ASSIGNED: &[&str] = &[
                "Shift+Super+KeyS",
                "Super+PrintScreen",
                "Alt+PrintScreen",
                "Alt+Super+PrintScreen",
            ];
            if RESERVED.contains(&canonical.as_str()) {
                return Some(SystemConflict::Reserved);
            }
            if canonical == "PrintScreen" && print_screen_to_system {
                return Some(SystemConflict::SystemAssigned {
                    settings_uri: Some(WINDOWS_KEYBOARD_SETTINGS),
                });
            }
            if ASSIGNED.contains(&canonical.as_str()) {
                return Some(SystemConflict::SystemAssigned { settings_uri: None });
            }
        }
        Platform::Macos => {
            const RESERVED: &[&str] = &["Super+Tab", "Shift+Super+Tab", "Alt+Super+Escape"];
            const ASSIGNED: &[&str] = &[
                "Shift+Super+Digit3",
                "Shift+Super+Digit4",
                "Shift+Super+Digit5",
                "Shift+Super+Digit6",
                "Super+Space",
                "Alt+Super+Space",
            ];
            if RESERVED.contains(&canonical.as_str()) {
                return Some(SystemConflict::Reserved);
            }
            if ASSIGNED.contains(&canonical.as_str()) {
                return Some(SystemConflict::SystemAssigned {
                    settings_uri: Some(MACOS_KEYBOARD_SETTINGS),
                });
            }
        }
    }
    None
}

/// Why a shortcut cannot be assigned at all (APP-017).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rejection {
    ForbiddenKey,
    ModifierOnly,
    NeedsModifier,
    Reserved,
}

impl Rejection {
    pub fn code(&self) -> &'static str {
        match self {
            Rejection::ForbiddenKey => "forbidden_key",
            Rejection::ModifierOnly => "modifier_only",
            Rejection::NeedsModifier => "needs_modifier",
            Rejection::Reserved => "reserved",
        }
    }

    fn i18n_key(&self) -> &'static str {
        match self {
            Rejection::ForbiddenKey => "shortcuts.reject.forbidden_key",
            Rejection::ModifierOnly => "shortcuts.reject.modifier_only",
            Rejection::NeedsModifier => "shortcuts.reject.needs_modifier",
            Rejection::Reserved => "shortcuts.reject.reserved",
        }
    }
}

pub fn validate(acc: &Accelerator, scope: ShortcutScope, platform: Platform) -> Result<(), Rejection> {
    // Explain OS-reserved combinations (Cmd+Tab, Win+L) as such, even if they use Tab.
    if system_conflict(acc, platform, false) == Some(SystemConflict::Reserved) {
        return Err(Rejection::Reserved);
    }
    if FORBIDDEN_CODES.contains(&acc.code.as_str()) {
        return Err(Rejection::ForbiddenKey);
    }
    if MODIFIER_CODES.contains(&acc.code.as_str()) {
        return Err(Rejection::ModifierOnly);
    }
    if scope == ShortcutScope::Global {
        let function_key = acc
            .code
            .strip_prefix('F')
            .and_then(|n| n.parse::<u8>().ok())
            .is_some_and(|n| (1..=24).contains(&n));
        let alone_ok = function_key || acc.code == "PrintScreen";
        if !acc.has_modifier() && !alone_ok {
            return Err(Rejection::NeedsModifier);
        }
        if system_conflict(acc, platform, false) == Some(SystemConflict::Reserved) {
            return Err(Rejection::Reserved);
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Registration status reported by the shell

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Status {
    Active,
    Unassigned,
    /// Global shortcuts paused from the tray menu or while recording a shortcut.
    Paused,
    /// Another XcreenShot command already has this combination.
    Duplicate {
        command: String,
    },
    System(SystemConflict),
    /// Another app registered it first (APP-019). The OS rarely says which.
    OtherApp,
    /// Taken from another app while XcreenShot runs (APP-020).
    TakenOver,
}

impl Status {
    pub fn is_conflict(&self) -> bool {
        matches!(
            self,
            Status::Duplicate { .. } | Status::System(_) | Status::OtherApp
        )
    }
}

// ---------------------------------------------------------------------------
// Store

#[derive(Debug, Default, Serialize, Deserialize)]
struct Stored {
    /// slot key -> accelerator, or `null` when the user removed the default.
    #[serde(default)]
    overrides: BTreeMap<String, Option<Accelerator>>,
}

/// Outcome of a successful `set`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetOutcome {
    /// Slot that lost its shortcut because the user chose Reassign.
    pub reassigned_from: Option<String>,
}

/// Rows of features on trial (APP-034): the row and the setting that turns it on.
/// While the setting is off the row does not exist (CAP-057).
pub const TRIAL_SLOTS: [(&str, &str); 1] = [("capture.quick_annotate", "experimental.quickAnnotate")];

pub struct ShortcutStore {
    path: Option<PathBuf>,
    platform: Platform,
    slots: Vec<Slot>,
    /// Rows of trial features that are off (APP-034).
    hidden: RwLock<Vec<&'static str>>,
    /// Rows that come and go with the user's things, with their own title: one per
    /// capture preset (CAP-062, APP-022).
    extra: RwLock<Vec<(Slot, String)>>,
    stored: RwLock<Stored>,
    statuses: RwLock<HashMap<String, Status>>,
    paused: AtomicBool,
    recording: AtomicBool,
}

impl ShortcutStore {
    pub fn load(path: PathBuf) -> ShortcutStore {
        let stored = std::fs::read(&path)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        ShortcutStore {
            path: Some(path),
            platform: Platform::current(),
            slots: slots(),
            hidden: RwLock::new(TRIAL_SLOTS.iter().map(|(id, _)| *id).collect()),
            extra: RwLock::new(Vec::new()),
            stored: RwLock::new(stored),
            statuses: RwLock::new(HashMap::new()),
            paused: AtomicBool::new(false),
            recording: AtomicBool::new(false),
        }
    }

    pub fn in_memory(platform: Platform) -> ShortcutStore {
        ShortcutStore {
            path: None,
            platform,
            slots: slots(),
            hidden: RwLock::new(TRIAL_SLOTS.iter().map(|(id, _)| *id).collect()),
            extra: RwLock::new(Vec::new()),
            stored: RwLock::new(Stored::default()),
            statuses: RwLock::new(HashMap::new()),
            paused: AtomicBool::new(false),
            recording: AtomicBool::new(false),
        }
    }

    pub fn platform(&self) -> Platform {
        self.platform
    }

    /// Shows or hides the rows of trial features as `on` says for each setting
    /// (APP-034). Returns whether anything changed.
    pub fn set_trials(&self, on: impl Fn(&str) -> bool) -> bool {
        let hidden: Vec<&'static str> = TRIAL_SLOTS
            .iter()
            .filter(|(_, key)| !on(key))
            .map(|(id, _)| *id)
            .collect();
        let mut current = self.hidden.write().unwrap();
        let changed = *current != hidden;
        *current = hidden;
        changed
    }

    /// Every row: the app's, then the presets' (CAP-062).
    pub fn slots(&self) -> Vec<Slot> {
        let hidden = self.hidden.read().unwrap().clone();
        let mut all: Vec<Slot> = self
            .slots
            .iter()
            .filter(|s| !hidden.contains(&s.id))
            .copied()
            .collect();
        all.extend(self.extra.read().unwrap().iter().map(|(s, _)| *s));
        all
    }

    /// The rows of the user's things (one per preset) and their titles; they replace
    /// the ones there were (CAP-062).
    pub fn set_extra(&self, rows: Vec<(Slot, String)>) {
        *self.extra.write().unwrap() = rows;
    }

    /// The title of a row of the user's things.
    pub fn extra_title(&self, slot: &Slot) -> Option<String> {
        self.extra
            .read()
            .unwrap()
            .iter()
            .find(|(s, _)| s == slot)
            .map(|(_, t)| t.clone())
    }

    /// Forgets the shortcut of a row that is gone (a deleted preset), so the keys are
    /// free again (CAP-060).
    pub fn forget(&self, key: &str) -> Option<Option<Accelerator>> {
        let old = self.stored.write().unwrap().overrides.remove(key);
        self.save();
        old
    }

    /// Puts back a row's shortcut as it was (Undo of a delete, CAP-060).
    pub fn put_back(&self, key: &str, acc: Option<Accelerator>) {
        self.stored
            .write()
            .unwrap()
            .overrides
            .insert(key.to_string(), acc);
        self.save();
    }

    pub fn find_slot(&self, id: &str, scope: ShortcutScope) -> Option<Slot> {
        self.slots().into_iter().find(|s| s.id == id && s.scope == scope)
    }

    /// The user's own shortcuts: slot key → shortcut, or `None` where a default was
    /// removed (APP-011).
    pub fn overrides(&self) -> BTreeMap<String, Option<Accelerator>> {
        self.stored.read().unwrap().overrides.clone()
    }

    /// The slot with this key (`<id>` or `<id>@<scope>`).
    pub fn slot_by_key(&self, key: &str) -> Option<Slot> {
        self.slots().into_iter().find(|s| s.key() == key)
    }

    /// The user's shortcuts become `wanted` (APP-011): what equals the profile's default
    /// is not kept as an override; slots not named go back to their default.
    pub fn replace_overrides(&self, wanted: &BTreeMap<String, Option<Accelerator>>, profile: Profile) {
        {
            let mut stored = self.stored.write().unwrap();
            // A preset's shortcut goes with its preset, not with the app's (APP-021).
            stored.overrides.retain(|k, _| is_preset_key(k));
            let all = self.slots();
            for (key, acc) in wanted {
                let Some(slot) = all.iter().find(|s| s.key() == *key) else {
                    continue;
                };
                if *acc != default_for(slot, profile, self.platform) {
                    stored.overrides.insert(key.clone(), acc.clone());
                }
            }
        }
        self.save();
    }

    pub fn is_custom(&self, slot: &Slot) -> bool {
        self.stored.read().unwrap().overrides.contains_key(&slot.key())
    }

    /// Effective shortcut of a slot: the user's override, else the profile default.
    pub fn binding(&self, slot: &Slot, profile: Profile) -> Option<Accelerator> {
        match self.stored.read().unwrap().overrides.get(&slot.key()) {
            Some(custom) => custom.clone(),
            None => default_for(slot, profile, self.platform),
        }
    }

    pub fn bindings(&self, profile: Profile) -> Vec<(Slot, Accelerator)> {
        self.slots()
            .into_iter()
            .filter_map(|s| self.binding(&s, profile).map(|a| (s, a)))
            .collect()
    }

    /// Global bindings the shell should register, without internal duplicates
    /// (the first slot keeps a duplicated combination).
    pub fn global_bindings(&self, profile: Profile) -> Vec<(Slot, Accelerator)> {
        let mut seen: Vec<Accelerator> = Vec::new();
        self.bindings(profile)
            .into_iter()
            .filter(|(s, a)| {
                if s.scope != ShortcutScope::Global || seen.contains(a) {
                    return false;
                }
                seen.push(a.clone());
                true
            })
            .collect()
    }

    /// Other slot that already uses `acc` where `slot` would also work (APP-017). An
    /// editor shortcut may repeat an app one: in the editor it wins.
    pub fn collision(&self, slot: &Slot, acc: &Accelerator, profile: Profile) -> Option<Slot> {
        self.bindings(profile).into_iter().find_map(|(other, a)| {
            (other != *slot && slot.scope.collides_with(other.scope) && a == *acc).then_some(other)
        })
    }

    /// Assigns (or removes, with `None`) a shortcut. With `reassign`, a colliding
    /// slot loses its shortcut; without it, a collision is an error (APP-018).
    pub fn set(
        &self,
        slot: &Slot,
        acc: Option<Accelerator>,
        reassign: bool,
        profile: Profile,
        lang: Lang,
    ) -> Result<SetOutcome, CommandError> {
        let mut outcome = SetOutcome {
            reassigned_from: None,
        };
        if let Some(acc) = &acc {
            validate(acc, slot.scope, self.platform).map_err(|r| {
                CommandError::new(ErrorCode::InvalidParams, tr(lang, r.i18n_key(), &[]))
                    .with_details(json!({ "reason": r.code() }))
            })?;
            if let Some(other) = self.collision(slot, acc, profile) {
                if !reassign {
                    return Err(CommandError::new(
                        ErrorCode::Busy,
                        tr(lang, "shortcuts.in_use", &[("command", other.id)]),
                    )
                    .with_details(
                        json!({ "conflict": "duplicate", "command": other.id, "slot": other.key() }),
                    ));
                }
                self.stored.write().unwrap().overrides.insert(other.key(), None);
                outcome.reassigned_from = Some(other.key());
            }
        }
        {
            let mut stored = self.stored.write().unwrap();
            if acc == default_for(slot, profile, self.platform) {
                stored.overrides.remove(&slot.key());
            } else {
                stored.overrides.insert(slot.key(), acc);
            }
        }
        self.save();
        Ok(outcome)
    }

    /// Restores one slot (or all) to the profile default.
    pub fn reset(&self, slot: Option<&Slot>) {
        {
            let mut stored = self.stored.write().unwrap();
            match slot {
                Some(s) => {
                    stored.overrides.remove(&s.key());
                }
                // Presets keep theirs: they belong to the preset (CAP-062, APP-021).
                None => stored.overrides.retain(|k, _| is_preset_key(k)),
            }
        }
        self.save();
    }

    /// What switching profile would change (APP-021): rows whose effective value changes,
    /// and custom shortcuts that would collide with a new default.
    pub fn preview_profile(&self, from: Profile, to: Profile, reset_custom: bool) -> Value {
        let mut changes = Vec::new();
        let mut conflicts = Vec::new();
        for slot in &self.slots {
            // The presets' rows have no profile default and keep their shortcut.
            let before = self.binding(slot, from);
            let after = if reset_custom || !self.is_custom(slot) {
                default_for(slot, to, self.platform)
            } else {
                before.clone()
            };
            if before != after {
                changes.push(json!({
                    "slot": slot.key(), "command": slot.id,
                    "from": before.as_ref().map(|a| a.display(self.platform)),
                    "to": after.as_ref().map(|a| a.display(self.platform)),
                }));
            }
            if !reset_custom && self.is_custom(slot) {
                if let Some(acc) = &before {
                    for other in &self.slots {
                        if other != slot
                            && !self.is_custom(other)
                            && default_for(other, to, self.platform).as_ref() == Some(acc)
                            && slot.scope.collides_with(other.scope)
                        {
                            conflicts.push(json!({"slot": slot.key(), "with": other.key(), "shortcut": acc.display(self.platform)}));
                        }
                    }
                }
            }
        }
        json!({ "changes": changes, "conflicts": conflicts })
    }

    fn save(&self) {
        let Some(path) = &self.path else { return };
        let stored = self.stored.read().unwrap();
        if let Ok(bytes) = serde_json::to_vec_pretty(&*stored) {
            if let Err(e) = write_atomic(path, &bytes) {
                tracing::error!(error = %e, "cannot save shortcuts");
            }
        }
    }

    pub fn set_status(&self, slot: &Slot, status: Status) {
        self.statuses.write().unwrap().insert(slot.key(), status);
    }

    pub fn status(&self, slot: &Slot, profile: Profile) -> Status {
        let Some(acc) = self.binding(slot, profile) else {
            return Status::Unassigned;
        };
        if slot.scope == ShortcutScope::App {
            return Status::Active;
        }
        // A global shortcut takes the keys first; of two editor shortcuts with the same
        // keys, the first one runs.
        if let Some((winner, _)) = self.bindings(profile).into_iter().find(|(s, a)| {
            *a == acc
                && (s.scope == ShortcutScope::Global
                    || (slot.scope == ShortcutScope::Editor && s.scope == ShortcutScope::Editor))
        }) {
            if winner != *slot {
                return Status::Duplicate {
                    command: winner.id.to_string(),
                };
            }
        }
        if slot.scope == ShortcutScope::Editor {
            return Status::Active;
        }
        if self.paused() {
            return Status::Paused;
        }
        self.statuses
            .read()
            .unwrap()
            .get(&slot.key())
            .cloned()
            .unwrap_or(Status::Active)
    }

    pub fn paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst) || self.recording.load(Ordering::SeqCst)
    }

    /// Paused from the tray menu (APP-002).
    pub fn user_paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst)
    }

    pub fn set_paused(&self, paused: bool) {
        self.paused.store(paused, Ordering::SeqCst);
    }

    /// Global shortcuts are suspended while the user records one (APP-018).
    pub fn set_recording(&self, recording: bool) -> bool {
        self.recording.swap(recording, Ordering::SeqCst) != recording
    }

    /// Global shortcuts that could not be activated, for the single startup notice (APP-019).
    pub fn failed(&self, profile: Profile) -> Vec<Slot> {
        self.slots()
            .into_iter()
            .filter(|s| s.scope == ShortcutScope::Global && self.status(s, profile).is_conflict())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn acc(s: &str) -> Accelerator {
        s.parse().unwrap()
    }

    /// CAP-057, APP-034: a trial feature's shortcut exists only while it is on.
    #[test]
    fn a_trial_feature_shortcut_comes_and_goes_with_it() {
        let store = ShortcutStore::in_memory(Platform::Windows);
        let has = |s: &ShortcutStore| s.slots().iter().any(|x| x.id == "capture.quick_annotate");
        assert!(!has(&store));
        assert!(store.set_trials(|k| k == "experimental.quickAnnotate"));
        assert!(has(&store));
        let slot = store
            .find_slot("capture.quick_annotate", ShortcutScope::Global)
            .unwrap();
        assert_eq!(
            default_for(&slot, Profile::Coexistence, Platform::Windows),
            Some(acc("Ctrl+Alt+Shift+KeyQ"))
        );
        assert!(!store.set_trials(|k| k == "experimental.quickAnnotate"));
        store.set_trials(|_| false);
        assert!(!has(&store));
    }

    #[test]
    fn parses_aliases_into_physical_codes() {
        assert_eq!(acc("Ctrl+Alt+A").to_string(), "Ctrl+Alt+KeyA");
        assert_eq!(acc("cmd+shift+5").to_string(), "Shift+Super+Digit5");
        assert_eq!(acc("PrtScn").to_string(), "PrintScreen");
        assert_eq!(
            acc("Control+Option+Shift+KeyC").to_string(),
            "Ctrl+Alt+Shift+KeyC"
        );
        assert_eq!(acc("Ctrl+,").to_string(), "Ctrl+Comma");
        assert_eq!(acc("F13").to_string(), "F13");
        assert!("Ctrl+".parse::<Accelerator>().is_err());
        assert!("Ctrl+Bogus+K".parse::<Accelerator>().is_err());
    }

    #[test]
    fn displays_per_platform() {
        let a = acc("Ctrl+Alt+Shift+K");
        assert_eq!(a.display(Platform::Windows), "Ctrl+Alt+Shift+K");
        assert_eq!(a.display(Platform::Macos), "⌃⌥⇧K");
        assert_eq!(
            acc("Ctrl+Shift+PrtScn").display(Platform::Windows),
            "Ctrl+Shift+PrtScn"
        );
    }

    /// APP-017 criterion: `A` alone is rejected as global, `Ctrl+Alt+A` accepted.
    #[test]
    fn global_rules() {
        let p = Platform::Windows;
        assert_eq!(
            validate(&acc("A"), ShortcutScope::Global, p),
            Err(Rejection::NeedsModifier)
        );
        assert!(validate(&acc("Ctrl+Alt+A"), ShortcutScope::Global, p).is_ok());
        assert!(validate(&acc("PrtScn"), ShortcutScope::Global, p).is_ok());
        assert!(validate(&acc("F9"), ShortcutScope::Global, p).is_ok());
        assert!(validate(&acc("A"), ShortcutScope::App, p).is_ok());
        assert_eq!(
            validate(&acc("Esc"), ShortcutScope::App, p),
            Err(Rejection::ForbiddenKey)
        );
        assert_eq!(
            validate(&acc("Ctrl+Tab"), ShortcutScope::App, p),
            Err(Rejection::ForbiddenKey)
        );
        assert_eq!(
            validate(&acc("Ctrl+Enter"), ShortcutScope::Global, p),
            Err(Rejection::ForbiddenKey)
        );
        assert_eq!(
            validate(&acc("Ctrl+ShiftLeft"), ShortcutScope::Global, p),
            Err(Rejection::ModifierOnly)
        );
        assert_eq!(
            validate(&acc("Win+L"), ShortcutScope::Global, p),
            Err(Rejection::Reserved)
        );
        assert_eq!(
            validate(&acc("Cmd+Tab"), ShortcutScope::Global, Platform::Macos),
            Err(Rejection::Reserved)
        );
    }

    #[test]
    fn system_assigned_combinations() {
        assert_eq!(
            system_conflict(&acc("PrtScn"), Platform::Windows, true),
            Some(SystemConflict::SystemAssigned {
                settings_uri: Some(WINDOWS_KEYBOARD_SETTINGS)
            })
        );
        assert_eq!(system_conflict(&acc("PrtScn"), Platform::Windows, false), None);
        assert_eq!(
            system_conflict(&acc("Cmd+Shift+4"), Platform::Macos, false),
            Some(SystemConflict::SystemAssigned {
                settings_uri: Some(MACOS_KEYBOARD_SETTINGS)
            })
        );
    }

    /// CAP-065: Coexistence defaults never use the system capture combinations.
    #[test]
    fn coexistence_defaults_avoid_system_captures() {
        for platform in [Platform::Windows, Platform::Macos] {
            let store = ShortcutStore::in_memory(platform);
            for (slot, a) in store.bindings(Profile::Coexistence) {
                if slot.scope == ShortcutScope::Global {
                    assert_eq!(system_conflict(&a, platform, true), None, "{}", slot.key());
                    assert!(validate(&a, slot.scope, platform).is_ok());
                }
            }
        }
    }

    /// EDT-116 criterion: no two actions share a combination where both would work.
    #[test]
    fn no_default_combination_is_shared() {
        for platform in [Platform::Windows, Platform::Macos] {
            for profile in [Profile::Coexistence, Profile::Compatibility] {
                let store = ShortcutStore::in_memory(platform);
                let bindings = store.bindings(profile);
                for (i, (a, acc_a)) in bindings.iter().enumerate() {
                    for (b, acc_b) in &bindings[i + 1..] {
                        assert!(
                            !(acc_a == acc_b && a.scope.collides_with(b.scope)),
                            "{platform:?} {profile:?}: {} and {} share {acc_a}",
                            a.key(),
                            b.key()
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn editor_shortcuts_win_over_app_ones_but_not_global_ones() {
        let store = ShortcutStore::in_memory(Platform::Windows);
        let close = store.find_slot("editor.close", ShortcutScope::Editor).unwrap();
        let app_close = store.find_slot("app.close_window", ShortcutScope::App).unwrap();
        // Both use Ctrl+W: in the editor it closes the document (EDT-108).
        assert_eq!(store.binding(&close, Profile::Coexistence), Some(acc("Ctrl+W")));
        assert_eq!(
            store.binding(&app_close, Profile::Coexistence),
            Some(acc("Ctrl+W"))
        );
        assert_eq!(store.status(&close, Profile::Coexistence), Status::Active);
        let e = store
            .set(
                &close,
                Some(acc("Ctrl+Alt+Shift+K")),
                false,
                Profile::Coexistence,
                Lang::En,
            )
            .unwrap_err();
        assert_eq!(e.details.unwrap()["command"], "app.ping");
        // A single key is fine in the editor (EDT-115).
        let arrow = store
            .find_slot("editor.tool_arrow", ShortcutScope::Editor)
            .unwrap();
        assert!(store
            .set(&arrow, Some(acc("J")), false, Profile::Coexistence, Lang::En)
            .is_ok());
        // Two editor actions cannot share one.
        let text = store
            .find_slot("editor.tool_text", ShortcutScope::Editor)
            .unwrap();
        assert!(store
            .set(&text, Some(acc("J")), false, Profile::Coexistence, Lang::En)
            .is_err());
    }

    #[test]
    fn editor_defaults_follow_the_platform() {
        let mac = ShortcutStore::in_memory(Platform::Macos);
        let slot = |id| mac.find_slot(id, ShortcutScope::Editor).unwrap();
        assert_eq!(
            mac.binding(&slot("editor.save"), Profile::Coexistence),
            Some(acc("Cmd+S"))
        );
        assert_eq!(
            mac.binding(&slot("editor.copy_all"), Profile::Compatibility),
            Some(acc("Option+Cmd+C"))
        );
        assert_eq!(
            mac.binding(&slot("editor.tool_crop"), Profile::Compatibility),
            Some(acc("Control+C"))
        );
        assert_eq!(mac.binding(&slot("editor.redo_alt"), Profile::Coexistence), None);
    }

    #[test]
    fn app_shortcut_cannot_match_a_global_one() {
        let store = ShortcutStore::in_memory(Platform::Windows);
        let settings_app = store.find_slot("settings.open", ShortcutScope::App).unwrap();
        let e = store
            .set(
                &settings_app,
                Some(acc("Ctrl+Alt+Shift+K")),
                false,
                Profile::Coexistence,
                Lang::En,
            )
            .unwrap_err();
        assert_eq!(e.details.unwrap()["command"], "app.ping");
    }

    /// APP-018: reassign leaves the other command without a shortcut.
    #[test]
    fn reassign_moves_the_combination() {
        let store = ShortcutStore::in_memory(Platform::Windows);
        let show = store.find_slot("app.show", ShortcutScope::Global).unwrap();
        let ping = store.find_slot("app.ping", ShortcutScope::Global).unwrap();
        let p = Profile::Coexistence;
        assert!(store
            .set(&show, Some(acc("Ctrl+Alt+Shift+K")), false, p, Lang::En)
            .is_err());
        let out = store
            .set(&show, Some(acc("Ctrl+Alt+Shift+K")), true, p, Lang::En)
            .unwrap();
        assert_eq!(out.reassigned_from.as_deref(), Some("app.ping@global"));
        assert_eq!(store.binding(&ping, p), None);
        assert_eq!(store.binding(&show, p), Some(acc("Ctrl+Alt+Shift+K")));
    }

    /// APP-021: custom shortcuts survive a profile change unless reset is chosen.
    #[test]
    fn profile_switch_keeps_custom_shortcuts() {
        let store = ShortcutStore::in_memory(Platform::Windows);
        let quit = store.find_slot("app.quit", ShortcutScope::Global).unwrap();
        let ping = store.find_slot("app.ping", ShortcutScope::Global).unwrap();
        assert_eq!(store.binding(&quit, Profile::Coexistence), None);
        assert_eq!(
            store.binding(&quit, Profile::Compatibility),
            Some(acc("Ctrl+Shift+X"))
        );
        store
            .set(
                &ping,
                Some(acc("Ctrl+Alt+Shift+J")),
                false,
                Profile::Coexistence,
                Lang::En,
            )
            .unwrap();
        assert_eq!(
            store.binding(&ping, Profile::Compatibility),
            Some(acc("Ctrl+Alt+Shift+J"))
        );
        let preview = store.preview_profile(Profile::Coexistence, Profile::Compatibility, false);
        // Only the defaults that differ (Capture, the three of video, Quit, and Trim
        // edges, which gives Ctrl+Shift+X to Quit); the custom one stays.
        assert_eq!(preview["changes"].as_array().unwrap().len(), 6);
        store.reset(None);
        assert_eq!(
            store.binding(&ping, Profile::Compatibility),
            Some(acc("Ctrl+Alt+Shift+K"))
        );
    }

    #[test]
    fn statuses_and_pause() {
        let store = ShortcutStore::in_memory(Platform::Windows);
        let ping = store.find_slot("app.ping", ShortcutScope::Global).unwrap();
        let show = store.find_slot("app.show", ShortcutScope::Global).unwrap();
        let p = Profile::Coexistence;
        assert_eq!(store.status(&ping, p), Status::Active);
        assert_eq!(store.status(&show, p), Status::Unassigned);
        store.set_status(&ping, Status::OtherApp);
        assert_eq!(store.failed(p), vec![ping]);
        store.set_paused(true);
        assert_eq!(store.status(&ping, p), Status::Paused);
    }
}
