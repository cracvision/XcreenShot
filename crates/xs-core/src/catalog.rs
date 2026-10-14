//! Static command catalog. The CLI reads it for parsing and help without the app
//! running; [`crate::core::Core`] attaches the handlers.

use serde_json::{json, Value};

use crate::command::{CommandSpec, Scope};

const ALL: &[Scope] = &[Scope::Ui, Scope::Shortcut, Scope::Cli, Scope::Mcp];
const UI_CLI_MCP: &[Scope] = &[Scope::Ui, Scope::Cli, Scope::Mcp];
const UI_SHORTCUT_CLI: &[Scope] = &[Scope::Ui, Scope::Shortcut, Scope::Cli];
const UI_CLI: &[Scope] = &[Scope::Ui, Scope::Cli];
const UI_SHORTCUT: &[Scope] = &[Scope::Ui, Scope::Shortcut];
const UI: &[Scope] = &[Scope::Ui];

fn no_params() -> Value {
    json!({"type": "object", "properties": {}, "additionalProperties": false})
}

fn params(props: Value, required: &[&str]) -> Value {
    json!({"type": "object", "properties": props, "required": required, "additionalProperties": false})
}

fn empty_result() -> Value {
    json!({"type": "object"})
}

struct B(CommandSpec);

impl B {
    fn new(id: &'static str, spec_ids: &'static [&'static str]) -> B {
        B(CommandSpec {
            id,
            params: no_params(),
            result: empty_result(),
            scopes: UI_CLI_MCP,
            interactive: false,
            mutates: false,
            network: false,
            destructive: false,
            assignable: false,
            positional: &[],
            spec_ids,
        })
    }
    fn params(mut self, p: Value) -> B {
        self.0.params = p;
        self
    }
    fn result(mut self, r: Value) -> B {
        self.0.result = r;
        self
    }
    fn scopes(mut self, s: &'static [Scope]) -> B {
        self.0.scopes = s;
        self
    }
    fn mutates(mut self) -> B {
        self.0.mutates = true;
        self
    }
    fn destructive(mut self) -> B {
        self.0.mutates = true;
        self.0.destructive = true;
        self
    }
    fn assignable(mut self) -> B {
        self.0.assignable = true;
        self
    }
    fn positional(mut self, p: &'static [&'static str]) -> B {
        self.0.positional = p;
        self
    }
}

const PERMISSIONS: [&str; 4] = ["screen_recording", "accessibility", "camera", "microphone"];

pub fn specs() -> Vec<CommandSpec> {
    let section = json!({"type": "string", "enum": crate::settings::SECTIONS});
    let shortcut_scope = json!({"type": "string", "enum": ["global", "app"]});
    vec![
        B::new("app.ping", &["APP-027"])
            .scopes(ALL)
            .assignable()
            .result(json!({
                "type": "object",
                "properties": {
                    "version": {"type": "string"},
                    "platform": {"type": "string", "enum": ["windows", "macos"]},
                    "arch": {"type": "string"},
                    "state": {"type": "string", "enum": ["idle", "capturing", "recording", "busy"]},
                    "automationEnabled": {"type": "boolean"},
                    "language": {"type": "string"},
                    "systemLanguage": {"type": "string"},
                    "source": {"type": "string"}
                },
                "required": ["version", "platform", "arch", "state", "automationEnabled"]
            })),
        B::new("app.commands", &["APP-027"]).result(json!({"type": "array", "items": {"type": "object"}})),
        B::new("app.show", &["APP-001"]).scopes(ALL).assignable(),
        B::new("app.close_window", &["APP-004"]).scopes(UI_SHORTCUT).assignable(),
        B::new("app.quit", &["APP-004"]).scopes(ALL).assignable(),
        B::new("app.diagnostics", &["APP-046"])
            .params(params(json!({"copy": {"type": "boolean", "default": false}}), &[]))
            .result(json!({"type": "object", "properties": {"text": {"type": "string"}}, "required": ["text"]})),
        B::new("app.open_logs", &["APP-046"]).scopes(UI_CLI),
        B::new("app.permissions", &["APP-024"]).result(json!({"type": "array", "items": {"type": "object"}})),
        B::new("app.request_permission", &["APP-024"])
            .scopes(UI_CLI)
            .positional(&["permission"])
            .params(params(json!({"permission": {"type": "string", "enum": PERMISSIONS}}), &["permission"])),
        B::new("app.open_permission_settings", &["APP-024"])
            .scopes(UI_CLI)
            .positional(&["permission"])
            .params(params(json!({"permission": {"type": "string", "enum": PERMISSIONS}}), &["permission"])),
        B::new("settings.open", &["APP-006"])
            .scopes(UI_SHORTCUT_CLI)
            .assignable()
            .params(params(json!({"section": section, "key": {"type": "string"}}), &[])),
        B::new("settings.get", &["APP-006"])
            .positional(&["key"])
            .params(params(json!({"key": {"type": "string"}}), &["key"]))
            .result(json!({"type": "object", "properties": {"key": {"type": "string"}, "value": {}}})),
        B::new("settings.set", &["APP-006", "APP-032"])
            .mutates()
            .positional(&["key", "value"])
            .params(params(json!({"key": {"type": "string"}, "value": {}}), &["key", "value"]))
            .result(json!({"type": "object", "properties": {"key": {"type": "string"}, "value": {}, "changed": {"type": "boolean"}}})),
        B::new("settings.list", &["APP-006"]).result(json!({"type": "array", "items": {"type": "object"}})),
        B::new("settings.schema", &["APP-006"]).result(json!({"type": "array", "items": {"type": "object"}})),
        B::new("shortcuts.list", &["APP-017", "APP-018"])
            .result(json!({"type": "object", "properties": {"profile": {"type": "string"}, "paused": {"type": "boolean"}, "rows": {"type": "array"}}})),
        B::new("shortcuts.set", &["APP-017", "APP-018", "APP-019"])
            .mutates()
            .positional(&["command", "shortcut"])
            .params(params(
                json!({
                    "command": {"type": "string"},
                    "shortcut": {"type": ["string", "null"]},
                    "scope": shortcut_scope,
                    "reassign": {"type": "boolean", "default": false}
                }),
                &["command", "shortcut"],
            ))
            .result(json!({"type": "object"})),
        B::new("shortcuts.reset", &["APP-018"])
            .destructive()
            .scopes(UI_CLI)
            .positional(&["command"])
            .params(params(json!({"command": {"type": "string"}, "scope": shortcut_scope}), &[])),
        B::new("shortcuts.pause", &["APP-002"])
            .scopes(UI_SHORTCUT_CLI)
            .assignable()
            .mutates()
            .params(params(json!({"paused": {"type": "boolean"}}), &[]))
            .result(json!({"type": "object", "properties": {"paused": {"type": "boolean"}}})),
        B::new("shortcuts.set_profile", &["APP-021"])
            .scopes(UI_CLI)
            .mutates()
            .positional(&["profile"])
            .params(params(
                json!({
                    "profile": {"type": "string", "enum": ["coexistence", "compatibility"]},
                    "resetCustom": {"type": "boolean", "default": false},
                    "dryRun": {"type": "boolean", "default": false}
                }),
                &["profile"],
            )),
        B::new("shortcuts.retry", &["APP-019"]).scopes(UI_CLI).mutates(),
        B::new("shortcuts.open_system_settings", &["APP-019", "APP-020"]).scopes(UI_CLI),
        B::new("shortcuts.recording", &["APP-018"])
            .scopes(UI)
            .params(params(json!({"active": {"type": "boolean"}}), &["active"])),
    ]
    .into_iter()
    .map(|b| b.0)
    .collect()
}
