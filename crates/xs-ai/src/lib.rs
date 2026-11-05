//! Cloud AI (SMT-039 to SMT-042), a feature of Carlos's own (IDEA-002): ask Claude
//! about the image of a document, and suggest a file name for it.
//!
//! Guarantees (SMT-042):
//! - off by default; only the Preferences window turns it on (`ai.enabled`);
//! - nothing leaves without the user asking, about one image at a time;
//! - one destination, the Claude API, with the user's own key from the keychain;
//! - logs keep the model, tokens, time and outcome, never the image, the text or the key.

pub mod client;
pub mod keychain;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::time::Instant;

use serde_json::{json, Value};
use xs_core::command::Context;
use xs_core::core::{Core, Extension};
use xs_core::error::{CommandError, ErrorCode};
use xs_core::i18n::{tr, Lang};

use client::{Answer, Api, ApiError, Request};
use keychain::{SystemVault, Vault};

/// Longest side of the image sent: the most the models make use of (SMT-040).
pub const MAX_SIDE: u32 = 2576;
/// Largest encoded image: 5 MB once in base64.
const MAX_BYTES: usize = 3_750_000;

/// Where requests go and where the key is kept. Tests point both elsewhere.
pub struct Config {
    pub base_url: String,
    pub vault: Box<dyn Vault>,
}

impl Config {
    /// The real API and the system keychain. Debug builds accept
    /// `XCREENSHOT_AI_BASE_URL` to test against a local stand-in.
    pub fn system() -> Config {
        let instance = std::env::var("XCREENSHOT_DATA_DIR").ok();
        #[allow(unused_mut)]
        let mut base_url = client::API_URL.to_string();
        #[cfg(debug_assertions)]
        if let Ok(url) = std::env::var("XCREENSHOT_AI_BASE_URL") {
            base_url = url;
        }
        Config {
            base_url,
            vault: Box::new(SystemVault::new(instance.as_deref())),
        }
    }
}

struct Ai {
    config: Config,
    /// The request in progress, if any: one at a time.
    running: Mutex<Option<Arc<AtomicBool>>>,
}

/// Marks a request as running until dropped.
struct Running<'a> {
    ai: &'a Ai,
    cancel: Arc<AtomicBool>,
}

impl Drop for Running<'_> {
    fn drop(&mut self) {
        *self.ai.running.lock().unwrap() = None;
    }
}

impl Ai {
    fn begin(&self, lang: Lang) -> Result<Running<'_>, CommandError> {
        let mut running = self.running.lock().unwrap();
        if running.is_some() {
            return Err(CommandError::new(
                ErrorCode::Busy,
                tr(lang, "errors.ai_busy", &[]),
            ));
        }
        let cancel = Arc::new(AtomicBool::new(false));
        *running = Some(cancel.clone());
        Ok(Running { ai: self, cancel })
    }

    fn key(&self, lang: Lang) -> Result<String, CommandError> {
        match self.config.vault.get() {
            Ok(Some(key)) => Ok(key),
            Ok(None) => Err(disabled(lang, "errors.ai_no_key")),
            Err(e) => Err(keychain_error(lang, &e)),
        }
    }

    fn status(&self, lang: Lang) -> Result<Value, CommandError> {
        let key = self.config.vault.get().map_err(|e| keychain_error(lang, &e))?;
        Ok(match key {
            Some(k) => {
                let tail: String = k
                    .chars()
                    .rev()
                    .take(4)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect();
                json!({ "saved": true, "hint": tail })
            }
            None => json!({ "saved": false }),
        })
    }

    fn api(&self) -> Result<Api, CommandError> {
        Api::new(&self.config.base_url).map_err(CommandError::internal)
    }
}

fn disabled(lang: Lang, key: &str) -> CommandError {
    CommandError::new(ErrorCode::FeatureDisabled, tr(lang, key, &[]))
        .with_details(json!({ "settings": "ai" }))
}

fn keychain_error(lang: Lang, reason: &str) -> CommandError {
    CommandError::internal(tr(lang, "errors.ai_keychain", &[("reason", reason)]))
}

/// A failed request in the user's language, with `details.reason` for the dialog.
fn api_error(lang: Lang, e: &ApiError) -> CommandError {
    let code = match e {
        ApiError::Auth | ApiError::Forbidden | ApiError::NoCredit => ErrorCode::PermissionDenied,
        ApiError::RateLimited | ApiError::Overloaded => ErrorCode::Busy,
        ApiError::Timeout => ErrorCode::Timeout,
        ApiError::Cancelled => ErrorCode::Cancelled,
        ApiError::UnknownModel | ApiError::TooLarge => ErrorCode::InvalidParams,
        ApiError::Refused | ApiError::Network | ApiError::Other(_) => ErrorCode::Internal,
    };
    let message = match e {
        ApiError::Other(text) => tr(lang, "errors.ai_other", &[("reason", text)]),
        other => tr(lang, &format!("errors.ai_{}", other.reason()), &[]),
    };
    CommandError::new(code, message).with_details(json!({ "reason": e.reason() }))
}

fn ai(core: &Core) -> Arc<Ai> {
    core.service::<Ai>(|| unreachable!("xs-ai registers its service with its extension"))
}

fn model(core: &Core, p: &Value, setting: &str) -> String {
    p.get("model")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| core.settings.get_str(setting))
}

fn language_name(lang: Lang) -> &'static str {
    match lang {
        Lang::Es => "Spanish",
        _ => "English",
    }
}

/// The image of a document as the API takes it: at most [`MAX_SIDE`] on its longest
/// side and [`MAX_BYTES`] encoded; PNG, or JPEG if the PNG is too heavy.
pub fn prepare_image(image: &image::RgbaImage) -> Result<(Vec<u8>, &'static str, u32, u32), String> {
    let scaled;
    let image = if image.width().max(image.height()) > MAX_SIDE {
        let (w, h) = image.dimensions();
        let longest = w.max(h) as u64;
        let tw = ((w as u64 * MAX_SIDE as u64) / longest).max(1) as u32;
        let th = ((h as u64 * MAX_SIDE as u64) / longest).max(1) as u32;
        scaled = image::imageops::resize(image, tw, th, image::imageops::FilterType::Lanczos3);
        &scaled
    } else {
        image
    };
    let (w, h) = image.dimensions();
    let png = xs_image::encode::png(image, 0.0, true)?;
    if png.len() <= MAX_BYTES {
        return Ok((png, "image/png", w, h));
    }
    // Transparent areas go white, as on a page.
    let mut rgb = image::RgbImage::new(w, h);
    for (x, y, p) in image.enumerate_pixels() {
        let a = p[3] as u32;
        let mix = |c: u8| ((c as u32 * a + 255 * (255 - a)) / 255) as u8;
        rgb.put_pixel(x, y, image::Rgb([mix(p[0]), mix(p[1]), mix(p[2])]));
    }
    for quality in [90u8, 75, 60] {
        let mut jpeg = Vec::new();
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, quality)
            .encode_image(&rgb)
            .map_err(|e| e.to_string())?;
        if jpeg.len() <= MAX_BYTES {
            return Ok((jpeg, "image/jpeg", w, h));
        }
    }
    Err("the image is too large to send".to_string())
}

fn ask_system(lang: Lang) -> String {
    format!(
        "You are the assistant inside XcreenShot, an app for screenshots and image annotation. \
         The user attached one image: a screenshot or a photo, possibly with arrows, text, shapes \
         or blurred areas drawn over it. Do what the user asks with this image. Answer in {} \
         unless the user asks for another language. Write plain text that can be pasted into an \
         email, a document or onto the image: no Markdown headings, tables or code blocks unless \
         the user asks for them. Blurred or pixelated areas were hidden on purpose: do not try to \
         guess what they contain.",
        language_name(lang)
    )
}

/// The user turn: earlier questions and answers as context, then the new request
/// (SMT-040). Each question is a request of its own with the same image.
fn ask_text(prompt: &str, history: &[(String, String)]) -> String {
    if history.is_empty() {
        return prompt.to_string();
    }
    let mut text = String::from("Earlier in this conversation about the same image:\n");
    for (q, a) in history {
        text.push_str(&format!("\nUser: {q}\n\nYou: {a}\n"));
    }
    text.push_str(&format!("\nNow the user asks:\n{prompt}"));
    text
}

fn name_system(lang: Lang) -> String {
    format!(
        "You name image files for XcreenShot. Reply with the file name only: 3 to 8 words in {} \
         that say what the image shows, so its owner can find it later. No date, no file \
         extension, no quotes, and none of these characters: \\ / : * ? \" < > |. Use spaces \
         between words and capitalize only the first word and proper names.",
        language_name(lang)
    )
}

/// The model's suggestion as a usable file name (SMT-041).
pub fn clean_suggestion(text: &str) -> Option<String> {
    let line = text.lines().map(str::trim).find(|l| !l.is_empty())?;
    let line = line.trim_matches(|c: char| {
        c.is_whitespace() || matches!(c, '"' | '\'' | '`' | '«' | '»' | '“' | '”' | '*' | '.')
    });
    let line = line
        .strip_suffix(".png")
        .or_else(|| line.strip_suffix(".jpg"))
        .unwrap_or(line);
    xs_library::file_name(line)
}

fn usage(answer: &Answer) -> Value {
    json!({ "inputTokens": answer.input_tokens, "outputTokens": answer.output_tokens })
}

fn log_request(command: &str, model: &str, started: Instant, outcome: Result<&Answer, &ApiError>) {
    let ms = started.elapsed().as_millis() as u64;
    match outcome {
        Ok(a) => tracing::info!(
            command,
            model,
            input_tokens = a.input_tokens,
            output_tokens = a.output_tokens,
            ms,
            "cloud AI request done"
        ),
        Err(e) => tracing::warn!(command, model, ms, reason = e.reason(), "cloud AI request failed"),
    }
}

/// Checks that the AI is on and returns the key (SMT-039).
fn ready(core: &Core, ai: &Ai, lang: Lang) -> Result<String, CommandError> {
    if !core.settings.get_bool("ai.enabled") {
        return Err(disabled(lang, "errors.ai_disabled"));
    }
    ai.key(lang)
}

fn document_image(
    core: &Core,
    lang: Lang,
    p: &Value,
) -> Result<(Vec<u8>, &'static str, u32, u32), CommandError> {
    let id = xs_editor::resolve_document(core, lang, p)?;
    let image = xs_editor::final_image(core, &id).map_err(CommandError::internal)?;
    prepare_image(&image).map_err(|_| api_error(lang, &ApiError::TooLarge))
}

fn ask(core: &Core, ctx: &Context, p: &Value) -> Result<Value, CommandError> {
    let lang = ctx.lang;
    let ai = ai(core);
    let key = ready(core, &ai, lang)?;
    let prompt = p["prompt"].as_str().unwrap_or_default().trim().to_string();
    if prompt.is_empty() {
        return Err(CommandError::new(
            ErrorCode::InvalidParams,
            tr(
                lang,
                "errors.invalid_param",
                &[("param", "prompt"), ("reason", "empty")],
            ),
        ));
    }
    let history: Vec<(String, String)> = p["history"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .map(|h| {
                    let s = |k: &str| h[k].as_str().unwrap_or_default().to_string();
                    (s("prompt"), s("answer"))
                })
                .collect()
        })
        .unwrap_or_default();
    let model = model(core, p, "ai.model");
    let (image, media_type, width, height) = document_image(core, lang, p)?;
    let running = ai.begin(lang)?;
    let request_id = p["requestId"].as_str().unwrap_or_default().to_string();
    let system = ask_system(lang);
    let text = ask_text(&prompt, &history);
    let request = Request {
        model: &model,
        system: &system,
        image: &image,
        media_type,
        text: &text,
        max_tokens: 16000,
        effort: "medium",
    };
    let host = core.host();
    let started = Instant::now();
    // Each event carries the whole answer so far: a window that hears it twice, or
    // misses one, still shows the right text.
    let mut so_far = String::new();
    let result = ai.api()?.ask(&key, &request, &running.cancel, |piece| {
        so_far.push_str(piece);
        host.editor_event("ai", json!({ "requestId": request_id, "text": so_far }));
    });
    log_request("ai.ask", &model, started, result.as_ref());
    let answer = result.map_err(|e| api_error(lang, &e))?;
    Ok(json!({
        "text": answer.text,
        "model": answer.model,
        "stopReason": answer.stop_reason,
        "usage": usage(&answer),
        "width": width,
        "height": height,
    }))
}

fn suggest_name(core: &Core, ctx: &Context, p: &Value) -> Result<Value, CommandError> {
    let lang = ctx.lang;
    let ai = ai(core);
    let key = ready(core, &ai, lang)?;
    let model = model(core, p, "ai.namingModel");
    let (image, media_type, _, _) = document_image(core, lang, p)?;
    let avoid: Vec<String> = p["avoid"]
        .as_array()
        .map(|a| a.iter().filter_map(Value::as_str).map(str::to_string).collect())
        .unwrap_or_default();
    let mut text = String::from("Suggest a file name for this image.");
    if !avoid.is_empty() {
        text.push_str(&format!(
            " Give a different one from these: {}.",
            avoid.join("; ")
        ));
    }
    let running = ai.begin(lang)?;
    let system = name_system(lang);
    let request = Request {
        model: &model,
        system: &system,
        image: &image,
        media_type,
        text: &text,
        max_tokens: 4000,
        effort: "low",
    };
    let started = Instant::now();
    let result = ai.api()?.ask(&key, &request, &running.cancel, |_| {});
    log_request("ai.suggest_name", &model, started, result.as_ref());
    let answer = result.map_err(|e| api_error(lang, &e))?;
    let name = clean_suggestion(&answer.text)
        .ok_or_else(|| api_error(lang, &ApiError::Other("empty suggestion".to_string())))?;
    Ok(json!({ "name": name, "model": answer.model, "usage": usage(&answer) }))
}

fn key_set(core: &Core, lang: Lang, p: &Value) -> Result<Value, CommandError> {
    let key = p["key"].as_str().unwrap_or_default().trim();
    let valid = !key.is_empty() && key.len() <= 400 && key.chars().all(|c| c.is_ascii_graphic());
    if !valid {
        return Err(CommandError::new(
            ErrorCode::InvalidParams,
            tr(lang, "errors.ai_key_format", &[]),
        ));
    }
    let ai = ai(core);
    ai.config.vault.set(key).map_err(|e| keychain_error(lang, &e))?;
    tracing::info!("cloud AI key saved in the keychain");
    ai.status(lang)
}

fn key_test(core: &Core, lang: Lang, p: &Value) -> Result<Value, CommandError> {
    let ai = ai(core);
    let key = ai.key(lang)?;
    let model = model(core, p, "ai.model");
    let started = Instant::now();
    let result = ai.api()?.check(&key, &model);
    let ms = started.elapsed().as_millis() as u64;
    match &result {
        Ok(()) => tracing::info!(model, ms, "cloud AI key checked"),
        Err(e) => tracing::warn!(model, ms, reason = e.reason(), "cloud AI key check failed"),
    }
    result.map_err(|e| api_error(lang, &e))?;
    Ok(json!({ "ok": true, "model": model }))
}

/// Registers the `ai.*` commands with the real API and keychain.
pub fn extension() -> Extension {
    extension_with(Config::system())
}

/// Registers the `ai.*` commands with another API address or vault (tests).
pub fn extension_with(config: Config) -> Extension {
    Box::new(move |reg, weak: Weak<Core>| {
        if let Some(core) = weak.upgrade() {
            core.service(|| Ai {
                config,
                running: Mutex::new(None),
            });
        }
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
        on!("ai.key_set", |core, ctx, p| key_set(&core, ctx.lang, &p));
        on!("ai.key_clear", |core, ctx, _p| {
            let ai = ai(&core);
            ai.config
                .vault
                .delete()
                .map_err(|e| keychain_error(ctx.lang, &e))?;
            tracing::info!("cloud AI key removed from the keychain");
            ai.status(ctx.lang)
        });
        on!("ai.key_status", |core, ctx, _p| ai(&core).status(ctx.lang));
        on!("ai.key_test", |core, ctx, p| key_test(&core, ctx.lang, &p));
        on!("ai.open_console", |core, ctx, _p| {
            core.host().open_uri(client::CONSOLE_URL)?;
            Ok(json!({}))
        });
        on!("ai.ask", |core, ctx, p| ask(&core, ctx, &p));
        on!("ai.suggest_name", |core, ctx, p| suggest_name(&core, ctx, &p));
        on!("ai.cancel", |core, ctx, _p| {
            let running = ai(&core).running.lock().unwrap().clone();
            if let Some(flag) = &running {
                flag.store(true, Ordering::Relaxed);
            }
            Ok(json!({ "cancelled": running.is_some() }))
        });
    })
}

#[cfg(test)]
mod tests;
