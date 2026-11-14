//! Ask for text when capturing (CAP-051): with Capture details and its "Ask for text
//! when capturing", a short box appears when each capture ends; what is typed becomes
//! the effect's own text, next to the capture's details. Esc leaves the text empty and
//! does not cancel the capture. Series and scheduled captures do not ask (nobody may be
//! there): they keep the effect's own text as it is.

use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{json, Value};
use xs_core::i18n::tr;
use xs_core::{CommandError, CommandResult, Core, ErrorCode, Lang};

/// The effect option that asks.
pub const ASK_KEY: &str = "askText";
/// How long the box waits before the capture goes on without text.
const WAIT: Duration = Duration::from_secs(300);

#[derive(Default)]
pub struct AskText {
    waiting: Mutex<Option<mpsc::Sender<String>>>,
}

fn service(core: &Core) -> Arc<AskText> {
    core.service(AskText::default)
}

fn asks(effect: &Value) -> bool {
    effect["type"] == "details" && effect[ASK_KEY] == true && effect["enabled"] != false
}

/// Whether any of `effects` asks for text.
pub fn wants(effects: &[Value]) -> bool {
    effects.iter().any(asks)
}

/// `effects` ready to apply: with `ask`, the text typed (empty with Esc) as the own
/// text of each Capture details that asks; without, as they are. The option itself
/// does not reach the document.
pub fn fill(core: &Core, effects: &[Value], ask: bool) -> Vec<Value> {
    let text = if ask && wants(effects) {
        Some(prompt(core).unwrap_or_default())
    } else {
        None
    };
    with_text(effects, text.as_deref())
}

/// The effects with `text` as the own text of the Capture details that ask.
pub fn with_text(effects: &[Value], text: Option<&str>) -> Vec<Value> {
    effects
        .iter()
        .map(|e| {
            let mut e = e.clone();
            let asked = asks(&e);
            if let Some(o) = e.as_object_mut() {
                o.remove(ASK_KEY);
                if let (true, Some(text)) = (asked, text) {
                    o.insert("customText".into(), json!(text));
                }
            }
            e
        })
        .collect()
}

/// Shows the box and waits for its answer: `None` if it could not be shown or nobody
/// answered in time.
fn prompt(core: &Core) -> Option<String> {
    let (tx, rx) = mpsc::channel();
    *service(core).waiting.lock().unwrap() = Some(tx);
    let host = core.host();
    if let Err(e) = host.ask_text_window(true) {
        tracing::warn!(error = %e.message, "text box for the capture details not shown");
        service(core).waiting.lock().unwrap().take();
        return None;
    }
    let text = rx.recv_timeout(WAIT).ok();
    service(core).waiting.lock().unwrap().take();
    let _ = host.ask_text_window(false);
    text
}

/// `capture.details_text`: the box's answer (empty: no text).
pub fn answer(core: &Core, lang: Lang, p: &Value) -> CommandResult {
    let text = p.get("text").and_then(Value::as_str).unwrap_or("").trim();
    let Some(tx) = service(core).waiting.lock().unwrap().take() else {
        return Err(CommandError::new(
            ErrorCode::NotFound,
            tr(lang, "errors.nothing_asked", &[]),
        ));
    };
    let _ = tx.send(text.to_string());
    Ok(json!({ "accepted": true }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_text_goes_only_to_details_that_ask() {
        let effects = vec![
            json!({"type": "border", "width": 4}),
            json!({"type": "details", "askText": true, "customText": "old"}),
            json!({"type": "details", "customText": "kept"}),
        ];
        assert!(wants(&effects));
        let out = with_text(&effects, Some("Ticket 42"));
        assert_eq!(out[0], json!({"type": "border", "width": 4}));
        assert_eq!(out[1], json!({"type": "details", "customText": "Ticket 42"}));
        assert_eq!(out[2], json!({"type": "details", "customText": "kept"}));
    }

    #[test]
    fn esc_leaves_it_empty_and_series_keep_their_text() {
        let effects = vec![json!({"type": "details", "askText": true, "customText": "old"})];
        assert_eq!(with_text(&effects, Some(""))[0]["customText"], "");
        // Without asking (a series): the option goes, the text stays.
        assert_eq!(
            with_text(&effects, None)[0],
            json!({"type": "details", "customText": "old"})
        );
    }

    #[test]
    fn switched_off_details_do_not_ask() {
        let effects = vec![json!({"type": "details", "askText": true, "enabled": false})];
        assert!(!wants(&effects));
        assert!(!wants(&[json!({"type": "details"})]));
    }
}
