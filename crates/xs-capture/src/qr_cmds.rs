//! QR codes (SMT-036, SMT-038): `smart.qr_create` makes a code (a preview, a file or a
//! QR object of a document) and checks it reads; `smart.qr_read` reads the codes of a
//! document, a part of it or a file. All on this computer: nothing goes to the network.

use std::path::PathBuf;

use serde_json::{json, Value};
use xs_core::i18n::tr;
use xs_core::{CommandError, CommandResult, Context, Core, ErrorCode};
use xs_image::qr::{self, QrStyle};

fn invalid(lang: xs_core::Lang, param: &str, reason: &str) -> CommandError {
    CommandError::new(
        ErrorCode::InvalidParams,
        tr(
            lang,
            "errors.invalid_param",
            &[("param", param), ("reason", reason)],
        ),
    )
}

/// The style asked, over the defaults.
fn style_of(lang: xs_core::Lang, p: &Value) -> Result<QrStyle, CommandError> {
    let mut v = serde_json::to_value(QrStyle::default()).unwrap_or_default();
    for k in ["kind", "ecc", "color", "background", "margin"] {
        if let Some(x) = p.get(k) {
            v[k] = x.clone();
        }
    }
    let style: QrStyle = serde_json::from_value(v).map_err(|e| invalid(lang, "style", &e.to_string()))?;
    qr::check_style(&style).map_err(|e| invalid(lang, "style", &e))?;
    Ok(style)
}

/// `smart.qr_create` (SMT-036).
pub fn create(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let style = style_of(lang, p)?;
    let raw = p.get("content").and_then(Value::as_str).unwrap_or_default();
    let content = if style.kind == "link" {
        qr::link(raw)
    } else {
        raw.to_string()
    };
    if content.is_empty() {
        return Err(invalid(lang, "content", "empty"));
    }
    let capacity = style.ecc.capacity();
    if content.len() > capacity {
        return Err(CommandError::new(
            ErrorCode::InvalidParams,
            tr(
                lang,
                "errors.qr_too_long",
                &[
                    ("bytes", &content.len().to_string()),
                    ("max", &capacity.to_string()),
                ],
            ),
        )
        .with_details(json!({"bytes": content.len(), "max": capacity})));
    }
    let (modules, _) = qr::matrix(&content, style.ecc).map_err(|e| invalid(lang, "content", &e))?;
    // Read with our own reader before anyone relies on it.
    let check = qr::picture(&content, &style, (modules as u32 + 2 * style.margin) * 8)
        .map(|img| qr::read(&img).iter().any(|f| f.text == content))
        .unwrap_or(false);
    let mut out = json!({
        "content": content,
        "modules": modules,
        "bytes": content.len(),
        "max": capacity,
        "readable": check,
        "hardToRead": qr::hard_to_read(&style),
    });
    if p.get("preview").and_then(Value::as_bool) == Some(true) {
        out["svg"] = json!(qr::svg(&content, &style).map_err(|e| invalid(lang, "content", &e))?);
    }
    // A file: PNG or SVG by its extension.
    if let Some(path) = p.get("path").and_then(Value::as_str) {
        let path = PathBuf::from(path);
        let svg = path
            .extension()
            .is_some_and(|e| e.to_string_lossy().eq_ignore_ascii_case("svg"));
        if svg {
            let text = qr::svg(&content, &style).map_err(|e| invalid(lang, "content", &e))?;
            std::fs::write(&path, text).map_err(|e| CommandError::internal(e.to_string()))?;
        } else {
            let size = p.get("size").and_then(Value::as_u64).unwrap_or(512) as u32;
            let img = qr::picture(&content, &style, size).map_err(CommandError::internal)?;
            let png = xs_image::encode::png(&img, 0.0, true).map_err(CommandError::internal)?;
            std::fs::write(&path, png).map_err(|e| CommandError::internal(e.to_string()))?;
        }
        out["path"] = json!(path);
    }
    // A QR object of a document: new, or the one `objectId` changed.
    if let Some(doc) = p.get("documentId").and_then(Value::as_str) {
        let id = xs_editor::resolve_document(core, lang, &json!({"documentId": doc}))?;
        let style_v = serde_json::to_value(&style).unwrap_or_default();
        match p.get("objectId").and_then(Value::as_str) {
            Some(oid) => {
                core.call(
                    ctx.source,
                    "document.update_object",
                    json!({"documentId": id, "objectIds": [oid], "set": {"content": content, "style": style_v}}),
                    Some(lang),
                )?;
                out["objectId"] = json!(oid);
            }
            None => {
                let rect = match p.get("rect") {
                    Some(r) => r.clone(),
                    None => {
                        // In the middle, a third of the shorter side (at most 240 px).
                        let (w, h) = xs_editor::editor(core)
                            .store
                            .with(&id, |d| {
                                (d.document.canvas.width as f64, d.document.canvas.height as f64)
                            })
                            .unwrap_or((600.0, 400.0));
                        let side = (w.min(h) / 3.0).clamp(48.0, 240.0).round();
                        json!({"x": ((w - side) / 2.0).round(), "y": ((h - side) / 2.0).round(), "w": side, "h": side})
                    }
                };
                let r = core.call(
                    ctx.source,
                    "document.annotate",
                    json!({"documentId": id, "objects": [{"type": "qr", "rect": rect, "content": content, "style": style_v}]}),
                    Some(lang),
                )?;
                out["objectId"] = r["ids"][0].clone();
            }
        }
        out["documentId"] = json!(id);
    }
    Ok(out)
}

/// What a code holds: a link, Wi-Fi details (the password only on demand) or text.
fn described(text: &str) -> Value {
    let lower = text.to_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") {
        let host = text
            .split_once("://")
            .map(|(_, rest)| rest.split(['/', '?', '#']).next().unwrap_or_default())
            .unwrap_or_default();
        return json!({"kind": "url", "host": host});
    }
    if let Some(rest) = text.strip_prefix("WIFI:") {
        let mut wifi = json!({"kind": "wifi"});
        for part in rest.split(';') {
            if let Some((k, v)) = part.split_once(':') {
                match k {
                    "S" => wifi["ssid"] = json!(v),
                    "T" => wifi["security"] = json!(v),
                    "P" => wifi["password"] = json!(v),
                    _ => {}
                }
            }
        }
        return wifi;
    }
    json!({"kind": "text"})
}

/// The QR codes on the frozen screens while capturing (SMT-038): what each says and
/// where it is on the virtual screen (physical px). `monitors` are the selection
/// request's (their `rect`), in the order of `frames`.
pub fn codes_on_screen(frames: &[xs_core::host::Frame], monitors: &[Value]) -> Vec<Value> {
    let mut out = Vec::new();
    for (frame, m) in frames.iter().zip(monitors) {
        let ox = m["rect"]["x"].as_f64().unwrap_or(0.0);
        let oy = m["rect"]["y"].as_f64().unwrap_or(0.0);
        let Some(img) = image::RgbaImage::from_raw(frame.width, frame.height, frame.rgba.as_ref().clone())
        else {
            continue;
        };
        out.extend(codes_in(&img, (ox, oy)));
    }
    out
}

/// The QR codes of `img`, what each is and where (moved by `offset`) (SMT-038).
pub fn codes_in(img: &image::RgbaImage, offset: (f64, f64)) -> Vec<Value> {
    xs_image::qr::read(img)
        .into_iter()
        .map(|f| {
            let mut v = described(&f.text);
            v["text"] = json!(f.text);
            v["rect"] = json!([f.rect[0] + offset.0, f.rect[1] + offset.1, f.rect[2], f.rect[3]]);
            v
        })
        .collect()
}

/// `smart.qr_read` (SMT-038).
pub fn read(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let (image, offset, id) = match p.get("path").and_then(Value::as_str) {
        Some(path) => {
            let img = image::open(path)
                .map_err(|e| invalid(lang, "path", &e.to_string()))?
                .to_rgba8();
            (img, (0.0, 0.0), None)
        }
        None => {
            let id = xs_editor::resolve_document(core, lang, p)?;
            xs_editor::video::refuse_for_video(core, lang, &json!({"documentId": id}))?;
            // What shows, objects too.
            let snap = xs_editor::editor(core)
                .store
                .snapshot(&id)
                .map_err(CommandError::internal)?;
            let (img, origin) = snap.render_with_origin();
            (img, (origin[0] as f64, origin[1] as f64), Some(id))
        }
    };
    // A part of it (canvas pixels).
    let (img, shift) = match p.get("rect").and_then(Value::as_array) {
        Some(r) if r.len() == 4 => {
            let n = |i: usize| r[i].as_f64().unwrap_or(0.0);
            let x = (n(0) + offset.0).max(0.0) as u32;
            let y = (n(1) + offset.1).max(0.0) as u32;
            let w = (n(2).max(1.0) as u32).min(image.width().saturating_sub(x));
            let h = (n(3).max(1.0) as u32).min(image.height().saturating_sub(y));
            (
                image::imageops::crop_imm(&image, x, y, w.max(1), h.max(1)).to_image(),
                (x as f64 - offset.0, y as f64 - offset.1),
            )
        }
        _ => (image, (-offset.0, -offset.1)),
    };
    let codes: Vec<Value> = qr::read(&img)
        .into_iter()
        .map(|f| {
            let mut v = described(&f.text);
            v["text"] = json!(f.text);
            v["rect"] = json!([f.rect[0] + shift.0, f.rect[1] + shift.1, f.rect[2], f.rect[3]]);
            v
        })
        .collect();
    // Counts only, never the contents.
    tracing::info!(codes = codes.len(), "qr read");
    let mut out = json!({"codes": codes});
    if let Some(id) = id {
        out["documentId"] = json!(id);
    }
    Ok(out)
}

/// `smart.qr_open` (SMT-038): opens a code's web link in the browser. Only `http` and
/// `https`, and only when the user said yes in the panel: nothing opens by itself.
pub fn open(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let url = p.get("url").and_then(Value::as_str).unwrap_or_default().trim();
    let lower = url.to_lowercase();
    if !(lower.starts_with("http://") || lower.starts_with("https://")) {
        return Err(invalid(lang, "url", "only http and https links open"));
    }
    core.host().open_uri(url)?;
    Ok(json!({"opened": true}))
}

pub fn register(reg: &mut xs_core::command::Registry, weak: &std::sync::Weak<Core>) {
    type Run = fn(&Core, &Context, &Value) -> CommandResult;
    let handlers: [(&str, Run); 3] = [
        ("smart.qr_create", create),
        ("smart.qr_read", read),
        ("smart.qr_open", open),
    ];
    for (name, run) in handlers {
        let w = weak.clone();
        reg.handle(name, move |ctx, p| {
            let core = w
                .upgrade()
                .ok_or_else(|| CommandError::internal("shutting down"))?;
            run(&core, ctx, &p)
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use xs_core::Source;

    /// SMT-038, while capturing: the codes of two frozen monitors (the second left of
    /// the first), on the virtual screen, with what each is.
    #[test]
    fn codes_on_the_frozen_screens() {
        let style = xs_image::qr::QrStyle::default();
        let frame = |content: &str, at: (i64, i64)| {
            let mut img = image::RgbaImage::from_pixel(800, 600, image::Rgba([250, 250, 250, 255]));
            let code = xs_image::qr::picture(content, &style, 240).unwrap();
            image::imageops::overlay(&mut img, &code, at.0, at.1);
            xs_core::host::Frame {
                width: 800,
                height: 600,
                rgba: Arc::new(img.into_raw()),
            }
        };
        let frames = vec![
            frame("https://example.com/menu", (400, 300)),
            frame("WIFI:S:Casa;T:WPA;P:secreta;;", (100, 50)),
        ];
        let monitors = vec![
            json!({"rect": {"x": 0, "y": 0, "width": 800, "height": 600}}),
            json!({"rect": {"x": -800, "y": 120, "width": 800, "height": 600}}),
        ];
        let codes = codes_on_screen(&frames, &monitors);
        assert_eq!(codes.len(), 2, "{codes:?}");
        let url = codes.iter().find(|c| c["kind"] == "url").unwrap();
        assert_eq!(url["host"], "example.com");
        let r = url["rect"].as_array().unwrap();
        assert!((r[0].as_f64().unwrap() - 400.0).abs() < 40.0, "{r:?}");
        let wifi = codes.iter().find(|c| c["kind"] == "wifi").unwrap();
        assert_eq!(wifi["ssid"], "Casa");
        let r = wifi["rect"].as_array().unwrap();
        assert!(
            r[0].as_f64().unwrap() < -600.0 && r[1].as_f64().unwrap() > 150.0,
            "{r:?}"
        );
    }

    /// How long the codes of a 4K screen take (SMT-038: in the background, never
    /// before the crosshair). `cargo test -p xs-capture --release -- --ignored --nocapture a_4k_screen`
    #[test]
    #[ignore]
    fn a_4k_screen() {
        let style = xs_image::qr::QrStyle::default();
        let mut img = image::RgbaImage::from_pixel(3840, 2160, image::Rgba([240, 240, 240, 255]));
        let code = xs_image::qr::picture("https://example.com", &style, 300).unwrap();
        image::imageops::overlay(&mut img, &code, 2000, 1200);
        let frames = vec![xs_core::host::Frame {
            width: 3840,
            height: 2160,
            rgba: Arc::new(img.into_raw()),
        }];
        let monitors = vec![json!({"rect": {"x": 0, "y": 0}})];
        let t = std::time::Instant::now();
        let codes = codes_on_screen(&frames, &monitors);
        println!("{} code(s) in {} ms", codes.len(), t.elapsed().as_millis());
        assert_eq!(codes.len(), 1);
    }

    fn core() -> (Arc<Core>, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let core = Core::for_tests_with(
            xs_core::platform::Platform::Windows,
            vec![crate::extension(), xs_editor::extension()],
        );
        xs_editor::editor(&core).set_folders(dir.path().join("lib"), dir.path().join("rec"));
        (core, dir)
    }

    /// SMT-036: a code of example.com inserted and exported reads back as the link; a
    /// light gray code warns; double-click's change updates it; enlarged 4×, sharp.
    #[test]
    fn create_insert_and_read() {
        let (core, dir) = core();
        let call = |cmd: &str, p: Value| core.call(Source::Cli, cmd, p, Some(xs_core::Lang::Es));
        let (id, _) = xs_editor::add_capture(
            &core,
            &image::RgbaImage::from_pixel(800, 600, image::Rgba([240, 240, 240, 255])),
            json!({}),
            1.0,
            None,
        )
        .unwrap();
        let r = call(
            "smart.qr_create",
            json!({"content": "example.com", "documentId": id}),
        )
        .unwrap();
        assert_eq!(
            (r["content"].as_str(), r["readable"].as_bool()),
            (Some("https://example.com"), Some(true)),
            "{r}"
        );
        let oid = r["objectId"].as_str().unwrap().to_string();
        // Exported to PNG, it reads back.
        let out = dir.path().join("out.png");
        call(
            "document.export",
            json!({"documentId": id, "format": "png", "path": out}),
        )
        .unwrap();
        let r = call("smart.qr_read", json!({"path": out})).unwrap();
        assert_eq!(r["codes"][0]["text"], "https://example.com", "{r}");
        assert_eq!(r["codes"][0]["kind"], "url");
        assert_eq!(r["codes"][0]["host"], "example.com");

        // Changed (double-click's panel): the code follows.
        call(
            "smart.qr_create",
            json!({"content": "https://example.org/x", "documentId": id, "objectId": oid}),
        )
        .unwrap();
        let r = call("smart.qr_read", json!({"documentId": id})).unwrap();
        assert_eq!(r["codes"][0]["text"], "https://example.org/x", "{r}");

        // 4× bigger: the modules stay sharp (only the code's two colors).
        call(
            "document.update_object",
            json!({"documentId": id, "objectIds": [oid], "set": {"rect": {"x": 0, "y": 0, "w": 600, "h": 600}}}),
        )
        .unwrap();
        let img = xs_editor::editor(&core).store.snapshot(&id).unwrap().render();
        for y in (40..560).step_by(7) {
            for x in (40..560).step_by(7) {
                let p = img.get_pixel(x, y).0;
                assert!(p[0] == 0 || p[0] == 255, "a soft edge at {x},{y}: {p:?}");
            }
        }

        // Light gray on white: hard to read.
        let r = call(
            "smart.qr_create",
            json!({"content": "hola", "kind": "text", "color": "#D9D9D9"}),
        )
        .unwrap();
        assert_eq!(r["hardToRead"], true, "{r}");
        // Too long for the correction chosen.
        let e = call(
            "smart.qr_create",
            json!({"content": "a".repeat(1300), "kind": "text", "ecc": "H"}),
        )
        .unwrap_err();
        assert_eq!(e.code, ErrorCode::InvalidParams);
    }

    /// SMT-038: three codes in one image are read, each with its place; Wi-Fi details
    /// are told apart; files PNG and SVG are written.
    #[test]
    fn three_codes_and_files() {
        let (core, dir) = core();
        let call = |cmd: &str, p: Value| core.call(Source::Cli, cmd, p, Some(xs_core::Lang::Es)).unwrap();
        let mut canvas = image::RgbaImage::from_pixel(1000, 400, image::Rgba([255, 255, 255, 255]));
        for (i, text) in [
            "https://a.example",
            "texto libre",
            "WIFI:T:WPA;S:Oficina;P:secreto;;",
        ]
        .iter()
        .enumerate()
        {
            let code = qr::picture(text, &QrStyle::default(), 260).unwrap();
            image::imageops::overlay(&mut canvas, &code, 40 + i as i64 * 320, 70);
        }
        let file = dir.path().join("three.png");
        canvas.save(&file).unwrap();
        let r = call("smart.qr_read", json!({"path": file}));
        let codes = r["codes"].as_array().unwrap();
        assert_eq!(codes.len(), 3, "{r}");
        let wifi = codes
            .iter()
            .find(|c| c["kind"] == "wifi")
            .expect("the Wi-Fi code");
        assert_eq!(
            (wifi["ssid"].as_str(), wifi["password"].as_str()),
            (Some("Oficina"), Some("secreto"))
        );
        assert!(codes.iter().any(|c| c["kind"] == "url") && codes.iter().any(|c| c["kind"] == "text"));
        // Part of it: only the middle one, placed in the whole image's pixels.
        let r = call("smart.qr_read", json!({"path": file, "rect": [330, 0, 300, 400]}));
        assert_eq!(r["codes"].as_array().unwrap().len(), 1, "{r}");
        assert!(r["codes"][0]["rect"][0].as_f64().unwrap() > 330.0);

        let png = dir.path().join("code.png");
        let svg = dir.path().join("code.svg");
        call(
            "smart.qr_create",
            json!({"content": "https://example.com", "path": png}),
        );
        call(
            "smart.qr_create",
            json!({"content": "https://example.com", "path": svg}),
        );
        assert!(std::fs::read_to_string(&svg).unwrap().starts_with("<svg"));
        let r = call("smart.qr_read", json!({"path": png}));
        assert_eq!(r["codes"][0]["text"], "https://example.com");
    }
}
