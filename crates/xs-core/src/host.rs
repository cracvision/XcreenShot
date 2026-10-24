//! What the core needs from the desktop shell. The Tauri app implements [`Host`];
//! tests and headless tools use [`NullHost`].

use serde::Serialize;
use serde_json::Value;

use crate::command::{CommandSpec, Context};
use crate::error::{CommandError, CommandResult, ErrorCode};

/// OS permissions XcreenShot may need (APP-024).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    ScreenRecording,
    Accessibility,
    Camera,
    Microphone,
}

impl Permission {
    pub const ALL: [Permission; 4] = [
        Permission::ScreenRecording,
        Permission::Accessibility,
        Permission::Camera,
        Permission::Microphone,
    ];

    pub fn parse(s: &str) -> Option<Permission> {
        Permission::ALL.into_iter().find(|p| p.as_str() == s)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Permission::ScreenRecording => "screen_recording",
            Permission::Accessibility => "accessibility",
            Permission::Camera => "camera",
            Permission::Microphone => "microphone",
        }
    }

    /// System Settings page for this permission on macOS.
    pub fn macos_settings_uri(self) -> &'static str {
        match self {
            Permission::ScreenRecording => {
                "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture"
            }
            Permission::Accessibility => {
                "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"
            }
            Permission::Camera => "x-apple.systempreferences:com.apple.preference.security?Privacy_Camera",
            Permission::Microphone => {
                "x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionState {
    Granted,
    Denied,
    NotDetermined,
}

#[derive(Debug, Clone, Serialize)]
pub struct PermissionStatus {
    pub permission: Permission,
    pub state: PermissionState,
    /// The system still lets the app show its own request dialog.
    pub can_request: bool,
}

/// Frozen image of one monitor, RGBA, row by row (CAP-013).
#[derive(Clone)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub rgba: std::sync::Arc<Vec<u8>>,
}

fn unsupported() -> CommandError {
    CommandError::new(ErrorCode::NotSupportedOnPlatform, "not available in this context")
}

#[allow(unused_variables)]
pub trait Host: Send + Sync {
    /// Shows the main window (editor and library).
    fn show_main_window(&self) -> Result<(), CommandError> {
        Err(unsupported())
    }
    fn close_focused_window(&self) -> Result<(), CommandError> {
        Err(unsupported())
    }
    fn open_settings(&self, section: Option<&str>, key: Option<&str>) -> Result<(), CommandError> {
        Err(unsupported())
    }
    /// Starts the quit sequence (APP-004).
    fn quit(&self) {}
    /// Shows what needs a decision before quitting (APP-004), in the main window.
    /// Without a UI nothing is asked.
    fn confirm_quit(&self, pending: &[Value]) {}
    /// The editor view must reflect a change (`document`, `documents`, `tool`, `zoom`,
    /// `recovery`…), whoever made it: UI, shortcut or CLI (EDT-119).
    fn editor_event(&self, kind: &str, payload: Value) {}
    /// A setting changed; apply it (theme, language, icon, login item…).
    fn settings_changed(&self, key: &str, value: &Value) {}
    /// Shortcuts or their pause state changed; re-register global shortcuts.
    fn shortcuts_changed(&self) {}
    fn monitors(&self) -> Vec<Value> {
        Vec::new()
    }
    /// Permission status; empty on platforms that do not gate capture (Windows in M0).
    fn permissions(&self) -> Vec<PermissionStatus> {
        Vec::new()
    }
    fn request_permission(&self, permission: Permission) -> Result<(), CommandError> {
        Err(unsupported())
    }
    fn open_uri(&self, uri: &str) -> Result<(), CommandError> {
        Err(unsupported())
    }
    fn open_path(&self, path: &std::path::Path) -> Result<(), CommandError> {
        Err(unsupported())
    }
    fn copy_text(&self, text: &str) -> Result<(), CommandError> {
        Err(unsupported())
    }
    /// Current OS login item state, if the shell can read it (APP-003).
    fn launch_at_login(&self) -> Option<bool> {
        None
    }
    /// Called after every command (UI events, notifications for shortcut-triggered tests).
    fn command_finished(&self, ctx: &Context, spec: &CommandSpec, result: &CommandResult) {}
    /// System notification (APP-016). Categories: `captures`, `share`, `problems`,
    /// `updates`, `automation`.
    fn notify(&self, category: &str, title: &str, body: &str) {}
    /// A capture finished; `open_editor` asks to show it (CAP-045, CAP-054).
    fn capture_finished(&self, result: &Value, thumbnail_png: Option<&[u8]>, open_editor: bool) {}
    /// Hide XcreenShot windows before a capture, and show them again after (CAP-023).
    fn hide_for_capture(&self, hide: bool) {}
    /// A capture starts (`true`) or ends: the capture panel steps aside meanwhile and
    /// comes back afterwards (CAP-001).
    fn capture_session(&self, active: bool) {}
    /// Shows the delay countdown and blocks until it ends (CAP-048). Returns `false` if
    /// the user cancelled it. Without a UI it simply waits.
    fn countdown(&self, seconds: f64) -> bool {
        std::thread::sleep(std::time::Duration::from_secs_f64(seconds.max(0.0)));
        true
    }
    /// Shows, hides or (with `None`) toggles the capture panel (CAP-001, CAP-065).
    /// Returns whether it is visible afterwards.
    fn show_capture_panel(&self, show: Option<bool>) -> Result<bool, CommandError> {
        Err(unsupported())
    }
    /// Shows the on-screen selection over the frozen `frames` (one per monitor, in
    /// the order of `request.monitors`) and blocks until the user finishes or cancels
    /// (CAP-009, CAP-013, CAP-025, CAP-027). Returns `{"kind": "rect", "rect": {…}}`,
    /// `{"kind": "window", "id": …}` or `{"kind": "cancelled"}`.
    fn select_on_screen(&self, request: Value, frames: Vec<Frame>) -> Result<Value, CommandError> {
        Err(unsupported())
    }
}

pub struct NullHost;

impl Host for NullHost {}
