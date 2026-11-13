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
const CLI: &[Scope] = &[Scope::Cli];

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
    fn interactive(mut self) -> B {
        self.0.interactive = true;
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
    fn network(mut self) -> B {
        self.0.network = true;
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

/// The sections of Preferences (APP-006), in their order.
pub const SETTING_SECTIONS: [&str; 10] = [
    "general",
    "capture",
    "video",
    "editor",
    "library",
    "share",
    "ai",
    "shortcuts",
    "advanced",
    "experimental",
];

/// Command areas whose handlers another crate registers through `Core::with_extensions`.
pub const EXTERNAL_AREAS: &[&str] = &[
    "capture", "document", "editor", "ai", "library", "tray", "preset", "smart",
];

/// Options shared by every capture command (CAP-008), plus the mode's own.
fn capture_params(own: Value) -> Value {
    let mut props = json!({
        "cursor": {"type": "boolean", "description": "Include the pointer (CAP-047)"},
        "copy": {"type": "boolean", "description": "Copy to the clipboard (CAP-046)"},
        "open": {"type": "string", "enum": ["editor", "none"], "description": "Open in (CAP-045)"},
        "out": {"type": "string", "description": "Also write a PNG to this path"},
        "delay": {"type": "number", "minimum": 0, "maximum": 60, "description": "Seconds to wait first (CAP-048)"},
        "effects": {"type": "array", "items": {"type": "object"}, "description": "Effects for the capture (CAP-051), the editor's entries {type, …settings}: border, shadow, edges, corners, scale, rotate, filters, watermark…; editable in the document, applied in the clipboard and files"},
        "share": {"type": "array", "items": {"type": "string", "enum": ["file"]}, "description": "Destinations the capture also goes to (CAP-052): file"},
        "file": {"type": "object", "description": "The File destination (CAP-053): folder, prefix, pattern ({prefix}_{date}_{time} by default), format png|jpg|xshot, quality 1-100, onExists number|overwrite"}
    });
    if let (Some(p), Some(o)) = (props.as_object_mut(), own.as_object()) {
        p.extend(o.clone());
    }
    params(props, &[])
}

fn capture_result() -> Value {
    json!({
        "type": "object",
        "properties": {
            "status": {"type": "string", "enum": ["completed", "cancelled", "failed", "busy", "none"], "description": "none: nothing to repeat yet (CAP-007)"},
            "panel": {"type": "boolean", "description": "The capture panel was opened instead"},
            "documentId": {"type": "string", "description": "The capture as an open document, for document.* commands"},
            "path": {"type": "string", "description": "The .xshot in the library, or the --out file"},
            "files": {"type": "array", "items": {"type": "string"}},
            "width": {"type": "integer"},
            "height": {"type": "integer"},
            "metadata": {"type": "object"}
        },
        "required": ["status"]
    })
}

/// What the multi-capture panel shows (CAP-067).
fn ai_model() -> Value {
    json!({"type": "string", "enum": crate::settings::AI_MODELS, "description": "Default: the model chosen in Preferences > AI"})
}

fn ai_key_status() -> Value {
    json!({
        "type": "object",
        "properties": {
            "saved": {"type": "boolean"},
            "hint": {"type": "string", "description": "Last four characters of the saved key"}
        }
    })
}

fn multi_state() -> Value {
    json!({
        "type": "object",
        "properties": {
            "capturing": {"type": "boolean", "description": "Clicks are being captured"},
            "items": {
                "type": "array",
                "description": "Captures of the session, oldest first, saved in the library",
                "items": {
                    "type": "object",
                    "properties": {
                        "id": {"type": "string"},
                        "path": {"type": "string"},
                        "width": {"type": "integer"},
                        "height": {"type": "integer"}
                    }
                }
            }
        },
        "required": ["capturing", "items"]
    })
}

const PERMISSIONS: [&str; 4] = ["screen_recording", "accessibility", "camera", "microphone"];

pub fn specs() -> Vec<CommandSpec> {
    let section = json!({"type": "string", "enum": crate::settings::SECTIONS});
    let shortcut_scope = json!({"type": "string", "enum": ["global", "app", "editor"]});
    // Built in parts: as one list, its frame was too big for the main thread of a
    // debug build (STATUS_STACK_OVERFLOW).
    let mut all: Vec<B> = Vec::new();
    all.extend(specs_1(&section, &shortcut_scope));
    all.extend(specs_2(&section, &shortcut_scope));
    all.extend(specs_3(&section, &shortcut_scope));
    all.extend(specs_4(&section, &shortcut_scope));
    all.extend(specs_5(&section, &shortcut_scope));
    all.extend(specs_6(&section, &shortcut_scope));
    all.into_iter().map(|b| b.0).collect()
}

#[inline(never)]
#[allow(unused_variables)]
fn specs_1(section: &Value, shortcut_scope: &Value) -> Vec<B> {
    vec![
        B::new("app.ping", &["APP-027"])
            .scopes(ALL)
            .assignable()
            .result(json!({
                "type": "object",
                "properties": {
                    "version": {"type": "string"},
                    "build": {"type": "string"},
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
        B::new("app.quit", &["APP-004"])
            .scopes(ALL)
            .assignable()
            .params(params(
                json!({"force": {"type": "boolean", "default": false, "description": "Quit without asking about unsaved documents (they stay recoverable)"}}),
                &[],
            ))
            .result(json!({
                "type": "object",
                "properties": {
                    "status": {"type": "string", "enum": ["quitting", "needs_decision"]},
                    "pending": {"type": "array", "items": {"type": "object"}}
                }
            })),
        B::new("app.diagnostics", &["APP-046"])
            .params(params(json!({"copy": {"type": "boolean", "default": false}}), &[]))
            .result(json!({"type": "object", "properties": {"text": {"type": "string"}}, "required": ["text"]})),
        B::new("events.watch", &["APP-031"])
            .scopes(CLI)
            .params(params(
                json!({"filter": {"type": "string", "description": "Events to show, separated by commas; a * at the end matches the rest (capture.*,file.written). Without it, all"}}),
                &[],
            ))
            .result(json!({"type": "object", "properties": {"watching": {"type": "boolean"}}})),
        B::new("app.welcome", &["APP-005"]).scopes(UI_CLI),
        B::new("app.help", &["APP-015"])
            .scopes(UI_CLI)
            .assignable()
            .params(params(
                json!({"topic": {"type": "string", "enum": ["start", "capture", "editor", "library", "shortcuts", "permissions", "automation", "video", "share", "about"]}}),
                &[],
            )),
        B::new("app.about", &["APP-015"])
            .params(params(
                json!({"open": {"type": "boolean", "default": false, "description": "Also show it, in the Help window"}}),
                &[],
            ))
            .result(json!({"type": "object", "properties": {
            "version": {"type": "string"}, "build": {"type": "string"}, "date": {"type": "string"},
            "platform": {"type": "string"}, "arch": {"type": "string"}, "channel": {"type": "string"},
            "dataDir": {"type": "string"}, "configDir": {"type": "string"}, "managed": {"type": "string"},
            "network": {"type": "array", "items": {"type": "string"}}
        }})),
        B::new("app.feedback", &["APP-015"])
            .scopes(UI_CLI)
            .result(json!({"type": "object", "properties": {"url": {"type": "string"}}})),
        B::new("app.open_logs", &["APP-046"]).scopes(UI_CLI),
        B::new("app.open_data_folder", &["APP-009", "APP-046"])
            .scopes(UI_CLI)
            .params(params(
                json!({"open": {"type": "boolean", "default": true, "description": "Open the folder; false only returns where it is"}}),
                &[],
            ))
            .result(json!({"type": "object", "properties": {"data": {"type": "string"}, "config": {"type": "string"}}})),
        B::new("app.mcp_config", &["APP-029", "SMT-043"])
            .scopes(UI_CLI)
            .result(json!({
                "type": "object",
                "properties": {
                    "command": {"type": "string", "description": "The installed CLI"},
                    "args": {"type": "array", "items": {"type": "string"}},
                    "config": {"type": "string", "description": "JSON block for an MCP client's configuration"},
                    "claudeCode": {"type": "string", "description": "Command that adds the server to Claude Code"}
                }
            })),
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
        B::new("settings.export", &["APP-011", "APP-030"])
            .params(params(
                json!({
                    "out": {"type": "string", "description": "The .xsettings file to write"},
                    "groups": {"type": "array", "items": {"type": "string", "enum": ["preferences", "shortcuts", "themes", "toolbar", "presets"]}, "description": "Default: all of them"}
                }),
                &["out"],
            ))
            .result(json!({"type": "object", "properties": {"path": {"type": "string"}, "groups": {"type": "array", "items": {"type": "string"}}}})),
        B::new("settings.import", &["APP-011", "APP-030"])
            .destructive()
            .positional(&["path"])
            .params(params(
                json!({
                    "path": {"type": "string", "description": "An .xsettings file"},
                    "groups": {"type": "array", "items": {"type": "string", "enum": ["preferences", "shortcuts", "themes", "toolbar", "presets"]}, "description": "Default: all of them"},
                    "convertShortcuts": {"type": "boolean", "default": true, "description": "From the other platform: convert Ctrl and Cmd, Alt and Option, Win and Control; false keeps this computer's shortcuts"},
                    "onConflict": {"type": "string", "enum": ["replace", "keepBoth", "skip"], "default": "keepBoth", "description": "A theme or saved effect with the name of an existing one"},
                    "dryRun": {"type": "boolean", "default": false, "description": "Only say what would change, by group"}
                }),
                &["path"],
            ))
            .result(json!({"type": "object", "properties": {
                "platform": {"type": "string"},
                "samePlatform": {"type": "boolean"},
                "dryRun": {"type": "boolean"},
                "groups": {"type": "object", "description": "By group: preferences {changes, notApplied}, shortcuts {changes, kept, unconvertible, conflicts}, themes {themes, effects, favorites}, toolbar {changes}"}
            }})),
        B::new("settings.reset", &["APP-010", "APP-030"])
            .destructive()
            .params(params(
                json!({
                    "section": {"type": "string", "enum": SETTING_SECTIONS, "description": "Only this section; without it, every setting and every shortcut"}
                }),
                &[],
            ))
            .result(json!({"type": "object", "properties": {"reset": {"type": "array", "items": {"type": "string"}}}})),
        B::new("app.hints", &["APP-014"])
            .scopes(UI)
            .params(params(
                json!({"seen": {"type": "string", "description": "A hint just seen and closed: it does not show again"}}),
                &[],
            ))
            .result(json!({"type": "object", "properties": {"enabled": {"type": "boolean"}, "seen": {"type": "array", "items": {"type": "string"}}}})),
        B::new("app.warnings", &["APP-010", "APP-014"])
            .params(params(
                json!({
                    "reset": {"type": "boolean", "default": false, "description": "Show again every warning answered with Don't ask again"}
                }),
                &[],
            ))
            .result(json!({"type": "object", "properties": {"hidden": {"type": "integer"}, "restored": {"type": "integer"}}})),
        B::new("settings.list", &["APP-006"]).result(json!({"type": "array", "items": {"type": "object"}})),
        B::new("settings.schema", &["APP-006"]).result(json!({"type": "array", "items": {"type": "object"}})),
        B::new("shortcuts.list", &["APP-017", "APP-018"])
            .result(json!({"type": "object", "properties": {"profile": {"type": "string"}, "paused": {"type": "boolean"}, "rows": {"type": "array"}}})),
        B::new("shortcuts.export", &["APP-023"])
            .params(params(
                json!({
                    "format": {"type": "string", "enum": ["pdf", "html", "json"], "default": "pdf"},
                    "out": {"type": "string", "description": "The file to write; without it, the sheet comes back as data"}
                }),
                &[],
            ))
            .result(json!({"type": "object", "properties": {
                "path": {"type": ["string", "null"]},
                "sheet": {"type": "object", "description": "title, profile, platform, date, groups [{id, title, rows [{title, keys, scope, inactive}]}]"}
            }})),
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
    ]
}

#[inline(never)]
#[allow(unused_variables)]
fn specs_2(section: &Value, shortcut_scope: &Value) -> Vec<B> {
    vec![
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
        // Capture (CAP-008). Handlers live in xs-capture.
        B::new("capture.monitors", &["CAP-021", "CAP-008"]).result(json!({"type": "array", "items": {"type": "object"}})),
        B::new("capture.fullscreen", &["CAP-028", "CAP-021", "CAP-008"])
            .scopes(ALL)
            .assignable()
            .mutates()
            .params(capture_params(json!({
                "monitor": {"type": "string", "description": "pointer (default), primary, all, or a monitor name"}
            })))
            .result(capture_result()),
        B::new("capture.capture", &["CAP-006", "CAP-001"])
            .scopes(ALL)
            .assignable()
            .mutates()
            .interactive()
            .params(capture_params(json!({})))
            .result(capture_result()),
        B::new("capture.image", &["CAP-065", "CAP-002"])
            .scopes(ALL)
            .assignable()
            .mutates()
            .interactive()
            .params(capture_params(json!({})))
            .result(capture_result()),
        B::new("capture.repeat_last", &["CAP-007"])
            .scopes(ALL)
            .assignable()
            .mutates()
            .interactive()
            .result(capture_result()),
        B::new("capture.repeat_last_area", &["CAP-007"])
            .scopes(ALL)
            .assignable()
            .mutates()
            .params(capture_params(json!({})))
            .result(capture_result()),
        B::new("capture.panel", &["CAP-001", "CAP-065"])
            .scopes(UI_SHORTCUT_CLI)
            .assignable()
            .params(params(json!({"show": {"type": "boolean"}}), &[]))
            .result(json!({"type": "object", "properties": {"visible": {"type": "boolean"}}})),
        B::new("capture.multi_start", &["CAP-067"])
            .scopes(ALL)
            .assignable()
            .interactive()
            .mutates()
            .result(multi_state()),
        B::new("capture.multi_stop", &["CAP-067"])
            .params(params(
                json!({"close": {"type": "boolean", "default": false, "description": "Also close the panel and forget the session's captures (they stay in the library)"}}),
                &[],
            ))
            .result(multi_state()),
        B::new("capture.multi_state", &["CAP-067"]).result(multi_state()),
        B::new("capture.multi_remove", &["CAP-067"])
            .mutates()
            .params(params(
                json!({"ids": {"type": "array", "items": {"type": "string"}, "description": "Ids of captures of the session"}}),
                &["ids"],
            ))
            .result(multi_state()),
        B::new("capture.multi_clear", &["CAP-067"])
            .mutates()
            .result(multi_state()),
        // Scrolling capture (CAP-034 to CAP-036).
        B::new("capture.scrolling", &["CAP-034", "CAP-035", "CAP-036", "CAP-008"])
            .scopes(ALL)
            .assignable()
            .mutates()
            .interactive()
            .params(capture_params(json!({
                "type": {"type": "string", "enum": ["manual", "auto"], "description": "manual: the user scrolls and presses Done; auto: XcreenShot scrolls (default: the Image tab's)"},
                "direction": {"type": "string", "enum": ["vertical", "horizontal", "both"], "description": "Automatic: which way to scroll (default: the Image tab's)"},
                "startFrom": {"type": "string", "enum": ["start", "current"], "description": "Automatic: from the beginning or from where the content is"},
                "wait": {"type": "number", "minimum": 0, "maximum": 5, "description": "Automatic: seconds between steps, for content that loads late"},
                "restore": {"type": "boolean", "description": "Automatic: put the content back where it was"},
                "rect": {"type": "object", "description": "The area, a rectangle of the desktop in physical pixels, without selecting on screen"}
            })))
            .result(capture_result()),
        B::new("capture.scroll_action", &["CAP-034", "CAP-035"])
            .positional(&["action"])
            .params(params(
                json!({"action": {"type": "string", "enum": ["start", "done", "cancel"], "description": "start (with Ask before scrolling), done (keep what was captured; Stop here) or cancel"}}),
                &["action"],
            ))
            .result(json!({"type": "object", "properties": {"running": {"type": "boolean"}}})),
        B::new("capture.scroll_state", &["CAP-034", "CAP-035"]).result(json!({"type": "object", "properties": {
            "running": {"type": "boolean"}, "auto": {"type": "boolean"}, "started": {"type": "boolean"},
            "width": {"type": "integer"}, "height": {"type": "integer"}, "full": {"type": "boolean"}
        }})),
        B::new("capture.settings", &["CAP-001", "CAP-002", "CAP-010"]).result(json!({"type": "object"})),
        B::new("capture.configure", &["CAP-001", "CAP-002", "CAP-010"])
            .mutates()
            .params(params(
                json!({
                    "tab": {"type": "string", "enum": ["universal", "image"]},
                    "expanded": {"type": "boolean"},
                    "universal": {"type": "object"},
                    "image": {"type": "object"}
                }),
                &[],
            ))
            .result(json!({"type": "object"})),
        // Extract text (SMT-003 to SMT-006).
        B::new("smart.extract_text", &["SMT-005", "SMT-003", "APP-030"])
            .scopes(ALL)
            .mutates()
            .interactive()
            .positional(&["source"])
            .params(params(
                json!({
                    "source": {"type": "string", "enum": ["screen", "file"], "default": "screen", "description": "screen: the user selects a zone (or rect); file: an image file"},
                    "path": {"type": "string", "description": "The image file, with source file"},
                    "rect": {"type": "object", "description": "A rectangle of the desktop in physical pixels, without selecting on screen"},
                    "language": {"type": ["string", "array"], "items": {"type": "string"}, "description": "auto, or up to 3 languages (es, en, pt, fr, de, ja, zh-Hans…); default: the setting"},
                    "copy": {"type": "boolean", "description": "Copy the text (default: yes from the app; no from the CLI)"},
                    "show": {"type": "boolean", "description": "Open the results window (default: the setting; no from the CLI)"},
                    "delay": {"type": "number", "minimum": 0, "maximum": 60}
                }),
                &[],
            ))
            .result(json!({"type": "object", "properties": {
                "id": {"type": "string"}, "found": {"type": "boolean"}, "words": {"type": "integer"},
                "language": {"type": "string"}, "text": {"type": "string"}, "html": {"type": "string"},
                "copied": {"type": "boolean"}
            }})),
        B::new("smart.ocr_languages", &["SMT-004"])
            .result(json!({"type": "object", "properties": {"languages": {"type": "array"}, "installed": {"type": "array"}, "settingsUri": {"type": "string"}}})),
        B::new("smart.language_settings", &["SMT-004"])
            .scopes(UI_CLI)
            .result(json!({"type": "object", "properties": {"uri": {"type": "string"}}})),
        B::new("smart.text_result", &["SMT-006"])
            .scopes(UI)
            .positional(&["id"])
            .params(params(json!({"id": {"type": "string"}}), &["id"]))
            .result(json!({"type": "object"})),
        B::new("smart.text_again", &["SMT-006", "SMT-004"])
            .scopes(UI)
            .params(params(
                json!({"id": {"type": "string"}, "language": {"type": ["string", "array"], "items": {"type": "string"}}}),
                &["id", "language"],
            ))
            .result(json!({"type": "object"})),
        B::new("smart.text_close", &["SMT-006", "SMT-001"])
            .scopes(UI)
            .params(params(json!({"id": {"type": "string"}}), &["id"]))
            .result(json!({"type": "object"})),
        B::new("smart.show_text", &["SMT-006"])
            .scopes(UI)
            .params(params(json!({"id": {"type": "string"}}), &["id"]))
            .result(json!({"type": "object"})),
        B::new("smart.copy_text", &["SMT-006"])
            .scopes(UI)
            .mutates()
            .params(params(
                json!({"text": {"type": "string"}, "html": {"type": ["string", "null"]}}),
                &["text"],
            ))
            .result(json!({"type": "object"})),
        B::new("smart.save_text", &["SMT-006"])
            .scopes(UI)
            .mutates()
            .params(params(
                json!({"path": {"type": "string"}, "text": {"type": "string"}}),
                &["path", "text"],
            ))
            .result(json!({"type": "object", "properties": {"path": {"type": "string"}}})),
        // Presets (CAP-058 to CAP-064).
        B::new("preset.list", &["CAP-058", "APP-030"])
            .result(json!({"type": "object", "properties": {"presets": {"type": "array", "description": "In order: id, name, kind, settings, summary, builtin, shortcut, shortcutDisplay, slot, running, warnings"}}})),
        B::new("preset.run", &["CAP-058", "CAP-062", "APP-030"])
            .scopes(ALL)
            .mutates()
            .interactive()
            .positional(&["name"])
            .params(params(
                json!({
                    "id": {"type": "string"},
                    "name": {"type": "string", "description": "The preset's name, any case"}
                }),
                &[],
            ))
            .result(json!({"type": "object", "description": "The capture's result"})),
        B::new("preset.create", &["CAP-059"])
            .mutates()
            .positional(&["name"])
            .params(params(
                json!({
                    "name": {"type": "string", "description": "1 to 64 characters; a taken one gets \" (2)\". Without it, a name that says what it does"},
                    "kind": {"type": "string", "enum": ["universal", "image"], "default": "image"},
                    "settings": {"type": "object", "description": "The tab's settings (capture.settings shows them); missing ones take their defaults"},
                    "fromCurrent": {"type": "boolean", "description": "Copy the capture panel's active tab"}
                }),
                &[],
            ))
            .result(json!({"type": "object"})),
        B::new("preset.update", &["CAP-060"])
            .mutates()
            .positional(&["id"])
            .params(params(
                json!({
                    "id": {"type": "string"},
                    "name": {"type": "string"},
                    "settings": {"type": "object", "description": "Settings to change; the rest stay"},
                    "fromCurrent": {"type": "boolean", "description": "Take the panel's active tab (same kind only)"}
                }),
                &[],
            ))
            .result(json!({"type": "object"})),
        B::new("preset.rename", &["CAP-060"])
            .mutates()
            .positional(&["id", "to"])
            .params(params(
                json!({"id": {"type": "string"}, "to": {"type": "string", "description": "The new name; its shortcut stays"}}),
                &["id", "to"],
            ))
            .result(json!({"type": "object"})),
    ]
}

#[inline(never)]
#[allow(unused_variables)]
fn specs_3(section: &Value, shortcut_scope: &Value) -> Vec<B> {
    vec![
        B::new("preset.duplicate", &["CAP-060"])
            .mutates()
            .positional(&["id"])
            .params(params(json!({"id": {"type": "string"}, "name": {"type": "string"}}), &[]))
            .result(json!({"type": "object"})),
        B::new("preset.delete", &["CAP-060"])
            .mutates()
            .destructive()
            .positional(&["ids"])
            .params(params(
                json!({"ids": {"type": "array", "items": {"type": "string"}}, "id": {"type": "string"}}),
                &[],
            ))
            .result(json!({"type": "object", "properties": {"deleted": {"type": "integer"}, "undo": {"type": "string", "description": "For preset.restore"}}})),
        B::new("preset.restore", &["CAP-060"])
            .mutates()
            .positional(&["undo"])
            .params(params(json!({"undo": {"type": "string"}}), &["undo"]))
            .result(json!({"type": "object", "properties": {"restored": {"type": "integer"}}})),
        B::new("preset.move", &["CAP-061"])
            .mutates()
            .positional(&["id", "index"])
            .params(params(
                json!({"id": {"type": "string"}, "index": {"type": "integer", "minimum": 0, "description": "0 = first"}}),
                &["id", "index"],
            ))
            .result(json!({"type": "object", "properties": {"index": {"type": "integer"}}})),
        B::new("preset.restore_builtins", &["CAP-064"])
            .mutates()
            .params(params(
                json!({"reset": {"type": "boolean", "default": false, "description": "Also put back the settings and name of the ones that were changed"}}),
                &[],
            ))
            .result(json!({"type": "object", "properties": {"restored": {"type": "array"}, "modified": {"type": "array"}}})),
        B::new("preset.export", &["CAP-063", "APP-030"])
            .positional(&["ids"])
            .params(params(
                json!({
                    "ids": {"type": "array", "items": {"type": "string"}, "description": "Without it, every preset"},
                    "out": {"type": "string", "description": "The .xpreset file to write; without it, the file comes back as data"}
                }),
                &[],
            ))
            .result(json!({"type": "object"})),
        B::new("preset.import", &["CAP-063", "APP-030"])
            .mutates()
            .destructive()
            .positional(&["paths"])
            .params(params(
                json!({
                    "paths": {"type": "array", "items": {"type": "string"}, "description": ".xpreset files"},
                    "dryRun": {"type": "boolean", "description": "Only list what the files bring"},
                    "only": {"type": "array", "items": {"type": "string"}, "description": "Only these names"},
                    "onConflict": {"type": "string", "enum": ["keepBoth", "replace", "skip"], "default": "keepBoth"},
                    "convertShortcuts": {"type": "boolean", "default": true}
                }),
                &["paths"],
            ))
            .result(json!({"type": "object"})),
        B::new("capture.reset_settings", &["CAP-003"])
            .scopes(UI_CLI)
            .destructive()
            .result(json!({"type": "object"})),
        B::new("capture.universal", &["CAP-009", "CAP-011", "CAP-012", "CAP-013"])
            .scopes(ALL)
            .interactive()
            .assignable()
            .mutates()
            .params(capture_params(json!({})))
            .result(capture_result()),
        B::new("capture.region", &["CAP-025", "CAP-008"])
            .scopes(ALL)
            .assignable()
            .mutates()
            .params(capture_params(json!({
                "rect": {
                    "type": "object",
                    "description": "Physical pixels on the virtual desktop, or from the top left of `monitor` (CAP-026). Omit for an interactive selection.",
                    "properties": {
                        "x": {"type": "integer"}, "y": {"type": "integer"},
                        "width": {"type": "integer", "minimum": 1}, "height": {"type": "integer", "minimum": 1},
                        "monitor": {"type": "string", "description": "A monitor's name (capture.monitors): x and y are measured from its top left"}
                    },
                    "required": ["x", "y", "width", "height"],
                    "additionalProperties": false
                },
                "multiple": {
                    "type": "object",
                    "description": "Multiple areas (CAP-033): the user picks several areas (drag or click each) and Enter ends; they become one image",
                    "properties": {
                        "composition": {"type": "string", "enum": ["original", "stacked", "row"], "default": "original"},
                        "gap": {"type": "integer", "minimum": 0, "maximum": 500, "default": 10}
                    },
                    "additionalProperties": false
                },
                "size": {
                    "type": "object",
                    "description": "Fixed region (CAP-026): a rectangle of this size follows the pointer; a click places it and captures",
                    "properties": {
                        "width": {"type": "integer", "minimum": 1}, "height": {"type": "integer", "minimum": 1}
                    },
                    "required": ["width", "height"],
                    "additionalProperties": false
                }
            })))
            .result(capture_result()),
        B::new("capture.window_picker", &["CAP-030", "CAP-065"])
            .scopes(ALL)
            .assignable()
            .mutates()
            .interactive()
            .params(capture_params(json!({})))
            .result(capture_result()),
        B::new("capture.picker_state", &["CAP-030"]).result(json!({"type": "object", "properties": {
            "open": {"type": "boolean"}, "windows": {"type": "array", "items": {"type": "object"}}
        }})),
        B::new("capture.picker_choose", &["CAP-030"])
            .params(params(
                json!({"id": {"type": "integer", "description": "The window to capture; without it, the picker is cancelled"}}),
                &[],
            ))
            .result(json!({"type": "object"})),
        B::new("capture.menu", &["CAP-031", "CAP-065"])
            .scopes(ALL)
            .assignable()
            .mutates()
            .interactive()
            .params(capture_params(json!({
                "submenus": {"type": "boolean", "default": true, "description": "Include the open submenus"},
                "menubar": {"type": "boolean", "default": false, "description": "Include the menu bar item the menu hangs from"}
            })))
            .result(capture_result()),
        B::new("capture.file_name", &["CAP-053"])
            .params(params(
                json!({
                    "file": {"type": "object", "description": "The File destination's options"},
                    "next": {"type": "integer", "minimum": 1, "description": "Set its counter's next number"}
                }),
                &[],
            ))
            .result(json!({"type": "object", "properties": {
                "folder": {"type": "string"}, "name": {"type": "string"}, "next": {"type": "integer"},
                "counter": {"type": "boolean"}, "markers": {"type": "array"}
            }})),
        B::new("capture.pick_area", &["CAP-026"])
            .scopes(UI_CLI)
            .interactive()
            .result(json!({"type": "object", "properties": {
                "monitor": {"type": "string"}, "x": {"type": "integer"}, "y": {"type": "integer"},
                "width": {"type": "integer"}, "height": {"type": "integer"}
            }})),
        B::new("capture.window", &["CAP-027", "CAP-008"])
            .scopes(ALL)
            .assignable()
            .mutates()
            .params(capture_params(json!({
                "target": {"type": "string", "enum": ["active", "interactive", "point", "title"], "default": "active"},
                "x": {"type": "integer"},
                "y": {"type": "integer"},
                "title": {"type": "string", "description": "Part of the window title or process name (first match)"}
            })))
            .result(capture_result()),
        // Editor and documents (EDT-119). Handlers live in xs-editor.
        B::new("document.open", &["EDT-105", "EDT-119"])
            .scopes(ALL)
            .mutates()
            .positional(&["path"])
            .params(params(
                json!({
                    "path": {"type": "string", "description": "A .xshot, PNG or JPG file"},
                    "show": {"type": "boolean", "description": "Show it in the editor (default: false from the CLI and MCP)"}
                }),
                &["path"],
            ))
            .result(document_summary()),
        B::new("document.list", &["EDT-001", "TRY-002"]).result(json!({
            "type": "object",
            "properties": {
                "active": {"type": ["string", "null"]},
                "documents": {"type": "array", "items": document_summary()}
            }
        })),
        B::new("document.info", &["EDT-119"])
            .params(params(json!({"documentId": document_id()}), &[]))
            .result(json!({"type": "object"})),
        B::new("document.annotate", &["EDT-119", "EDT-024"])
            .mutates()
            .params(params(
                json!({
                    "documentId": document_id(),
                    "objects": {"type": "array", "items": {"type": "object"}, "description": "Objects to add: type, geometry and style. All or none are created."}
                }),
                &["objects"],
            ))
            .result(json!({"type": "object", "properties": {"ids": {"type": "array", "items": {"type": "string"}}}})),
        B::new("document.update_object", &["EDT-119", "EDT-014", "EDT-042"])
            .mutates()
            .params(params(
                json!({
                    "documentId": document_id(),
                    "objectId": {"type": "string"},
                    "objectIds": {"type": "array", "items": {"type": "string"}},
                    "set": {"type": "object", "description": "Geometry to replace and style properties to change"},
                    "move": {"type": "array", "items": {"type": "number"}, "minItems": 2, "maxItems": 2, "description": "Move every object by [dx, dy] canvas pixels, in one undo step"},
                    "moves": {"type": "object", "description": "A move for each object, {\"o1\": [dx, dy], …}, in one undo step (aligning, EDT-083)"}
                }),
                &[],
            ))
            .result(json!({"type": "object", "properties": {"objects": {"type": "array"}}})),
        B::new("document.arrange", &["EDT-081", "EDT-119"])
            .mutates()
            .params(params(
                json!({
                    "documentId": document_id(),
                    "objectId": {"type": "string"},
                    "objectIds": {"type": "array", "items": {"type": "string"}},
                    "to": {"type": "string", "enum": ["front", "forward", "backward", "back"]}
                }),
                &["to"],
            ))
            .result(json!({"type": "object", "properties": {"order": {"type": "array", "items": {"type": "string"}}}})),
        B::new("document.simplify_view", &["EDT-041"])
            .mutates()
            .params(params(
                json!({
                    "documentId": document_id(),
                    "showOriginal": {"type": "boolean", "description": "Show the original image under the simplified shapes"},
                    "opacity": {"type": "number", "minimum": 0, "maximum": 100, "description": "How strongly it shows, %"}
                }),
                &[],
            ))
            .result(json!({"type": "object", "properties": {"showOriginal": {"type": "boolean"}, "opacity": {"type": "number"}}})),
        B::new("editor.stamps", &["EDT-031"])
            .params(params(
                json!({
                    "query": {"type": "string", "description": "Words of its name or keywords, in English or Spanish"},
                    "category": {"type": "string", "enum": ["indicators", "cursors", "keys", "devices", "interface", "labels", "arrows"]}
                }),
                &[],
            ))
            .result(json!({"type": "object", "properties": {"categories": {"type": "array"}, "stamps": {"type": "array"}}})),
        B::new("document.steps", &["EDT-035"])
            .mutates()
            .positional(&["action"])
            .params(params(
                json!({
                    "documentId": document_id(),
                    "action": {"type": "string", "enum": ["restart", "renumber"], "description": "restart: the next step shows the first value again; renumber: number the steps again by the order they were made, without gaps (one undo step)"},
                    "sequence": {"type": "string", "enum": ["number", "upper", "lower", "romanUpper", "romanLower"], "description": "Default: the Step tool's"}
                }),
                &["action"],
            ))
            .result(json!({"type": "object", "properties": {"next": {"type": "string"}, "renumbered": {"type": "integer"}}})),
        B::new("document.flatten", &["EDT-091", "EDT-119"])
            .mutates()
            .params(params(
                json!({
                    "documentId": document_id(),
                    "objectIds": {"type": "array", "items": {"type": "string"}},
                    "all": {"type": "boolean", "default": false, "description": "Flatten every object"}
                }),
                &[],
            ))
            .result(json!({"type": "object", "properties": {
                "layers": {"type": "array", "items": {"type": "string"}, "description": "New layers, oldest first: each blur has one of its own"},
                "permanent": {"type": "boolean", "description": "A blur was flattened: it cannot become objects again"}
            }})),
        B::new("document.unflatten", &["EDT-091", "EDT-119"])
            .mutates()
            .params(params(
                json!({
                    "documentId": document_id(),
                    "layerId": {"type": "string", "description": "A layer of `flattened` in document.info"},
                    "all": {"type": "boolean", "default": false, "description": "Every layer that can"}
                }),
                &[],
            ))
            .result(json!({"type": "object", "properties": {"ids": {"type": "array", "items": {"type": "string"}}}})),
        B::new("document.fill", &["EDT-047", "EDT-119"])
            .mutates()
            .params(params(
                json!({
                    "documentId": document_id(),
                    "at": {"type": "array", "items": {"type": "number"}, "minItems": 2, "maxItems": 2, "description": "The pixel clicked, in canvas pixels"},
                    "color": {"type": ["string", "object", "null"], "description": "#RRGGBB or #RRGGBBAA, a gradient {type: linear | radial, angle, stops: [{color, pos}]} over the filled area, or null to make the pixels transparent (the canvas color shows)"},
                    "tolerance": {"type": "number", "minimum": 0, "maximum": 100, "default": 10, "description": "How different a color may be from the clicked one and still fill, %; 0 = only that exact color"},
                    "opacity": {"type": "number", "minimum": 0, "maximum": 100, "default": 100},
                    "global": {"type": "boolean", "default": false, "description": "Every pixel of that color in the image, touching or not"},
                    "objectId": {"type": "string", "description": "A floating image to fill instead of the image (`at` is still in canvas pixels)"}
                }),
                &["at"],
            ))
            .result(json!({"type": "object", "properties": {"changed": {"type": "boolean", "description": "False when the click is outside the pixels"}}})),
        B::new("document.erase", &["EDT-048", "EDT-119"])
            .mutates()
            .params(params(
                json!({
                    "documentId": document_id(),
                    "points": {"type": "array", "items": {"type": "array", "items": {"type": "number"}, "minItems": 2, "maxItems": 2}, "minItems": 1, "description": "The stroke, in canvas pixels"},
                    "width": {"type": "number", "minimum": 1, "maximum": 200, "default": 20, "description": "Thickness, px"},
                    "shape": {"type": "string", "enum": ["round", "square"], "default": "round"},
                    "opacity": {"type": "number", "minimum": 0, "maximum": 100, "default": 100, "description": "How much it erases, %"},
                    "antialias": {"type": "boolean", "default": true, "description": "Smooth edges"},
                    "objectId": {"type": "string", "description": "A floating image to erase instead of the image"}
                }),
                &["points"],
            ))
            .result(json!({"type": "object", "properties": {"changed": {"type": "boolean"}}})),
        B::new("document.select_pixels", &["EDT-043", "EDT-046", "EDT-119"])
            .positional(&["shape"])
            .params(params(
                json!({
                    "documentId": document_id(),
                    "shape": {"type": "string", "enum": ["rectangle", "ellipse", "polygon", "freehand", "wand", "all", "invert", "none"], "default": "rectangle", "description": "What to select: a drawn shape, the magic wand at `at`, everything; invert or drop the selection"},
                    "rect": {"type": "object", "description": "Rectangle or ellipse, in canvas pixels", "properties": {"x": {"type": "number"}, "y": {"type": "number"}, "w": {"type": "number"}, "h": {"type": "number"}}},
                    "points": {"type": "array", "items": {"type": "array", "items": {"type": "number"}, "minItems": 2, "maxItems": 2}, "description": "Polygon or freehand loop, in canvas pixels; it closes by itself"},
                    "at": {"type": "array", "items": {"type": "number"}, "minItems": 2, "maxItems": 2, "description": "Magic wand: the pixel clicked, in canvas pixels"},
                    "tolerance": {"type": "number", "minimum": 0, "maximum": 100, "default": 15, "description": "Magic wand: how different a color may be, %"},
                    "global": {"type": "boolean", "default": false, "description": "Magic wand: every pixel of that color, touching or not"},
                    "mode": {"type": "string", "enum": ["replace", "add", "subtract"], "default": "replace"},
                    "antialias": {"type": "boolean", "default": true, "description": "Smooth edges"},
                    "feather": {"type": "number", "minimum": 0, "maximum": 50, "default": 0, "description": "Soft edges, px"},
                    "snap": {"type": "boolean", "default": false, "description": "Rectangle or ellipse: shrink to the visual element inside it"},
                    "level": {"type": "number", "minimum": 0, "maximum": 100, "default": 50, "description": "Snap: from the smallest element to the widest group"},
                    "objectId": {"type": "string", "description": "Select on this floating image instead of the image"}
                }),
                &[],
            ))
            .result(json!({"type": "object", "properties": {"bounds": {"description": "The rectangle around the selection, in canvas pixels; null without one"}}})),
        B::new("document.selection_action", &["EDT-044", "EDT-045", "EDT-119"])
            .mutates()
            .positional(&["action"])
            .params(params(
                json!({
                    "documentId": document_id(),
                    "action": {"type": "string", "enum": ["delete", "cut", "copy", "move", "float", "crop", "effect"], "description": "delete or cut leave the gap filled; copy puts the pixels on the clipboard; move takes them by `offset`; float makes them a floating image; crop crops to them; effect fixes `effect` into them"},
                    "effect": {"type": "object", "description": "effect: a filters, color, replace or spotlight effect (spotlight: dim, blur, zoom 100–400, borderColor, borderWidth)"},
                    "offset": {"type": "array", "items": {"type": "number"}, "minItems": 2, "maxItems": 2, "description": "move: [dx, dy] in canvas pixels"},
                    "gapFill": {"type": "string", "enum": ["smart", "transparent", "canvas"], "default": "smart", "description": "What fills the place the pixels leave"},
                    "quality": {"type": "string", "enum": ["fast", "photo"], "default": "fast", "description": "Smart fill: fast for interfaces and flat backgrounds; photo rebuilds textures (slower; editor.cancel stops it)"}
                }),
                &["action"],
            ))
            .result(json!({"type": "object", "properties": {"changed": {"type": "boolean"}, "ids": {"type": "array", "items": {"type": "string"}, "description": "float: the new floating image"}}})),
        B::new("document.effects", &["EDT-053", "EDT-055", "EDT-056", "EDT-057", "EDT-058", "EDT-059", "EDT-060", "EDT-061", "EDT-062", "EDT-063", "EDT-064", "EDT-066", "EDT-067", "EDT-068", "EDT-119"])
            .mutates()
            .positional(&["action"])
            .params(params(
                json!({
                    "documentId": document_id(),
                    "action": {"type": "string", "enum": ["set", "add", "update", "remove", "move", "clear"], "default": "set", "description": "set: the whole stack; add: one at the end; update: change `index` with `effect`'s values; remove / move (`to`); clear: every effect off"},
                    "effects": {"type": "array", "items": {"type": "object"}, "description": "set: the stack, in order"},
                    "effect": {"type": "object", "description": "{type, enabled, …settings}. Types: border, edges, shadow, perspective, pageCurl, reflection, fade, filters, color, replace, watermark, details, trim, corners, scale, rotate"},
                    "index": {"type": "integer", "minimum": 0},
                    "to": {"type": "integer", "minimum": 0},
                    "objectId": {"type": "string", "description": "A floating image's own stack instead of the document's"}
                }),
                &[],
            ))
            .result(json!({"type": "object", "properties": {"effects": {"type": "array"}}})),
        B::new("editor.saved_effects", &["EDT-054"])
            .params(params(json!({}), &[]))
            .result(json!({"type": "object", "properties": {"effects": {"type": "array", "description": "[{id, name, stack}]"}}})),
        B::new("editor.saved_effect", &["EDT-054", "EDT-016"])
            .positional(&["action"])
            .params(params(
                json!({
                    "action": {"type": "string", "enum": ["apply", "save", "update", "rename", "delete", "move", "favorite"], "description": "apply: the saved effect becomes the target's stack (with append, it goes at the end); save / update: from the target's stack; favorite: a copy in Favorites"},
                    "id": {"type": "string"},
                    "name": {"type": "string"},
                    "to": {"type": "integer", "minimum": 0},
                    "append": {"type": "boolean", "default": false},
                    "documentId": document_id(),
                    "objectId": {"type": "string", "description": "A floating image instead of the document"}
                }),
                &["action"],
            ))
            .result(json!({"type": "object"})),
        B::new("document.resize", &["EDT-069", "EDT-119"])
            .mutates()
            .params(params(
                json!({
                    "documentId": document_id(),
                    "width": {"type": "number", "minimum": 1, "description": "New width of the image, px (alone: the height keeps the proportion)"},
                    "height": {"type": "number", "minimum": 1},
                    "percent": {"type": "number", "exclusiveMinimum": 0, "description": "Instead of width and height"},
                    "resample": {"type": "string", "enum": ["auto", "bicubic", "bilinear", "lanczos", "nearest"], "default": "auto"},
                    "resampleImage": {"type": "boolean", "default": true, "description": "false: the pixels stay and only the resolution changes"},
                    "dpi": {"type": "number", "minimum": 1, "maximum": 10000}
                }),
                &[],
            ))
            .result(json!({"type": "object", "properties": {"width": {"type": "integer"}, "height": {"type": "integer"}, "dpi": {"type": "number"}}})),
        B::new("document.rotate", &["EDT-072", "EDT-119"])
            .mutates()
            .positional(&["degrees"])
            .params(params(
                json!({
                    "documentId": document_id(),
                    "degrees": {"type": "number", "minimum": -359.9, "maximum": 359.9, "description": "Clockwise; 90, 180, 270 or any angle"},
                    "background": {"type": "string", "enum": ["transparent", "canvas"], "default": "transparent", "description": "The new corners of an angle that is not a quarter turn"}
                }),
                &["degrees"],
            ))
            .result(json!({"type": "object"})),
        B::new("document.flip", &["EDT-073", "EDT-119"])
            .mutates()
            .positional(&["direction"])
            .params(params(
                json!({
                    "documentId": document_id(),
                    "direction": {"type": "string", "enum": ["horizontal", "vertical"], "default": "horizontal"},
                    "objectIds": {"type": "array", "items": {"type": "string"}, "description": "Only these objects, around the box that holds them; without it, the image and every object"}
                }),
                &[],
            ))
            .result(json!({"type": "object"})),
    ]
}

#[inline(never)]
#[allow(unused_variables)]
fn specs_4(section: &Value, shortcut_scope: &Value) -> Vec<B> {
    vec![
        B::new("document.trim", &["EDT-051", "EDT-119"])
            .mutates()
            .positional(&["tolerance"])
            .params(params(
                json!({
                    "documentId": document_id(),
                    "tolerance": {"type": "number", "minimum": 0, "maximum": 20, "default": 2, "description": "Percent: how far a border pixel may be from the top-left one"}
                }),
                &[],
            ))
            .result(json!({"type": "object", "properties": {"changed": {"type": "boolean", "description": "False (with a message) when there are no uniform borders"}}})),
        B::new("document.print_pages", &["EDT-114", "EDT-102"])
            .params(params(
                json!({
                    "documentId": document_id(),
                    "paths": {"type": "array", "items": {"type": "string"}, "description": "Or image files (library items), one fitted per page, without opening them (LIB-017)"},
                    "setup": {"type": "object", "description": "{paper: a4 | letter | legal | a3, orientation: auto | portrait | landscape, margin (mm), size: fit | actual | percent, percent, position: center | topLeft}"},
                    "rect": {"type": "object", "description": "Only this part of the canvas {x, y, w, h}: printing a selection"}
                }),
                &[],
            ))
            .result(json!({"type": "object", "properties": {"pages": {"type": "array", "description": "{png (base64), paper {w, h}, at {x, y}, size {w, h}} in millimeters"}}})),
        B::new("document.print", &["EDT-114", "EDT-102", "EDT-119"])
            .params(params(
                json!({
                    "documentId": document_id(),
                    "paths": {"type": "array", "items": {"type": "string"}, "description": "As document.print_pages (LIB-017)"},
                    "setup": {"type": "object", "description": "As document.print_pages"},
                    "rect": {"type": "object", "description": "Only this part of the canvas {x, y, w, h}"},
                    "open": {"type": "boolean", "default": true, "description": "Open the PDF in the system's viewer to print it"}
                }),
                &[],
            ))
            .result(json!({"type": "object", "properties": {"path": {"type": "string"}, "pages": {"type": "integer"}}})),
        B::new("document.batch_export", &["EDT-113", "EDT-119"])
            .params(params(
                json!({
                    "paths": {"type": "array", "items": {"type": "string"}, "description": "Files (.xshot, PNG, JPG) and folders of them, in this order"},
                    "format": {"type": "string", "enum": ["png", "jpg", "gif", "bmp", "tiff", "tif", "pdf", "ico", "psd"]},
                    "options": {"type": "object", "description": "As document.export (EDT-112)"},
                    "effects": {"type": ["string", "array", "null"], "description": "A saved effect's id (EDT-054) or a stack, added to each file's own effects"},
                    "folder": {"type": "string", "description": "Where the files go"},
                    "nextToOriginal": {"type": "boolean", "default": false, "description": "Each next to its source instead"},
                    "naming": {"type": "object", "description": "{mode: keep | base | name, base, start (1), digits (2), prefix, suffix, name}: base gives guia_01, guia_02…; name gives that name (one file, LIB-018)"},
                    "onConflict": {"type": "string", "enum": ["keepBoth", "replace", "skip"], "default": "keepBoth"}
                }),
                &["paths", "format"],
            ))
            .result(json!({"type": "object", "properties": {
                "exported": {"type": "array", "description": "{source, path} of each file written"},
                "failed": {"type": "array", "description": "{source, reason}"},
                "skipped": {"type": "array", "description": "{source, path} that already existed (onConflict: skip)"},
                "cancelled": {"type": "boolean"}
            }})),
        B::new("document.new", &["EDT-103", "EDT-119"])
            .params(params(
                json!({
                    "width": {"type": "integer", "minimum": 1, "maximum": 32000, "description": "Defaults to editor.newImage.width"},
                    "height": {"type": "integer", "minimum": 1, "maximum": 32000, "description": "Defaults to editor.newImage.height"},
                    "color": {"type": ["string", "object", "null"], "description": "Canvas color #RRGGBB(AA), a gradient, or null for transparent; defaults to editor.newImage.color"},
                    "dpi": {"type": "number", "minimum": 1, "maximum": 10000, "default": 96},
                    "remember": {"type": "boolean", "default": false, "description": "Make the size and color the defaults"}
                }),
                &[],
            ))
            .result(json!({"type": "object", "properties": {"documentId": {"type": "string"}, "path": {"type": "string"}}})),
        B::new("document.new_from_clipboard", &["EDT-104", "EDT-119"])
            .params(params(json!({}), &[]))
            .result(json!({"type": "object", "properties": {"documentId": {"type": "string"}, "path": {"type": ["string", "null"]}}})),
        B::new("document.cut_out", &["EDT-052", "EDT-119"])
            .mutates()
            .params(params(
                json!({
                    "documentId": document_id(),
                    "direction": {"type": "string", "enum": ["horizontal", "vertical"], "default": "horizontal", "description": "horizontal: a strip of the whole width (the height changes); vertical: of the whole height"},
                    "from": {"type": "number", "description": "Where the strip starts, in canvas pixels along the direction's axis (y for horizontal)"},
                    "to": {"type": "number", "description": "Where it ends"},
                    "mode": {"type": "string", "enum": ["remove", "add"], "default": "remove", "description": "remove the strip, or add that much space of the canvas color"},
                    "join": {"type": "string", "enum": ["straight", "torn", "wavy", "zigzag"], "default": "straight", "description": "How the parts join; all but straight leave a gap"},
                    "gap": {"type": "number", "minimum": 8, "maximum": 60, "default": 16, "description": "The gap, px"}
                }),
                &["from", "to"],
            ))
            .result(json!({"type": "object", "properties": {
                "crossing": {"type": "array", "items": {"type": "string"}, "description": "Objects across the strip: they stay where they were"}
            }})),
        B::new("document.fit_canvas", &["EDT-051", "EDT-119"])
            .mutates()
            .params(params(json!({"documentId": document_id()}), &[]))
            .result(json!({"type": "object", "properties": {"changed": {"type": "boolean"}}})),
        B::new("document.crop_to_canvas", &["EDT-051", "EDT-119"])
            .mutates()
            .params(params(json!({"documentId": document_id()}), &[]))
            .result(json!({"type": "object", "properties": {
                "changed": {"type": "boolean"},
                "sticking": {"type": "array", "items": {"type": "string"}, "description": "Annotations that still stick out: they are not cut"}
            }})),
        B::new("document.revert", &["EDT-076", "EDT-119"])
            .mutates()
            .params(params(json!({"documentId": document_id()}), &[]))
            .result(json!({"type": "object"})),
        B::new("document.canvas", &["EDT-074", "EDT-075", "EDT-077", "EDT-119"])
            .mutates()
            .params(params(
                json!({
                    "documentId": document_id(),
                    "color": {"type": ["string", "object", "null"], "description": "Canvas color #RRGGBB(AA), a gradient {type: linear | radial, angle, stops: [{color, pos}]} over the whole canvas, or null for transparent"},
                    "cornerRadius": {"type": ["number", "string"], "description": "Rounded corners, px, or \"circle\" (first a centered square)"},
                    "dpi": {"type": "number", "minimum": 1, "maximum": 10000, "description": "Resolution: the pixels do not change"}
                }),
                &[],
            ))
            .result(json!({"type": "object"})),
        B::new("document.canvas_size", &["EDT-070", "EDT-119"])
            .mutates()
            .params(params(
                json!({
                    "documentId": document_id(),
                    "width": {"type": "number", "minimum": 1, "maximum": 32000},
                    "height": {"type": "number", "minimum": 1, "maximum": 32000},
                    "anchor": {"type": "string", "enum": ["topLeft", "top", "topRight", "left", "center", "right", "bottomLeft", "bottom", "bottomRight"], "default": "center", "description": "What stays put: the canvas grows or shrinks on the other sides"}
                }),
                &[],
            ))
            .result(json!({"type": "object"})),
        B::new("document.transform", &["EDT-080", "EDT-083", "EDT-119"])
            .mutates()
            .params(params(
                json!({
                    "documentId": document_id(),
                    "objectIds": {"type": "array", "items": {"type": "string"}},
                    "scale": {
                        "type": "object",
                        "description": "Stretch them as one: from the box `from` (default: the box around them) to `to`, or by `factor` (a number or [x, y]) from its top left corner",
                        "properties": {
                            "from": {"type": "array", "items": {"type": "number"}, "minItems": 4, "maxItems": 4},
                            "to": {"type": "array", "items": {"type": "number"}, "minItems": 4, "maxItems": 4},
                            "factor": {"type": ["number", "array"]}
                        }
                    },
                    "strokes": {"type": "boolean", "description": "With scale: line widths and text sizes too. Default: the editor.scaleStrokes setting"},
                    "rotate": {"type": "number", "description": "Turn them as one, in degrees clockwise"},
                    "around": {"type": "array", "items": {"type": "number"}, "minItems": 2, "maxItems": 2, "description": "With rotate: the center (default: the center of the box around them)"},
                    "equal": {"type": "string", "enum": ["width", "height"], "description": "Give them all the same width or height"},
                    "size": {"type": "number", "minimum": 1, "description": "With equal: the size (default: the largest)"}
                }),
                &["objectIds"],
            ))
            .result(json!({"type": "object", "properties": {"objects": {"type": "array"}}})),
        B::new("document.group", &["EDT-082", "EDT-119"])
            .mutates()
            .params(params(
                json!({
                    "documentId": document_id(),
                    "objectIds": {"type": "array", "items": {"type": "string"}},
                    "ungroup": {"type": "boolean", "default": false, "description": "Take them out of their outermost group instead"}
                }),
                &["objectIds"],
            ))
            .result(json!({"type": "object", "properties": {"group": {"type": ["string", "null"]}}})),
        B::new("document.duplicate", &["EDT-088", "EDT-119"])
            .mutates()
            .params(params(
                json!({
                    "documentId": document_id(),
                    "objectId": {"type": "string"},
                    "objectIds": {"type": "array", "items": {"type": "string"}},
                    "offset": {"type": "array", "items": {"type": "number"}, "minItems": 2, "maxItems": 2, "description": "Where the copies go from the originals, [dx, dy] (default [10, 10])"}
                }),
                &[],
            ))
            .result(json!({"type": "object", "properties": {"ids": {"type": "array", "items": {"type": "string"}}}})),
        B::new("document.delete_object", &["EDT-119", "EDT-042"])
            .mutates()
            .params(params(
                json!({
                    "documentId": document_id(),
                    "objectId": {"type": "string"},
                    "objectIds": {"type": "array", "items": {"type": "string"}}
                }),
                &[],
            ))
            .result(json!({"type": "object", "properties": {"deleted": {"type": "array"}}})),
        B::new("document.undo", &["EDT-092"])
            .scopes(ALL)
            .assignable()
            .mutates()
            .params(params(json!({"documentId": document_id()}), &[]))
            .result(history_result()),
        B::new("document.redo", &["EDT-092"])
            .scopes(ALL)
            .assignable()
            .mutates()
            .params(params(json!({"documentId": document_id()}), &[]))
            .result(history_result()),
        B::new("document.save", &["EDT-107", "EDT-106", "LIB-021"])
            .scopes(ALL)
            .assignable()
            .mutates()
            .params(params(
                json!({
                    "documentId": document_id(),
                    "path": {"type": "string", "description": "Save as: another file. Omit to save in its own file"},
                    "format": {"type": "string", "enum": ["xshot", "png", "jpg"], "description": "Default: from the path extension"},
                    "confirmFlatten": {"type": "boolean", "default": false, "description": "The user accepted that a PNG or JPG keeps the objects flattened (UI)"},
                    "dontAskAgain": {"type": "boolean", "default": false, "description": "Do not warn about flattening again (UI)"}
                }),
                &[],
            ))
            .result(json!({
                "type": "object",
                "properties": {
                    "status": {"type": "string", "enum": ["saved", "needs_decision"]},
                    "reason": {"type": "string", "enum": ["flatten"]},
                    "path": {"type": "string"},
                    "linked": {"type": "boolean"}
                }
            })),
        B::new("document.export", &["EDT-107", "EDT-110"])
            .mutates()
            .params(params(
                json!({
                    "documentId": document_id(),
                    "documentIds": {"type": "array", "items": {"type": "string"}, "description": "Several documents in one PDF or TIFF, one page each, in this order (EDT-111)"},
                    "format": {"type": "string", "enum": ["png", "jpg", "gif", "bmp", "tiff", "tif", "pdf", "ico", "psd"]},
                    "path": {"type": "string", "description": "Required unless estimate"},
                    "quality": {"type": "integer", "minimum": 1, "maximum": 100, "description": "JPG quality (default 90); also options.quality"},
                    "options": {"type": "object", "description": "Format options (EDT-112): quality, pngCompression (fast | max), pngColors256, gifTransparent, tiffCompression (none | lzw | zip), pdfPage (auto | a4 | letter | legal | a3), pdfOrientation (auto | portrait | landscape), pdfMargin (mm), pdfFitWidth, pdfSinglePage, pdfLossless, icoSizes ([16…256]), scale (25–400 %), colorProfile"},
                    "estimate": {"type": "boolean", "default": false, "description": "Only say how many bytes it would take; nothing is written"},
                    "onConflict": {"type": "string", "enum": ["keepBoth", "replace", "fail"], "default": "keepBoth"}
                }),
                &["format"],
            ))
            .result(json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string"},
                    "width": {"type": "integer"},
                    "height": {"type": "integer"},
                    "bytes": {"type": "integer"},
                    "pages": {"type": "integer", "description": "Images in the file"}
                }
            })),
        B::new("document.crop", &["EDT-049", "EDT-050", "EDT-119"])
            .scopes(ALL)
            .mutates()
            .params(params(
                json!({
                    "documentId": document_id(),
                    "rect": {
                        "type": "object",
                        "description": "New canvas in canvas pixels. It may reach past the canvas, up to the edges of the original image (EDT-050)",
                        "properties": {
                            "x": {"type": "number"}, "y": {"type": "number"},
                            "w": {"type": "number", "minimum": 1}, "h": {"type": "number", "minimum": 1}
                        },
                        "required": ["x", "y", "w", "h"],
                        "additionalProperties": false
                    },
                    "trim": {"type": "boolean", "description": "Trim plain borders (EDT-051, M2)"}
                }),
                &[],
            ))
            .result(json!({
                "type": "object",
                "properties": {
                    "width": {"type": "integer"},
                    "height": {"type": "integer"},
                    "outside": {"type": "integer", "description": "Objects now partly or wholly outside the canvas"}
                }
            })),
        B::new("document.copy_objects", &["EDT-089"])
            .scopes(ALL)
            .assignable()
            .params(params(
                json!({
                    "documentId": document_id(),
                    "objectIds": {"type": "array", "items": {"type": "string"}},
                    "cut": {"type": "boolean", "default": false, "description": "Cut: copy and delete them"}
                }),
                &["objectIds"],
            ))
            .result(json!({"type": "object", "properties": {"count": {"type": "integer"}}})),
        B::new("document.insert_images", &["EDT-078", "CAP-067"])
            .mutates()
            .params(params(
                json!({
                    "documentId": document_id(),
                    "paths": {"type": "array", "items": {"type": "string"}, "minItems": 1, "description": "Library documents (.xshot) or PNG/JPG files"},
                    "center": {"type": "array", "items": {"type": "number"}, "description": "Canvas point [x, y] the first image is centered on (default: the canvas center)"}
                }),
                &["paths"],
            ))
            .result(json!({
                "type": "object",
                "properties": {
                    "documentId": {"type": "string"},
                    "ids": {"type": "array", "items": {"type": "string"}},
                    "created": {"type": "boolean", "description": "A new document was made for them"}
                }
            })),
        B::new("document.rename", &["SMT-041", "TRY-006"])
            .mutates()
            .params(params(
                json!({
                    "documentId": document_id(),
                    "name": {"type": "string", "minLength": 1, "description": "New file name, without folder or extension"}
                }),
                &["name"],
            ))
            .result(json!({"type": "object", "properties": {"path": {"type": "string"}, "name": {"type": "string"}}})),
        B::new("document.locate", &["TRY-002"])
            .mutates()
            .params(params(
                json!({
                    "documentId": document_id(),
                    "path": {"type": "string", "description": "Where the file is now"}
                }),
                &["path"],
            ))
            .result(document_summary()),
        B::new("document.image", &["SMT-043", "APP-029"])
            .params(params(
                json!({
                    "documentId": document_id(),
                    "maxSize": {"type": "integer", "minimum": 64, "maximum": 2576, "default": 1568, "description": "Longest side in pixels; larger images are scaled down"}
                }),
                &[],
            ))
            .result(json!({
                "type": "object",
                "properties": {
                    "image": {"type": "object", "description": "PNG: {mimeType, data (base64)}. MCP clients get it as an image"},
                    "width": {"type": "integer"},
                    "height": {"type": "integer"},
                    "name": {"type": "string"},
                    "path": {"type": "string"}
                }
            })),
        B::new("document.paste", &["EDT-089"])
            .scopes(ALL)
            .assignable()
            .mutates()
            .params(params(
                json!({
                    "documentId": document_id(),
                    "inPlace": {"type": "boolean", "default": false, "description": "Paste copied objects at their original place"},
                    "center": {"type": "array", "items": {"type": "number"}, "description": "Canvas point [x, y] to center what is pasted on (the middle of the view)"}
                }),
                &[],
            ))
            .result(json!({
                "type": "object",
                "properties": {
                    "pasted": {"type": "string", "enum": ["objects", "text", "document"]},
                    "documentId": {"type": "string"},
                    "ids": {"type": "array", "items": {"type": "string"}}
                }
            })),
        B::new("document.copy_all", &["EDT-090", "SHR-019"])
            .scopes(ALL)
            .assignable()
            .params(params(json!({"documentId": document_id()}), &[])),
        B::new("document.close", &["EDT-108", "EDT-012"])
            .scopes(ALL)
            .assignable()
            .mutates()
            .params(params(
                json!({
                    "documentId": document_id(),
                    "documentIds": {"type": "array", "items": {"type": "string"}, "description": "Several documents at once (TRY-008)"},
                    "all": {"type": "boolean", "default": false},
                    "save": {"type": "array", "items": {"type": "string"}, "description": "Documents with unsaved changes to save first"},
                    "discard": {"type": "array", "items": {"type": "string"}, "description": "Documents whose unsaved changes may be lost"}
                }),
                &[],
            ))
            .result(json!({
                "type": "object",
                "properties": {
                    "status": {"type": "string", "enum": ["closed", "needs_decision"]},
                    "closed": {"type": "array", "items": {"type": "string"}},
                    "pending": {"type": "array", "items": {"type": "object"}}
                }
            })),
        B::new("editor.show", &["EDT-001", "EDT-119"])
            .scopes(ALL)
            .params(params(
                json!({
                    "view": {"type": "string", "enum": ["editor", "library"], "default": "editor"},
                    "documentId": document_id()
                }),
                &[],
            )),
        B::new("editor.state", &["EDT-014", "EDT-003"]).result(json!({"type": "object"})),
        B::new("editor.fonts", &["EDT-026"]).result(json!({
            "type": "object",
            "properties": {"fonts": {"type": "array", "items": {"type": "string"}}}
        })),
    ]
}

#[inline(never)]
#[allow(unused_variables)]
fn specs_5(section: &Value, shortcut_scope: &Value) -> Vec<B> {
    vec![
        B::new("editor.select_tool", &["EDT-003", "EDT-119"])
            .scopes(ALL)
            .positional(&["tool"])
            .params(params(json!({"tool": {"type": "string", "enum": TOOLS}}), &["tool"]))
            .result(json!({"type": "object", "properties": {"tool": {"type": "string"}}})),
        B::new("editor.action", &["EDT-002", "EDT-116", "EDT-119"])
            .scopes(ALL)
            .assignable()
            .positional(&["action"])
            .params(params(
                json!({"action": {"type": "string", "enum": EDITOR_ACTIONS.as_slice()}}),
                &["action"],
            ))
            .result(json!({"type": "object"})),
        B::new("editor.toolbar", &["EDT-003", "EDT-004"])
            .params(params(
                json!({
                    "action": {"type": "string", "enum": ["get", "set", "reset"], "default": "get"},
                    "items": {"type": "array", "items": {"type": "string"}, "description": "For set: tool:<id>, action:<editor action> (commands and action:effect_…), sep or space, in order"}
                }),
                &[],
            ))
            .result(json!({"type": "object", "properties": {
                "items": {"type": "array", "items": {"type": "string"}},
                "default": {"type": "array", "items": {"type": "string"}},
                "available": {"type": "object", "description": "{tools, actions, effects} that can go on it"}
            }})),
        B::new("editor.templates", &["EDT-095", "EDT-096", "EDT-101"])
            .mutates()
            .positional(&["action"])
            .params(params(
                json!({
                    "action": {"type": "string", "enum": ["list", "rename", "duplicate", "delete", "import", "export"], "default": "list"},
                    "id": {"type": "string", "description": "An own template, own:<name> (rename, duplicate, delete, export)"},
                    "name": {"type": "string", "description": "The new name (rename)"},
                    "path": {"type": "string", "description": "The .xtemplate file to import, or where to export"}
                }),
                &[],
            ))
            .result(json!({"type": "object", "properties": {
                "templates": {"type": "array", "description": "{id, name, family, min, max, own} in the picker's order; own ones add path and zones"},
                "max": {"type": "integer", "description": "Most images a template takes"},
                "folder": {"type": "string"},
                "result": {"description": "The new id (import, rename, duplicate) or the exported file"}
            }})),
        B::new("document.from_template", &["EDT-095", "EDT-096", "EDT-097"])
            .positional(&["template"])
            .params(params(
                json!({
                    "template": {"type": "string", "description": "A built-in template id (editor.templates), own:<name> or a .xtemplate file"},
                    "images": {"type": "array", "maxItems": 100, "items": {"type": ["string", "object"]}, "description": "Files (paths) or {documentId} of open documents, in zone order; the first 20 are used"},
                    "settings": {"type": "object", "description": "Template settings (EDT-097): scale fit|fill, title, captions, footer, author system|custom, authorText, font, textColor, background, numbering, numberColor, numberShape, reversed, theme"}
                }),
                &["template"],
            ))
            .result(json!({"type": "object", "properties": {
                "documentId": {"type": "string"},
                "path": {"type": "string"},
                "zones": {"type": "integer"},
                "placed": {"type": "integer"},
                "warnings": {"type": "array", "items": {"type": "string"}}
            }})),
        B::new("document.template", &["EDT-097", "EDT-098", "EDT-099", "EDT-101"])
            .mutates()
            .positional(&["action"])
            .params(params(
                json!({
                    "documentId": document_id(),
                    "action": {"type": "string", "enum": ["info", "settings", "reverse", "add_section", "remove_section", "resize_section", "move_section", "fill_zone", "clear_zone", "swap_zones", "frame_zone", "remove_caption", "convert_zone"], "default": "info"},
                    "settings": {"type": "object", "description": "settings: the settings to change (EDT-097)"},
                    "kind": {"type": "string", "enum": ["step", "imageText", "text", "twoImages", "title", "footer"], "description": "add_section: its kind"},
                    "index": {"type": "integer", "minimum": 0, "description": "add_section, move_section: its place among the sections"},
                    "sectionId": {"type": "string"},
                    "height": {"type": "number", "minimum": 40, "maximum": 5000, "description": "resize_section"},
                    "zone": {"type": "string", "description": "A zone: its object id or slot (zone:<section>:<n>)"},
                    "with": {"type": "string", "description": "swap_zones: the other zone"},
                    "path": {"type": "string", "description": "fill_zone: an image or .xshot file"},
                    "sourceDocumentId": {"type": "string", "description": "fill_zone: an open document's final image"},
                    "fromObject": {"type": "string", "description": "fill_zone: a floating image that goes into the zone"},
                    "clipboard": {"type": "boolean", "description": "fill_zone: the image on the clipboard"},
                    "zoom": {"type": "number", "minimum": 0.1, "maximum": 10, "description": "frame_zone: 1 = fitted or filled"},
                    "zoomBy": {"type": "number", "description": "frame_zone: steps of 10 % (+1, -1)"},
                    "offset": {"type": "array", "items": {"type": "number"}, "description": "frame_zone: [x, y] px from the centered place"},
                    "reset": {"type": "boolean", "description": "frame_zone: back to as placed"},
                    "objectId": {"type": "string", "description": "convert_zone: a floating image"}
                }),
                &[],
            ))
            .result(json!({"type": "object", "properties": {
                "isTemplate": {"type": "boolean"},
                "template": {"type": ["object", "null"]},
                "sections": {"type": "array", "description": "{id, kind, top, height} of the sections shown"},
                "zones": {"type": "array", "description": "{objectId, slot, section, rect, filled, zoom}"}
            }})),
        B::new("document.save_as_template", &["EDT-101"])
            .positional(&["name"])
            .params(params(
                json!({
                    "documentId": document_id(),
                    "name": {"type": "string"},
                    "family": {"type": "string", "enum": ["steps", "timeline", "compare", "collage", "mine"], "default": "mine"},
                    "emptyZones": {"type": "boolean", "default": true},
                    "replace": {"type": "boolean", "default": false, "description": "Overwrite an own template of that name"}
                }),
                &["name"],
            ))
            .result(json!({"type": "object", "properties": {"id": {"type": "string"}, "path": {"type": "string"}}})),
        B::new("document.apply_theme", &["EDT-100"])
            .mutates()
            .positional(&["themeId"])
            .params(params(
                json!({
                    "documentId": document_id(),
                    "themeId": {"type": "string"},
                    "objectIds": {"type": "array", "items": {"type": "string"}, "description": "Only these; without it, every object"},
                    "fromThemeId": {"type": "string", "description": "The theme the colors come from; default: the template's or the active one"}
                }),
                &["themeId"],
            ))
            .result(json!({"type": "object", "properties": {"changed": {"type": "integer"}}})),
        B::new("editor.clipboard", &["EDT-103", "EDT-104"])
            .params(params(json!({}), &[]))
            .result(json!({"type": "object", "properties": {
                "image": {"type": ["object", "null"], "description": "{width, height} of the image in the clipboard"},
                "files": {"type": "integer", "description": "Image files copied in the file manager"},
                "text": {"type": "boolean"}
            }})),
        B::new("editor.cancel", &["EDT-045"])
            .params(params(json!({}), &[]))
            .result(json!({"type": "object", "properties": {"cancelled": {"type": "boolean", "description": "Whether something was running"}}})),
        B::new("editor.menu_state", &["EDT-002"])
            .scopes(UI)
            .params(params(
                json!({
                    "enabled": {"type": "object", "description": "Menu entry id -> whether it can be used now"},
                    "checked": {"type": "object", "description": "Menu entry id -> whether it is ticked"}
                }),
                &[],
            )),
        B::new("editor.set_style", &["EDT-014"])
            .mutates()
            .params(params(
                json!({"tool": {"type": "string", "enum": TOOLS}, "style": {"type": "object"}}),
                &["tool", "style"],
            ))
            .result(json!({"type": "object", "properties": {"style": {"type": "object"}}})),
        B::new("editor.styles", &["EDT-015", "EDT-016"])
            .params(params(json!({}), &[]))
            .result(json!({"type": "object", "properties": {
                "theme": {"type": "object"},
                "tools": {"type": "object", "description": "Saved styles of the active theme by tool: [{id, name, style}]"},
                "favorites": {"type": "array", "description": "[{id, tool, style}]"}
            }})),
        B::new("editor.style", &["EDT-015", "EDT-016"])
            .positional(&["action"])
            .params(params(
                json!({
                    "action": {"type": "string", "enum": ["apply", "save", "update", "rename", "delete", "move", "reset", "favorite", "unfavorite", "move_favorite", "apply_favorite"], "description": "apply: the tool takes the style (and objectIds, if given); save: the tool's current style as a new one; update: a style becomes the current one; reset: back to the theme's styles of the tool; favorite: a copy in Favorites (of styleId, or of the current style)"},
                    "tool": {"type": "string", "enum": ["arrow", "line", "text", "callout", "shape", "step", "highlight", "pen", "magnify", "spotlight", "blur"]},
                    "styleId": {"type": "string"},
                    "favoriteId": {"type": "string"},
                    "name": {"type": "string"},
                    "to": {"type": "integer", "minimum": 0, "description": "move: the new position, 0 is first"},
                    "documentId": document_id(),
                    "objectIds": {"type": "array", "items": {"type": "string"}, "description": "apply: objects of that tool that take the style too (not their position, size or text)"}
                }),
                &["action"],
            ))
            .result(json!({"type": "object"})),
        B::new("editor.colors", &["EDT-019"])
            .scopes(ALL)
            .params(params(json!({}), &[]))
            .result(json!({"type": "object", "properties": {"mine": {"type": "array", "items": {"type": "string"}}, "recent": {"type": "array", "items": {"type": "string"}}, "gradients": {"type": "array"}}})),
        B::new("editor.color", &["EDT-019"])
            .scopes(ALL)
            .positional(&["action"])
            .params(params(
                json!({
                    "action": {"type": "string", "enum": ["save", "remove", "used", "save_gradient", "remove_gradient"], "description": "save: add to My colors (up to 24); remove: take it out; used: first of the 8 recent colors; save_gradient / remove_gradient: My gradients (up to 16)"},
                    "color": {"type": "string", "description": "#RRGGBB or #RRGGBBAA"},
                    "gradient": {"type": "object", "description": "{type: linear | radial, angle, stops: [{color, pos}]}"}
                }),
                &["action"],
            ))
            .result(json!({"type": "object", "properties": {"mine": {"type": "array"}, "recent": {"type": "array"}}})),
        B::new("editor.themes", &["EDT-017"])
            .params(params(json!({}), &[]))
            .result(json!({"type": "object", "properties": {"active": {"type": "string"}, "themes": {"type": "array"}}})),
        B::new("editor.theme", &["EDT-017", "EDT-018"])
            .positional(&["action"])
            .params(params(
                json!({
                    "action": {"type": "string", "enum": ["activate", "save", "preview", "duplicate", "rename", "delete", "export", "import"], "description": "preview: how `theme` would look, without saving it (PNG samples)"},
                    "id": {"type": "string", "description": "The theme; save without it creates one"},
                    "theme": {"type": "object", "description": "save: {name, colors (2 to 8, none twice), font (empty: the system's), shadow, shadowOff (tools)}; its styles are made again", "properties": {
                        "name": {"type": "string"},
                        "colors": {"type": "array", "items": {"type": "string"}, "minItems": 2, "maxItems": 8},
                        "font": {"type": ["string", "null"]},
                        "shadow": {"type": "boolean"},
                        "shadowOff": {"type": "array", "items": {"type": "string"}}
                    }},
                    "name": {"type": "string"},
                    "path": {"type": "string", "description": "export or import: a .xtheme file"},
                    "onConflict": {"type": "string", "enum": ["replace", "keep"], "description": "import: a theme of the user with that name exists"}
                }),
                &["action"],
            ))
            .result(json!({"type": "object"})),
        B::new("editor.zoom", &["EDT-007", "EDT-119"])
            .scopes(ALL)
            .assignable()
            .params(params(
                json!({
                    "mode": {"type": "string", "enum": ["actual", "fit", "original", "in", "out", "percent"]},
                    "value": {"type": "number", "minimum": 10, "maximum": 3200, "description": "Percent, with mode percent"},
                    "documentId": document_id()
                }),
                &["mode"],
            ))
            .result(json!({"type": "object", "properties": {"zoom": {"type": "number"}}})),
        B::new("editor.view", &["EDT-007"])
            .scopes(UI)
            .params(params(
                json!({
                    "documentId": {"type": "string"},
                    "zoom": {"type": "number"},
                    "x": {"type": "number"},
                    "y": {"type": "number"},
                    "viewportWidth": {"type": "number"},
                    "viewportHeight": {"type": "number"},
                    "dpr": {"type": "number"}
                }),
                &["documentId", "zoom", "x", "y", "viewportWidth", "viewportHeight", "dpr"],
            )),
        // Cloud AI (SMT-039 to SMT-042): never from an agent, only on the user's request.
        B::new("ai.key_set", &["SMT-039"])
            .scopes(UI)
            .mutates()
            .params(params(json!({"key": {"type": "string", "minLength": 1}}), &["key"]))
            .result(ai_key_status()),
        // The library (LIB-); the rest of the area arrives with M2 (LIB-034).
        B::new("library.list", &["LIB-001", "LIB-002", "LIB-004", "LIB-005", "LIB-006", "LIB-007", "LIB-022", "LIB-034"])
            .params(params(
                json!({
                    "sort": {"type": "string", "enum": ["modified", "captured", "name", "size", "type", "app"], "default": "modified"},
                    "order": {"type": "string", "enum": ["asc", "desc"], "description": "Default: newest and largest first, names A to Z"},
                    "query": {"type": "string", "description": "Search (LIB-007): words by prefix in any field, a quoted phrase, tag:, app:, site:, type: (image, video, gif, guide); no case or accents"},
                    "filters": {"type": "object", "description": "{types, dates (2026, 2026-11, 2026-11-12, today, yesterday, last7, month), apps, sites, tags, favorites}: values of a section add up, sections cross", "properties": {
                        "types": {"type": "array", "items": {"type": "string"}},
                        "dates": {"type": "array", "items": {"type": "string"}},
                        "apps": {"type": "array", "items": {"type": "string"}},
                        "sites": {"type": "array", "items": {"type": "string"}},
                        "tags": {"type": "array", "items": {"type": "string"}},
                        "favorites": {"type": "boolean"}
                    }},
                    "offset": {"type": "integer", "minimum": 0, "default": 0},
                    "limit": {"type": "integer", "minimum": 1, "description": "Default: all"}
                }),
                &[],
            ))
            .result(json!({
                "type": "object",
                "properties": {
                    "root": {"type": "string"},
                    "synced": {"type": "boolean"},
                    "total": {"type": "integer", "description": "Items in the library"},
                    "count": {"type": "integer", "description": "Items that pass the filters and the search"},
                    "items": {"type": "array", "items": library_item()},
                    "facets": {"type": "object", "description": "Counts of each filter value over the other sections: types, dates (years, months, days), quickDates, apps, sites, tags, favorites"}
                }
            })),
        B::new("library.search", &["LIB-034", "LIB-005", "LIB-007"])
            .positional(&["query"])
            .params(params(
                json!({
                    "query": {"type": "string", "description": "Words of the name, tags, app, site or annotations (LIB-007 rules)"},
                    "tag": {"type": ["string", "array"], "items": {"type": "string"}},
                    "type": {"type": ["string", "array"], "items": {"type": "string"}, "description": "image, video, gif, guide"},
                    "app": {"type": ["string", "array"], "items": {"type": "string"}, "description": "Part of the application's name"},
                    "site": {"type": ["string", "array"], "items": {"type": "string"}, "description": "Part of the website"},
                    "from": {"type": "string", "description": "First capture day, YYYY-MM-DD"},
                    "to": {"type": "string", "description": "Last capture day, YYYY-MM-DD"},
                    "favorite": {"type": "boolean"},
                    "sort": {"type": "string", "enum": ["modified", "captured", "name", "size", "type", "app"]},
                    "order": {"type": "string", "enum": ["asc", "desc"]},
                    "offset": {"type": "integer", "minimum": 0},
                    "limit": {"type": "integer", "minimum": 1}
                }),
                &[],
            ))
            .result(json!({"type": "object", "properties": {
                "total": {"type": "integer"},
                "count": {"type": "integer"},
                "items": {"type": "array", "items": library_item()}
            }})),
        B::new("library.reveal", &["LIB-014", "LIB-034"])
            .scopes(UI_CLI_MCP)
            .params(params(
                json!({
                    "id": {"type": "string"},
                    "path": {"type": "string", "description": "A file of the library, instead of its ID"}
                }),
                &[],
            ))
            .result(library_item()),
        // The recent tray (TRY-017): the documents shown in the editor, in order.
        B::new("tray.list", &["TRY-017", "TRY-002"])
            .params(params(json!({}), &[]))
            .result(json!({"type": "object", "properties": {
                "items": {"type": "array", "description": "{documentId, name, path, libraryId, pinned, dirty, active, missing}, left to right"},
                "capacity": {"type": "integer"}
            }})),
        B::new("tray.open", &["TRY-017"])
            .mutates()
            .positional(&["id"])
            .params(params(
                json!({
                    "id": {"type": "string", "description": "A library item (library.list)"},
                    "path": {"type": "string", "description": "Or a file"},
                    "activate": {"type": "boolean", "default": true, "description": "Show it in the editor; false only puts it in the tray"}
                }),
                &[],
            ))
            .result(json!({"type": "object", "properties": {"documentId": {"type": "string"}}})),
        B::new("tray.activate", &["TRY-017", "TRY-013"])
            .positional(&["documentId"])
            .params(params(json!({"documentId": {"type": "string"}}), &["documentId"]))
            .result(json!({"type": "object", "properties": {"documentId": {"type": "string"}}})),
        B::new("tray.pin", &["TRY-017", "TRY-004"])
            .mutates()
            .positional(&["documentId"])
            .params(params(
                json!({"documentId": document_id(), "documentIds": {"type": "array", "items": {"type": "string"}}}),
                &[],
            ))
            .result(json!({"type": "object", "properties": {"changed": {"type": "boolean"}, "items": {"type": "array"}}})),
        B::new("tray.unpin", &["TRY-017", "TRY-004"])
            .mutates()
            .positional(&["documentId"])
            .params(params(
                json!({"documentId": document_id(), "documentIds": {"type": "array", "items": {"type": "string"}}}),
                &[],
            ))
            .result(json!({"type": "object", "properties": {"changed": {"type": "boolean"}, "items": {"type": "array"}}})),
        B::new("tray.move", &["TRY-017", "TRY-005"])
            .mutates()
            .positional(&["documentId", "index"])
            .params(params(
                json!({
                    "documentId": {"type": "string"},
                    "index": {"type": "integer", "minimum": 0, "description": "Its new place, 0 is the left end; among the pinned ones it is pinned, among the rest unpinned"}
                }),
                &["documentId", "index"],
            ))
            .result(json!({"type": "object", "properties": {"index": {"type": "integer"}, "pinned": {"type": "boolean"}}})),
        B::new("tray.close", &["TRY-017", "TRY-008"])
            .mutates()
            .positional(&["documentId"])
            .params(params(
                json!({
                    "documentId": document_id(),
                    "documentIds": {"type": "array", "items": {"type": "string"}},
                    "which": {"type": "string", "enum": ["this", "others", "all", "unpinned"], "default": "this", "description": "this: the documents named; others: all but them; all; unpinned"},
                    "save": {"type": "array", "items": {"type": "string"}, "description": "As document.close"},
                    "discard": {"type": "array", "items": {"type": "string"}, "description": "As document.close"}
                }),
                &[],
            ))
            .result(json!({"type": "object", "properties": {
                "status": {"type": "string", "enum": ["closed", "needs_decision"]},
                "closed": {"type": "array", "items": {"type": "string"}},
                "pending": {"type": "array"}
            }})),
        B::new("library.duplicate", &["TRY-010", "LIB-034"])
            .mutates()
            .positional(&["ids"])
            .params(params(
                json!({
                    "ids": {"type": "array", "items": {"type": "string"}, "description": "Library items"},
                    "documentIds": {"type": "array", "items": {"type": "string"}, "description": "Or open documents (their unsaved changes go into the copy)"},
                    "paths": {"type": "array", "items": {"type": "string"}, "description": "Or files"},
                    "open": {"type": "boolean", "default": false, "description": "Put the copies in the tray next to their originals; a single copy is shown"}
                }),
                &[],
            ))
            .result(json!({"type": "object", "properties": {
                "copies": {"type": "array", "description": "{source, path, id, documentId} of each copy"}
            }})),
    ]
}

#[inline(never)]
#[allow(unused_variables)]
fn specs_6(section: &Value, shortcut_scope: &Value) -> Vec<B> {
    vec![
        B::new("library.import", &["LIB-024", "LIB-034"])
            .mutates()
            .positional(&["paths"])
            .params(params(
                json!({
                    "paths": {"type": "array", "items": {"type": "string"}, "description": "Files (.xshot, PNG, JPG) and folders of them; they are copied, the originals do not change"},
                    "tagFolders": {"type": "boolean", "default": true, "description": "A folder's name becomes a tag of the images that came from it"}
                }),
                &["paths"],
            ))
            .result(json!({"type": "object", "properties": {
                "imported": {"type": "array", "description": "{source, path, id} of each copy"},
                "skipped": {"type": "array", "description": "{source, reason}: unsupported, in_library or unreadable"}
            }})),
        B::new("library.merge", &["LIB-031", "LIB-034"])
            .mutates()
            .positional(&["path"])
            .params(params(
                json!({
                    "path": {"type": "string", "description": "The other library's folder; nothing in it changes"},
                    "background": {"type": "boolean", "default": false, "description": "Return at once and report with library_merge events (the window)"},
                    "cancel": {"type": "boolean", "description": "Cancel the merge in progress: what was copied is removed"}
                }),
                &[],
            ))
            .result(json!({"type": "object", "properties": {
                "added": {"type": "integer", "description": "Items copied here"},
                "duplicates": {"type": "integer", "description": "Items not copied: the same file is already here"},
                "enriched": {"type": "integer", "description": "Items here that gained tags or the favorite from a duplicate"},
                "renamed": {"type": "array", "description": "{from, path}: copies that got \" (2)\" because the name was taken"},
                "failed": {"type": "array", "items": {"type": "string"}},
                "cancelled": {"type": "boolean"},
                "started": {"type": "boolean"},
                "total": {"type": "integer"}
            }})),
        B::new("library.export", &["LIB-033", "LIB-018", "LIB-034"])
            .positional(&["folder"])
            .params(params(
                json!({
                    "folder": {"type": "string", "description": "Where the copy goes: a folder outside the library"},
                    "format": {"type": "string", "enum": ["png", "jpg"], "default": "png", "description": "What .xshot images become, with their annotations; PNG and JPG files are copied as they are"},
                    "ids": {"type": "array", "items": {"type": "string"}, "description": "Only these items"},
                    "filters": {"type": "object", "description": "Only what these filters show, as library.list"},
                    "query": {"type": "string", "description": "Only what this search shows, as library.list"},
                    "metadata": {"type": "boolean", "default": true, "description": "Add a CSV with each item's name, dates, app, website and tags"}
                }),
                &["folder"],
            ))
            .result(json!({"type": "object", "properties": {
                "folder": {"type": "string"},
                "exported": {"type": "integer", "description": ".xshot images written as PNG or JPG"},
                "copied": {"type": "integer", "description": "PNG and JPG files copied"},
                "failed": {"type": "array", "description": "{source, name, reason}"},
                "cancelled": {"type": "boolean", "description": "Stopped with editor.cancel; what was written stays"},
                "csv": {"type": ["string", "null"]}
            }})),
        B::new("library.resolve", &["LIB-029", "LIB-034"])
            .mutates()
            .positional(&["id"])
            .params(params(
                json!({
                    "id": {"type": "string", "description": "A conflict copy (conflict: true in library.list)"},
                    "keep": {"type": "string", "enum": ["this", "original", "both"], "default": "both"}
                }),
                &["id"],
            ))
            .result(json!({"type": "object", "properties": {"kept": {"type": "string"}, "undo": {"type": ["string", "null"]}}})),
        B::new("library.tag", &["LIB-009", "LIB-034"])
            .mutates()
            .params(params(
                json!({
                    "ids": {"type": "array", "items": {"type": "string"}, "minItems": 1},
                    "add": {"type": ["array", "string"], "items": {"type": "string"}, "description": "Tags to add: 1 to 50 characters; case does not make a new tag"},
                    "remove": {"type": ["array", "string"], "items": {"type": "string"}}
                }),
                &["ids"],
            ))
            .result(json!({"type": "object", "properties": {"items": {"type": "array", "items": library_item()}}})),
        B::new("library.tags", &["LIB-010", "LIB-034"])
            .mutates()
            .positional(&["action"])
            .params(params(
                json!({
                    "action": {"type": "string", "enum": ["list", "rename", "merge", "delete"], "default": "list"},
                    "tag": {"type": "string"},
                    "to": {"type": "string", "description": "rename: the new name"},
                    "merge": {"type": "boolean", "default": false, "description": "rename onto a tag that exists: merge them"},
                    "into": {"type": "string", "description": "merge: the tag that stays"},
                    "sort": {"type": "string", "enum": ["name", "count"], "default": "name"}
                }),
                &[],
            ))
            .result(json!({"type": "object", "properties": {
                "tags": {"type": "array", "description": "{tag, count}"},
                "changed": {"type": ["integer", "null"], "description": "Items changed"}
            }})),
        B::new("library.location", &["LIB-027", "LIB-028", "LIB-034"])
            .positional(&["action"])
            .params(params(
                json!({
                    "action": {"type": "string", "enum": ["get", "inspect"], "default": "get"},
                    "path": {"type": "string", "description": "inspect: a folder, what it holds"}
                }),
                &[],
            ))
            .result(json!({"type": "object", "properties": {
                "path": {"type": "string"},
                "default": {"type": "string"},
                "synced": {"type": "boolean"},
                "moving": {"type": "boolean"},
                "items": {"type": "integer", "description": "inspect: items in the folder"},
                "library": {"type": "boolean", "description": "inspect: it holds a library of XcreenShot"},
                "size": {"type": "integer", "description": "inspect: bytes of the current library"},
                "free": {"type": ["integer", "null"], "description": "inspect: bytes free on that folder's disk"},
                "enough": {"type": "boolean", "description": "inspect: room to move the library there (always on the same disk)"},
                "bigSync": {"type": "boolean", "description": "inspect: a synced folder and a library of more than 5 GB"}
            }})),
        B::new("library.relocate", &["LIB-028"])
            .scopes(UI)
            .mutates()
            .params(params(
                json!({
                    "path": {"type": "string", "description": "The new folder"},
                    "mode": {"type": "string", "enum": ["move", "empty", "use"], "default": "move", "description": "move the items there, start an empty library there, or use the library already there"},
                    "cancel": {"type": "boolean", "description": "Cancel the move in progress: what was moved goes back"},
                    "retry": {"type": "array", "items": {"type": "string"}, "description": "Files the last move left behind, to move again"}
                }),
                &[],
            ))
            .result(json!({"type": "object", "properties": {"path": {"type": "string"}, "started": {"type": "boolean"}, "total": {"type": "integer"}}})),
        B::new("library.rebuild", &["LIB-023", "LIB-034"])
            .mutates()
            .params(params(json!({}), &[]))
            .result(json!({"type": "object", "properties": {"items": {"type": "integer"}}})),
        B::new("library.check", &["LIB-023", "LIB-034"])
            .mutates()
            .positional(&["action"])
            .params(params(
                json!({"action": {"type": "string", "enum": ["report", "clean"], "default": "report", "description": "clean: delete sidecars whose file is gone"}}),
                &[],
            ))
            .result(json!({"type": "object", "properties": {
                "damaged": {"type": "array", "items": {"type": "string"}, "description": "Files that cannot be read"},
                "orphans": {"type": "integer", "description": "Sidecars (.xmeta) without their file"},
                "writable": {"type": "boolean"},
                "cleaned": {"type": ["integer", "null"]}
            }})),
        B::new("library.favorite", &["LIB-011", "LIB-034"])
            .mutates()
            .params(params(
                json!({
                    "ids": {"type": "array", "items": {"type": "string"}, "minItems": 1},
                    "value": {"type": "boolean", "description": "Without it: all become favorites, unless all already are"}
                }),
                &["ids"],
            ))
            .result(json!({"type": "object", "properties": {"favorite": {"type": "boolean"}, "count": {"type": "integer"}}})),
        B::new("library.get", &["LIB-034", "LIB-012"])
            .params(params(
                json!({
                    "id": {"type": "string"},
                    "details": {"type": "boolean", "default": false, "description": "Add `details`: capture metadata (app and version, website, system, user, display scale) and the file's dates (LIB-012)"}
                }),
                &["id"],
            ))
            .positional(&["id"])
            .result(library_item()),
        B::new("library.open_url", &["LIB-012"])
            .scopes(UI_CLI)
            .positional(&["id"])
            .params(params(json!({"id": {"type": "string"}}), &["id"]))
            .result(json!({"type": "object", "properties": {"url": {"type": "string"}}})),
        B::new("library.delete", &["LIB-015", "TRY-009"])
            .destructive()
            .params(params(
                json!({
                    "ids": {"type": "array", "items": {"type": "string"}, "minItems": 1, "description": "Library item IDs"},
                    "paths": {"type": "array", "items": {"type": "string"}, "minItems": 1, "description": "Files, in the library or not (ids or paths, or both)"},
                    "permanent": {"type": "boolean", "default": false, "description": "Delete for good instead of moving to the trash"}
                }),
                &[],
            ))
            .result(json!({"type": "object", "properties": {"deleted": {"type": "integer"}, "undo": {"type": ["string", "null"]}}})),
        B::new("library.restore", &["LIB-015", "TRY-009"])
            .mutates()
            .params(params(json!({"undo": {"type": "string"}}), &["undo"]))
            .result(json!({"type": "object", "properties": {"restored": {"type": "integer"}}})),
        B::new("library.rename", &["TRY-006", "LIB-002"])
            .mutates()
            .params(params(
                json!({"id": {"type": "string"}, "name": {"type": "string", "minLength": 1}}),
                &["id", "name"],
            ))
            .result(library_item()),
        B::new("library.show_in_folder", &["EDT-109", "LIB-002"])
            .scopes(UI_CLI)
            .params(params(
                json!({"id": {"type": "string"}, "path": {"type": "string"}}),
                &[],
            )),
        B::new("ai.key_clear", &["SMT-039"]).scopes(UI_CLI).destructive().result(ai_key_status()),
        B::new("ai.key_status", &["SMT-039"]).scopes(UI_CLI).result(ai_key_status()),
        B::new("ai.key_test", &["SMT-039"])
            .scopes(UI_CLI)
            .network()
            .params(params(json!({"model": ai_model()}), &[]))
            .result(json!({"type": "object", "properties": {"ok": {"type": "boolean"}, "model": {"type": "string"}}})),
        B::new("ai.ask", &["SMT-040", "SMT-042"])
            .scopes(UI_CLI)
            .network()
            .params(params(
                json!({
                    "documentId": document_id(),
                    "prompt": {"type": "string", "minLength": 1, "description": "What to do with the image"},
                    "history": {
                        "type": "array",
                        "description": "Earlier questions and answers about the same image, oldest first",
                        "items": {
                            "type": "object",
                            "properties": {"prompt": {"type": "string"}, "answer": {"type": "string"}},
                            "required": ["prompt", "answer"]
                        }
                    },
                    "model": ai_model(),
                    "requestId": {"type": "string", "description": "Tags the answer so far, sent to the editor window while it arrives"}
                }),
                &["prompt"],
            ))
            .result(json!({
                "type": "object",
                "properties": {
                    "text": {"type": "string"},
                    "model": {"type": "string"},
                    "stopReason": {"type": "string"},
                    "usage": {"type": "object", "properties": {"inputTokens": {"type": "integer"}, "outputTokens": {"type": "integer"}}},
                    "width": {"type": "integer", "description": "Size of the image sent"},
                    "height": {"type": "integer"}
                }
            })),
        B::new("ai.suggest_name", &["SMT-041", "SMT-042"])
            .scopes(UI_CLI)
            .network()
            .params(params(
                json!({
                    "documentId": document_id(),
                    "model": ai_model(),
                    "avoid": {"type": "array", "items": {"type": "string"}, "description": "Names already suggested"}
                }),
                &[],
            ))
            .result(json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string"},
                    "model": {"type": "string"},
                    "usage": {"type": "object"}
                }
            })),
        B::new("ai.open_console", &["SMT-039"]).scopes(UI_CLI),
        B::new("ai.cancel", &["SMT-040"])
            .scopes(UI_CLI)
            .result(json!({"type": "object", "properties": {"cancelled": {"type": "boolean"}}})),
        B::new("editor.recovery_list", &["APP-047", "EDT-012"]).result(json!({
            "type": "object",
            "properties": {"items": {"type": "array", "items": {"type": "object"}}}
        })),
        B::new("editor.recovery_restore", &["APP-047", "EDT-012"])
            .scopes(UI_CLI)
            .mutates()
            .params(params(json!({"ids": {"type": "array", "items": {"type": "string"}}}), &["ids"]))
            .result(json!({"type": "object", "properties": {"documents": {"type": "array"}}})),
        B::new("editor.recovery_discard", &["APP-047", "EDT-012"])
            .scopes(UI_CLI)
            .destructive()
            .params(params(json!({"ids": {"type": "array", "items": {"type": "string"}}}), &["ids"])),
    ]
}

/// Editor tools of M1 (EDT-003). The ones of a later build answer `not_found`.
pub const TOOLS: [&str; 21] = [
    "favorites",
    "move",
    "arrow",
    "line",
    "text",
    "callout",
    "shape",
    "stamp",
    "step",
    "highlight",
    "pen",
    "magnify",
    "spotlight",
    "blur",
    "simplify",
    "select",
    "wand",
    "fill",
    "eraser",
    "crop",
    "cutout",
];

/// What `editor.action` can do in the editor window: the entries of its menus and
/// shortcuts that act on what the window shows (EDT-002, EDT-116).
pub const EDITOR_ACTIONS: [&str; 106] = [
    "open",
    "save",
    "save_as",
    "export",
    "close",
    "close_all",
    "undo",
    "redo",
    "cut",
    "copy",
    "paste",
    "paste_in_place",
    "copy_all",
    "delete",
    "deselect",
    "side_panel",
    "toolbar_labels",
    "recover",
    "ai_ask",
    "ai_name",
    "view_editor",
    "view_library",
    "tray",
    "prev_item",
    "next_item",
    "close_others",
    "rename_item",
    "pin_item",
    "duplicate_item",
    "shortcut_sheet",
    "move_computer",
    "show_in_folder",
    "delete_item",
    "select_all",
    "duplicate",
    "bring_front",
    "bring_forward",
    "send_backward",
    "send_back",
    "lock",
    "unlock_all",
    "align_left",
    "align_hcenter",
    "align_right",
    "align_top",
    "align_vcenter",
    "align_bottom",
    "distribute_h",
    "distribute_v",
    "group",
    "ungroup",
    "equal_width",
    "equal_height",
    "flatten",
    "flatten_all",
    "unflatten_all",
    "invert_selection",
    "float_selection",
    "crop_selection",
    "import_theme",
    // The Image menu (EDT-069 to EDT-077) and Object > Flip (EDT-073).
    "resize_image",
    "canvas_size",
    "canvas_color",
    "rotate_right",
    "rotate_left",
    "rotate_180",
    "rotate_custom",
    "flip_horizontal",
    "flip_vertical",
    "corners",
    "resolution",
    "revert",
    "flip_objects_horizontal",
    "flip_objects_vertical",
    "crop_to_canvas",
    "fit_canvas",
    "trim",
    // Image > Snapping (EDT-084).
    "snap",
    // File > New image and New from clipboard (EDT-103, EDT-104).
    "new_image",
    "new_from_clipboard",
    // File > Show in library (LIB-014).
    "show_in_library",
    // File > New from template, Apply template, Save as template, Import > Template
    // (EDT-095, EDT-101).
    "new_from_template",
    "apply_template",
    "save_as_template",
    "import_template",
    // File > Batch export (EDT-113).
    "batch_export",
    // File > Print (EDT-114).
    "print",
    // View > Pixel grid and Full screen (EDT-008, EDT-009).
    "pixel_grid",
    "fullscreen",
    // View > Toolbar > Customize (EDT-004).
    "customize_toolbar",
    // Image > Effects: turns one on and shows it in the Effects tab (EDT-053).
    "effect_border",
    "effect_edges",
    "effect_shadow",
    "effect_perspective",
    "effect_page_curl",
    "effect_reflection",
    "effect_fade",
    "effect_filters",
    "effect_color",
    "effect_replace",
    "effect_watermark",
    "effect_details",
    "effect_trim",
    "effect_corners",
    "effect_scale",
    "effect_rotate",
];

fn library_item() -> Value {
    json!({
        "type": "object",
        "properties": {
            "id": {"type": "string", "description": "Stable: it survives renames (LIB-034)"},
            "name": {"type": "string"},
            "path": {"type": "string"},
            "format": {"type": "string", "enum": ["xshot", "png", "jpg"]},
            "captured": {"type": "string"},
            "modified": {"type": "string"},
            "width": {"type": ["integer", "null"]},
            "height": {"type": ["integer", "null"]},
            "app": {"type": ["string", "null"]},
            "size": {"type": "integer"}
        }
    })
}

fn document_id() -> Value {
    json!({"type": "string", "description": "Default: the document shown in the editor"})
}

fn document_summary() -> Value {
    json!({
        "type": "object",
        "properties": {
            "documentId": {"type": "string"},
            "name": {"type": "string"},
            "path": {"type": ["string", "null"]},
            "kind": {"type": "string", "enum": ["library", "external"]},
            "width": {"type": "integer"},
            "height": {"type": "integer"},
            "objects": {"type": "integer"},
            "dirty": {"type": "boolean"}
        }
    })
}

fn history_result() -> Value {
    json!({
        "type": "object",
        "properties": {
            "label": {"type": ["string", "null"], "description": "Name of the step undone or redone"},
            "canUndo": {"type": "boolean"},
            "canRedo": {"type": "boolean"}
        }
    })
}
