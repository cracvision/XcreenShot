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
}

pub struct NullHost;

impl Host for NullHost {}
