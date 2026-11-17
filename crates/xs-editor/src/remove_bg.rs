//! Remove background (SMT-020): on the image (the part the canvas shows) or on a
//! floating image, with the classical method of `xs_image::background` for now. One
//! undo step. On the image, the canvas becomes transparent (EDT-074). Options of the
//! notice after it: `invert` keeps the background and removes the subject; `floating`
//! leaves the original as it was and adds the subject on top as a floating image.

use super::*;

/// `smart.remove_background` (SMT-020).
pub fn command(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let id = resolve(core, lang, p)?;
    let ed = editor(core);
    let object = p.get("objectId").and_then(Value::as_str).map(str::to_string);
    let invert = p.get("invert").and_then(Value::as_bool).unwrap_or(false);
    let floating = p.get("floating").and_then(Value::as_bool).unwrap_or(false);
    let doc = ed.store.with(&id, |d| d.document.clone()).ok_or_else(closed)?;

    // What it works on: its pixels, and where they are on the canvas.
    let (pixels, place) = match &object {
        None => {
            let full = image_pixels(core, &id).map_err(|e| internal(lang, "", e))?;
            let c = doc.crop;
            let x0 = c.x.clamp(0, full.width() as i64) as u32;
            let y0 = c.y.clamp(0, full.height() as i64) as u32;
            let x1 = (c.x + c.width as i64).clamp(0, full.width() as i64) as u32;
            let y1 = (c.y + c.height as i64).clamp(0, full.height() as i64) as u32;
            if x1 <= x0 || y1 <= y0 {
                return Err(no_subject(lang));
            }
            let area = [x0, y0, x1 - x0, y1 - y0];
            let rect = [
                (x0 as i64 - c.x) as f64,
                (y0 as i64 - c.y) as f64,
                area[2] as f64,
                area[3] as f64,
            ];
            (pixels::cut_area(&full, area), Place::Image { area, rect })
        }
        Some(oid) => {
            let o = doc
                .object(oid)
                .filter(|o| o.kind == "image")
                .ok_or_else(|| bad(lang, "objectId", "a floating image"))?;
            let name = o
                .props
                .get("image")
                .and_then(Value::as_str)
                .ok_or_else(|| bad(lang, "objectId", "a floating image"))?
                .to_string();
            let bytes = ed
                .store
                .image_file(&id, &name)
                .ok_or_else(|| bad(lang, "objectId", "a floating image"))?;
            let picture = image::load_from_memory(&bytes)
                .map_err(|e| internal(lang, "", e))?
                .to_rgba8();
            let r = xs_image::shapes::rect(o.props.get("rect")).map_err(|e| internal(lang, "", e))?;
            let rotation = o.props.get("rotation").and_then(Value::as_f64).unwrap_or(0.0);
            (
                picture,
                Place::Object {
                    id: oid.clone(),
                    rect: [r.x, r.y, r.w, r.h],
                    rotation,
                },
            )
        }
    };

    let alpha = xs_image::background::subject(&pixels).ok_or_else(|| no_subject(lang))?;
    let out = xs_image::background::apply(&pixels, &alpha, invert);
    let kept = alpha.iter().filter(|a| **a > 0).count() as f64 / alpha.len().max(1) as f64;

    // Floating: only what is left, as a floating image where it was.
    let added = if floating {
        let bounds = xs_image::background::visible_bounds(&out).ok_or_else(|| no_subject(lang))?;
        let piece = pixels::cut_area(&out, bounds);
        let name = store_picture(core, &id, &piece, lang)?;
        let (rect, rotation) = match &place {
            Place::Image { rect, .. } => (
                [
                    rect[0] + bounds[0] as f64,
                    rect[1] + bounds[1] as f64,
                    bounds[2] as f64,
                    bounds[3] as f64,
                ],
                0.0,
            ),
            Place::Object { rect, rotation, .. } => {
                let (sx, sy) = (
                    rect[2] / pixels.width().max(1) as f64,
                    rect[3] / pixels.height().max(1) as f64,
                );
                (
                    [
                        rect[0] + bounds[0] as f64 * sx,
                        rect[1] + bounds[1] as f64 * sy,
                        bounds[2] as f64 * sx,
                        bounds[3] as f64 * sy,
                    ],
                    *rotation,
                )
            }
        };
        let mut props = objects::image_object(&name, rect).map_err(invalid)?;
        if rotation.abs() > 0.01 {
            props.insert("rotation".into(), json!(rotation));
        }
        Some(props)
    } else {
        None
    };
    let name = if floating {
        None
    } else {
        Some(store_picture(core, &id, &out, lang)?)
    };
    let made = ed
        .store
        .mutate(&id, "history.remove_background", |doc| {
            if let Some(props) = &added {
                let oid = doc.next_object_id();
                doc.objects.push(Object {
                    id: oid.clone(),
                    kind: "image".to_string(),
                    props: props.clone(),
                });
                return Ok::<_, CommandError>(Some(oid));
            }
            let Some(name) = &name else { return Ok(None) };
            match &place {
                Place::Image { area, .. } => {
                    pixels::add_patch(doc, pixels::rect_of(*area), name.clone());
                    // What is left shows on a transparent canvas (EDT-074).
                    doc.canvas.color = None;
                    doc.canvas.gradient = None;
                }
                Place::Object { id: oid, .. } => {
                    if let Some(o) = doc.objects.iter_mut().find(|o| &o.id == oid) {
                        o.props.insert("image".into(), json!(name));
                    }
                }
            }
            Ok(None)
        })
        .ok_or_else(closed)??;
    changed(core, ctx, &id);
    Ok(json!({
        "documentId": id,
        "objectId": made.or(object),
        "subject": (kept * 1000.0).round() / 1000.0,
        "inverted": invert,
        "floating": floating,
    }))
}

/// Where the pixels worked on are.
enum Place {
    /// The image: the part of the base shown (base pixels) and where it is on the canvas.
    Image { area: [u32; 4], rect: [f64; 4] },
    /// A floating image: its rectangle and turn on the canvas.
    Object {
        id: String,
        rect: [f64; 4],
        rotation: f64,
    },
}

fn no_subject(lang: Lang) -> CommandError {
    CommandError::new(ErrorCode::NotFound, tr(lang, "errors.no_subject", &[]))
}

fn closed() -> CommandError {
    CommandError::internal("document closed")
}

fn bad(lang: Lang, param: &str, reason: &str) -> CommandError {
    invalid(tr(
        lang,
        "errors.invalid_param",
        &[("param", param), ("reason", reason)],
    ))
}
