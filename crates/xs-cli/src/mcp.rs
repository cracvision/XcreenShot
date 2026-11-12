//! `xcreenshot mcp`: the MCP server over standard input and output (APP-029,
//! SMT-043). The MCP client starts it as a child process; it answers JSON-RPC
//! messages, one per line, and runs each tool as the command of the same ID in the
//! running app, with the MCP source and its rules (APP-032). Nothing but JSON-RPC
//! goes to standard output; nothing listens on the network.
//!
//! Besides the tools: read-only resources (the command guide, the recent captures,
//! the recent tray and the latest events) and the app's events (APP-031) for clients
//! that subscribe to `xcreenshot://events`: each one arrives as
//! `notifications/resources/updated` and `resources/read` gives the latest ones.

use std::collections::VecDeque;
use std::io::{BufRead, Write};
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use serde_json::{json, Map, Value};
use xs_core::i18n::tr;
use xs_core::ipc::{self, Request, PROTOCOL_VERSION};
use xs_core::{CommandSpec, Lang, Scope, Source};

/// Protocol revisions this server speaks, newest first.
const PROTOCOLS: [&str; 4] = ["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05"];

const INSTRUCTIONS: &str = "XcreenShot is a screenshot and image annotation app running on this computer. \
Each tool is an XcreenShot command (see its AGENTS.md). To look at a capture, call \
document_list or document_open, then document_image: it returns the final image so you \
can describe it, write about it or suggest a name, and document_rename renames its file. \
XcreenShot sends nothing to the cloud through these tools.";

/// The command guide (APP-033), as a resource.
const AGENTS: &str = include_str!("../../../AGENTS.md");

/// Longest side of an image sent with `include_image` (APP-029, design decision).
pub const IMAGE_SIDE: u64 = 1568;

/// Events kept for `resources/read` of `xcreenshot://events`.
const EVENTS_KEPT: usize = 50;

/// The resources (APP-029): uri, name, description, type.
pub fn resources() -> Vec<Value> {
    let r = |uri: &str, name: &str, description: &str, mime: &str| json!({"uri": uri, "name": name, "description": description, "mimeType": mime});
    vec![
        r(
            "xcreenshot://agents",
            "Command guide",
            "AGENTS.md: every command with an example and what it does",
            "text/markdown",
        ),
        r(
            "xcreenshot://library/recent",
            "Recent captures",
            "The 20 most recent captures in the library",
            "application/json",
        ),
        r(
            "xcreenshot://tray",
            "Recent tray",
            "The items of the editor's recent tray, left to right",
            "application/json",
        ),
        r(
            "xcreenshot://events",
            "Events",
            "The latest app events (APP-031); subscribe to be told of each new one",
            "application/json",
        ),
    ]
}

/// Tool name of a command: its ID with `_` for `.`, valid in every client.
pub fn tool_name(id: &str) -> String {
    id.replace('.', "_")
}

/// The tools: every command an agent may run (MCP scope).
pub fn tools(specs: &[CommandSpec]) -> Vec<Value> {
    specs
        .iter()
        .filter(|s| s.has_scope(Scope::Mcp))
        .map(|s| {
            // A tool that gives a document (a capture, an open file) can also send its
            // image, scaled down, for the agent to see.
            let mut input = s.params.clone();
            if s.result["properties"]["documentId"].is_object() {
                input["properties"]["include_image"] = json!({
                    "type": "boolean",
                    "default": false,
                    "description": format!("Also send the image, at most {IMAGE_SIDE} px on its longest side")
                });
            }
            json!({
                "name": tool_name(s.id),
                "title": s.title(Lang::En),
                "description": s.description(Lang::En),
                "inputSchema": input,
                "annotations": {
                    "title": s.title(Lang::En),
                    "readOnlyHint": !s.mutates,
                    "destructiveHint": s.destructive,
                    "idempotentHint": !s.mutates,
                    "openWorldHint": s.network
                }
            })
        })
        .collect()
}

/// A command result as MCP content: an image when it carries one (`image.mimeType`,
/// `image.data`), and the rest as JSON text and structured content.
pub fn content(result: Value) -> Value {
    let mut blocks = Vec::new();
    let mut rest = result;
    if let Some(image) = rest.get_mut("image").and_then(Value::as_object_mut) {
        if let (Some(mime), Some(data)) = (
            image.get("mimeType").and_then(Value::as_str).map(str::to_string),
            image.remove("data").and_then(|d| d.as_str().map(str::to_string)),
        ) {
            blocks.push(json!({"type": "image", "data": data, "mimeType": mime}));
            image.insert("data".to_string(), json!("(sent as image content)"));
        }
    }
    blocks.insert(
        0,
        json!({"type": "text", "text": serde_json::to_string(&rest).unwrap_or_default()}),
    );
    let mut out = json!({ "content": blocks, "isError": false });
    if rest.is_object() {
        out["structuredContent"] = rest;
    }
    out
}

fn error_content(code: &str, message: &str) -> Value {
    json!({
        "content": [{"type": "text", "text": format!("{message} ({code})")}],
        "isError": true
    })
}

struct Server<'a> {
    specs: &'a [CommandSpec],
    no_launch: bool,
    lang: Lang,
    out: Arc<Mutex<std::io::Stdout>>,
    events: Arc<Mutex<VecDeque<Value>>>,
    watching: Arc<AtomicBool>,
}

/// Writes one JSON-RPC message, alone on its line.
fn send(out: &Mutex<std::io::Stdout>, message: &Value) -> bool {
    let mut text = serde_json::to_string(message).unwrap_or_default();
    text.push('\n');
    let mut out = out.lock().unwrap();
    out.write_all(text.as_bytes()).and_then(|_| out.flush()).is_ok()
}

impl Server<'_> {
    /// Runs a command in the app as the MCP source; `None` when the app is not there.
    fn run(&self, id: &str, params: Value) -> Result<Value, (String, String)> {
        let request = Request {
            v: PROTOCOL_VERSION,
            command: id.to_string(),
            params,
            source: Source::Mcp,
            lang: Some(Lang::En),
        };
        match ipc::send(&ipc::endpoint(), &request, None) {
            Ok(r) if r.ok => Ok(r.result.unwrap_or(Value::Null)),
            Ok(r) => Err(r
                .error
                .map(|e| (e.code.as_str().to_string(), e.message))
                .unwrap_or_else(|| ("internal".into(), "the command failed".into()))),
            Err(e) => Err(("not_running".into(), e.to_string())),
        }
    }

    fn running(&self) -> bool {
        let endpoint = ipc::endpoint();
        ipc::is_running(&endpoint) || (!self.no_launch && super::launch(&endpoint, self.lang, true))
    }

    fn read_resource(&self, uri: &str) -> Result<Value, (i64, String)> {
        let json_text = |v: &Value| json!({"contents": [{"uri": uri, "mimeType": "application/json", "text": serde_json::to_string_pretty(v).unwrap_or_default()}]});
        match uri {
            "xcreenshot://agents" => {
                Ok(json!({"contents": [{"uri": uri, "mimeType": "text/markdown", "text": AGENTS}]}))
            }
            "xcreenshot://events" => {
                let list: Vec<Value> = self.events.lock().unwrap().iter().cloned().collect();
                Ok(json_text(&json!({ "events": list })))
            }
            "xcreenshot://library/recent" | "xcreenshot://tray" => {
                if !self.running() {
                    return Err((-32002, tr(self.lang, "cli.not_running", &[])));
                }
                let (id, params) = if uri.ends_with("tray") {
                    ("tray.list", json!({}))
                } else {
                    (
                        "library.list",
                        json!({"sort": "captured", "order": "desc", "limit": 20}),
                    )
                };
                let mut v = self
                    .run(id, params)
                    .map_err(|(code, msg)| (-32603, format!("{msg} ({code})")))?;
                // The captures without their facets: what an agent needs to pick one.
                if let Some(o) = v.as_object_mut() {
                    o.remove("facets");
                }
                Ok(json_text(&v))
            }
            _ => Err((-32002, format!("Resource not found: {uri}"))),
        }
    }

    /// `resources/subscribe` to the events: a thread follows them (APP-031) and tells
    /// the client of each one.
    fn subscribe(&self, uri: &str) -> Result<Value, (i64, String)> {
        if uri != "xcreenshot://events" {
            return Err((-32002, format!("Resource not found: {uri}")));
        }
        if self.watching.swap(true, Ordering::SeqCst) {
            return Ok(json!({}));
        }
        if !self.running() {
            self.watching.store(false, Ordering::SeqCst);
            return Err((-32002, tr(self.lang, "cli.not_running", &[])));
        }
        let (out, events, watching) = (self.out.clone(), self.events.clone(), self.watching.clone());
        std::thread::spawn(move || {
            let request = Request {
                v: PROTOCOL_VERSION,
                command: "events.watch".to_string(),
                params: json!({}),
                source: Source::Mcp,
                lang: Some(Lang::En),
            };
            let _ = ipc::stream(&ipc::endpoint(), &request, |line| {
                if !watching.load(Ordering::SeqCst) {
                    return false;
                }
                // The first line says whether the app watches; then one per event.
                if line.get("ok").is_some() {
                    return line["ok"] == json!(true);
                }
                {
                    let mut list = events.lock().unwrap();
                    list.push_back(line);
                    while list.len() > EVENTS_KEPT {
                        list.pop_front();
                    }
                }
                send(
                    &out,
                    &json!({"jsonrpc": "2.0", "method": "notifications/resources/updated", "params": {"uri": "xcreenshot://events"}}),
                )
            });
            watching.store(false, Ordering::SeqCst);
        });
        Ok(json!({}))
    }

    fn call(&self, name: &str, arguments: Value) -> Result<Value, (i64, String)> {
        let Some(spec) = self
            .specs
            .iter()
            .find(|s| s.has_scope(Scope::Mcp) && tool_name(s.id) == name)
        else {
            return Err((-32602, format!("Unknown tool: {name}")));
        };
        if !self.running() {
            return Ok(error_content(
                "not_running",
                &tr(self.lang, "cli.not_running", &[]),
            ));
        }
        let mut params = match arguments {
            Value::Null => Value::Object(Map::new()),
            other => other,
        };
        // `include_image` is the server's, not the command's (APP-029).
        let include = params
            .as_object_mut()
            .and_then(|o| o.remove("include_image"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        Ok(match self.run(spec.id, params) {
            Ok(mut result) => {
                if include {
                    if let Some(doc) = result["documentId"].as_str().map(str::to_string) {
                        if let Ok(img) = self.run(
                            "document.image",
                            json!({"documentId": doc, "maxSize": IMAGE_SIDE}),
                        ) {
                            result["image"] = img["image"].clone();
                        }
                    }
                }
                content(result)
            }
            Err((code, message)) => error_content(&code, &message),
        })
    }

    fn handle(&self, method: &str, params: &Value) -> Result<Value, (i64, String)> {
        match method {
            "initialize" => {
                let asked = params["protocolVersion"].as_str().unwrap_or_default();
                let version = PROTOCOLS.iter().find(|v| **v == asked).unwrap_or(&PROTOCOLS[0]);
                Ok(json!({
                    "protocolVersion": version,
                    "capabilities": {
                        "tools": {"listChanged": false},
                        "resources": {"subscribe": true, "listChanged": false}
                    },
                    "serverInfo": {"name": "xcreenshot", "title": "XcreenShot", "version": xs_core::platform::VERSION},
                    "instructions": INSTRUCTIONS
                }))
            }
            "ping" => Ok(json!({})),
            "tools/list" => Ok(json!({ "tools": tools(self.specs) })),
            "tools/call" => {
                let name = params["name"].as_str().unwrap_or_default();
                self.call(name, params.get("arguments").cloned().unwrap_or(Value::Null))
            }
            "resources/list" => Ok(json!({ "resources": resources() })),
            "resources/templates/list" => Ok(json!({ "resourceTemplates": [] })),
            "resources/read" => self.read_resource(params["uri"].as_str().unwrap_or_default()),
            "resources/subscribe" => self.subscribe(params["uri"].as_str().unwrap_or_default()),
            "resources/unsubscribe" => {
                self.watching.store(false, Ordering::SeqCst);
                Ok(json!({}))
            }
            _ => Err((-32601, format!("Method not found: {method}"))),
        }
    }
}

/// Serves until standard input closes.
pub fn serve(specs: &[CommandSpec], no_launch: bool, lang: Lang) -> ExitCode {
    let server = Server {
        specs,
        no_launch,
        lang,
        out: Arc::new(Mutex::new(std::io::stdout())),
        events: Arc::new(Mutex::new(VecDeque::new())),
        watching: Arc::new(AtomicBool::new(false)),
    };
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let reply = match serde_json::from_str::<Value>(&line) {
            Err(e) => Some(json!({
                "jsonrpc": "2.0", "id": null,
                "error": {"code": -32700, "message": format!("Parse error: {e}")}
            })),
            Ok(message) => {
                let id = message.get("id").cloned();
                let method = message["method"].as_str().unwrap_or_default();
                // Notifications (no id) and replies from the client need no answer.
                match id {
                    None | Some(Value::Null) => None,
                    Some(_) if method.is_empty() => None,
                    Some(id) => Some(match server.handle(method, &message["params"]) {
                        Ok(result) => json!({"jsonrpc": "2.0", "id": id, "result": result}),
                        Err((code, msg)) => json!({
                            "jsonrpc": "2.0", "id": id,
                            "error": {"code": code, "message": msg}
                        }),
                    }),
                }
            }
        };
        if let Some(reply) = reply {
            if !send(&server.out, &reply) {
                break;
            }
        }
    }
    server.watching.store(false, Ordering::SeqCst);
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tools_are_the_mcp_commands() {
        let specs = xs_core::catalog::specs();
        let tools = tools(&specs);
        let names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
        for expected in ["document_image", "document_rename", "document_list", "app_ping"] {
            assert!(names.contains(&expected), "{expected} missing");
        }
        // The cloud AI is never an agent's to use, nor its key (SMT-042).
        for hidden in ["ai_ask", "ai_suggest_name", "ai_key_set", "ai_key_status"] {
            assert!(!names.contains(&hidden), "{hidden} must not be a tool");
        }
        let image = tools.iter().find(|t| t["name"] == "document_image").unwrap();
        assert_eq!(image["annotations"]["readOnlyHint"], true);
        assert_eq!(image["inputSchema"]["type"], "object");
    }

    #[test]
    fn image_tools_offer_their_image_and_resources_are_listed() {
        let specs = xs_core::catalog::specs();
        let tools = tools(&specs);
        let full = tools.iter().find(|t| t["name"] == "capture_fullscreen").unwrap();
        assert_eq!(
            full["inputSchema"]["properties"]["include_image"]["type"],
            "boolean"
        );
        let list = tools.iter().find(|t| t["name"] == "document_list").unwrap();
        assert!(list["inputSchema"]["properties"].get("include_image").is_none());
        let listed = resources();
        let uris: Vec<&str> = listed.iter().filter_map(|r| r["uri"].as_str()).collect();
        assert!(uris.contains(&"xcreenshot://agents") && uris.contains(&"xcreenshot://events"));
        assert!(
            AGENTS.contains("| `library.list` |"),
            "the guide is the real AGENTS.md"
        );
    }

    #[test]
    fn images_become_image_content() {
        let out = content(json!({
            "image": {"mimeType": "image/png", "data": "iVBORw0KGgo="},
            "width": 10, "height": 5
        }));
        let blocks = out["content"].as_array().unwrap();
        assert_eq!(
            blocks[1],
            json!({"type": "image", "data": "iVBORw0KGgo=", "mimeType": "image/png"})
        );
        assert!(!blocks[0]["text"].as_str().unwrap().contains("iVBORw0KGgo="));
        assert_eq!(out["structuredContent"]["width"], 10);
        assert_eq!(out["isError"], false);
        let plain = content(json!([1, 2]));
        assert!(plain.get("structuredContent").is_none());
    }
}
