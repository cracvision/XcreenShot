//! [`xs_core::host::Host`] implemented on top of Tauri.

use std::path::Path;
use std::time::Duration;

use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_autostart::ManagerExt as _;
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_opener::OpenerExt;
use xs_core::host::{Host, Permission, PermissionStatus};
use xs_core::i18n::tr;
use xs_core::{CommandError, CommandResult, CommandSpec, Context, Source};

use crate::{global, permissions, tray, windows};

pub struct TauriHost {
    pub app: AppHandle,
}

fn internal(e: impl std::fmt::Display) -> CommandError {
    CommandError::internal(e.to_string())
}

/// Real state of the login item (APP-003): registered, and on Windows not turned off
/// in Settings > Apps > Startup.
pub fn login_item_enabled(app: &AppHandle) -> Option<bool> {
    let registered = app.autolaunch().is_enabled().ok()?;
    #[cfg(windows)]
    if registered && xs_core::winreg::login_item_turned_off(&app.package_info().name) {
        return Some(false);
    }
    Some(registered)
}

pub fn notify(app: &AppHandle, title: &str, body: &str) {
    if let Err(e) = app.notification().builder().title(title).body(body).show() {
        tracing::warn!(error = %e, "notification failed");
    }
}

impl Host for TauriHost {
    fn show_main_window(&self) -> Result<(), CommandError> {
        windows::show_main(&self.app).map_err(internal)
    }

    fn close_focused_window(&self) -> Result<(), CommandError> {
        windows::close_focused(&self.app);
        Ok(())
    }

    fn open_settings(&self, section: Option<&str>, key: Option<&str>) -> Result<(), CommandError> {
        windows::open_preferences(&self.app, section, key).map_err(internal)
    }

    fn quit(&self) {
        tracing::info!("quit requested");
        crate::overlay::cancel(&self.app);
        let app = self.app.clone();
        // Let the caller (e.g. the CLI) receive its answer first.
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(150));
            app.exit(0);
        });
    }

    fn settings_changed(&self, key: &str, value: &Value) {
        match key {
            "general.theme" => windows::apply_theme(&self.app),
            "general.language" => {
                windows::retitle(&self.app);
                crate::rebuild_app_menu(&self.app);
                tray::refresh(&self.app);
            }
            "general.showIcon" | "general.monochromeIcon" => tray::refresh(&self.app),
            "shortcuts.takeOver" => self.shortcuts_changed(),
            "general.launchAtLogin" => {
                let auto = self.app.autolaunch();
                let r = if value.as_bool().unwrap_or(false) {
                    // A previous "off" in Windows Settings would otherwise keep it disabled.
                    #[cfg(windows)]
                    xs_core::winreg::clear_login_item_override(&self.app.package_info().name);
                    auto.enable()
                } else {
                    auto.disable()
                };
                if let Err(e) = r {
                    tracing::error!(error = %e, "could not change the login item");
                }
            }
            _ => {}
        }
        let _ = self
            .app
            .emit("xs://settings", json!({ "key": key, "value": value }));
    }

    fn shortcuts_changed(&self) {
        global::apply(&self.app);
        tray::refresh(&self.app);
        let _ = self.app.emit("xs://shortcuts", ());
    }

    fn monitors(&self) -> Vec<Value> {
        let Some(w) = self.app.webview_windows().into_values().next() else {
            return self
                .app
                .primary_monitor()
                .ok()
                .flatten()
                .map(|m| vec![monitor(&m)])
                .unwrap_or_default();
        };
        w.available_monitors()
            .map(|ms| ms.iter().map(monitor).collect())
            .unwrap_or_default()
    }

    fn permissions(&self) -> Vec<PermissionStatus> {
        permissions::all()
    }

    fn request_permission(&self, permission: Permission) -> Result<(), CommandError> {
        let r = permissions::request(permission, |uri| self.open_uri(uri));
        let _ = self.app.emit("xs://permissions", ());
        tray::refresh(&self.app);
        r
    }

    fn open_uri(&self, uri: &str) -> Result<(), CommandError> {
        self.app.opener().open_url(uri, None::<&str>).map_err(internal)
    }

    fn open_path(&self, path: &Path) -> Result<(), CommandError> {
        self.app
            .opener()
            .open_path(path.to_string_lossy(), None::<&str>)
            .map_err(internal)
    }

    fn copy_text(&self, text: &str) -> Result<(), CommandError> {
        self.app
            .clipboard()
            .write_text(text.to_string())
            .map_err(internal)
    }

    fn launch_at_login(&self) -> Option<bool> {
        login_item_enabled(&self.app)
    }

    fn hide_for_capture(&self, hide: bool) {
        // Apply on the UI thread and wait until it is done, so the capture never
        // races a busy event loop (seen right after the app starts).
        let (tx, rx) = std::sync::mpsc::channel();
        let app = self.app.clone();
        let queued = self.app.run_on_main_thread(move || {
            windows::exclude_from_capture(&app, hide);
            let _ = tx.send(());
        });
        if queued.is_ok() {
            let _ = rx.recv_timeout(std::time::Duration::from_millis(500));
        }
        if hide {
            // One or two compositor frames for the change to reach the screen.
            std::thread::sleep(std::time::Duration::from_millis(34));
        }
    }

    fn capture_session(&self, active: bool) {
        windows::panel_capture_session(&self.app, active);
    }

    fn countdown(&self, seconds: f64) -> bool {
        crate::countdown::run(&self.app, seconds)
    }

    fn show_capture_panel(&self, show: Option<bool>) -> Result<bool, CommandError> {
        let (tx, rx) = std::sync::mpsc::channel();
        let app = self.app.clone();
        self.app
            .run_on_main_thread(move || {
                let _ = tx.send(windows::capture_panel(&app, show));
            })
            .map_err(internal)?;
        rx.recv_timeout(Duration::from_secs(5))
            .map_err(internal)?
            .map_err(internal)
    }

    fn select_on_screen(
        &self,
        request: Value,
        frames: Vec<xs_core::host::Frame>,
    ) -> Result<Value, CommandError> {
        crate::overlay::select(&self.app, request, frames)
    }

    fn notify(&self, _category: &str, title: &str, body: &str) {
        // Per-category switches arrive with Preferences > Notifications (APP-016).
        notify(&self.app, title, body);
    }

    fn capture_finished(&self, result: &Value, thumbnail_png: Option<&[u8]>, open_editor: bool) {
        use base64::Engine as _;
        let thumbnail = thumbnail_png.map(|png| {
            format!(
                "data:image/png;base64,{}",
                base64::engine::general_purpose::STANDARD.encode(png)
            )
        });
        let _ = self.app.emit(
            "xs://capture",
            json!({ "result": result, "thumbnail": thumbnail }),
        );
        // "Open in = Editor" (CAP-045): the capture becomes the document shown.
        if open_editor {
            let app = self.app.clone();
            let id = result["documentId"].clone();
            let _ = self.app.run_on_main_thread(move || {
                // The panel comes back first, so the editor ends up in front of it.
                windows::restore_capture_panel(&app);
                let core = app.state::<crate::AppState>().core.clone();
                std::thread::spawn(move || {
                    if let Err(e) = core.call(Source::Ui, "editor.show", json!({ "documentId": id }), None) {
                        tracing::warn!(code = e.code.as_str(), "could not show the capture in the editor");
                    }
                });
            });
        }
    }

    fn confirm_quit(&self, pending: &[Value]) {
        let _ = windows::show_main(&self.app);
        let _ = self.app.emit(
            "xs://editor",
            json!({ "kind": "quit", "payload": { "pending": pending } }),
        );
    }

    fn editor_event(&self, kind: &str, payload: Value) {
        if kind == "title" {
            let app = self.app.clone();
            let title = payload["title"].as_str().unwrap_or_default().to_string();
            let _ = self
                .app
                .run_on_main_thread(move || windows::set_main_title(&app, &title));
            return;
        }
        let _ = self
            .app
            .emit("xs://editor", json!({ "kind": kind, "payload": payload }));
    }

    fn command_finished(&self, ctx: &Context, spec: &CommandSpec, result: &CommandResult) {
        let payload = json!({
            "id": spec.id,
            "source": ctx.source,
            "ok": result.is_ok(),
            "result": if spec.id == "app.ping" { result.as_ref().ok().cloned() } else { None },
            "error": result.as_ref().err(),
            "at": now_millis(),
        });
        let _ = self.app.emit("xs://command", payload);
        // M0 exit criterion: the test command visibly answers a global shortcut.
        if spec.id == "app.ping" && ctx.source == Source::Shortcut && result.is_ok() {
            let source = tr(ctx.lang, "source.shortcut", &[]);
            notify(
                &self.app,
                &tr(ctx.lang, "notify.ping_title", &[]),
                &tr(ctx.lang, "notify.ping_body", &[("source", &source)]),
            );
        }
        if spec.id == "app.diagnostics" && ctx.source == Source::Ui && result.is_ok() {
            notify(
                &self.app,
                &tr(ctx.lang, "app.name", &[]),
                &tr(ctx.lang, "notify.diagnostics_copied", &[]),
            );
        }
    }
}

fn monitor(m: &tauri::Monitor) -> Value {
    json!({
        "width": m.size().width,
        "height": m.size().height,
        "scale": m.scale_factor(),
    })
}

fn now_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}
