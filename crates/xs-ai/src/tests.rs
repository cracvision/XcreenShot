//! The `ai.*` commands against a local stand-in for the API: no request leaves this
//! computer, and the key stays in memory.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use super::*;
use keychain::MemoryVault;
use xs_core::command::Source;
use xs_core::platform::Platform;

/// One request as the stand-in received it.
#[derive(Debug, Clone)]
struct Seen {
    line: String,
    headers: Vec<(String, String)>,
    body: String,
}

impl Seen {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }
    fn json(&self) -> Value {
        serde_json::from_str(&self.body).unwrap()
    }
}

/// A reply: status, content type and body, sent in parts with a pause between them.
struct Reply {
    status: u16,
    content_type: &'static str,
    parts: Vec<String>,
    pause: Duration,
}

fn reply(status: u16, content_type: &'static str, body: String) -> Reply {
    Reply {
        status,
        content_type,
        parts: vec![body],
        pause: Duration::ZERO,
    }
}

/// Serves `replies` in order, one per connection, and reports what it received.
fn stand_in(replies: Vec<Reply>) -> (String, mpsc::Receiver<Seen>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        for r in replies {
            let Ok((stream, _)) = listener.accept() else {
                return;
            };
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let mut headers = Vec::new();
            loop {
                let mut h = String::new();
                reader.read_line(&mut h).unwrap();
                let h = h.trim_end().to_string();
                if h.is_empty() {
                    break;
                }
                if let Some((k, v)) = h.split_once(':') {
                    headers.push((k.trim().to_string(), v.trim().to_string()));
                }
            }
            let length = headers
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("content-length"))
                .and_then(|(_, v)| v.parse::<usize>().ok())
                .unwrap_or(0);
            let mut body = vec![0; length];
            reader.read_exact(&mut body).unwrap();
            let _ = tx.send(Seen {
                line: line.trim_end().to_string(),
                headers,
                body: String::from_utf8(body).unwrap(),
            });
            let mut out = stream;
            let head = format!(
                "HTTP/1.1 {} X\r\nContent-Type: {}\r\nConnection: close\r\n\r\n",
                r.status, r.content_type
            );
            out.write_all(head.as_bytes()).unwrap();
            for (i, part) in r.parts.iter().enumerate() {
                if i > 0 {
                    std::thread::sleep(r.pause);
                }
                if out.write_all(part.as_bytes()).is_err() {
                    break;
                }
                let _ = out.flush();
            }
        }
    });
    (url, rx)
}

fn sse(events: &[Value]) -> String {
    events
        .iter()
        .map(|e| format!("event: {}\ndata: {}\n\n", e["type"].as_str().unwrap(), e))
        .collect()
}

/// A streamed answer that writes `pieces`.
fn answer(model: &str, pieces: &[&str], stop: &str) -> String {
    let mut events = vec![
        json!({"type": "message_start", "message": {"id": "msg_1", "model": model, "usage": {"input_tokens": 1200, "output_tokens": 1}}}),
        json!({"type": "content_block_start", "index": 0, "content_block": {"type": "thinking", "thinking": ""}}),
        json!({"type": "content_block_stop", "index": 0}),
        json!({"type": "content_block_start", "index": 1, "content_block": {"type": "text", "text": ""}}),
        json!({"type": "ping"}),
    ];
    for p in pieces {
        events.push(
            json!({"type": "content_block_delta", "index": 1, "delta": {"type": "text_delta", "text": p}}),
        );
    }
    events.push(json!({"type": "content_block_stop", "index": 1}));
    events.push(
        json!({"type": "message_delta", "delta": {"stop_reason": stop}, "usage": {"output_tokens": 42}}),
    );
    events.push(json!({"type": "message_stop"}));
    sse(&events)
}

fn error_body(kind: &str, message: &str) -> String {
    json!({"type": "error", "error": {"type": kind, "message": message}}).to_string()
}

struct Env {
    core: Arc<Core>,
    _library: tempfile::TempDir,
    _recovery: tempfile::TempDir,
    doc: String,
    path: PathBuf,
}

const KEY: &str = "sk-ant-test-0000-abcd";

fn env(url: &str) -> Env {
    let core = Core::for_tests_with(
        Platform::Windows,
        vec![
            xs_editor::extension(),
            extension_with(Config {
                base_url: url.to_string(),
                vault: Box::<MemoryVault>::default(),
            }),
        ],
    );
    let library = tempfile::tempdir().unwrap();
    let recovery = tempfile::tempdir().unwrap();
    xs_editor::editor(&core).set_folders(library.path().to_path_buf(), recovery.path().to_path_buf());
    let picture = image::RgbaImage::from_pixel(400, 200, image::Rgba([20, 120, 220, 255]));
    let (doc, path) =
        xs_editor::add_capture(&core, &picture, json!({"mode": "region"}), 1.0, Some("Test")).unwrap();
    Env {
        core,
        _library: library,
        _recovery: recovery,
        doc,
        path,
    }
}

fn call(core: &Core, source: Source, id: &str, p: Value) -> Result<Value, CommandError> {
    core.call(source, id, p, Some(Lang::En))
}

/// `ai.ask` about the test document, from the editor window.
fn ask(e: &Env, mut p: Value) -> Result<Value, CommandError> {
    p["documentId"] = json!(e.doc);
    call(&e.core, Source::Ui, "ai.ask", p)
}

/// Turned on and with a key, as after Preferences > AI.
fn ready_env(url: &str) -> Env {
    let e = env(url);
    call(
        &e.core,
        Source::Ui,
        "settings.set",
        json!({"key": "ai.enabled", "value": true}),
    )
    .unwrap();
    call(&e.core, Source::Ui, "ai.key_set", json!({"key": KEY})).unwrap();
    e
}

/// SMT-039: off by default; only the Preferences window turns it on; without a key
/// nothing is sent.
#[test]
fn off_by_default_and_needs_a_key() {
    let (url, seen) = stand_in(vec![]);
    let e = env(&url);
    let err = call(&e.core, Source::Cli, "ai.ask", json!({"prompt": "Describe"})).unwrap_err();
    assert_eq!(err.code, ErrorCode::FeatureDisabled);
    assert_eq!(err.details.unwrap()["settings"], "ai");

    let err = call(
        &e.core,
        Source::Cli,
        "settings.set",
        json!({"key": "ai.enabled", "value": true}),
    )
    .unwrap_err();
    assert_eq!(err.code, ErrorCode::PermissionDenied);
    call(
        &e.core,
        Source::Ui,
        "settings.set",
        json!({"key": "ai.enabled", "value": true}),
    )
    .unwrap();

    let err = call(&e.core, Source::Ui, "ai.ask", json!({"prompt": "Describe"})).unwrap_err();
    assert_eq!(err.code, ErrorCode::FeatureDisabled);
    assert!(err.message.contains("key"), "{}", err.message);
    let err = call(&e.core, Source::Ui, "ai.suggest_name", json!({})).unwrap_err();
    assert_eq!(err.code, ErrorCode::FeatureDisabled);
    assert!(seen.try_recv().is_err(), "nothing was sent");
}

/// SMT-039: the key is set from the Preferences window only, and shown by its last
/// four characters.
#[test]
fn the_key_is_set_only_from_the_window() {
    let (url, _) = stand_in(vec![]);
    let e = env(&url);
    assert!(call(&e.core, Source::Cli, "ai.key_set", json!({"key": KEY})).is_err());
    assert!(call(&e.core, Source::Mcp, "ai.key_status", json!({})).is_err());
    let err = call(
        &e.core,
        Source::Ui,
        "ai.key_set",
        json!({"key": "has spaces in it"}),
    )
    .unwrap_err();
    assert_eq!(err.code, ErrorCode::InvalidParams);
    let status = call(
        &e.core,
        Source::Ui,
        "ai.key_set",
        json!({"key": format!("  {KEY}\n")}),
    )
    .unwrap();
    assert_eq!(status, json!({"saved": true, "hint": "abcd"}));
    assert_eq!(
        call(&e.core, Source::Cli, "ai.key_status", json!({})).unwrap()["saved"],
        true
    );
    let cleared = call(&e.core, Source::Cli, "ai.key_clear", json!({})).unwrap();
    assert_eq!(cleared, json!({"saved": false}));
}

/// SMT-040: the image of the document goes with the request; the answer streams in
/// and comes back with what it cost.
#[test]
fn asking_sends_the_image_and_streams_the_answer() {
    let (url, seen) = stand_in(vec![reply(
        200,
        "text/event-stream",
        answer("claude-opus-5-5", &["A blue ", "rectangle."], "end_turn"),
    )]);
    let e = ready_env(&url);
    let out = call(
        &e.core,
        Source::Ui,
        "ai.ask",
        json!({"documentId": e.doc, "prompt": "Describe this image", "requestId": "r1"}),
    )
    .unwrap();
    assert_eq!(out["text"], "A blue rectangle.");
    assert_eq!(out["model"], "claude-opus-5-5");
    assert_eq!(out["stopReason"], "end_turn");
    assert_eq!(out["usage"], json!({"inputTokens": 1200, "outputTokens": 42}));
    assert_eq!(
        (out["width"].as_u64(), out["height"].as_u64()),
        (Some(400), Some(200))
    );

    let r = seen.recv_timeout(Duration::from_secs(5)).unwrap();
    assert_eq!(r.line, "POST /v1/messages HTTP/1.1");
    assert_eq!(r.header("x-api-key"), Some(KEY));
    assert_eq!(r.header("anthropic-version"), Some("2026-10-21"));
    assert_eq!(
        r.header("anthropic-beta"),
        Some("server-side-fallback-2026-07-01")
    );
    let body = r.json();
    assert_eq!(body["model"], "claude-opus-5-5");
    assert_eq!(body["fallbacks"], "default");
    assert_eq!(body["stream"], true);
    assert_eq!(body["output_config"]["effort"], "medium");
    assert!(body.get("temperature").is_none() && body.get("thinking").is_none());
    assert!(body["system"].as_str().unwrap().contains("Answer in English"));
    let content = &body["messages"][0]["content"];
    assert_eq!(content[0]["source"]["media_type"], "image/png");
    use base64::Engine as _;
    let png = base64::engine::general_purpose::STANDARD
        .decode(content[0]["source"]["data"].as_str().unwrap())
        .unwrap();
    let sent = image::load_from_memory(&png).unwrap();
    assert_eq!((sent.width(), sent.height()), (400, 200));
    assert_eq!(content[1]["text"], "Describe this image");
}

/// SMT-040: a follow-up carries the earlier questions and answers as context; a model
/// without server-side fallback gets no fallback.
#[test]
fn follow_ups_carry_the_conversation() {
    let (url, seen) = stand_in(vec![reply(
        200,
        "text/event-stream",
        answer("claude-haiku-5-5", &["Shorter."], "end_turn"),
    )]);
    let e = ready_env(&url);
    ask(
        &e,
        json!({
            "prompt": "Make it shorter",
            "model": "claude-haiku-5-5",
            "history": [{"prompt": "Write a poem", "answer": "Blue skies above"}]
        }),
    )
    .unwrap();
    let r = seen.recv_timeout(Duration::from_secs(5)).unwrap();
    assert_eq!(r.header("anthropic-beta"), None);
    let body = r.json();
    assert!(body.get("fallbacks").is_none());
    let text = body["messages"][0]["content"][1]["text"].as_str().unwrap();
    assert!(
        text.contains("User: Write a poem") && text.contains("You: Blue skies above"),
        "{text}"
    );
    assert!(text.ends_with("Now the user asks:\nMake it shorter"), "{text}");
}

/// SMT-040: failures come back in plain words, with a reason the dialog can act on.
#[test]
fn failures_are_explained() {
    let (url, _) = stand_in(vec![
        reply(
            401,
            "application/json",
            error_body("authentication_error", "invalid x-api-key"),
        ),
        reply(
            529,
            "application/json",
            error_body("overloaded_error", "Overloaded"),
        ),
        reply(
            400,
            "application/json",
            error_body(
                "invalid_request_error",
                "Your credit balance is too low to access the Anthropic API.",
            ),
        ),
        reply(
            200,
            "text/event-stream",
            answer("claude-opus-5-5", &["Partial"], "refusal"),
        ),
    ]);
    let e = ready_env(&url);
    let ask = || ask(&e, json!({"prompt": "Describe"})).unwrap_err();
    let reason = |err: &CommandError| {
        err.details.as_ref().unwrap()["reason"]
            .as_str()
            .unwrap()
            .to_string()
    };

    let err = ask();
    assert_eq!(
        (err.code, reason(&err).as_str()),
        (ErrorCode::PermissionDenied, "auth")
    );
    assert!(err.message.contains("key"), "{}", err.message);
    let err = ask();
    assert_eq!((err.code, reason(&err).as_str()), (ErrorCode::Busy, "overloaded"));
    let err = ask();
    assert_eq!(reason(&err), "no_credit");
    let err = ask();
    assert_eq!(reason(&err), "refused");
}

/// SMT-040: without a connection the request fails cleanly.
#[test]
fn no_connection_is_explained() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);
    let e = ready_env(&url);
    let err = ask(&e, json!({"prompt": "Describe"})).unwrap_err();
    assert_eq!(err.details.unwrap()["reason"], "network");
}

/// SMT-040: Stop cuts the answer short.
#[test]
fn cancelling_stops_the_answer() {
    let full = answer("claude-opus-5-5", &["First part. ", "Second part."], "end_turn");
    let split = full.find("Second").unwrap();
    let cut = full[..split].rfind("event:").unwrap();
    let (url, seen) = stand_in(vec![Reply {
        status: 200,
        content_type: "text/event-stream",
        parts: vec![full[..cut].to_string(), full[cut..].to_string()],
        pause: Duration::from_millis(800),
    }]);
    let e = ready_env(&url);
    let core = e.core.clone();
    let doc = e.doc.clone();
    let asking = std::thread::spawn(move || {
        call(
            &core,
            Source::Ui,
            "ai.ask",
            json!({"prompt": "Describe", "documentId": doc}),
        )
    });
    seen.recv_timeout(Duration::from_secs(5)).unwrap();
    std::thread::sleep(Duration::from_millis(200));
    let busy = call(
        &e.core,
        Source::Ui,
        "ai.suggest_name",
        json!({"documentId": e.doc}),
    )
    .unwrap_err();
    assert_eq!(busy.code, ErrorCode::Busy);
    let cancelled = call(&e.core, Source::Ui, "ai.cancel", json!({})).unwrap();
    assert_eq!(cancelled["cancelled"], true);
    let err = asking.join().unwrap().unwrap_err();
    assert_eq!(err.code, ErrorCode::Cancelled);
    assert_eq!(
        call(&e.core, Source::Ui, "ai.cancel", json!({})).unwrap()["cancelled"],
        false
    );
}

/// SMT-041: names come from the cheaper model at low effort, cleaned up to be a
/// valid file name; the previous suggestions are asked to be avoided.
#[test]
fn suggested_names_are_valid_file_names() {
    let (url, seen) = stand_in(vec![reply(
        200,
        "text/event-stream",
        answer(
            "claude-haiku-5-5",
            &["\"Blue banner: ", "draft/v2.png\"\n"],
            "end_turn",
        ),
    )]);
    let e = ready_env(&url);
    let out = call(
        &e.core,
        Source::Ui,
        "ai.suggest_name",
        json!({"documentId": e.doc, "avoid": ["Blue box"]}),
    )
    .unwrap();
    assert_eq!(out["name"], "Blue banner draft v2");
    let body = seen.recv_timeout(Duration::from_secs(5)).unwrap().json();
    assert_eq!(body["model"], "claude-haiku-5-5");
    assert_eq!(body["output_config"]["effort"], "low");
    assert!(body["messages"][0]["content"][1]["text"]
        .as_str()
        .unwrap()
        .contains("Blue box"));

    assert_eq!(
        clean_suggestion("  \n«Factura enero 2026».\n"),
        Some("Factura enero 2026".to_string())
    );
    assert_eq!(clean_suggestion("***"), None);
}

/// SMT-039: Test the key asks the Models API, without any image.
#[test]
fn testing_the_key_sends_no_image() {
    let (url, seen) = stand_in(vec![
        reply(
            200,
            "application/json",
            json!({"id": "claude-opus-5-5"}).to_string(),
        ),
        reply(
            404,
            "application/json",
            error_body("not_found_error", "model not found"),
        ),
    ]);
    let e = ready_env(&url);
    let ok = call(&e.core, Source::Ui, "ai.key_test", json!({})).unwrap();
    assert_eq!(ok, json!({"ok": true, "model": "claude-opus-5-5"}));
    let r = seen.recv_timeout(Duration::from_secs(5)).unwrap();
    assert_eq!(r.line, "GET /v1/models/claude-opus-5-5 HTTP/1.1");
    assert!(r.body.is_empty());
    let err = call(
        &e.core,
        Source::Ui,
        "ai.key_test",
        json!({"model": "claude-sonnet-5-5"}),
    )
    .unwrap_err();
    assert_eq!(err.details.unwrap()["reason"], "unknown_model");
}

/// SMT-040: large images go at most 2576 px on their longest side.
#[test]
fn large_images_are_scaled_down() {
    let wide = image::RgbaImage::from_pixel(5000, 1000, image::Rgba([0, 0, 0, 255]));
    let (bytes, media, w, h) = prepare_image(&wide).unwrap();
    assert_eq!((media, w, h), ("image/png", 2576, 515));
    assert_eq!(image::load_from_memory(&bytes).unwrap().width(), 2576);
    let small = image::RgbaImage::new(300, 100);
    assert_eq!(prepare_image(&small).unwrap().2, 300);
}

/// SMT-040: asking reads the document; it never changes its file.
#[test]
fn the_document_file_is_untouched() {
    let (url, _) = stand_in(vec![reply(
        200,
        "text/event-stream",
        answer("claude-opus-5-5", &["Ok"], "end_turn"),
    )]);
    let e = ready_env(&url);
    let before = std::fs::read(&e.path).unwrap();
    ask(&e, json!({"prompt": "Describe"})).unwrap();
    assert_eq!(std::fs::read(&e.path).unwrap(), before);
}
