//! XcreenShot desktop shell (Tauri 2). Owns the background process, the icon,
//! global shortcuts and the windows; every action goes through `xs_core`'s registry.

#[cfg(windows)]
mod freeze_win;
mod global;
mod host;
mod overlay;
mod permissions;
mod tray;
mod windows;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{json, Value};
use tauri::{AppHandle, Manager, RunEvent, WindowEvent};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt as _};
use xs_core::ipc::{self, Request, Response};
use xs_core::paths::Paths;
use xs_core::{logging, CommandError, Core, Source};

pub struct AppState {
    pub core: Arc<Core>,
    /// Native menu id -> (command, params).
    pub actions: Mutex<tray::Actions>,
    /// OS shortcut id -> command.
    pub shortcut_map: Mutex<HashMap<u32, String>>,
    /// Section/key to show when the Preferences window finishes loading.
    pub pending_navigation: Mutex<Option<Value>>,
    /// On-screen selection session (CAP-013).
    pub overlays: overlay::Overlays,
}

/// The single entry point from the webviews: run a registered command.
/// The UI can only claim to be the UI or an in-app shortcut, never CLI or MCP.
#[tauri::command]
async fn xs_run(
    state: tauri::State<'_, AppState>,
    id: String,
    params: Option<Value>,
    source: Option<String>,
) -> Result<Value, CommandError> {
    let source = match source.as_deref() {
        Some("shortcut") => Source::Shortcut,
        _ => Source::Ui,
    };
    let core = state.core.clone();
    let params = params.unwrap_or_else(|| json!({}));
    tauri::async_runtime::spawn_blocking(move || core.call(source, &id, params, None))
        .await
        .map_err(|e| CommandError::internal(e.to_string()))?
}

/// A selection window drew its frozen frame (CAP-013).
#[tauri::command]
fn xs_overlay_ready(app: AppHandle, seq: u64, index: usize, timings: Option<Value>) {
    if let Some(t) = timings {
        tracing::info!(index, timings = %t, "selection frame drawn");
    }
    overlay::ready(&app, seq, index);
}

/// Raw pixels around the pointer for the magnifier (CAP-015).
#[tauri::command]
fn xs_frame_patch(
    app: AppHandle,
    seq: u64,
    index: usize,
    x: i32,
    y: i32,
    w: u32,
    h: u32,
) -> tauri::ipc::Response {
    tauri::ipc::Response::new(overlay::patch(&app, seq, index, x, y, w, h))
}

/// The user finished or cancelled the on-screen selection.
#[tauri::command]
fn xs_overlay_finish(app: AppHandle, seq: u64, outcome: Value) {
    overlay::finish(&app, seq, outcome);
}

/// Preferences window: where to go after loading (APP-006 deep links).
#[tauri::command]
fn xs_take_navigation(state: tauri::State<'_, AppState>) -> Option<Value> {
    state.pending_navigation.lock().unwrap().take()
}

pub(crate) fn rebuild_app_menu(app: &AppHandle) {
    #[cfg(target_os = "macos")]
    {
        let state = app.state::<AppState>();
        let mut actions = state.actions.lock().unwrap();
        let entries = xs_core::menus::app_menu(xs_core::platform::Platform::Macos);
        if let Ok(menu) = tray::build_menu(app, &state.core, &entries, false, &mut actions) {
            let _ = app.set_menu(menu);
        }
    }
    #[cfg(windows)]
    {
        let state = app.state::<AppState>();
        if let Some(w) = app.get_webview_window(windows::MAIN) {
            let mut actions = state.actions.lock().unwrap();
            let entries = xs_core::menus::app_menu(xs_core::platform::Platform::Windows);
            if let Ok(menu) = tray::build_menu(app, &state.core, &entries, false, &mut actions) {
                let _ = w.set_menu(menu);
            }
        }
    }
}

/// APP-040: refuse to start below Windows 10 22H2 with a clear message.
#[cfg(windows)]
fn check_windows_version(lang: xs_core::Lang) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};
    const MIN_BUILD: u32 = 19045;
    if xs_core::core::windows_build().is_none_or(|b| b >= MIN_BUILD) {
        return true;
    }
    let wide = |s: String| s.encode_utf16().chain(Some(0)).collect::<Vec<u16>>();
    let title = wide(xs_core::i18n::tr(lang, "system.unsupported_title", &[]));
    let body = wide(xs_core::i18n::tr(lang, "system.unsupported_windows", &[]));
    // SAFETY: null-terminated UTF-16 buffers that outlive the call.
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            body.as_ptr(),
            title.as_ptr(),
            MB_OK | MB_ICONERROR,
        )
    };
    false
}

/// Requests from the CLI. The claimed source is not trusted: only a second app
/// launch may ask, as the UI, to show the main window (single instance, APP-001).
fn serve_ipc(core: Arc<Core>) -> std::io::Result<()> {
    let handler: ipc::RequestHandler = Arc::new(move |req: Request| {
        let source = match (req.source, req.command.as_str()) {
            (Source::Ui, "app.show") => Source::Ui,
            (Source::Mcp, _) => Source::Mcp,
            _ => Source::Cli,
        };
        Response::from(core.call(source, &req.command, req.params, req.lang))
    });
    ipc::serve(&ipc::endpoint(), handler).map(|_| ())
}

pub fn run() {
    let started = std::time::Instant::now();
    let background = std::env::args().any(|a| a == "--background");
    let paths = Paths::standard();
    let log = logging::init(&paths.logs_dir(), false);
    let endpoint = ipc::endpoint();

    // Single instance (APP-001): hand over to the running one and leave.
    if ipc::is_running(&endpoint) {
        if !background {
            let req = Request {
                v: ipc::PROTOCOL_VERSION,
                command: "app.show".into(),
                params: json!({}),
                source: Source::Ui,
                lang: None,
            };
            let _ = ipc::send(&endpoint, &req, Some(Duration::from_secs(5)));
        }
        return;
    }

    let core = Core::with_extensions(paths.clone(), Some(log), vec![xs_capture::extension()]);
    #[cfg(windows)]
    if !check_windows_version(core.lang()) {
        return;
    }
    tracing::info!(
        version = xs_core::platform::VERSION,
        platform = xs_core::platform::Platform::current().as_str(),
        arch = xs_core::platform::arch(),
        background,
        "XcreenShot starting"
    );
    let first_run_marker = paths.config_dir.join(".first-run-done");

    let app = tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(global::on_shortcut)
                .build(),
        )
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--background"]),
        ))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        // Remembers size and position of normal windows (APP-004, APP-006). Selection
        // windows are left out: the plugin shows any window it has no saved state for,
        // which made a hidden selection window appear at startup.
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_filter(|label| !label.starts_with(overlay::LABEL_PREFIX))
                .build(),
        )
        .manage(AppState {
            core: core.clone(),
            actions: Mutex::new(HashMap::new()),
            shortcut_map: Mutex::new(HashMap::new()),
            pending_navigation: Mutex::new(None),
            overlays: overlay::Overlays::default(),
        })
        .invoke_handler(tauri::generate_handler![
            xs_run,
            xs_take_navigation,
            xs_overlay_ready,
            xs_overlay_finish,
            xs_frame_patch
        ])
        .register_asynchronous_uri_scheme_protocol("xsframe", |ctx, request, responder| {
            let app = ctx.app_handle().clone();
            std::thread::spawn(move || responder.respond(overlay::serve_frame(&app, &request)));
        })
        .setup(move |app| {
            let handle = app.handle().clone();
            core.set_host(Arc::new(host::TauriHost { app: handle.clone() }));
            permissions::init(core.paths.config_dir.clone());

            if let Err(e) = serve_ipc(core.clone()) {
                tracing::error!(error = %e, "could not open the local automation channel");
            }

            // First run of a release build: propose starting at sign-in (APP-003, APP-005).
            if !first_run_marker.exists() {
                if !cfg!(debug_assertions) && core.settings.get_bool("general.launchAtLogin") {
                    let _ = handle.autolaunch().enable();
                }
                let _ = std::fs::create_dir_all(&core.paths.config_dir);
                let _ = std::fs::write(&first_run_marker, b"");
            }
            if let Some(enabled) = host::login_item_enabled(&handle) {
                core.settings
                    .sync_external("general.launchAtLogin", json!(enabled));
            }

            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            rebuild_app_menu(&handle);

            // Shortcuts and icon first; the window (if any) after (APP-003, APP-051).
            global::apply(&handle);
            tray::create(&handle)?;
            global::startup_notice(&handle);
            // Time from process start to "icon and shortcuts work" (APP-003, APP-051).
            tracing::info!(
                elapsed_ms = started.elapsed().as_millis() as u64,
                "ready: icon and shortcuts active"
            );
            if !background {
                windows::show_main(&handle)?;
            }
            // Selection windows wait hidden so the crosshair appears fast (APP-048).
            overlay::prewarm(&handle, xs_capture::monitors().map(|m| m.len()).unwrap_or(1));

            // Verbose logging expires after 24 h even if the app never restarts (APP-046).
            let ticker = core.clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(Duration::from_secs(600));
                ticker.expire_verbose();
            });
            Ok(())
        })
        .on_menu_event(|app, event| {
            let state = app.state::<AppState>();
            let action = state.actions.lock().unwrap().get(event.id().as_ref()).cloned();
            if let Some((command, params)) = action {
                let core = state.core.clone();
                std::thread::spawn(move || {
                    if let Err(e) = core.call(Source::Ui, &command, params, None) {
                        tracing::warn!(command, code = e.code.as_str(), "menu command failed");
                    }
                });
            }
        })
        .on_window_event(|window, event| {
            let app = window.app_handle();
            match event {
                WindowEvent::CloseRequested { api, .. } => {
                    let keep_loaded = app
                        .state::<AppState>()
                        .core
                        .settings
                        .get_bool("advanced.keepEditorLoaded");
                    if window.label() == windows::MAIN && keep_loaded {
                        api.prevent_close();
                        let _ = window.hide();
                    }
                    windows::after_window_gone(app, window.label());
                }
                WindowEvent::Focused(true) => {
                    // Permissions may have changed in System Settings (APP-024).
                    if cfg!(target_os = "macos") {
                        let _ = tauri::Emitter::emit(app, "xs://permissions", ());
                        tray::refresh(app);
                    }
                }
                _ => {}
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building XcreenShot");

    app.run(|app, event| match event {
        // Closing the last window must not end the process (APP-004); quitting is explicit.
        RunEvent::ExitRequested { code: None, api, .. } => api.prevent_exit(),
        RunEvent::Exit => tracing::info!("XcreenShot exiting"),
        #[cfg(target_os = "macos")]
        RunEvent::Reopen { .. } => {
            let _ = windows::show_main(app);
        }
        _ => {
            let _ = app;
        }
    });
}
