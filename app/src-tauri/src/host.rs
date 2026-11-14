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

/// Whether the user wants notifications of this category (APP-016). Categories without
/// a switch yet (share, updates) are on.
fn category_on(app: &AppHandle, category: &str) -> bool {
    let core = &app.state::<crate::AppState>().core;
    core.settings
        .get(&format!("notifications.{category}"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true)
}

/// The kind of notification that opens a capture, with its two actions (CAP-055).
pub const CAPTURE_ACTIONS: &str = "capture";

/// Registers the notification actions and what a click does (APP-016, CAP-055): a
/// click or Open shows the capture in the editor; Show in folder, its file.
pub fn setup_notifications(app: &AppHandle) {
    use tauri_plugin_notification::{Action, ActionType};
    let core = app.state::<crate::AppState>().core.clone();
    let lang = core.settings.lang();
    let actions = ActionType::builder(CAPTURE_ACTIONS)
        .actions(vec![
            Action::builder("open", tr(lang, "notify.action_open", &[])).build(),
            Action::builder("folder", tr(lang, "notify.action_folder", &[])).build(),
        ])
        .build();
    if let Err(e) = app.notification().register_action_types(vec![actions]) {
        tracing::warn!(error = %e, "notification actions not registered");
    }
    let _ = app.notification().on_action(move |performed| {
        let Some(open) = performed
            .notification()
            .and_then(|n| n.extra().get("open").cloned())
        else {
            return;
        };
        let core = core.clone();
        let action = performed.action_id().to_string();
        // Off the notification thread: opening may read a file.
        std::thread::spawn(move || {
            // Extract text (SMT-005): View opens its results window.
            if let Some(id) = open["textResult"].as_str() {
                let _ = core.call(Source::Ui, "smart.show_text", json!({ "id": id }), None);
                return;
            }
            let path = open["path"].as_str().unwrap_or_default().to_string();
            if action == "folder" {
                let _ = core.call(
                    Source::Ui,
                    "library.show_in_folder",
                    json!({ "path": path }),
                    None,
                );
                return;
            }
            // The capture may have been closed since: open its file again.
            let opened = core
                .call(
                    Source::Ui,
                    "document.open",
                    json!({ "path": path, "show": true }),
                    None,
                )
                .ok()
                .and_then(|r| r["documentId"].as_str().map(String::from))
                .or_else(|| open["documentId"].as_str().map(String::from));
            if let Some(id) = opened {
                let _ = core.call(Source::Ui, "editor.show", json!({ "documentId": id }), None);
            }
        });
    });
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

    fn open_welcome(&self) -> Result<(), CommandError> {
        windows::open_welcome(&self.app).map_err(internal)
    }

    fn open_help(&self, topic: Option<&str>) -> Result<(), CommandError> {
        windows::open_help(&self.app, topic).map_err(internal)
    }

    fn open_text_result(&self, id: &str) -> Result<(), CommandError> {
        // Window work belongs on the main thread.
        let app = self.app.clone();
        let id = id.to_string();
        self.app
            .run_on_main_thread(move || {
                if let Err(e) = windows::open_text_result(&app, &id) {
                    tracing::warn!(error = %e, "text results window not opened");
                }
            })
            .map_err(internal)
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
            // Capture interface: the edge launcher comes or goes (CAP-004).
            "capture.interface" => {
                let app = self.app.clone();
                let _ = self.app.run_on_main_thread(move || crate::launcher::sync(&app));
            }
            "shortcuts.takeOver" => self.shortcuts_changed(),
            // A smart feature on or off: its menus and shortcuts come or go (SMT-002).
            k if xs_core::features::is_setting(k) => self.shortcuts_changed(),
            // A test copy never changes how the installed app starts.
            "general.launchAtLogin" if xs_core::paths::isolated() => {
                tracing::info!("isolated instance: login item left alone");
            }
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
        // The menus show each entry's shortcut (EDT-116).
        crate::rebuild_app_menu(&self.app);
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
        let _lock = xs_core::clipboard_lock();
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

    fn multicapture_panel(&self, show: bool) {
        let app = self.app.clone();
        let _ = self.app.run_on_main_thread(move || {
            if let Err(e) = windows::multicapture_panel(&app, show) {
                tracing::error!(error = %e, "multi-capture panel");
            }
        });
    }

    fn multicapture_event(&self, state: Value) {
        let _ = self.app.emit("xs://multicapture", state);
    }

    fn scroll_panel(&self, place: Option<Value>) {
        let app = self.app.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        let queued = self.app.run_on_main_thread(move || {
            if let Err(e) = windows::scroll_panel(&app, place) {
                tracing::error!(error = %e, "scrolling capture panel");
            }
            // The icon menu offers Finish scrolling meanwhile (CAP-034).
            crate::tray::refresh(&app);
            let _ = tx.send(());
        });
        // Wait until it is shown or gone: the first frame must not race with it.
        if queued.is_ok() {
            let _ = rx.recv_timeout(Duration::from_secs(2));
        }
    }

    fn scroll_event(&self, state: Value) {
        let _ = self.app.emit("xs://scroll", state);
    }

    fn pick_files(
        &self,
        multiple: bool,
        filters: &[(String, Vec<String>)],
    ) -> Option<Vec<std::path::PathBuf>> {
        use tauri_plugin_dialog::DialogExt;
        let mut dialog = self.app.dialog().file();
        for (name, extensions) in filters {
            let exts: Vec<&str> = extensions.iter().map(String::as_str).collect();
            dialog = dialog.add_filter(name, &exts);
        }
        let to_paths = |f: tauri_plugin_dialog::FilePath| f.into_path().ok();
        if multiple {
            dialog
                .blocking_pick_files()
                .map(|files| files.into_iter().filter_map(to_paths).collect())
        } else {
            dialog.blocking_pick_file().and_then(to_paths).map(|p| vec![p])
        }
    }

    fn pick_save(&self, suggested: &std::path::Path) -> Option<std::path::PathBuf> {
        use tauri_plugin_dialog::DialogExt;
        let mut dialog = self.app.dialog().file();
        if let Some(dir) = suggested.parent() {
            dialog = dialog.set_directory(dir);
        }
        if let Some(name) = suggested.file_name() {
            dialog = dialog.set_file_name(name.to_string_lossy());
        }
        dialog = dialog
            .add_filter("PNG", &["png"])
            .add_filter("JPG", &["jpg", "jpeg"])
            .add_filter("XcreenShot", &["xshot"]);
        dialog.blocking_save_file().and_then(|f| f.into_path().ok())
    }

    fn pdf_window(&self, id: &str) -> Result<(), CommandError> {
        let app = self.app.clone();
        let id = id.to_string();
        self.app
            .run_on_main_thread(move || {
                if let Err(e) = windows::open_pdf(&app, &id) {
                    tracing::warn!(error = %e, "Import PDF window not opened");
                }
            })
            .map_err(internal)
    }

    fn pdf_event(&self, state: Value) {
        let _ = self.app.emit("xs://pdf", state);
    }

    fn load_webpage(&self, request: &Value) -> Result<Value, String> {
        crate::webpage::capture(&self.app, request)
    }

    fn cancel_webpage(&self) {
        crate::webpage::cancel();
    }

    fn working(&self, zone: Option<Value>) {
        let app = self.app.clone();
        let _ = self.app.run_on_main_thread(move || {
            if let Err(e) = windows::working(&app, zone) {
                tracing::warn!(error = %e, "working indicator");
            }
        });
    }

    fn webpage_prompt(&self) -> Result<(), CommandError> {
        let app = self.app.clone();
        self.app
            .run_on_main_thread(move || {
                if let Err(e) = windows::open_webpage_prompt(&app) {
                    tracing::warn!(error = %e, "web address window not opened");
                }
            })
            .map_err(internal)
    }

    fn ask_text_window(&self, open: bool) -> Result<(), CommandError> {
        let app = self.app.clone();
        self.app
            .run_on_main_thread(move || {
                if let Err(e) = windows::ask_text(&app, open) {
                    tracing::warn!(error = %e, "text box for the capture details not shown");
                }
            })
            .map_err(internal)
    }

    fn recording_bar(&self, state: Option<Value>) {
        let app = self.app.clone();
        let _ = self.app.run_on_main_thread(move || {
            if let Err(e) = windows::recording_bar(&app, state) {
                tracing::warn!(error = %e, "recording bar");
            }
        });
    }

    fn steps_bar(&self, state: Option<Value>) {
        let app = self.app.clone();
        let _ = self.app.run_on_main_thread(move || {
            if let Err(e) = windows::steps_bar(&app, state) {
                tracing::warn!(error = %e, "step recorder bar");
            }
            // Pause, Resume and Finish in the icon's menu (SMT-023).
            crate::tray::refresh(&app);
        });
    }

    fn steps_notice(&self, show: bool) {
        let app = self.app.clone();
        let _ = self.app.run_on_main_thread(move || {
            if let Err(e) = windows::steps_notice(&app, show) {
                tracing::warn!(error = %e, "step recorder notice");
            }
        });
    }

    fn live_draw(&self, area: Option<(i32, i32, u32, u32)>) {
        let app = self.app.clone();
        let _ = self.app.run_on_main_thread(move || {
            if let Err(e) = windows::live_draw(&app, area) {
                tracing::warn!(error = %e, "live drawing windows");
            }
        });
    }

    fn live_draw_state(&self, state: Value) {
        // The recording bar shows whether it is on (its Draw button).
        for label in [windows::LIVEDRAW, windows::DRAWBAR, windows::RECBAR] {
            let _ = self.app.emit_to(label, "xs://livedraw", state.clone());
        }
    }

    fn recording_event(&self, state: Value) {
        let _ = self.app.emit("xs://record", state);
        // Stop recording in the icon's menu, and its clock (VID-020).
        let app = self.app.clone();
        let _ = self.app.run_on_main_thread(move || crate::tray::refresh(&app));
    }

    fn interval_event(&self, state: Value) {
        let _ = self.app.emit("xs://interval", state);
        // The clock on the icon and its Stop capture entries (APP-002, CAP-005).
        let app = self.app.clone();
        let _ = self.app.run_on_main_thread(move || crate::tray::refresh(&app));
    }

    fn picker_window(&self, show: bool) {
        let app = self.app.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        let queued = self.app.run_on_main_thread(move || {
            if let Err(e) = windows::picker_window(&app, show) {
                tracing::error!(error = %e, "window picker");
            }
            let _ = tx.send(());
        });
        // Gone before the window is captured.
        if queued.is_ok() {
            let _ = rx.recv_timeout(Duration::from_secs(2));
        }
    }

    fn picker_event(&self, state: Value) {
        let _ = self.app.emit("xs://picker", state);
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

    fn notify(&self, category: &str, title: &str, body: &str) {
        if category_on(&self.app, category) {
            notify(&self.app, title, body);
        }
    }

    fn notify_opening(&self, category: &str, title: &str, body: &str, open: Value) {
        if !category_on(&self.app, category) {
            return;
        }
        static NEXT: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(1);
        let shown = self
            .app
            .notification()
            .builder()
            .id(NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
            .title(title)
            .body(body)
            .action_type_id(CAPTURE_ACTIONS)
            .extra("open", open)
            .show();
        if let Err(e) = shown {
            tracing::warn!(error = %e, "notification failed");
        }
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
        if kind == "menu_state" {
            *self.app.state::<crate::AppState>().menu_state.lock().unwrap() = payload;
            crate::apply_menu_state(&self.app);
            return;
        }
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
