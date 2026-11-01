//! Requests to the Claude API over plain HTTPS (SMT-040, SMT-041, SMT-042): the
//! Messages API with streaming, and the Models API to check a key. No SDK exists for
//! Rust, so this follows the documented HTTP shapes.

use std::io::{BufRead, BufReader};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde_json::{json, Value};

pub const API_URL: &str = "https://api.anthropic.com";
/// Where API keys are created, opened in the browser on request (SMT-039).
pub const CONSOLE_URL: &str = "https://platform.claude.com/settings/keys";
const API_VERSION: &str = "2026-10-21";
/// Server-side fallback when a model declines (`fallbacks: "default"`).
const FALLBACK_BETA: &str = "server-side-fallback-2026-07-01";

/// What went wrong, in terms the dialog can explain (SMT-040).
#[derive(Debug, Clone, PartialEq)]
pub enum ApiError {
    /// 401: the key is wrong or was revoked.
    Auth,
    /// 403: the key may not use this model or feature.
    Forbidden,
    /// The account has no credit left.
    NoCredit,
    /// 404 on a model: not available to this key.
    UnknownModel,
    /// 429: too many requests or tokens for now.
    RateLimited,
    /// 529 / 503: the service is overloaded.
    Overloaded,
    /// The image or the request is too large.
    TooLarge,
    /// The model (and its fallback) declined the request.
    Refused,
    /// No connection to the service (or the proxy).
    Network,
    /// The service stopped answering.
    Timeout,
    /// The user stopped it.
    Cancelled,
    /// Anything else, with the service's own words.
    Other(String),
}

impl ApiError {
    pub fn reason(&self) -> &'static str {
        match self {
            ApiError::Auth => "auth",
            ApiError::Forbidden => "forbidden",
            ApiError::NoCredit => "no_credit",
            ApiError::UnknownModel => "unknown_model",
            ApiError::RateLimited => "rate_limited",
            ApiError::Overloaded => "overloaded",
            ApiError::TooLarge => "too_large",
            ApiError::Refused => "refused",
            ApiError::Network => "network",
            ApiError::Timeout => "timeout",
            ApiError::Cancelled => "cancelled",
            ApiError::Other(_) => "other",
        }
    }
}

/// The text the model wrote, and what it cost.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Answer {
    pub text: String,
    pub model: String,
    pub stop_reason: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

/// One user turn: an image and a text.
pub struct Request<'a> {
    pub model: &'a str,
    pub system: &'a str,
    pub image: &'a [u8],
    pub media_type: &'a str,
    pub text: &'a str,
    pub max_tokens: u32,
    pub effort: &'a str,
}

pub struct Api {
    base: String,
    http: reqwest::blocking::Client,
}

impl Api {
    pub fn new(base: &str) -> Result<Api, String> {
        let http = reqwest::blocking::Client::builder()
            .connect_timeout(Duration::from_secs(20))
            // Per read: a long answer streams for minutes, but never goes silent that
            // long (the service sends pings).
            .timeout(Duration::from_secs(120))
            .user_agent(concat!("XcreenShot/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| e.to_string())?;
        Ok(Api {
            base: base.trim_end_matches('/').to_string(),
            http,
        })
    }

    /// Checks that `key` works with `model`, without sending any image (SMT-039).
    pub fn check(&self, key: &str, model: &str) -> Result<(), ApiError> {
        let response = self
            .http
            .get(format!("{}/v1/models/{model}", self.base))
            .header("x-api-key", key)
            .header("anthropic-version", API_VERSION)
            .send()
            .map_err(transport)?;
        let status = response.status().as_u16();
        if status == 200 {
            return Ok(());
        }
        let body = response.text().unwrap_or_default();
        Err(match status {
            404 => ApiError::UnknownModel,
            _ => status_error(status, &body),
        })
    }

    /// Sends one request and streams the answer: `on_text` gets each piece as it
    /// arrives. `cancel` stops it between pieces.
    pub fn ask(
        &self,
        key: &str,
        request: &Request,
        cancel: &AtomicBool,
        mut on_text: impl FnMut(&str),
    ) -> Result<Answer, ApiError> {
        use base64::Engine as _;
        let data = base64::engine::general_purpose::STANDARD.encode(request.image);
        let mut body = json!({
            "model": request.model,
            "max_tokens": request.max_tokens,
            "stream": true,
            "system": request.system,
            "output_config": {"effort": request.effort},
            "messages": [{
                "role": "user",
                "content": [
                    {"type": "image", "source": {"type": "base64", "media_type": request.media_type, "data": data}},
                    {"type": "text", "text": request.text}
                ]
            }]
        });
        let mut http = self
            .http
            .post(format!("{}/v1/messages", self.base))
            .header("x-api-key", key)
            .header("anthropic-version", API_VERSION)
            .header("content-type", "application/json");
        if has_fallback(request.model) {
            body["fallbacks"] = json!("default");
            http = http.header("anthropic-beta", FALLBACK_BETA);
        }
        let response = http.body(body.to_string()).send().map_err(transport)?;
        let status = response.status().as_u16();
        if status != 200 {
            let text = response.text().unwrap_or_default();
            return Err(status_error(status, &text));
        }
        let mut answer = Answer {
            model: request.model.to_string(),
            ..Answer::default()
        };
        let reader = BufReader::new(response);
        for line in reader.lines() {
            if cancel.load(Ordering::Relaxed) {
                return Err(ApiError::Cancelled);
            }
            let line = line.map_err(|e| {
                if e.kind() == std::io::ErrorKind::TimedOut {
                    ApiError::Timeout
                } else {
                    ApiError::Network
                }
            })?;
            let Some(data) = line.strip_prefix("data:") else {
                continue;
            };
            let Ok(event) = serde_json::from_str::<Value>(data.trim()) else {
                continue;
            };
            match event["type"].as_str().unwrap_or_default() {
                "message_start" => {
                    let message = &event["message"];
                    if let Some(model) = message["model"].as_str() {
                        answer.model = model.to_string();
                    }
                    read_usage(&message["usage"], &mut answer);
                }
                "content_block_delta" if event["delta"]["type"] == "text_delta" => {
                    let piece = event["delta"]["text"].as_str().unwrap_or_default();
                    answer.text.push_str(piece);
                    on_text(piece);
                }
                "message_delta" => {
                    if let Some(reason) = event["delta"]["stop_reason"].as_str() {
                        answer.stop_reason = reason.to_string();
                    }
                    read_usage(&event["usage"], &mut answer);
                }
                "message_stop" => break,
                "error" => {
                    let kind = event["error"]["type"].as_str().unwrap_or_default();
                    let message = event["error"]["message"].as_str().unwrap_or_default();
                    return Err(error_type(kind, message));
                }
                _ => {}
            }
        }
        // A decline, even after some text, leaves nothing usable (the partial is
        // discarded).
        if answer.stop_reason == "refusal" {
            return Err(ApiError::Refused);
        }
        Ok(answer)
    }
}

/// Models that take the server-side fallback (`fallbacks: "default"`); Haiku has none.
fn has_fallback(model: &str) -> bool {
    model.starts_with("claude-opus-5-5") || model.starts_with("claude-sonnet-5-5")
}

fn read_usage(usage: &Value, answer: &mut Answer) {
    if let Some(n) = usage["input_tokens"].as_u64() {
        answer.input_tokens = n;
    }
    if let Some(n) = usage["output_tokens"].as_u64() {
        answer.output_tokens = n;
    }
}

fn transport(e: reqwest::Error) -> ApiError {
    if e.is_timeout() {
        ApiError::Timeout
    } else {
        ApiError::Network
    }
}

fn status_error(status: u16, body: &str) -> ApiError {
    let parsed: Value = serde_json::from_str(body).unwrap_or(Value::Null);
    let kind = parsed["error"]["type"].as_str().unwrap_or_default();
    let message = parsed["error"]["message"].as_str().unwrap_or(body);
    match status {
        401 => ApiError::Auth,
        403 => ApiError::Forbidden,
        413 => ApiError::TooLarge,
        429 => ApiError::RateLimited,
        503 | 529 => ApiError::Overloaded,
        _ => error_type(kind, message),
    }
}

/// The error object of the API (`{"type": …, "message": …}`).
fn error_type(kind: &str, message: &str) -> ApiError {
    let lower = message.to_lowercase();
    match kind {
        "authentication_error" => ApiError::Auth,
        "permission_error" => ApiError::Forbidden,
        "not_found_error" => ApiError::UnknownModel,
        "rate_limit_error" => ApiError::RateLimited,
        "overloaded_error" => ApiError::Overloaded,
        "request_too_large" => ApiError::TooLarge,
        _ if lower.contains("credit balance") || lower.contains("billing") => ApiError::NoCredit,
        _ if lower.contains("image") && (lower.contains("exceed") || lower.contains("too large")) => {
            ApiError::TooLarge
        }
        _ => ApiError::Other(message.chars().take(300).collect()),
    }
}
