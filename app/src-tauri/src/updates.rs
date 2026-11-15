//! Updates (APP-036): the core says where the manifest of the channel is; here the
//! updater checks it, verifies the package's signature with the public key built into
//! the app, and installs it now or when the app closes. A package whose signature does
//! not match is never installed.

use std::sync::Mutex;
use std::time::Duration;

use serde_json::{json, Value};
use tauri::{AppHandle, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};

/// The version found by the last check, and its package once downloaded.
#[derive(Default)]
pub struct UpdateState {
    update: Mutex<Option<Update>>,
    bytes: Mutex<Option<Vec<u8>>>,
}

pub fn check(app: &AppHandle, endpoint: &str) -> Result<Value, String> {
    let url: tauri::Url = endpoint.parse().map_err(|e| format!("{e}"))?;
    let updater = app
        .updater_builder()
        .endpoints(vec![url])
        .map_err(|e| e.to_string())?
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;
    let found = tauri::async_runtime::block_on(updater.check()).map_err(|e| e.to_string())?;
    let current = app.package_info().version.to_string();
    let state = app.state::<UpdateState>();
    Ok(match found {
        Some(u) => {
            let v = json!({
                "available": true,
                "current": current,
                "version": u.version,
                "notes": u.body,
                "date": u.date.map(|d| d.to_string()),
            });
            *state.update.lock().unwrap() = Some(u);
            v
        }
        None => {
            json!({ "available": false, "current": current, "version": null, "notes": null, "date": null })
        }
    })
}

/// `now`: downloads, verifies and runs the installer (the app closes and comes back).
/// `quit`: downloads and verifies now; installs when the app closes.
pub fn install(app: &AppHandle, when: &str) -> Result<Value, String> {
    let state = app.state::<UpdateState>();
    let Some(update) = state.update.lock().unwrap().clone() else {
        return Ok(json!({ "status": "none" }));
    };
    if when == "quit" {
        let bytes =
            tauri::async_runtime::block_on(update.download(|_, _| {}, || {})).map_err(|e| e.to_string())?;
        *state.bytes.lock().unwrap() = Some(bytes);
        return Ok(json!({ "status": "at_quit" }));
    }
    tracing::info!(version = %update.version, "installing update");
    tauri::async_runtime::block_on(update.download_and_install(|_, _| {}, || {}))
        .map_err(|e| e.to_string())?;
    app.restart();
}

/// At quit, the update downloaded for then (APP-036).
pub fn install_pending(app: &AppHandle) {
    let state = app.state::<UpdateState>();
    let (Some(update), Some(bytes)) = (
        state.update.lock().unwrap().clone(),
        state.bytes.lock().unwrap().take(),
    ) else {
        return;
    };
    tracing::info!(version = %update.version, "installing update at quit");
    if let Err(e) = update.install(bytes) {
        tracing::warn!(error = %e, "update at quit failed");
    }
}

/// The automatic check (APP-036): a while after start, then daily or weekly, only while
/// it is on and not turned off by policy; with automatic install, the package waits for
/// the app to close.
pub fn watch(app: AppHandle, core: std::sync::Arc<xs_core::Core>) {
    std::thread::Builder::new()
        .name("xs-updates".into())
        .spawn(move || {
            std::thread::sleep(Duration::from_secs(60));
            loop {
                if due(&core) {
                    if let Ok(r) = core.call(xs_core::Source::Scheduled, "app.check_update", json!({}), None)
                    {
                        if r["available"] == true {
                            let lang = core.lang();
                            let version = r["version"].as_str().unwrap_or_default();
                            core.host().notify(
                                "updates",
                                &xs_core::i18n::tr(lang, "notify.update_title", &[("version", version)]),
                                &xs_core::i18n::tr(lang, "notify.update", &[]),
                            );
                            core.host().editor_event("update", r.clone());
                            if core.settings.get_bool("updates.autoInstall") {
                                let _ = install(&app, "quit");
                            }
                        }
                    }
                }
                std::thread::sleep(Duration::from_secs(3600));
            }
        })
        .ok();
}

fn due(core: &xs_core::Core) -> bool {
    if !core.settings.get_bool("updates.autoCheck") || core.policy("disableUpdates") {
        return false;
    }
    // Never from a development or isolated copy.
    if cfg!(debug_assertions) || xs_core::paths::isolated() {
        return false;
    }
    let every = if core.settings.get_str("updates.frequency") == "weekly" {
        chrono::Duration::days(7)
    } else {
        chrono::Duration::days(1)
    };
    let last = core.recall("updates.lastCheck").and_then(|v| {
        v.as_str()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
    });
    last.is_none_or(|t| chrono::Utc::now() - t.with_timezone(&chrono::Utc) >= every)
}
