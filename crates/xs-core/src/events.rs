//! Events that scripts and agents can follow (APP-031): `capture.started`,
//! `capture.succeeded`, `capture.failed`, `capture.cancelled`, `file.written` and
//! `app.state_changed`, for what the user does in the interface as much as for the
//! CLI. Each one is a JSON object with `event`, `time`, `source` and its own data:
//! metadata and paths only, never the image or recognized text.
//!
//! `xcreenshot events watch` streams them, one JSON object per line.

use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Mutex;

use serde_json::{json, Map, Value};

use crate::command::Source;
use crate::error::{CommandResult, ErrorCode};

/// Commands that take a capture: they publish the capture events.
pub const CAPTURE_COMMANDS: &[&str] = &[
    "capture.capture",
    "capture.image",
    "capture.universal",
    "capture.region",
    "capture.window",
    "capture.fullscreen",
    "capture.repeat_last",
    "capture.repeat_last_area",
    "capture.scrolling",
    "capture.menu",
    "capture.window_picker",
    "capture.clipboard",
    "capture.quick_annotate",
    "capture.file",
    "capture.pdf",
];

/// Commands whose result names a file they wrote.
const WRITING_COMMANDS: &[&str] = &["document.save", "document.export"];

#[derive(Default)]
pub struct Events {
    subscribers: Mutex<Vec<Sender<Value>>>,
}

impl Events {
    /// A new listener: it receives every event published from now on.
    pub fn subscribe(&self) -> Receiver<Value> {
        let (tx, rx) = mpsc::channel();
        self.subscribers.lock().unwrap().push(tx);
        rx
    }

    /// Publishes an event to every listener still there.
    pub fn publish(&self, event: &str, source: Source, data: Value) {
        let mut subscribers = self.subscribers.lock().unwrap();
        if subscribers.is_empty() {
            return;
        }
        let mut message = Map::new();
        message.insert("event".into(), json!(event));
        message.insert("time".into(), json!(chrono::Utc::now().to_rfc3339()));
        message.insert("source".into(), json!(source));
        if let Value::Object(fields) = data {
            message.extend(fields);
        }
        let message = Value::Object(message);
        subscribers.retain(|tx| tx.send(message.clone()).is_ok());
    }

    /// Before a command runs: a capture starts.
    pub fn before(&self, id: &str, source: Source) {
        if CAPTURE_COMMANDS.contains(&id) {
            self.publish("app.state_changed", source, json!({ "state": "capturing" }));
            self.publish("capture.started", source, json!({ "command": id }));
        }
    }

    /// After a command ran: how the capture ended, and the files written.
    pub fn after(&self, id: &str, source: Source, result: &CommandResult) {
        let capture = CAPTURE_COMMANDS.contains(&id);
        match result {
            Ok(r) if capture => {
                self.publish(
                    "capture.succeeded",
                    source,
                    json!({
                        "command": id,
                        "documentId": r["documentId"],
                        "path": r["path"],
                        "width": r["width"],
                        "height": r["height"],
                    }),
                );
                for file in r["files"].as_array().into_iter().flatten() {
                    if let Some(path) = file.as_str() {
                        self.file_written(path, source);
                    }
                }
            }
            Ok(r) if WRITING_COMMANDS.contains(&id) && r["status"] != "needs_decision" => {
                if let Some(path) = r["path"].as_str() {
                    self.file_written(path, source);
                }
            }
            Err(e) if capture && e.code == ErrorCode::Cancelled => {
                self.publish("capture.cancelled", source, json!({ "command": id }));
            }
            Err(e) if capture => {
                self.publish(
                    "capture.failed",
                    source,
                    json!({ "command": id, "code": e.code, "message": e.message }),
                );
            }
            _ => {}
        }
        if capture {
            self.publish("app.state_changed", source, json!({ "state": "idle" }));
        }
    }

    fn file_written(&self, path: &str, source: Source) {
        let size = std::fs::metadata(path).map(|m| m.len()).ok();
        let kind = std::path::Path::new(path)
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        self.publish(
            "file.written",
            source,
            json!({ "path": path, "type": kind, "size": size }),
        );
    }
}

/// Whether `event` passes a filter like `capture.*,file.written` (empty: all).
pub fn matches(filter: &str, event: &str) -> bool {
    let patterns: Vec<&str> = filter
        .split(',')
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect();
    patterns.is_empty()
        || patterns.iter().any(|p| match p.strip_suffix('*') {
            Some(prefix) => event.starts_with(prefix),
            None => *p == event,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::CommandError;

    #[test]
    fn a_capture_tells_its_story_in_order() {
        let events = Events::default();
        let rx = events.subscribe();
        events.before("capture.region", Source::Shortcut);
        let file = std::env::temp_dir().join(format!("xs-events-{}.xshot", std::process::id()));
        std::fs::write(&file, b"12345").unwrap();
        let ok: CommandResult = Ok(json!({
            "documentId": "d1", "path": file, "files": [file], "width": 10, "height": 5
        }));
        events.after("capture.region", Source::Shortcut, &ok);
        let got: Vec<Value> = rx.try_iter().collect();
        let names: Vec<&str> = got.iter().filter_map(|e| e["event"].as_str()).collect();
        assert_eq!(
            names,
            [
                "app.state_changed",
                "capture.started",
                "capture.succeeded",
                "file.written",
                "app.state_changed"
            ]
        );
        assert!(got
            .iter()
            .all(|e| e["source"] == "shortcut" && e["time"].is_string()));
        assert_eq!(got[3]["size"], 5);
        assert_eq!(got[3]["type"], "xshot");
        let _ = std::fs::remove_file(file);

        let cancelled: CommandResult = Err(CommandError::new(ErrorCode::Cancelled, "Esc"));
        events.before("capture.region", Source::Ui);
        events.after("capture.region", Source::Ui, &cancelled);
        let names: Vec<String> = rx
            .try_iter()
            .filter_map(|e| e["event"].as_str().map(String::from))
            .collect();
        assert!(names.contains(&"capture.cancelled".to_string()));
        assert!(!names.contains(&"file.written".to_string()));
    }

    #[test]
    fn filters() {
        assert!(matches("", "capture.started"));
        assert!(matches("capture.*,file.written", "capture.failed"));
        assert!(matches("capture.*,file.written", "file.written"));
        assert!(!matches("capture.*", "app.state_changed"));
    }
}
