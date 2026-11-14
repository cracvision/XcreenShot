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
    /// XcreenShot's notifications, which the system can block (APP-016).
    Notifications,
}

impl Permission {
    pub const ALL: [Permission; 5] = [
        Permission::ScreenRecording,
        Permission::Accessibility,
        Permission::Camera,
        Permission::Microphone,
        Permission::Notifications,
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
            Permission::Notifications => "notifications",
        }
    }

    /// Settings page for this permission on Windows, if it has one.
    pub fn windows_settings_uri(self) -> Option<&'static str> {
        match self {
            Permission::Notifications => Some("ms-settings:notifications"),
            Permission::Camera => Some("ms-settings:privacy-webcam"),
            Permission::Microphone => Some("ms-settings:privacy-microphone"),
            _ => None,
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
            Permission::Notifications => "x-apple.systempreferences:com.apple.preference.notifications",
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
    /// The welcome window of the first run (APP-005).
    fn open_welcome(&self) -> Result<(), CommandError> {
        Err(unsupported())
    }
    /// The Help window at a topic, or at its first page (APP-015).
    fn open_help(&self, topic: Option<&str>) -> Result<(), CommandError> {
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
    /// A notification that opens something when clicked (APP-016, CAP-055): `open`
    /// names it, e.g. `{"documentId", "path"}` of a capture. With `folder`, it also
    /// offers Show in folder.
    fn notify_opening(&self, category: &str, title: &str, body: &str, open: Value) {
        let _ = open;
        self.notify(category, title, body);
    }
    /// Shows the results window of an Extract text (SMT-006).
    fn open_text_result(&self, id: &str) -> Result<(), CommandError> {
        let _ = id;
        Err(unsupported())
    }
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
    /// Shows (`true`) or closes the multi-capture panel (CAP-067).
    fn multicapture_panel(&self, show: bool) {}
    /// The multi-capture state changed (`{capturing, items}`): the panel shows it.
    fn multicapture_event(&self, state: Value) {}
    /// Shows the scrolling capture's panel beside `{rect, auto}` (CAP-034, CAP-035), or
    /// closes it with `None`. Never part of a capture.
    fn scroll_panel(&self, place: Option<Value>) {}
    /// The scrolling capture's state changed (`{running, auto, started, width, height,
    /// full, preview}`): the panel shows it.
    fn scroll_event(&self, state: Value) {}
    /// The interval series changed (CAP-049): started, a capture, stopped.
    fn interval_event(&self, state: Value) {}
    /// A video recording started, paused, resumed or stopped (VID-020, VID-024).
    fn recording_event(&self, state: Value) {}
    /// The step recorder's bar (SMT-023): shown with `Some(state)` (`status`, `steps`,
    /// `confirm`…), closed with `None`. Never in the steps' pictures.
    fn steps_bar(&self, state: Option<Value>) {}
    /// The notice "Recording steps: every click makes a step" (SMT-022), for 2 s.
    fn steps_notice(&self, show: bool) {}
    /// The recording bar and the border of the area (VID-011, VID-020): shown for
    /// `Some(state)` (`status`, `area`, `elapsed`, `width`, `height`), closed for `None`.
    fn recording_bar(&self, state: Option<Value>) {}
    /// Live drawing (VID-048 to VID-051): its overlay over the monitor that holds
    /// `area` (x, y, width, height in physical pixels of the virtual screen) and its
    /// toolbar; neither with `None`. The overlay shows in captures and videos; clicks go
    /// through it; the toolbar never shows in them.
    fn live_draw(&self, area: Option<(i32, i32, u32, u32)>) {}
    /// What the overlay and the toolbar show: the shapes, the tool, the colors.
    fn live_draw_state(&self, state: Value) {}
    /// The system's Open dialog: the files chosen, or none if cancelled. `filters`
    /// are (name, extensions) (CAP-040, CAP-041).
    fn pick_files(
        &self,
        multiple: bool,
        filters: &[(String, Vec<String>)],
    ) -> Option<Vec<std::path::PathBuf>> {
        None
    }
    /// The Import PDF window for an import id (CAP-041).
    fn pdf_window(&self, id: &str) -> Result<(), CommandError> {
        Err(unsupported())
    }
    /// How a PDF import goes: `{id, done, total}` (CAP-041).
    fn pdf_event(&self, state: Value) {}
    /// Loads a web page privately and takes it whole (CAP-037): `{png (base64), url,
    /// title, status, cut}`, or why not (`cancelled`, `timeout`, `unreachable`, …).
    fn load_webpage(&self, request: &Value) -> Result<Value, String> {
        Err("not available here".into())
    }
    /// Stops the page being loaded (CAP-037).
    fn cancel_webpage(&self) {}
    /// "Working" next to a zone of the screen (`{x, y, width, height}`, physical), or
    /// gone with `None` (SMT-005).
    fn working(&self, zone: Option<Value>) {}
    /// The system's Save dialog at `suggested` (folder and name): the path chosen, or
    /// none if cancelled (CAP-053).
    fn pick_save(&self, suggested: &std::path::Path) -> Option<std::path::PathBuf> {
        None
    }
    /// The window that asks for a web address (CAP-037).
    fn webpage_prompt(&self) -> Result<(), CommandError> {
        Err(unsupported())
    }
    /// Shows or closes the box that asks for the text of Capture details (CAP-051); it
    /// answers with `capture.details_text`.
    fn ask_text_window(&self, open: bool) -> Result<(), CommandError> {
        if open {
            Err(unsupported())
        } else {
            Ok(())
        }
    }
    /// Shows or closes the window picker (CAP-030): over the monitor under the pointer,
    /// never part of a capture.
    fn picker_window(&self, show: bool) {}
    /// The window picker's list changed (`{open, windows}`), e.g. a thumbnail is ready.
    fn picker_event(&self, state: Value) {}
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
