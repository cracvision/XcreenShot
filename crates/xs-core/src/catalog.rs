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
    fn assignable(mut self) -> B {
        self.0.assignable = true;
        self
    }
    fn positional(mut self, p: &'static [&'static str]) -> B {
        self.0.positional = p;
        self
    }
}

/// Command areas whose handlers another crate registers through `Core::with_extensions`.
pub const EXTERNAL_AREAS: &[&str] = &["capture", "document", "editor"];

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
                    "set": {"type": "object", "description": "Geometry to replace and style properties to change"}
                }),
                &["set"],
            ))
            .result(json!({"type": "object", "properties": {"objects": {"type": "array"}}})),
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
                    "view": {"type": "string", "enum": ["editor"], "default": "editor"},
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
        B::new("editor.set_style", &["EDT-014"])
            .mutates()
            .params(params(
                json!({"tool": {"type": "string", "enum": TOOLS}, "style": {"type": "object"}}),
                &["tool", "style"],
            ))
            .result(json!({"type": "object", "properties": {"style": {"type": "object"}}})),
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
pub const TOOLS: [&str; 6] = ["move", "arrow", "text", "shape", "blur", "crop"];

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
