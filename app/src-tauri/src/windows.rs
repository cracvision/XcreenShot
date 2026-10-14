//! Windows are created on demand and can close without ending the process (APP-001, APP-004).

use serde_json::json;
use tauri::{AppHandle, Emitter, Manager, Theme, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

use crate::AppState;

pub const MAIN: &str = "main";
pub const PREFERENCES: &str = "preferences";

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
        .inner_size(960.0, 640.0)
        .min_inner_size(640.0, 420.0)
        .theme(theme(app));
    #[cfg(windows)]
    {
        let state = app.state::<AppState>();
        let mut actions = state.actions.lock().unwrap();
        let menu = crate::tray::build_menu(
            app,
            &state.core,
            &xs_core::menus::app_menu(xs_core::platform::Platform::Windows),
            false,
            &mut actions,
        )?;
        builder = builder.menu(menu);
    }
    let w = builder.build()?;
    focus(&w);
    Ok(())
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

/// "Hide XcreenShot when capturing" (CAP-023): while a capture is taken the OS leaves
/// our windows out of it (Windows display affinity, macOS sharing type), without
/// hiding them on screen. Only during the capture, so screen sharing in meeting apps
/// still shows XcreenShot.
pub fn exclude_from_capture(app: &AppHandle, exclude: bool) {
    for w in app.webview_windows().values() {
        // Selection windows are always excluded.
        if w.label().starts_with(crate::overlay::LABEL_PREFIX) {
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
        app.exit(0);
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
