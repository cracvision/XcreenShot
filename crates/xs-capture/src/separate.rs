//! Separate elements (SMT-018): the interface elements and the pieces of text of the
//! image (or of a floating image) become floating images of their own, to move, resize
//! or delete; the gaps they leave are rebuilt from the background around them (Smart
//! fill, EDT-045). Detail regroups them (with changes made, it asks first); Reset puts
//! the image back as it was; Done flattens them into the image. The whole session is
//! one undo step. A piece of text, double-clicked, becomes a Text object (SMT-009).

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Mutex;

use serde_json::{json, Map, Value};
use xs_core::i18n::tr;
use xs_core::{CommandError, CommandResult, Context, Core, ErrorCode};
use xs_image::selection::Mask;
use xs_image::ui_detect;

use crate::redact;

pub const MARK: &str = "separated";

/// A session per document: where its undo steps started and what it works on.
struct Session {
    undo_len: usize,
    target: Option<String>,
}

static SESSIONS: Mutex<Vec<(String, Session)>> = Mutex::new(Vec::new());

fn session_key(core: &Core, id: &str) -> String {
    format!("{:p}/{id}", core)
}

fn fingerprint(props: &Map<String, Value>) -> String {
    let mut p = props.clone();
    p.remove(MARK);
    let mut h = DefaultHasher::new();
    serde_json::to_string(&p).unwrap_or_default().hash(&mut h);
    format!("{:016x}", h.finish())
}

/// The separated objects: their IDs, and how many the user changed or deleted.
fn separated(doc: &xs_core::document::Document) -> (Vec<String>, usize) {
    let mut ids = Vec::new();
    let mut edited = 0;
    let mut total = 0u64;
    for o in doc.objects.iter() {
        let Some(mark) = o.props.get(MARK) else { continue };
        ids.push(o.id.clone());
        total = mark.get("total").and_then(Value::as_u64).unwrap_or(total);
        if mark.get("fp").and_then(Value::as_str) != Some(fingerprint(&o.props).as_str()) {
            edited += 1;
        }
    }
    edited += (total as usize).saturating_sub(ids.len());
    (ids, edited)
}

/// A piece to separate: its place in the target's pixels, and its text if it is text.
struct Piece {
    rect: [u32; 4],
    text: Option<(String, String, f64)>,
}

fn holds(outer: [u32; 4], inner: [u32; 4]) -> bool {
    inner[0] >= outer[0]
        && inner[1] >= outer[1]
        && inner[0] + inner[2] <= outer[0] + outer[2]
        && inner[1] + inner[3] <= outer[1] + outer[3]
}

/// The pieces of `pixels` at `detail`: elements, and text by block, line or word.
fn pieces(pixels: &image::RgbaImage, text: Option<&xs_ocr::Recognized>, detail: u8) -> Vec<Piece> {
    let (w, h) = pixels.dimensions();
    let clamp = |r: [f64; 4], pad: f64| -> [u32; 4] {
        let x0 = (r[0] - pad).floor().max(0.0) as u32;
        let y0 = (r[1] - pad).floor().max(0.0) as u32;
        let x1 = ((r[0] + r[2] + pad).ceil().max(0.0) as u32).min(w);
        let y1 = ((r[1] + r[3] + pad).ceil().max(0.0) as u32).min(h);
        [x0, y0, x1.saturating_sub(x0).max(1), y1.saturating_sub(y0).max(1)]
    };
    let lines: Vec<&xs_ocr::Line> = text
        .iter()
        .flat_map(|r| r.blocks.iter().flat_map(|b| &b.lines))
        .collect();
    let line_rects: Vec<[f64; 4]> = lines.iter().map(|l| l.rect).collect();
    let mut out: Vec<Piece> = ui_detect::detect(pixels, &line_rects, detail)
        .into_iter()
        .map(|e| Piece {
            rect: e.rect,
            text: None,
        })
        .collect();
    let pad = 3.0;
    let mut texts: Vec<Piece> = Vec::new();
    if detail >= 8 {
        // Each word.
        for l in &lines {
            for word in &l.words {
                texts.push(Piece {
                    rect: clamp(word.rect, pad),
                    text: Some((word.text.clone(), l.color.clone(), l.rect[3])),
                });
            }
        }
    } else if detail >= 5 {
        for l in &lines {
            texts.push(Piece {
                rect: clamp(l.rect, pad),
                text: Some((l.text.clone(), l.color.clone(), l.rect[3])),
            });
        }
    } else {
        for b in ui_detect::text_blocks(&line_rects, detail) {
            let inside: Vec<&&xs_ocr::Line> = lines
                .iter()
                .filter(|l| {
                    l.rect[1] >= b.rect[1] - 1.0 && l.rect[1] + l.rect[3] <= b.rect[1] + b.rect[3] + 1.0
                })
                .collect();
            let words: Vec<String> = inside.iter().map(|l| l.text.clone()).collect();
            let color = inside.first().map(|l| l.color.clone()).unwrap_or_default();
            texts.push(Piece {
                rect: clamp(b.rect, pad),
                text: Some((words.join("\n"), color, b.line_height)),
            });
        }
    }
    // Text inside an element (a button's label) moves with it.
    texts.retain(|t| {
        !out.iter().any(|e| {
            let r = e.rect;
            let c = (t.rect[0] + t.rect[2] / 2, t.rect[1] + t.rect[3] / 2);
            holds(r, t.rect) || (c.0 >= r[0] && c.0 < r[0] + r[2] && c.1 >= r[1] && c.1 < r[1] + r[3])
        })
    });
    out.extend(texts);
    out
}

/// `smart.separate`: starts the session, or with another detail separates again.
pub fn command(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let id = xs_editor::resolve_document(core, lang, p)?;
    xs_editor::video::refuse_for_video(core, lang, &json!({"documentId": id}))?;
    let ed = xs_editor::editor(core);
    let key = session_key(core, &id);
    let detail = p.get("detail").and_then(Value::as_u64).unwrap_or(5).clamp(1, 10) as u8;
    let force = p.get("force").and_then(Value::as_bool) == Some(true);
    let existing = {
        let s = SESSIONS.lock().unwrap();
        s.iter()
            .find(|(k, _)| *k == key)
            .map(|(_, s)| (s.undo_len, s.target.clone()))
    };
    let target = match &existing {
        Some((_, t)) => t.clone(),
        None => p.get("objectId").and_then(Value::as_str).map(str::to_string),
    };
    if let Some((undo_len, _)) = existing {
        let (_, edited) = ed
            .store
            .with(&id, |d| separated(&d.document))
            .ok_or_else(|| CommandError::internal("document closed"))?;
        if edited > 0 && !force {
            return Ok(json!({"documentId": id, "needsConfirm": true, "edited": edited}));
        }
        // Back to before the session, to separate anew.
        ed.store.rewind(&id, undo_len);
    }
    let undo_len = ed.store.undo_len(&id).unwrap_or(0);

    // The pixels to separate, and how they map to the canvas.
    let (pixels, map) = xs_editor::separable_pixels(core, &id, target.as_deref())
        .map_err(|e| CommandError::new(ErrorCode::InvalidParams, e))?;
    let languages = crate::extract::wanted(core, None);
    let reading = redact::read(&pixels, &languages, lang)?;
    let found = pieces(&pixels, reading.text.as_ref(), detail);
    if found.is_empty() {
        return Err(CommandError::new(
            ErrorCode::NotFound,
            tr(lang, "errors.separate_nothing", &[]),
        ));
    }
    let (w, h) = pixels.dimensions();
    let mut mask = Mask::empty(w, h);
    for piece in &found {
        let [x, y, pw, ph] = piece.rect;
        for yy in y..(y + ph).min(h) {
            for xx in x..(x + pw).min(w) {
                mask.alpha[(yy * w + xx) as usize] = 255;
            }
        }
    }
    let total = found.len();
    let made: Vec<(image::RgbaImage, [u32; 4], Map<String, Value>)> = found
        .iter()
        .map(|piece| {
            let mut mark = json!({"total": total});
            if let Some((text, color, height)) = &piece.text {
                mark["text"] = json!({"text": text, "color": color, "height": height});
            }
            (
                image::imageops::crop_imm(
                    &pixels,
                    piece.rect[0],
                    piece.rect[1],
                    piece.rect[2],
                    piece.rect[3],
                )
                .to_image(),
                piece.rect,
                mark.as_object().cloned().unwrap_or_default(),
            )
        })
        .collect();
    let ids = xs_editor::separate_into(
        core,
        ctx,
        &id,
        target.as_deref(),
        &mask,
        &made,
        &map,
        MARK,
        fingerprint,
    )?;
    {
        let mut s = SESSIONS.lock().unwrap();
        s.retain(|(k, _)| *k != key);
        s.push((key, Session { undo_len, target }));
    }
    tracing::info!(pieces = ids.len(), detail, "separate elements");
    Ok(json!({"documentId": id, "pieces": ids.len(), "ids": ids, "detail": detail}))
}

fn end(core: &Core, id: &str) -> Option<Session> {
    let key = session_key(core, id);
    let mut s = SESSIONS.lock().unwrap();
    let at = s.iter().position(|(k, _)| *k == key)?;
    Some(s.remove(at).1)
}

/// `smart.separate_reset`: the image as it was before separating; the session ends.
pub fn reset(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let id = xs_editor::resolve_document(core, ctx.lang, p)?;
    let Some(session) = end(core, &id) else {
        return Ok(json!({"documentId": id, "active": false}));
    };
    xs_editor::editor(core).store.rewind(&id, session.undo_len);
    xs_editor::document_changed(core, &id);
    Ok(json!({"documentId": id, "active": false}))
}

/// `smart.separate_done`: the pieces flattened into the image; the session ends as one
/// undo step.
pub fn done(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let id = xs_editor::resolve_document(core, ctx.lang, p)?;
    let Some(session) = end(core, &id) else {
        return Ok(json!({"documentId": id, "active": false}));
    };
    let ed = xs_editor::editor(core);
    let ids = ed
        .store
        .with(&id, |d| separated(&d.document).0)
        .ok_or_else(|| CommandError::internal("document closed"))?;
    if !ids.is_empty() {
        core.call(
            ctx.source,
            "document.flatten",
            json!({"documentId": id, "objectIds": ids}),
            Some(ctx.lang),
        )?;
    }
    // Text pieces turned into Text objects stay objects; the marks go.
    ed.store.amend(&id, |doc| {
        let mut n = 0;
        for o in doc.objects.iter_mut() {
            if o.props.remove(MARK).is_some() {
                n += 1;
            }
        }
        n
    });
    ed.store.squash(&id, session.undo_len, "history.separate");
    xs_editor::document_changed(core, &id);
    Ok(json!({"documentId": id, "active": false, "flattened": ids.len()}))
}

/// `smart.separate_text`: a separated piece of text becomes a Text object, with its
/// size and color, to type into (SMT-009).
pub fn text(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let id = xs_editor::resolve_document(core, lang, p)?;
    let oid = p
        .get("objectId")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            CommandError::new(
                ErrorCode::InvalidParams,
                tr(lang, "errors.separate_not_text", &[]),
            )
        })?
        .to_string();
    let ed = xs_editor::editor(core);
    let (object, dpi) = ed
        .store
        .with(&id, |d| (d.document.object(&oid).cloned(), d.document.dpi))
        .ok_or_else(|| CommandError::internal("document closed"))?;
    let object = object
        .ok_or_else(|| CommandError::new(ErrorCode::NotFound, tr(lang, "errors.separate_not_text", &[])))?;
    let info = object
        .props
        .get(MARK)
        .and_then(|m| m.get("text"))
        .cloned()
        .ok_or_else(|| {
            CommandError::new(
                ErrorCode::InvalidParams,
                tr(lang, "errors.separate_not_text", &[]),
            )
        })?;
    let content = info["text"].as_str().unwrap_or_default().to_string();
    let color = info["color"].as_str().unwrap_or("#000000").to_string();
    let height = info["height"].as_f64().unwrap_or(16.0);
    let r = xs_image::shapes::rect(object.props.get("rect")).map_err(CommandError::internal)?;
    let first_line = content.lines().next().unwrap_or_default();
    let px = crate::edit_text::font_px(first_line, height * 0.94);
    let size = (px * 72.0 / dpi.max(1.0) * 2.0).round() / 2.0;
    let props = json!({
        "at": [r.x + 3.0 - 0.05 * px, r.y + 3.0 - 0.25 * px],
        "width": null,
        "text": content,
        "rotation": 0,
        "style": {"font": "Segoe UI", "size": size.clamp(4.0, 400.0), "bold": false, "italic": false,
                  "color": color, "align": "left", "opacity": 100}
    });
    let new_id = xs_editor::object_to_text(
        core,
        ctx,
        &id,
        &oid,
        props.as_object().cloned().unwrap_or_default(),
    )?;
    Ok(json!({"documentId": id, "id": new_id}))
}

pub fn register(reg: &mut xs_core::command::Registry, weak: &std::sync::Weak<Core>) {
    type Run = fn(&Core, &Context, &Value) -> CommandResult;
    let handlers: [(&str, Run); 4] = [
        ("smart.separate", command),
        ("smart.separate_reset", reset),
        ("smart.separate_done", done),
        ("smart.separate_text", text),
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

    fn core_with(dir: &std::path::Path) -> (Arc<Core>, String) {
        let core = Core::for_tests_with(
            xs_core::platform::Platform::Windows,
            vec![crate::extension(), xs_editor::extension()],
        );
        xs_editor::editor(&core).set_folders(dir.join("lib"), dir.join("rec"));
        let image = crate::simplify_auto::tests::settings_window();
        let (id, _) = xs_editor::add_capture(&core, &image, json!({}), 1.0, None).unwrap();
        (core, id)
    }

    fn close(a: &image::RgbaImage, b: &image::RgbaImage, tolerance: u8) -> bool {
        a.dimensions() == b.dimensions()
            && a.as_raw()
                .iter()
                .zip(b.as_raw())
                .all(|(x, y)| x.abs_diff(*y) <= tolerance)
    }

    fn render(core: &Core, id: &str) -> image::RgbaImage {
        xs_editor::editor(core).store.snapshot(id).unwrap().render()
    }

    fn pieces_of(core: &Core, id: &str) -> Vec<xs_core::document::Object> {
        xs_editor::editor(core)
            .store
            .with(id, |d| {
                d.document
                    .objects
                    .iter()
                    .filter(|o| o.props.contains_key(MARK))
                    .cloned()
                    .collect()
            })
            .unwrap()
    }

    /// SMT-018: with a lot of detail the three buttons are three objects, with little
    /// one; a button moved leaves its place filled with the background; changing the
    /// detail after a move asks first; Done flattens and undoes in one step.
    #[test]
    fn separate_a_settings_window() {
        if xs_ocr::languages().is_empty() {
            eprintln!("no recognition language on this computer: skipped");
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let (core, id) = core_with(dir.path());
        let call = |cmd: &str, p: Value| core.call(Source::Cli, cmd, p, Some(xs_core::Lang::Es)).unwrap();
        let before = render(&core, &id);

        let r = call("smart.separate", json!({"documentId": id, "detail": 1}));
        let buttons = |core: &Core| {
            pieces_of(core, &id)
                .into_iter()
                .filter(|o| o.props[MARK].get("text").is_none())
                .collect::<Vec<_>>()
        };
        assert_eq!(buttons(&core).len(), 1, "{r}");
        // Separated, the image looks the same.
        assert!(close(&before, &render(&core, &id), 3), "nothing moves yet");

        let r = call("smart.separate", json!({"documentId": id, "detail": 10}));
        assert_eq!(buttons(&core).len(), 3, "{r}");
        let texts = pieces_of(&core, &id)
            .into_iter()
            .filter(|o| o.props[MARK].get("text").is_some())
            .count();
        assert!(texts >= 3, "{texts}");

        // Move the first button 200 px down: where it was, the background.
        let first = buttons(&core)
            .into_iter()
            .min_by(|a, b| {
                a.props["rect"]["x"]
                    .as_f64()
                    .unwrap()
                    .total_cmp(&b.props["rect"]["x"].as_f64().unwrap())
            })
            .unwrap();
        let (bx, by) = (
            first.props["rect"]["x"].as_f64().unwrap(),
            first.props["rect"]["y"].as_f64().unwrap(),
        );
        call(
            "document.update_object",
            json!({"documentId": id, "objectIds": [first.id], "move": [0, -200]}),
        );
        let now = render(&core, &id);
        let p = now.get_pixel(bx as u32 + 20, by as u32 + 20).0;
        assert!(
            p[0] > 200 && p[1] > 200 && p[2] > 200,
            "the gap is the white background: {p:?}"
        );

        // Another detail now asks first.
        let r = call("smart.separate", json!({"documentId": id, "detail": 5}));
        assert_eq!(r["needsConfirm"], true, "{r}");

        // A piece of text becomes a Text object.
        let piece = pieces_of(&core, &id)
            .into_iter()
            .find(|o| o.props[MARK].get("text").is_some())
            .unwrap();
        let r = call(
            "smart.separate_text",
            json!({"documentId": id, "objectId": piece.id}),
        );
        let tid = r["id"].as_str().unwrap().to_string();
        let kind = xs_editor::editor(&core)
            .store
            .with(&id, |d| d.document.object(&tid).map(|o| o.kind.clone()))
            .flatten();
        assert_eq!(kind.as_deref(), Some("text"));

        // Done: flattened, and one undo goes back to the start.
        call("smart.separate_done", json!({"documentId": id}));
        assert!(pieces_of(&core, &id).is_empty());
        call("document.undo", json!({"documentId": id}));
        assert!(
            render(&core, &id).as_raw() == before.as_raw(),
            "one step back to the start"
        );
    }

    /// SMT-018: Reset puts the image back; on a floating image the image below stays.
    #[test]
    fn reset_and_floating_images() {
        if xs_ocr::languages().is_empty() {
            eprintln!("no recognition language on this computer: skipped");
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let (core, id) = core_with(dir.path());
        let call = |cmd: &str, p: Value| core.call(Source::Cli, cmd, p, Some(xs_core::Lang::Es)).unwrap();
        let before = render(&core, &id);
        call("smart.separate", json!({"documentId": id, "detail": 10}));
        call("smart.separate_reset", json!({"documentId": id}));
        assert!(render(&core, &id).as_raw() == before.as_raw());
        assert!(pieces_of(&core, &id).is_empty());

        // The window as a floating image over a white image: separating it leaves the
        // image below as it was.
        let picture = crate::simplify_auto::tests::settings_window();
        let (other, _) = xs_editor::add_capture(
            &core,
            &image::RgbaImage::from_pixel(1000, 500, image::Rgba([255, 255, 255, 255])),
            json!({}),
            1.0,
            None,
        )
        .unwrap();
        let mut png = Vec::new();
        picture
            .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .unwrap();
        let file = dir.path().join("window.png");
        std::fs::write(&file, png).unwrap();
        let r = call(
            "document.insert_images",
            json!({"documentId": other, "paths": [file]}),
        );
        let floating = r["ids"][0].as_str().unwrap().to_string();
        let base_before = xs_editor::image_pixels(&core, &other).unwrap();
        let r = call(
            "smart.separate",
            json!({"documentId": other, "objectId": floating, "detail": 10}),
        );
        assert!(r["pieces"].as_u64().unwrap() >= 3, "{r}");
        assert!(xs_editor::image_pixels(&core, &other).unwrap().as_raw() == base_before.as_raw());
    }
}
