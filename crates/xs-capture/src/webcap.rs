//! A web page by its address (CAP-037): the address is checked here and the page is
//! loaded and taken by the app (a private web view, `Host::load_webpage`); the image
//! then goes down the capture route like any other, with the final address and the
//! page's title in its metadata. It is one of the few things that use the network, and
//! only when the user asks (APP-054).

use base64::Engine;
use serde_json::{json, Value};
use xs_core::i18n::tr;
use xs_core::{CommandError, CommandResult, Context, Core, ErrorCode};

use crate::{finish, Options, Rect, Shot};

/// The address to load: `https://` added when there is no scheme; only `http` and
/// `https` (CAP-037).
pub fn normalize(text: &str) -> Result<String, String> {
    let t = text.trim();
    if t.is_empty() {
        return Err("empty".into());
    }
    let lower = t.to_ascii_lowercase();
    let full = if lower.starts_with("http://") || lower.starts_with("https://") {
        t.to_string()
    } else if lower.contains("://") || lower.starts_with("javascript:") || lower.starts_with("file:") {
        return Err("scheme".into());
    } else {
        format!("https://{t}")
    };
    let host = full
        .split("://")
        .nth(1)
        .unwrap_or_default()
        .split(['/', '?', '#'])
        .next()
        .unwrap_or_default();
    let host = host.rsplit('@').next().unwrap_or_default();
    let name = host.split(':').next().unwrap_or_default();
    if name.is_empty() || name.contains(' ') || !(name.contains('.') || name == "localhost") {
        return Err("host".into());
    }
    Ok(full)
}

/// Whether a text looks like a web address (to fill the address box from the
/// clipboard, CAP-037).
pub fn looks_like_url(text: &str) -> bool {
    let t = text.trim();
    !t.contains(char::is_whitespace)
        && t.len() < 2048
        && (t.starts_with("http://") || t.starts_with("https://") || t.starts_with("www."))
        && normalize(t).is_ok()
}

/// `capture.webpage`: loads the page and makes the capture.
pub fn run(core: &Core, ctx: &Context, opts: &Options, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let url = normalize(p.get("url").and_then(Value::as_str).unwrap_or_default())
        .map_err(|_| CommandError::new(ErrorCode::InvalidParams, tr(lang, "errors.webpage_url", &[])))?;
    core.remember(LAST_URL_KEY, json!(url));
    // Scale: 1×, 2×, or that of the main monitor (CAP-037).
    let scale = match p.get("scale").and_then(Value::as_f64) {
        Some(s) if s >= 1.0 => s.min(2.0).round(),
        _ => {
            let primary = crate::monitors()
                .ok()
                .and_then(|list| {
                    list.into_iter()
                        .find(|(_, i)| i.primary)
                        .map(|(_, i)| i.scale as f64)
                })
                .unwrap_or(1.0);
            if primary >= 1.5 {
                2.0
            } else {
                1.0
            }
        }
    };
    let request = json!({
        "url": url,
        "width": p.get("width").and_then(Value::as_u64).unwrap_or(1280).clamp(320, 3840),
        "scale": scale,
        "wait": p.get("wait").and_then(Value::as_f64).unwrap_or(1.0).clamp(0.0, 10.0),
        "lazy": p.get("lazy").and_then(Value::as_bool).unwrap_or(true),
    });
    tracing::info!(width = request["width"].as_u64(), scale, "web page: loading");
    let page = core
        .host()
        .load_webpage(&request)
        .map_err(|why| match why.as_str() {
            "cancelled" => crate::cancelled(lang),
            "timeout" => CommandError::new(
                ErrorCode::Timeout,
                tr(lang, "errors.webpage_timeout", &[("url", &url)]),
            ),
            "unreachable" => CommandError::new(
                ErrorCode::NotFound,
                tr(lang, "errors.webpage_unreachable", &[("url", &url)]),
            ),
            other => CommandError::new(
                ErrorCode::Internal,
                tr(lang, "errors.webpage_failed", &[("url", &url), ("reason", other)]),
            ),
        })?;
    let png = base64::engine::general_purpose::STANDARD
        .decode(page["png"].as_str().unwrap_or_default())
        .map_err(|e| CommandError::internal(e.to_string()))?;
    let image = xs_image::encode::decode(&png).map_err(CommandError::internal)?;
    let title = page["title"].as_str().unwrap_or_default().to_string();
    let final_url = page["url"].as_str().unwrap_or(&url).to_string();
    let status = page["status"].as_u64().unwrap_or(0);
    let mut opts = opts.clone();
    opts.cursor = false;
    opts.meta = Some(json!({ "url": final_url, "pageTitle": title, "httpStatus": status }));
    let shot = Shot {
        rect: Rect {
            x: 0,
            y: 0,
            width: image.width(),
            height: image.height(),
        },
        image,
        mode: "webpage",
        scale: scale as f32,
        monitor: String::new(),
        app: None,
        window_title: (!title.is_empty()).then_some(title),
        window_id: None,
        pointer: None,
        warnings: Vec::new(),
    };
    let result = finish(core, ctx, shot, &opts)?;
    let name = tr(lang, "app.name", &[]);
    // An error page is taken anyway, and said (CAP-037).
    if status >= 400 {
        core.host().notify(
            "problems",
            &name,
            &tr(lang, "notify.webpage_status", &[("code", &status.to_string())]),
        );
    }
    if page["cut"].as_bool() == Some(true) {
        core.host()
            .notify("problems", &name, &tr(lang, "notify.webpage_cut", &[]));
    }
    Ok(result)
}

/// The last address captured, to suggest it again.
pub const LAST_URL_KEY: &str = "capture.webpage.lastUrl";

/// `capture.webpage_suggest`: the address on the clipboard, else the last one.
pub fn suggest(core: &Core) -> Value {
    let clip = {
        let _lock = xs_core::clipboard_lock();
        arboard::Clipboard::new().ok().and_then(|mut c| c.get_text().ok())
    };
    if let Some(text) = clip.filter(|t| looks_like_url(t)) {
        return json!({ "url": text.trim(), "from": "clipboard" });
    }
    match core.recall(LAST_URL_KEY) {
        Some(url) => json!({ "url": url, "from": "last" }),
        None => json!({ "url": "", "from": "none" }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addresses() {
        assert_eq!(normalize("example.com").unwrap(), "https://example.com");
        assert_eq!(normalize(" https://a.org/x?y=1 ").unwrap(), "https://a.org/x?y=1");
        assert_eq!(
            normalize("http://localhost:8080/p").unwrap(),
            "http://localhost:8080/p"
        );
        assert!(normalize("ftp://a.org").is_err());
        assert!(normalize("javascript:alert(1)").is_err());
        assert!(normalize("file:///C:/x.html").is_err());
        assert!(normalize("not a url").is_err());
        assert!(normalize("").is_err());
    }

    #[test]
    fn clipboard_addresses() {
        assert!(looks_like_url("https://example.com/page"));
        assert!(looks_like_url("www.example.com"));
        assert!(!looks_like_url("hello world"));
        assert!(!looks_like_url("example"));
    }
}
