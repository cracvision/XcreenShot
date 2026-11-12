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
pub const EXTERNAL_AREAS: &[&str] = &["capture", "document", "editor", "ai", "library"];

/// Options shared by every capture command (CAP-008), plus the mode's own.
fn capture_params(own: Value) -> Value {
    let mut props = json!({
        "cursor": {"type": "boolean", "description": "Include the pointer (CAP-047)"},
        "copy": {"type": "boolean", "description": "Copy to the clipboard (CAP-046)"},
        "open": {"type": "string", "enum": ["editor", "none"], "description": "Open in (CAP-045)"},
        "out": {"type": "string", "description": "Also write a PNG to this path"},
        "delay": {"type": "number", "minimum": 0, "maximum": 60, "description": "Seconds to wait first (CAP-048)"}
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
                    "description": "Physical pixels on the virtual desktop. Omit for an interactive selection.",
                    "properties": {
                        "x": {"type": "integer"}, "y": {"type": "integer"},
                        "width": {"type": "integer", "minimum": 1}, "height": {"type": "integer", "minimum": 1}
                    },
                    "required": ["x", "y", "width", "height"],
                    "additionalProperties": false
                }
            })))
            .result(capture_result()),
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
                    "width": {"type": "number", "minimum": 1, "maximum": 50000},
                    "height": {"type": "number", "minimum": 1, "maximum": 50000},
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
                    "format": {"type": "string", "enum": ["png", "jpg"]},
                    "path": {"type": "string"},
                    "quality": {"type": "integer", "minimum": 1, "maximum": 100, "description": "JPG quality (default 90)"},
                    "onConflict": {"type": "string", "enum": ["keepBoth", "replace", "fail"], "default": "keepBoth"}
                }),
                &["format", "path"],
            ))
            .result(json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string"},
                    "width": {"type": "integer"},
                    "height": {"type": "integer"},
                    "bytes": {"type": "integer"}
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
        B::new("library.list", &["LIB-001", "LIB-002", "LIB-022", "LIB-034"])
            .params(params(
                json!({
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
                    "total": {"type": "integer"},
                    "items": {"type": "array", "items": library_item()}
                }
            })),
        B::new("library.get", &["LIB-034"])
            .params(params(json!({"id": {"type": "string"}}), &["id"]))
            .positional(&["id"])
            .result(library_item()),
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
    .into_iter()
    .map(|b| b.0)
    .collect()
}

/// Editor tools of M1 (EDT-003). The ones of a later build answer `not_found`.
pub const TOOLS: [&str; 20] = [
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
];

/// What `editor.action` can do in the editor window: the entries of its menus and
/// shortcuts that act on what the window shows (EDT-002, EDT-116).
pub const EDITOR_ACTIONS: [&str; 89] = [
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
