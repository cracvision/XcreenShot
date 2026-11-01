//! `xcreenshot mcp`: the MCP server over standard input and output (APP-029,
//! SMT-043). The MCP client starts it as a child process; it answers JSON-RPC
//! messages, one per line, and runs each tool as the command of the same ID in the
//! running app, with the MCP source and its rules (APP-032). Nothing but JSON-RPC
//! goes to standard output; nothing listens on the network.

use std::io::{BufRead, Write};
use std::process::ExitCode;

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
            json!({
                "name": tool_name(s.id),
                "title": s.title(Lang::En),
                "description": s.description(Lang::En),
                "inputSchema": s.params,
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
}

impl Server<'_> {
    fn call(&self, name: &str, arguments: Value) -> Result<Value, (i64, String)> {
        let Some(spec) = self
            .specs
            .iter()
            .find(|s| s.has_scope(Scope::Mcp) && tool_name(s.id) == name)
        else {
            return Err((-32602, format!("Unknown tool: {name}")));
        };
        let endpoint = ipc::endpoint();
        if !ipc::is_running(&endpoint) && (self.no_launch || !super::launch(&endpoint, self.lang, true)) {
            return Ok(error_content(
                "not_running",
                &tr(self.lang, "cli.not_running", &[]),
            ));
        }
        let params = match arguments {
            Value::Null => Value::Object(Map::new()),
            other => other,
        };
        let request = Request {
            v: PROTOCOL_VERSION,
            command: spec.id.to_string(),
            params,
            source: Source::Mcp,
            lang: Some(Lang::En),
        };
        Ok(match ipc::send(&endpoint, &request, None) {
            Ok(r) if r.ok => content(r.result.unwrap_or(Value::Null)),
            Ok(r) => match r.error {
                Some(e) => error_content(e.code.as_str(), &e.message),
                None => error_content("internal", "the command failed"),
            },
            Err(e) => error_content("not_running", &e.to_string()),
        })
    }

    fn handle(&self, method: &str, params: &Value) -> Result<Value, (i64, String)> {
        match method {
            "initialize" => {
                let asked = params["protocolVersion"].as_str().unwrap_or_default();
                let version = PROTOCOLS.iter().find(|v| **v == asked).unwrap_or(&PROTOCOLS[0]);
                Ok(json!({
                    "protocolVersion": version,
                    "capabilities": {"tools": {"listChanged": false}},
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
    };
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
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
            let mut text = serde_json::to_string(&reply).unwrap_or_default();
            text.push('\n');
            if stdout
                .write_all(text.as_bytes())
                .and_then(|_| stdout.flush())
                .is_err()
            {
                break;
            }
        }
    }
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
