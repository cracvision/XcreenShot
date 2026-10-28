//! Windows are created on demand and can close without ending the process (APP-001, APP-004).

use std::sync::atomic::Ordering;

use serde_json::json;
use tauri::{
    AppHandle, Emitter, LogicalSize, Manager, Theme, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};

use crate::AppState;

pub const MAIN: &str = "main";
pub const PREFERENCES: &str = "preferences";
pub const CAPTURE_PANEL: &str = "capture-panel";
/// Logical width of the capture panel; its height follows the content.
const PANEL_WIDTH: f64 = 400.0;

pub fn theme(app: &AppHandle) -> Option<Theme> {
    match app
        .state::<AppState>()
        .core
        .settings
        .get_str("general.theme")
        .as_str()
    {
        "light" => Some(Theme::Light),
        "dark" => Some(Theme::Dark),
        _ => None,
    }
}

fn focus(w: &WebviewWindow) {
    let _ = w.show();
    let _ = w.unminimize();
    let _ = w.set_focus();
}

/// On macOS the Dock icon only shows while a main window is open (APP-001).
fn dock(app: &AppHandle, visible: bool) {
    #[cfg(target_os = "macos")]
    {
        let policy = if visible {
            tauri::ActivationPolicy::Regular
        } else {
            tauri::ActivationPolicy::Accessory
        };
        let _ = app.set_activation_policy(policy);
    }
    #[cfg(not(target_os = "macos"))]
    let _ = (app, visible);
}

pub fn show_main(app: &AppHandle) -> tauri::Result<()> {
    dock(app, true);
    if let Some(w) = app.get_webview_window(MAIN) {
        focus(&w);
        return Ok(());
    }
    let title = xs_core::i18n::tr(app.state::<AppState>().core.lang(), "app.name", &[]);
    #[allow(unused_mut)]
    let mut builder = WebviewWindowBuilder::new(app, MAIN, WebviewUrl::App("index.html".into()))
        .title(title)
        .inner_size(1280.0, 800.0)
        // EDT-001: the editor stays usable down to 900 × 600.
        .min_inner_size(900.0, 600.0)
        .theme(theme(app));
    #[cfg(windows)]
    {
        let state = app.state::<AppState>();
        let mut actions = state.actions.lock().unwrap();
        let menu = crate::tray::build_menu(
            app,
            &state.core,
            &xs_core::menus::app_menu(xs_core::platform::Platform::Windows),
            crate::tray::Shortcuts::Text,
            &mut actions,
        )?;
        *state.window_menu.lock().unwrap() = Some(menu.clone());
        builder = builder.menu(menu);
    }
    let w = builder.build()?;
    focus(&w);
    Ok(())
}

/// Title of the main window: the document shown and its unsaved state (EDT-012).
pub fn set_main_title(app: &AppHandle, title: &str) {
    if let Some(w) = app.get_webview_window(MAIN) {
        let _ = w.set_title(title);
    }
}

pub fn open_preferences(app: &AppHandle, section: Option<&str>, key: Option<&str>) -> tauri::Result<()> {
    let nav = json!({ "section": section, "key": key });
    if let Some(w) = app.get_webview_window(PREFERENCES) {
        focus(&w);
        let _ = w.emit("xs://navigate", nav);
        return Ok(());
    }
    *app.state::<AppState>().pending_navigation.lock().unwrap() = Some(nav);
    let title = xs_core::i18n::tr(app.state::<AppState>().core.lang(), "ui.prefs.title", &[]);
    let w = WebviewWindowBuilder::new(app, PREFERENCES, WebviewUrl::App("index.html".into()))
        .title(title)
        .inner_size(820.0, 600.0)
        .min_inner_size(640.0, 420.0)
        .theme(theme(app))
        .build()?;
    focus(&w);
    Ok(())
}

/// Capture panel (CAP-001): a normal window with the native title bar that remembers
/// its position. `None` toggles it (its shortcut, CAP-065): it hides when it is the
/// window in front, and otherwise comes to the front.
pub fn capture_panel(app: &AppHandle, show: Option<bool>) -> tauri::Result<bool> {
    let existing = app.get_webview_window(CAPTURE_PANEL);
    let show = show.unwrap_or_else(|| {
        !existing.as_ref().is_some_and(|w| {
            w.is_visible().unwrap_or(false)
                && !w.is_minimized().unwrap_or(false)
                && w.is_focused().unwrap_or(false)
        })
    });
    match (existing, show) {
        (Some(w), true) => focus(&w),
        (Some(w), false) => w.hide()?,
        (None, true) => {
            let title = xs_core::i18n::tr(app.state::<AppState>().core.lang(), "ui.panel.title", &[]);
            let w = WebviewWindowBuilder::new(app, CAPTURE_PANEL, WebviewUrl::App("index.html".into()))
                .title(title)
                .inner_size(PANEL_WIDTH, 460.0)
                .resizable(false)
                .maximizable(false)
                .theme(theme(app))
                .build()?;
            focus(&w);
        }
        (None, false) => {}
    }
    Ok(show)
}

/// The panel fits its content (compact or expanded view, CAP-001).
pub fn fit_capture_panel(app: &AppHandle, height: f64) {
    if let Some(w) = app.get_webview_window(CAPTURE_PANEL) {
        let _ = w.set_size(LogicalSize::new(PANEL_WIDTH, height.clamp(120.0, 900.0)));
    }
}

/// While a capture runs the panel steps aside, and comes back afterwards in the state
/// it had (CAP-001). Runs on the UI thread and waits, so it is gone before the screen
/// is frozen.
pub fn panel_capture_session(app: &AppHandle, active: bool) {
    let (tx, rx) = std::sync::mpsc::channel();
    let handle = app.clone();
    let queued = app.run_on_main_thread(move || {
        let state = handle.state::<AppState>();
        if active {
            let shown = handle
                .get_webview_window(CAPTURE_PANEL)
                .filter(|w| w.is_visible().unwrap_or(false) && !w.is_minimized().unwrap_or(false));
            if let Some(w) = shown {
                let _ = w.hide();
                state.panel_stepped_aside.store(true, Ordering::SeqCst);
            }
        } else {
            restore_capture_panel(&handle);
        }
        let _ = tx.send(());
    });
    if queued.is_ok() {
        let _ = rx.recv_timeout(std::time::Duration::from_millis(500));
    }
}

/// Brings back the panel if a capture had put it aside (UI thread).
pub fn restore_capture_panel(app: &AppHandle) {
    let state = app.state::<AppState>();
    if state.panel_stepped_aside.swap(false, Ordering::SeqCst) {
        if let Some(w) = app.get_webview_window(CAPTURE_PANEL) {
            let _ = w.show();
        }
    }
}

/// "Hide XcreenShot when capturing" (CAP-023): while a capture is taken the OS leaves
/// our windows out of it (Windows display affinity, macOS sharing type), without
/// hiding them on screen. Only during the capture, so screen sharing in meeting apps
/// still shows XcreenShot.
pub fn exclude_from_capture(app: &AppHandle, exclude: bool) {
    for w in app.webview_windows().values() {
        // Selection windows and the countdown are always excluded.
        if w.label().starts_with(crate::overlay::LABEL_PREFIX) || w.label() == crate::countdown::LABEL {
            continue;
        }
        let _ = w.set_content_protected(exclude);
    }
}

pub fn apply_theme(app: &AppHandle) {
    let t = theme(app);
    for w in app.webview_windows().values() {
        let _ = w.set_theme(t);
    }
}

/// Retitles open windows after a language change.
pub fn retitle(app: &AppHandle) {
    let lang = app.state::<AppState>().core.lang();
    if let Some(w) = app.get_webview_window(MAIN) {
        let _ = w.set_title(&xs_core::i18n::tr(lang, "app.name", &[]));
    }
    if let Some(w) = app.get_webview_window(PREFERENCES) {
        let _ = w.set_title(&xs_core::i18n::tr(lang, "ui.prefs.title", &[]));
    }
    if let Some(w) = app.get_webview_window(CAPTURE_PANEL) {
        let _ = w.set_title(&xs_core::i18n::tr(lang, "ui.panel.title", &[]));
    }
}

/// Called after a window closed or hid. Hides the Dock icon when nothing is visible
/// and quits if "keep running in the background" is off (APP-004).
pub fn after_window_gone(app: &AppHandle, closing_label: &str) {
    let visible = app
        .webview_windows()
        .values()
        .any(|w| w.label() != closing_label && w.is_visible().unwrap_or(false));
    let main_visible = app
        .get_webview_window(MAIN)
        .is_some_and(|w| w.label() != closing_label && w.is_visible().unwrap_or(false));
    dock(app, main_visible);
    if !visible
        && !app
            .state::<AppState>()
            .core
            .settings
            .get_bool("general.keepRunning")
    {
        tracing::info!("last window closed and keepRunning is off: quitting");
        // Through the command, so unsaved documents are asked about first (APP-004).
        let core = app.state::<AppState>().core.clone();
        std::thread::spawn(move || {
            let _ = core.call(xs_core::Source::Ui, "app.quit", json!({}), None);
        });
    }
}

pub fn close_focused(app: &AppHandle) {
    if let Some(w) = app
        .webview_windows()
        .values()
        .find(|w| w.is_focused().unwrap_or(false))
    {
        let _ = w.close();
    }
}
