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

/// One assignable (command, scope) pair, shown as a row in Preferences > Shortcuts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Slot {
    pub command: &'static str,
    pub scope: ShortcutScope,
    /// Group in the Shortcuts table (APP-018).
    pub group: &'static str,
}

impl Slot {
    pub fn key(&self) -> String {
        let scope = match self.scope {
            ShortcutScope::Global => "global",
            ShortcutScope::App => "app",
        };
        format!("{}@{scope}", self.command)
    }
}

/// Assignable slots that exist in this build. Owners add theirs as features land.
pub fn slots() -> Vec<Slot> {
    use ShortcutScope::*;
    let s = |command, scope| Slot {
        command,
        scope,
        group: "app",
    };
    vec![
        // Capture actions (CAP-065); the others arrive with the capture panel.
        Slot {
            command: "capture.universal",
            scope: Global,
            group: "capture",
        },
        s("app.ping", Global),
        s("app.show", Global),
        s("settings.open", Global),
        s("settings.open", App),
        s("shortcuts.pause", Global),
        s("app.close_window", App),
        s("app.quit", Global),
        s("app.quit", App),
    ]
}

/// Default shortcut of a slot in a profile (APP-021, CAP-065 for capture commands).
pub fn default_for(slot: &Slot, profile: Profile, platform: Platform) -> Option<Accelerator> {
    use Platform::*;
    use Profile::*;
    use ShortcutScope::*;
    let s = match (slot.command, slot.scope, profile, platform) {
        // M0 test shortcut for the exit criterion. Provisional: Coexistence family
        // with a letter CAP-065 does not use (see spec/QUESTIONS.md).
        // CAP-065: Universal Capture. Compatibility on macOS uses Control+Shift.
        ("capture.universal", Global, Compatibility, Macos) => "Ctrl+Shift+KeyO",
        ("capture.universal", Global, _, _) => "Ctrl+Alt+Shift+KeyO",
        ("app.ping", Global, _, _) => "Ctrl+Alt+Shift+KeyK",
        ("app.quit", Global, Compatibility, Windows) => "Ctrl+Shift+KeyX",
        ("app.quit", App, _, Windows) => "Ctrl+KeyQ",
        ("app.quit", App, _, Macos) => "Super+KeyQ",
        ("app.close_window", App, _, Windows) => "Ctrl+KeyW",
        ("app.close_window", App, _, Macos) => "Super+KeyW",
        ("settings.open", App, _, Windows) => "Ctrl+Comma",
        ("settings.open", App, _, Macos) => "Super+Comma",
        _ => return None,
    };
    Some(s.parse().expect("default shortcuts parse"))
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

pub struct ShortcutStore {
    path: Option<PathBuf>,
    platform: Platform,
    slots: Vec<Slot>,
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
            stored: RwLock::new(Stored::default()),
            statuses: RwLock::new(HashMap::new()),
            paused: AtomicBool::new(false),
            recording: AtomicBool::new(false),
        }
    }

    pub fn platform(&self) -> Platform {
        self.platform
    }

    pub fn slots(&self) -> &[Slot] {
        &self.slots
    }

    pub fn find_slot(&self, command: &str, scope: ShortcutScope) -> Option<Slot> {
        self.slots
            .iter()
            .copied()
            .find(|s| s.command == command && s.scope == scope)
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
        self.slots
            .iter()
            .filter_map(|s| self.binding(s, profile).map(|a| (*s, a)))
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

    /// Other slot that already uses `acc` in a way that collides with `slot` (APP-017):
    /// global vs global, and app vs global in both directions.
    pub fn collision(&self, slot: &Slot, acc: &Accelerator, profile: Profile) -> Option<Slot> {
        self.bindings(profile).into_iter().find_map(|(other, a)| {
            let same_command_and_scope = other == *slot;
            let scopes_collide = slot.scope == ShortcutScope::Global || other.scope == ShortcutScope::Global;
            (!same_command_and_scope && scopes_collide && a == *acc).then_some(other)
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
                        tr(lang, "shortcuts.in_use", &[("command", other.command)]),
                    )
                    .with_details(
                        json!({ "conflict": "duplicate", "command": other.command, "slot": other.key() }),
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
                None => stored.overrides.clear(),
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
            let before = self.binding(slot, from);
            let after = if reset_custom || !self.is_custom(slot) {
                default_for(slot, to, self.platform)
            } else {
                before.clone()
            };
            if before != after {
                changes.push(json!({
                    "slot": slot.key(), "command": slot.command,
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
                            && (slot.scope == ShortcutScope::Global || other.scope == ShortcutScope::Global)
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
        if let Some((winner, _)) = self
            .bindings(profile)
            .into_iter()
            .find(|(s, a)| s.scope == ShortcutScope::Global && *a == acc)
        {
            if winner != *slot {
                return Status::Duplicate {
                    command: winner.command.to_string(),
                };
            }
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
        self.slots
            .iter()
            .filter(|s| s.scope == ShortcutScope::Global && self.status(s, profile).is_conflict())
            .copied()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn acc(s: &str) -> Accelerator {
        s.parse().unwrap()
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
        assert_eq!(preview["changes"].as_array().unwrap().len(), 1);
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
