//! Registration of global shortcuts with the OS (APP-017, APP-019, APP-020).

use std::str::FromStr;

use serde_json::json;
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutEvent, ShortcutState};
use xs_core::platform::Platform;
use xs_core::shortcuts::{system_conflict, Status, SystemConflict};
use xs_core::Source;

use crate::AppState;

fn print_screen_to_system() -> bool {
    #[cfg(windows)]
    {
        xs_core::winreg::print_screen_opens_system_capture()
    }
    #[cfg(not(windows))]
    {
        false
    }
}

/// Unregisters everything and registers the current global bindings, recording a
/// status per shortcut. Nothing is registered while paused (APP-002, APP-018).
pub fn apply(app: &AppHandle) {
    let state = app.state::<AppState>();
    let core = &state.core;
    // Holding the map lock serializes concurrent re-registrations.
    let mut map = state.shortcut_map.lock().unwrap();
    let gs = app.global_shortcut();
    let _ = gs.unregister_all();
    map.clear();
    if core.shortcuts.paused() {
        tracing::info!("global shortcuts paused");
        return;
    }
    let profile = core.profile();
    let platform = Platform::current();
    let print_screen = print_screen_to_system();
    let take_over = core.settings.get_bool("shortcuts.takeOver");
    for (slot, acc) in core.shortcuts.global_bindings(profile) {
        let conflict = system_conflict(&acc, platform, print_screen);
        if conflict == Some(SystemConflict::Reserved) || (conflict.is_some() && !take_over) {
            tracing::warn!(command = slot.command, shortcut = %acc, "shortcut assigned by the system");
            core.shortcuts.set_status(
                &slot,
                Status::System(conflict.unwrap_or(SystemConflict::Reserved)),
            );
            continue;
        }
        let status = match Shortcut::from_str(&acc.to_string()) {
            Ok(shortcut) => match gs.register(shortcut) {
                Ok(()) => {
                    map.insert(shortcut.id(), slot.command.to_string());
                    if conflict.is_some() {
                        Status::TakenOver
                    } else {
                        Status::Active
                    }
                }
                Err(e) => {
                    tracing::warn!(command = slot.command, shortcut = %acc, error = %e, "could not register global shortcut");
                    match conflict {
                        Some(c) => Status::System(c),
                        None => Status::OtherApp,
                    }
                }
            },
            Err(e) => {
                tracing::error!(shortcut = %acc, error = %e, "shortcut not understood by the OS layer");
                Status::OtherApp
            }
        };
        core.shortcuts.set_status(&slot, status);
    }
}

/// Plugin handler: a registered global shortcut was pressed.
pub fn on_shortcut(app: &AppHandle, shortcut: &Shortcut, event: ShortcutEvent) {
    if event.state() != ShortcutState::Pressed {
        return;
    }
    let state = app.state::<AppState>();
    let command = state.shortcut_map.lock().unwrap().get(&shortcut.id()).cloned();
    if let Some(command) = command {
        tracing::info!(command, "global shortcut pressed");
        let core = state.core.clone();
        // Off the event loop: commands may create windows.
        std::thread::spawn(move || {
            if let Err(e) = core.call(Source::Shortcut, &command, json!({}), None) {
                tracing::warn!(command, code = e.code.as_str(), "shortcut command failed");
            }
        });
    }
}

/// One notification at startup listing every global shortcut that failed (APP-019).
pub fn startup_notice(app: &AppHandle) {
    let core = &app.state::<AppState>().core;
    let failed = core.shortcuts.failed(core.profile());
    if failed.is_empty() {
        return;
    }
    let lang = core.lang();
    let registry = core.registry();
    let list = failed
        .iter()
        .map(|s| {
            let title = registry
                .spec(s.command)
                .map(|c| c.title(lang))
                .unwrap_or_default();
            let shortcut = core
                .shortcuts
                .binding(s, core.profile())
                .map(|a| a.display(Platform::current()))
                .unwrap_or_default();
            format!("{title} ({shortcut})")
        })
        .collect::<Vec<_>>()
        .join(", ");
    crate::host::notify(
        app,
        &xs_core::i18n::tr(lang, "notify.shortcuts_failed_title", &[]),
        &xs_core::i18n::tr(lang, "notify.shortcuts_failed_body", &[("list", &list)]),
    );
}
