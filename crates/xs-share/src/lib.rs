//! Sharing (SHR-): for now, the local network with other XcreenShot users (SHR-042 to
//! SHR-044, IDEA-005). Nothing leaves the local network, and nothing listens or is
//! announced while the preference is off (APP-054).

pub mod discovery;
pub mod identity;
pub mod lan;
pub mod store;
pub mod tls;
pub mod wire;

use std::sync::Arc;

use serde_json::{json, Value};
use xs_core::core::Extension;
use xs_core::i18n::tr;
use xs_core::{CommandError, Context, Core, ErrorCode, Lang, Source};

use lan::lan;

fn bad(lang: Lang, param: &str, reason: &str) -> CommandError {
    CommandError::new(
        ErrorCode::InvalidParams,
        tr(
            lang,
            "errors.invalid_param",
            &[("param", param), ("reason", reason)],
        ),
    )
}

fn off(lang: Lang) -> CommandError {
    CommandError::new(ErrorCode::FeatureDisabled, tr(lang, "errors.lan_off", &[]))
}

fn not_found(lang: Lang, what: &str) -> CommandError {
    CommandError::new(
        ErrorCode::NotFound,
        tr(lang, "errors.lan_not_found", &[("id", what)]),
    )
}

/// Accepting for the user from the command line or an agent needs its preference
/// (SHR-044).
fn automation_allowed(core: &Core, ctx: &Context) -> Result<(), CommandError> {
    if matches!(ctx.source, Source::Cli | Source::Mcp) && !core.settings.get_bool("share.lanAutomation") {
        return Err(CommandError::new(
            ErrorCode::PermissionDenied,
            tr(ctx.lang, "errors.lan_automation", &[]),
        ));
    }
    Ok(())
}

fn strings(v: &Value) -> Vec<String> {
    match v {
        Value::String(s) => vec![s.clone()],
        Value::Array(a) => a.iter().filter_map(Value::as_str).map(str::to_string).collect(),
        _ => Vec::new(),
    }
}

pub fn extension() -> Extension {
    Box::new(|reg, weak| {
        if let Some(core) = weak.upgrade() {
            let service = lan::install(&core, weak.clone());
            // On and off with its preference; the visible name announced again.
            let w = Arc::downgrade(&service);
            core.on_setting(Arc::new(move |key, _| {
                if key == "share.lan" || key == "share.lanName" {
                    if let Some(s) = w.upgrade() {
                        std::thread::spawn(move || s.apply());
                    }
                }
            }));
            if core.settings.get_bool("share.lan") {
                std::thread::spawn(move || service.apply());
            }
        }
        macro_rules! on {
            ($id:expr, |$core:ident, $ctx:ident, $p:ident| $body:expr) => {{
                let weak = weak.clone();
                reg.handle($id, move |ctx: &Context, $p: Value| {
                    let $core = weak
                        .upgrade()
                        .ok_or_else(|| CommandError::internal("core gone"))?;
                    let $ctx = ctx;
                    $body
                });
            }};
        }
        on!("share.lan_status", |core, _ctx, _p| Ok(lan(&core).status()));
        on!("share.lan_people", |core, _ctx, _p| Ok(
            json!({ "people": lan(&core).people() })
        ));
        on!("share.lan_pair", |core, ctx, p| {
            let fp = p["fingerprint"].as_str().unwrap_or_default();
            match lan(&core).pair(fp) {
                Ok((id, code)) => Ok(json!({ "id": id, "code": code })),
                Err(e) if e == "off" => Err(off(ctx.lang)),
                Err(e) if e == "offline" => Err(CommandError::new(
                    ErrorCode::NotRunning,
                    tr(ctx.lang, "errors.lan_offline", &[]),
                )),
                Err(e) => Err(CommandError::new(
                    ErrorCode::Internal,
                    tr(ctx.lang, "errors.lan_failed", &[("reason", &e)]),
                )),
            }
        });
        on!("share.lan_pairings", |core, _ctx, _p| Ok(
            json!({ "pairings": lan(&core).pairings() })
        ));
        on!("share.lan_pair_answer", |core, ctx, p| {
            automation_allowed(&core, ctx)?;
            let id = p["id"].as_str().unwrap_or_default();
            lan(&core)
                .answer_pairing(id, p["accept"].as_bool().unwrap_or(false))
                .map_err(|_| not_found(ctx.lang, id))?;
            Ok(json!({ "id": id }))
        });
        on!("share.lan_contact", |core, ctx, p| {
            let fp = p["fingerprint"].as_str().unwrap_or_default();
            let action = p["action"].as_str().unwrap_or_default();
            lan(&core).contact_action(fp, action, &p["value"]).map_err(|e| {
                if e == "missing" {
                    not_found(ctx.lang, fp)
                } else {
                    bad(
                        ctx.lang,
                        "action",
                        "rename, alwaysAccept, remove, block or unblock",
                    )
                }
            })?;
            Ok(json!({ "fingerprint": fp }))
        });
        on!("share.lan_send", |core, ctx, p| {
            let to = strings(&p["to"]);
            let paths = strings(&p["paths"]);
            if to.is_empty() {
                return Err(bad(ctx.lang, "to", "choose at least one contact"));
            }
            if paths.is_empty() {
                return Err(bad(ctx.lang, "paths", "choose at least one item"));
            }
            let sends = lan(&core)
                .send(&to, &paths)
                .map_err(|e| bad(ctx.lang, "paths", &e))?;
            Ok(json!({ "sends": sends }))
        });
        on!("share.lan_sends", |core, _ctx, _p| Ok(
            json!({ "sends": lan(&core).sends() })
        ));
        on!("share.lan_cancel", |core, ctx, p| {
            let id = p["id"].as_str().unwrap_or_default();
            lan(&core).cancel(id).map_err(|_| not_found(ctx.lang, id))?;
            Ok(json!({ "id": id }))
        });
        on!("share.lan_incoming", |core, _ctx, _p| Ok(
            json!({ "incoming": lan(&core).incoming() })
        ));
        on!("share.lan_answer", |core, ctx, p| {
            automation_allowed(&core, ctx)?;
            let id = p["id"].as_str().unwrap_or_default();
            lan(&core)
                .answer_incoming(id, p["accept"].as_bool().unwrap_or(false))
                .map_err(|_| not_found(ctx.lang, id))?;
            Ok(json!({ "id": id }))
        });
    })
}

#[cfg(test)]
mod tests;
