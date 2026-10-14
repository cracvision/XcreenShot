//! The app core: settings, shortcuts and the command registry with its handlers.
//! The shell creates one [`Core`], installs its [`Host`], and routes every action
//! (UI, shortcuts, tray, CLI) through [`Core::call`].

use std::sync::{Arc, RwLock, Weak};

use chrono::{DateTime, Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::catalog;
use crate::command::{CommandSpec, Context, Registry, Source};
use crate::error::{CommandError, CommandResult, ErrorCode};
use crate::host::{Host, NullHost, Permission};
use crate::i18n::{tr, Lang};
use crate::logging::{self, LogControl};
use crate::paths::{self, Paths};
use crate::platform::{self, Platform};
use crate::settings::SettingsStore;
use crate::shortcuts::{Accelerator, Profile, ShortcutScope, ShortcutStore, Slot, Status};

/// Small internal state that is not a preference.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppState {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    verbose_since: Option<DateTime<Utc>>,
}

pub struct Core {
    pub paths: Paths,
    pub settings: SettingsStore,
    pub shortcuts: ShortcutStore,
    host: RwLock<Arc<dyn Host>>,
    log: Option<LogControl>,
    state: RwLock<AppState>,
    registry: RwLock<Option<Arc<Registry>>>,
}

/// Verbose logging switches itself off after this long (APP-046).
const VERBOSE_FOR_HOURS: i64 = 24;

impl Core {
    pub fn new(paths: Paths, log: Option<LogControl>) -> Arc<Core> {
        Core::with_extensions(paths, log, Vec::new())
    }

    /// Like [`Core::new`], plus handlers contributed by other crates (capture, library…)
    /// for commands declared in the catalog. Commands are still declared once, here.
    pub fn with_extensions(paths: Paths, log: Option<LogControl>, extensions: Vec<Extension>) -> Arc<Core> {
        let state = std::fs::read(paths.state_file())
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        let core = Arc::new(Core {
            settings: SettingsStore::load(paths.settings_file()),
            shortcuts: ShortcutStore::load(paths.shortcuts_file()),
            paths,
            host: RwLock::new(Arc::new(NullHost)),
            log,
            state: RwLock::new(state),
            registry: RwLock::new(None),
        });
        let registry = build_registry(&core);
        let registry = apply_extensions(registry, &core, extensions);
        *core.registry.write().unwrap() = Some(Arc::new(registry));
        core.expire_verbose();
        if let Some(log) = &core.log {
            log.set_verbose(core.settings.get_bool("diagnostics.verbose"));
        }
        core
    }

    /// In-memory core for tests.
    pub fn for_tests(platform: Platform) -> Arc<Core> {
        Core::for_tests_with(platform, Vec::new())
    }

    /// In-memory core for tests, with extensions.
    pub fn for_tests_with(platform: Platform, extensions: Vec<Extension>) -> Arc<Core> {
        let core = Arc::new(Core {
            settings: SettingsStore::in_memory(),
            shortcuts: ShortcutStore::in_memory(platform),
            paths: Paths::rooted(&std::env::temp_dir().join(format!("xs-test-{}", std::process::id()))),
            host: RwLock::new(Arc::new(NullHost)),
            log: None,
            state: RwLock::new(AppState::default()),
            registry: RwLock::new(None),
        });
        let registry = build_registry(&core);
        let registry = apply_extensions(registry, &core, extensions);
        *core.registry.write().unwrap() = Some(Arc::new(registry));
        core
    }

    pub fn set_host(&self, host: Arc<dyn Host>) {
        *self.host.write().unwrap() = host;
    }

    pub fn host(&self) -> Arc<dyn Host> {
        self.host.read().unwrap().clone()
    }

    pub fn registry(&self) -> Arc<Registry> {
        self.registry.read().unwrap().clone().expect("registry built")
    }

    pub fn lang(&self) -> Lang {
        self.settings.lang()
    }

    pub fn profile(&self) -> Profile {
        Profile::parse(&self.settings.get_str("shortcuts.profile")).unwrap_or(Profile::Coexistence)
    }

    /// Runs a command. `lang` overrides the UI language for messages (CLI `--lang`).
    pub fn call(&self, source: Source, id: &str, params: Value, lang: Option<Lang>) -> CommandResult {
        let ctx = Context {
            source,
            lang: lang.unwrap_or_else(|| self.lang()),
        };
        self.registry().call(&ctx, id, params)
    }

    fn save_state(&self) {
        let state = self.state.read().unwrap().clone();
        if let Ok(bytes) = serde_json::to_vec_pretty(&state) {
            let _ = paths::write_atomic(&self.paths.state_file(), &bytes);
        }
    }

    /// Turns verbose logging off once it has been on for 24 hours. The shell calls
    /// this at startup and periodically.
    pub fn expire_verbose(&self) {
        if !self.settings.get_bool("diagnostics.verbose") {
            return;
        }
        let since = self.state.read().unwrap().verbose_since;
        let expired = match since {
            Some(t) => Utc::now() - t >= ChronoDuration::hours(VERBOSE_FOR_HOURS),
            None => {
                self.state.write().unwrap().verbose_since = Some(Utc::now());
                self.save_state();
                false
            }
        };
        if expired {
            tracing::info!("verbose logging expired after 24 hours");
            self.settings.sync_external("diagnostics.verbose", json!(false));
            self.state.write().unwrap().verbose_since = None;
            self.save_state();
            if let Some(log) = &self.log {
                log.set_verbose(false);
            }
            self.host().settings_changed("diagnostics.verbose", &json!(false));
        }
    }

    #[cfg(test)]
    fn backdate_verbose(&self, hours: i64) {
        self.state.write().unwrap().verbose_since = Some(Utc::now() - ChronoDuration::hours(hours));
    }

    fn on_setting_changed(&self, key: &str, value: &Value) {
        if key == "diagnostics.verbose" {
            let on = value.as_bool().unwrap_or(false);
            self.state.write().unwrap().verbose_since = on.then(Utc::now);
            self.save_state();
            if let Some(log) = &self.log {
                log.set_verbose(on);
            }
        }
        if key == "shortcuts.profile" {
            self.host().shortcuts_changed();
        }
        self.host().settings_changed(key, value);
    }

    /// Rows for Preferences > Shortcuts and `shortcuts.list` (APP-018).
    pub fn shortcut_rows(&self, lang: Lang) -> Value {
        let profile = self.profile();
        let platform = self.shortcuts.platform();
        let registry = self.registry();
        let rows: Vec<Value> = self
            .shortcuts
            .slots()
            .iter()
            .map(|slot| {
                let binding = self.shortcuts.binding(slot, profile);
                let default = crate::shortcuts::default_for(slot, profile, platform);
                let status = self.shortcuts.status(slot, profile);
                let title = registry
                    .spec(slot.command)
                    .map(|s| s.title(lang))
                    .unwrap_or_else(|| slot.command.to_string());
                json!({
                    "command": slot.command,
                    "scope": slot.scope,
                    "group": slot.group,
                    "title": title,
                    "shortcut": binding.as_ref().map(|a| a.to_string()),
                    "display": binding.as_ref().map(|a| a.display(platform)),
                    "default": default.as_ref().map(|a| a.display(platform)),
                    "custom": self.shortcuts.is_custom(slot),
                    "status": status,
                    "conflict": status.is_conflict(),
                    "statusText": status_text(&status, lang),
                    "canTakeOver": matches!(status, Status::OtherApp | Status::System(crate::shortcuts::SystemConflict::SystemAssigned { .. })),
                })
            })
            .collect();
        json!({
            "profile": profile,
            "paused": self.shortcuts.user_paused(),
            "platform": platform,
            "takeOver": self.settings.get_bool("shortcuts.takeOver"),
            "rows": rows,
        })
    }

    /// Text for "Copy diagnostic information" (APP-046). No library paths, no user data.
    pub fn diagnostics(&self) -> Value {
        let host = self.host();
        let profile = self.profile();
        let failed: Vec<Value> = self
            .shortcuts
            .failed(profile)
            .iter()
            .map(|s| {
                json!({
                    "command": s.command,
                    "shortcut": self.shortcuts.binding(s, profile).map(|a| a.to_string()),
                    "status": self.shortcuts.status(s, profile),
                })
            })
            .collect();
        let permissions = host.permissions();
        let monitors = host.monitors();
        let log_tail = logging::tail(&self.paths.logs_dir(), 200);
        let mut text = String::new();
        text.push_str(&format!("XcreenShot {}\n", platform::VERSION));
        text.push_str(&format!(
            "Build: {} · Channel: dev · Arch: {}\n",
            if cfg!(debug_assertions) {
                "debug"
            } else {
                "release"
            },
            platform::arch()
        ));
        text.push_str(&format!(
            "OS: {} {} · UI language: {}\n",
            Platform::current().as_str(),
            os_version(),
            self.lang().code()
        ));
        text.push_str(&format!("Monitors: {}\n", monitors.len()));
        for m in &monitors {
            text.push_str(&format!("  - {m}\n"));
        }
        text.push_str("Permissions:\n");
        if permissions.is_empty() {
            text.push_str("  (none required on this platform)\n");
        }
        for p in &permissions {
            text.push_str(&format!("  - {}: {:?}\n", p.permission.as_str(), p.state));
        }
        text.push_str(&format!("Shortcut profile: {profile:?}\n"));
        text.push_str(&format!("Shortcuts in conflict: {}\n", failed.len()));
        for f in &failed {
            text.push_str(&format!("  - {f}\n"));
        }
        text.push_str("Managed configuration: inactive\n");
        text.push_str("Experimental features: none\n");
        text.push_str(&format!("--- Last {} log lines ---\n", log_tail.len()));
        for line in &log_tail {
            text.push_str(line);
            text.push('\n');
        }
        json!({
            "text": text,
            "version": platform::VERSION,
            "platform": Platform::current(),
            "arch": platform::arch(),
            "os": os_version(),
            "language": self.lang().code(),
            "monitors": monitors,
            "permissions": permissions,
            "shortcutConflicts": failed,
            "logLines": log_tail.len(),
        })
    }
}

fn os_version() -> String {
    #[cfg(windows)]
    {
        windows_build().map(|b| format!("10.0.{b}")).unwrap_or_default()
    }
    #[cfg(not(windows))]
    {
        std::process::Command::new("sw_vers")
            .arg("-productVersion")
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default()
    }
}

/// Windows build number from the registry (e.g. 26200).
#[cfg(windows)]
pub fn windows_build() -> Option<u32> {
    crate::winreg::read_string(
        crate::winreg::HKLM,
        r"SOFTWARE\Microsoft\Windows NT\CurrentVersion",
        "CurrentBuildNumber",
    )
    .and_then(|s| s.parse().ok())
}

fn status_text(status: &Status, lang: Lang) -> String {
    match status {
        Status::Active => tr(lang, "shortcuts.status.active", &[]),
        Status::Unassigned => tr(lang, "shortcuts.status.unassigned", &[]),
        Status::Paused => tr(lang, "shortcuts.status.paused", &[]),
        Status::Duplicate { command } => tr(lang, "shortcuts.status.duplicate", &[("command", command)]),
        Status::System(crate::shortcuts::SystemConflict::Reserved) => {
            tr(lang, "shortcuts.status.reserved", &[])
        }
        Status::System(crate::shortcuts::SystemConflict::SystemAssigned { .. }) => {
            tr(lang, "shortcuts.status.system_assigned", &[])
        }
        Status::OtherApp => tr(lang, "shortcuts.status.other_app", &[]),
        Status::TakenOver => tr(lang, "shortcuts.status.taken_over", &[]),
    }
}

fn str_param<'a>(p: &'a Value, name: &str) -> Option<&'a str> {
    p.get(name).and_then(Value::as_str)
}

fn resolve_slot(core: &Core, command: &str, scope: Option<&str>, lang: Lang) -> Result<Slot, CommandError> {
    let scope = match scope {
        Some("app") => Some(ShortcutScope::App),
        Some("global") => Some(ShortcutScope::Global),
        _ => None,
    };
    let found = match scope {
        Some(s) => core.shortcuts.find_slot(command, s),
        None => core
            .shortcuts
            .find_slot(command, ShortcutScope::Global)
            .or_else(|| core.shortcuts.find_slot(command, ShortcutScope::App)),
    };
    found.ok_or_else(|| {
        CommandError::new(
            ErrorCode::NotFound,
            tr(lang, "errors.no_shortcut_slot", &[("command", command)]),
        )
    })
}

/// Adds handlers from another crate. Receives a weak handle so handlers do not keep
/// the core alive.
pub type Extension = Box<dyn FnOnce(&mut Registry, Weak<Core>) + Send>;

fn apply_extensions(mut registry: Registry, core: &Arc<Core>, extensions: Vec<Extension>) -> Registry {
    for extension in extensions {
        extension(&mut registry, Arc::downgrade(core));
    }
    registry
}

fn build_registry(core: &Arc<Core>) -> Registry {
    let mut reg = Registry::new(catalog::specs());
    let weak = Arc::downgrade(core);
    // Each handler holds a weak reference so the registry does not keep the core alive.
    macro_rules! on {
        ($id:literal, |$core:ident, $ctx:ident, $p:ident| $body:expr) => {{
            let weak = weak.clone();
            reg.handle($id, move |$ctx: &Context, $p: Value| {
                let $core = weak
                    .upgrade()
                    .ok_or_else(|| CommandError::internal("shutting down"))?;
                #[allow(unused_variables)]
                let $ctx = $ctx;
                $body
            });
        }};
    }

    on!("app.ping", |core, ctx, _p| Ok(json!({
        "version": platform::VERSION,
        "platform": Platform::current(),
        "arch": platform::arch(),
        "state": "idle",
        "automationEnabled": core.settings.get_bool("automation.enabled"),
        "language": ctx.lang.code(),
        "systemLanguage": Lang::system().code(),
        "source": ctx.source,
    })));
    on!("app.commands", |core, ctx, _p| Ok(Value::Array(
        core.registry()
            .specs()
            .iter()
            .map(|s| s.describe(ctx.lang))
            .collect()
    )));
    on!("app.show", |core, ctx, _p| core
        .host()
        .show_main_window()
        .map(|_| json!({})));
    on!("app.close_window", |core, ctx, _p| core
        .host()
        .close_focused_window()
        .map(|_| json!({})));
    on!("app.quit", |core, ctx, _p| {
        core.host().quit();
        Ok(json!({}))
    });
    on!("app.diagnostics", |core, ctx, p| {
        let info = core.diagnostics();
        if p.get("copy").and_then(Value::as_bool).unwrap_or(false) {
            core.host().copy_text(info["text"].as_str().unwrap_or_default())?;
        }
        Ok(info)
    });
    on!("app.open_logs", |core, ctx, _p| {
        let dir = core.paths.logs_dir();
        std::fs::create_dir_all(&dir).map_err(|e| CommandError::internal(e.to_string()))?;
        core.host().open_path(&dir)?;
        Ok(json!({ "path": dir }))
    });
    on!("app.permissions", |core, ctx, _p| Ok(serde_json::to_value(
        core.host().permissions()
    )
    .unwrap_or_default()));
    on!("app.request_permission", |core, ctx, p| {
        let perm = Permission::parse(str_param(&p, "permission").unwrap_or_default())
            .ok_or_else(|| CommandError::new(ErrorCode::InvalidParams, "permission"))?;
        core.host().request_permission(perm)?;
        Ok(json!({}))
    });
    on!("app.open_permission_settings", |core, ctx, p| {
        let perm = Permission::parse(str_param(&p, "permission").unwrap_or_default())
            .ok_or_else(|| CommandError::new(ErrorCode::InvalidParams, "permission"))?;
        if Platform::current() != Platform::Macos {
            return Err(CommandError::new(
                ErrorCode::NotSupportedOnPlatform,
                tr(ctx.lang, "errors.macos_only", &[]),
            ));
        }
        core.host().open_uri(perm.macos_settings_uri())?;
        Ok(json!({}))
    });
    on!("settings.open", |core, ctx, p| {
        core.host()
            .open_settings(str_param(&p, "section"), str_param(&p, "key"))?;
        Ok(json!({}))
    });
    on!("settings.get", |core, ctx, p| {
        let key = str_param(&p, "key").unwrap_or_default();
        let value = core.settings.get(key).ok_or_else(|| {
            CommandError::new(
                ErrorCode::NotFound,
                tr(ctx.lang, "errors.unknown_setting", &[("key", key)]),
            )
        })?;
        Ok(json!({ "key": key, "value": value }))
    });
    on!("settings.set", |core, ctx, p| {
        let key = str_param(&p, "key").unwrap_or_default().to_string();
        let value = p.get("value").cloned().unwrap_or(Value::Null);
        let changed = core.settings.set(&key, value.clone(), ctx.source, ctx.lang)?;
        if changed {
            tracing::info!(key, source = ?ctx.source, "setting changed");
            core.on_setting_changed(&key, &value);
        }
        Ok(json!({ "key": key, "value": value, "changed": changed }))
    });
    on!("settings.list", |core, ctx, _p| {
        if let Some(v) = core.host().launch_at_login() {
            if core.settings.sync_external("general.launchAtLogin", json!(v)) {
                core.host().settings_changed("general.launchAtLogin", &json!(v));
            }
        }
        Ok(Value::Array(core.settings.list(ctx.lang)))
    });
    on!("settings.schema", |core, ctx, _p| Ok(serde_json::to_value(
        core.settings.defs()
    )
    .unwrap_or_default()));
    on!("shortcuts.list", |core, ctx, _p| Ok(core.shortcut_rows(ctx.lang)));
    on!("shortcuts.set", |core, ctx, p| {
        let slot = resolve_slot(
            &core,
            str_param(&p, "command").unwrap_or_default(),
            str_param(&p, "scope"),
            ctx.lang,
        )?;
        let acc = match p.get("shortcut") {
            Some(Value::String(s)) if !s.is_empty() => Some(s.parse::<Accelerator>().map_err(|e| {
                CommandError::new(
                    ErrorCode::InvalidParams,
                    tr(ctx.lang, "shortcuts.reject.unparsable", &[("detail", &e)]),
                )
            })?),
            _ => None,
        };
        let reassign = p.get("reassign").and_then(Value::as_bool).unwrap_or(false);
        let outcome = core
            .shortcuts
            .set(&slot, acc, reassign, core.profile(), ctx.lang)?;
        tracing::info!(slot = slot.key(), source = ?ctx.source, "shortcut changed");
        core.host().shortcuts_changed();
        let row = core.shortcut_rows(ctx.lang)["rows"]
            .as_array()
            .and_then(|rows| {
                rows.iter()
                    .find(|r| r["command"] == slot.command && r["scope"] == json!(slot.scope))
                    .cloned()
            })
            .unwrap_or_default();
        Ok(json!({ "row": row, "reassignedFrom": outcome.reassigned_from }))
    });
    on!("shortcuts.reset", |core, ctx, p| {
        match str_param(&p, "command") {
            Some(command) => {
                let slot = resolve_slot(&core, command, str_param(&p, "scope"), ctx.lang)?;
                core.shortcuts.reset(Some(&slot));
            }
            None => core.shortcuts.reset(None),
        }
        core.host().shortcuts_changed();
        Ok(json!({}))
    });
    on!("shortcuts.pause", |core, ctx, p| {
        let paused = p
            .get("paused")
            .and_then(Value::as_bool)
            .unwrap_or(!core.shortcuts.user_paused());
        core.shortcuts.set_paused(paused);
        tracing::info!(paused, "global shortcuts pause changed");
        core.host().shortcuts_changed();
        Ok(json!({ "paused": paused }))
    });
    on!("shortcuts.set_profile", |core, ctx, p| {
        let to = Profile::parse(str_param(&p, "profile").unwrap_or_default())
            .ok_or_else(|| CommandError::new(ErrorCode::InvalidParams, "profile"))?;
        let reset = p.get("resetCustom").and_then(Value::as_bool).unwrap_or(false);
        let preview = core.shortcuts.preview_profile(core.profile(), to, reset);
        if p.get("dryRun").and_then(Value::as_bool).unwrap_or(false) {
            return Ok(preview);
        }
        if reset {
            core.shortcuts.reset(None);
        }
        let value = serde_json::to_value(to).unwrap_or_default();
        if core
            .settings
            .set("shortcuts.profile", value.clone(), Source::Ui, ctx.lang)?
        {
            core.on_setting_changed("shortcuts.profile", &value);
        } else {
            core.host().shortcuts_changed();
        }
        Ok(preview)
    });
    on!("shortcuts.retry", |core, ctx, _p| {
        core.host().shortcuts_changed();
        Ok(core.shortcut_rows(ctx.lang))
    });
    on!("shortcuts.open_system_settings", |core, ctx, _p| {
        let uri = match Platform::current() {
            Platform::Windows => crate::shortcuts::WINDOWS_KEYBOARD_SETTINGS,
            Platform::Macos => crate::shortcuts::MACOS_KEYBOARD_SETTINGS,
        };
        core.host().open_uri(uri)?;
        Ok(json!({}))
    });
    on!("shortcuts.recording", |core, ctx, p| {
        let active = p.get("active").and_then(Value::as_bool).unwrap_or(false);
        // Idempotent: re-registering on every repeat would churn the OS hotkeys.
        if core.shortcuts.set_recording(active) {
            core.host().shortcuts_changed();
        }
        Ok(json!({ "active": active }))
    });

    // APP-032: master switch for CLI and MCP. `app.ping` still answers.
    let weak_gate = weak.clone();
    reg.add_gate(Box::new(move |ctx: &Context, spec: &CommandSpec| {
        let Some(core) = weak_gate.upgrade() else {
            return Ok(());
        };
        if ctx.source.is_automation()
            && spec.id != "app.ping"
            && !core.settings.get_bool("automation.enabled")
        {
            return Err(CommandError::new(
                ErrorCode::DisabledByPolicy,
                tr(ctx.lang, "errors.automation_disabled", &[]),
            ));
        }
        Ok(())
    }));
    // APP-032 / APP-046: log every automation command with its outcome, never its params.
    let weak_obs = weak;
    reg.add_observer(Box::new(move |ctx: &Context, spec: &CommandSpec, result: &CommandResult| {
        if ctx.source.is_automation() {
            match result {
                Ok(_) => tracing::info!(command = spec.id, source = ?ctx.source, "automation command ok"),
                Err(e) => tracing::warn!(command = spec.id, source = ?ctx.source, code = e.code.as_str(), "automation command failed"),
            }
        }
        if let Some(core) = weak_obs.upgrade() {
            core.host().command_finished(ctx, spec, result);
        }
    }));
    reg
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menus;

    fn call(core: &Core, source: Source, id: &str, p: Value) -> CommandResult {
        core.call(source, id, p, Some(Lang::En))
    }

    #[test]
    fn every_spec_has_a_handler_and_valid_schemas() {
        let core = Core::for_tests(Platform::Windows);
        let reg = core.registry();
        for spec in reg.specs() {
            if !catalog::EXTERNAL_AREAS.contains(&spec.area()) {
                assert!(reg.has_handler(spec.id), "{} has no handler", spec.id);
            }
            assert_eq!(spec.params["type"], "object", "{}", spec.id);
            assert!(spec.result.is_object(), "{}", spec.id);
            assert!(spec.id.contains('.'), "{}", spec.id);
            for p in spec.positional {
                assert!(
                    spec.params["properties"].get(*p).is_some(),
                    "{} positional {p}",
                    spec.id
                );
            }
        }
    }

    /// APP-027: every menu entry is bound to a registered command.
    #[test]
    fn menus_only_reference_registered_commands() {
        let core = Core::for_tests(Platform::Windows);
        let reg = core.registry();
        let mut all = menus::tray_menu();
        all.extend(menus::app_menu(Platform::Windows));
        all.extend(menus::app_menu(Platform::Macos));
        for id in menus::commands_in(&all) {
            assert!(reg.spec(id).is_some(), "menu references unknown command {id}");
        }
        for label in menus::labels_in(&all) {
            assert_ne!(tr(Lang::Es, label, &[]), label, "missing label {label}");
        }
    }

    #[test]
    fn every_command_and_setting_is_translated() {
        let core = Core::for_tests(Platform::Windows);
        for spec in core.registry().specs() {
            for lang in [Lang::En, Lang::Es] {
                let title_key = format!("{}.title", spec.i18n_base());
                assert_ne!(spec.title(lang), title_key, "{title_key}");
                assert_ne!(
                    spec.description(lang),
                    format!("{}.description", spec.i18n_base())
                );
            }
        }
        for row in core.settings.list(Lang::Es) {
            assert!(!row["label"].as_str().unwrap().starts_with("settings."), "{row}");
            assert!(
                !row["description"].as_str().unwrap().starts_with("settings."),
                "{row}"
            );
        }
    }

    /// APP-027 criterion: `app.commands` returns every command with schemas.
    #[test]
    fn app_commands_lists_everything() {
        let core = Core::for_tests(Platform::Windows);
        let list = call(&core, Source::Cli, "app.commands", json!({})).unwrap();
        let ids: Vec<&str> = list
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["id"].as_str().unwrap())
            .collect();
        for spec in core.registry().specs() {
            assert!(ids.contains(&spec.id));
        }
    }

    #[test]
    fn ping_reports_source() {
        let core = Core::for_tests(Platform::Windows);
        for source in [Source::Ui, Source::Shortcut, Source::Cli] {
            let r = call(&core, source, "app.ping", json!({})).unwrap();
            assert_eq!(r["version"], platform::VERSION);
            assert_eq!(r["source"], serde_json::to_value(source).unwrap());
        }
    }

    /// APP-032: with automation off, CLI commands are rejected except `app.ping`.
    #[test]
    fn automation_switch() {
        let core = Core::for_tests(Platform::Windows);
        call(
            &core,
            Source::Ui,
            "settings.set",
            json!({"key": "automation.enabled", "value": false}),
        )
        .unwrap();
        let ping = call(&core, Source::Cli, "app.ping", json!({})).unwrap();
        assert_eq!(ping["automationEnabled"], false);
        let e = call(&core, Source::Cli, "settings.list", json!({})).unwrap_err();
        assert_eq!(e.code, ErrorCode::DisabledByPolicy);
        assert!(e.message.contains("Allow control"), "{}", e.message);
        assert!(call(&core, Source::Ui, "settings.list", json!({})).is_ok());
    }

    /// APP-032 criterion: `settings set automation.enabled false` is refused from the CLI.
    #[test]
    fn cli_cannot_disable_automation() {
        let core = Core::for_tests(Platform::Windows);
        let e = call(
            &core,
            Source::Cli,
            "settings.set",
            json!({"key": "automation.enabled", "value": false}),
        )
        .unwrap_err();
        assert_eq!(e.code.exit_code(), 6);
    }

    #[test]
    fn shortcuts_set_via_command() {
        let core = Core::for_tests(Platform::Windows);
        let e = call(
            &core,
            Source::Cli,
            "shortcuts.set",
            json!({"command": "app.show", "shortcut": "A"}),
        )
        .unwrap_err();
        assert_eq!(e.code, ErrorCode::InvalidParams);
        let r = call(
            &core,
            Source::Cli,
            "shortcuts.set",
            json!({"command": "app.show", "shortcut": "Ctrl+Alt+A"}),
        )
        .unwrap();
        assert_eq!(r["row"]["display"], "Ctrl+Alt+A");
        let e = call(
            &core,
            Source::Cli,
            "shortcuts.set",
            json!({"command": "settings.open", "shortcut": "Ctrl+Alt+A", "scope": "app"}),
        )
        .unwrap_err();
        assert!(e.message.contains("app.show"), "{}", e.message);
    }

    #[test]
    fn profile_switch_changes_quit_on_windows() {
        let core = Core::for_tests(Platform::Windows);
        let preview = call(
            &core,
            Source::Ui,
            "shortcuts.set_profile",
            json!({"profile": "compatibility", "dryRun": true}),
        )
        .unwrap();
        assert_eq!(preview["changes"][0]["command"], "app.quit");
        assert_eq!(core.profile(), Profile::Coexistence);
        call(
            &core,
            Source::Ui,
            "shortcuts.set_profile",
            json!({"profile": "compatibility"}),
        )
        .unwrap();
        assert_eq!(core.profile(), Profile::Compatibility);
    }

    #[test]
    fn verbose_logging_expires_after_a_day() {
        let core = Core::for_tests(Platform::Windows);
        call(
            &core,
            Source::Ui,
            "settings.set",
            json!({"key": "diagnostics.verbose", "value": true}),
        )
        .unwrap();
        core.expire_verbose();
        assert!(core.settings.get_bool("diagnostics.verbose"));
        core.backdate_verbose(25);
        core.expire_verbose();
        assert!(!core.settings.get_bool("diagnostics.verbose"));
    }

    #[test]
    fn diagnostics_has_the_required_blocks() {
        let core = Core::for_tests(Platform::Windows);
        let d = call(&core, Source::Cli, "app.diagnostics", json!({})).unwrap();
        let text = d["text"].as_str().unwrap();
        for needle in [
            "XcreenShot",
            "OS:",
            "Monitors:",
            "Permissions:",
            "Shortcuts in conflict:",
        ] {
            assert!(text.contains(needle), "{needle}");
        }
    }
}
