//! Edit text of the image (SMT-009): the text lines inside the pixel selection are
//! read, their letters' pixels rebuilt from the background around them (Smart fill,
//! EDT-045), and a Text object per paragraph put in their place with the original
//! size and color, ready to type. One undo step. A selection that cuts letters or lines
//! asks first and offers to fit the selection to the text.

use serde_json::{json, Map, Value};
use xs_core::i18n::tr;
use xs_core::{CommandError, CommandResult, Context, Core, ErrorCode};
use xs_image::selection::Mask;

use crate::redact;

fn rgb(hex: &str) -> [u8; 3] {
    xs_image::color::Rgba::parse(hex)
        .map(|c| [c.r, c.g, c.b])
        .unwrap_or([255, 255, 255])
}

fn far(a: [u8; 3], b: [u8; 3], by: i32) -> bool {
    (0..3).map(|i| (a[i] as i32 - b[i] as i32).abs()).sum::<i32>() > by
}

fn ascends(text: &str) -> bool {
    text.chars()
        .any(|c| c.is_uppercase() || c.is_ascii_digit() || "bdfhijklt'\"¿¡?!/()[]{}áéíóúñ".contains(c))
}

/// The letter size in pixels of a line from the height of its letters' pixels: which
/// letters it has tells how much of the size they take (measured with Segoe UI).
pub fn font_px(text: &str, glyph_height: f64) -> f64 {
    let descends = text.chars().any(|c| "gjpqyQ,;()[]{}".contains(c));
    let ratio = 0.50 + if ascends(text) { 0.25 } else { 0.0 } + if descends { 0.22 } else { 0.0 };
    glyph_height / ratio
}

/// From the top of a Text object to its letters' top, in pixels.
fn top_offset(text: &str, px: f64) -> f64 {
    if ascends(text) {
        0.25 * px
    } else {
        0.5 * px
    }
}

fn inside(mask: &Mask, x: f64, y: f64) -> bool {
    x >= 0.0
        && y >= 0.0
        && (x as u32) < mask.width
        && (y as u32) < mask.height
        && mask.at(x as u32, y as u32) >= 128
}

/// How much of a word's box the selection holds: none, part or all.
fn held(mask: &Mask, r: [f64; 4]) -> (bool, bool) {
    let pts = [
        (r[0] + 1.0, r[1] + 1.0),
        (r[0] + r[2] - 1.0, r[1] + 1.0),
        (r[0] + 1.0, r[1] + r[3] - 1.0),
        (r[0] + r[2] - 1.0, r[1] + r[3] - 1.0),
        (r[0] + r[2] / 2.0, r[1] + r[3] / 2.0),
    ];
    let n = pts.iter().filter(|(x, y)| inside(mask, *x, *y)).count();
    (n > 0, n == pts.len())
}

fn rect_mask(w: u32, h: u32, r: [f64; 4]) -> Mask {
    let mut m = Mask::empty(w, h);
    let x0 = r[0].floor().max(0.0) as u32;
    let y0 = r[1].floor().max(0.0) as u32;
    let x1 = ((r[0] + r[2]).ceil().max(0.0) as u32).min(w);
    let y1 = ((r[1] + r[3]).ceil().max(0.0) as u32).min(h);
    for y in y0..y1 {
        for x in x0..x1 {
            m.alpha[(y * w + x) as usize] = 255;
        }
    }
    m
}

/// `smart.edit_text` (SMT-009).
pub fn command(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let id = xs_editor::resolve_document(core, lang, p)?;
    xs_editor::video::refuse_for_video(core, lang, &json!({"documentId": id}))?;
    let pixels = xs_editor::image_pixels(core, &id).map_err(CommandError::internal)?;
    let (w, h) = pixels.dimensions();
    let (cx, cy) = xs_editor::crop_origin(core, &id).unwrap_or((0, 0));
    let (cx, cy) = (cx as f64, cy as f64);
    // The place: a rectangle asked (auto redact's menu), else the pixel selection.
    let rect_asked = p.get("rect").and_then(|v| {
        let n = |i: usize| v.get(i).and_then(Value::as_f64);
        Some([n(0)? + cx, n(1)? + cy, n(2)?, n(3)?])
    });
    let mut mask: Mask = match rect_asked {
        Some(r) => rect_mask(w, h, r),
        None => xs_editor::image_selection(core, &id)
            .map(|m| (*m).clone())
            .ok_or_else(|| {
                CommandError::new(ErrorCode::InvalidParams, tr(lang, "errors.edit_text_select", &[]))
            })?,
    };
    if (mask.width, mask.height) != (w, h) {
        return Err(CommandError::new(
            ErrorCode::InvalidParams,
            tr(lang, "errors.edit_text_select", &[]),
        ));
    }
    let languages = crate::extract::wanted(core, None);
    let reading = redact::read(&pixels, &languages, lang)?;
    let Some(text) = reading.text.as_ref() else {
        return Err(CommandError::new(
            ErrorCode::NotFound,
            tr(lang, "errors.edit_text_none", &[]),
        ));
    };
    // The lines the selection touches, and whether it cuts some word.
    let mut cut = false;
    let mut touched: Vec<(usize, usize)> = Vec::new();
    for (bi, b) in text.blocks.iter().enumerate() {
        for (li, l) in b.lines.iter().enumerate() {
            let mut any = false;
            for word in &l.words {
                let (some, all) = held(&mask, word.rect);
                any |= some;
                cut |= some && !all;
            }
            if any {
                touched.push((bi, li));
            }
        }
    }
    if touched.is_empty() {
        return Err(CommandError::new(
            ErrorCode::NotFound,
            tr(lang, "errors.edit_text_none", &[]),
        ));
    }
    let full_lines = |list: &[(usize, usize)]| -> [f64; 4] {
        let rects: Vec<[f64; 4]> = list.iter().map(|(b, l)| text.blocks[*b].lines[*l].rect).collect();
        let x0 = rects.iter().map(|r| r[0]).fold(f64::MAX, f64::min) - 3.0;
        let y0 = rects.iter().map(|r| r[1]).fold(f64::MAX, f64::min) - 3.0;
        let x1 = rects.iter().map(|r| r[0] + r[2]).fold(f64::MIN, f64::max) + 3.0;
        let y1 = rects.iter().map(|r| r[1] + r[3]).fold(f64::MIN, f64::max) + 3.0;
        [x0, y0, x1 - x0, y1 - y0]
    };
    let fit = p.get("fit").and_then(Value::as_bool) == Some(true);
    if cut && !fit && p.get("anyway").and_then(Value::as_bool) != Some(true) {
        let r = full_lines(&touched);
        return Ok(json!({
            "documentId": id,
            "cut": true,
            "fitRect": [r[0] - cx, r[1] - cy, r[2], r[3]],
        }));
    }
    if fit {
        mask = rect_mask(w, h, full_lines(&touched));
    }

    // The letters' pixels: what differs from each line's background, even their soft
    // edges, two pixels wider; and
    // where the letters are, line by line ([x0, y0, x1, y1]).
    let mut letters = Mask::empty(w, h);
    let mut rough = false;
    let mut extents: Vec<Option<[u32; 4]>> = Vec::with_capacity(touched.len());
    for (bi, li) in &touched {
        let l = &text.blocks[*bi].lines[*li];
        let bg = rgb(&l.background);
        let r = l.rect;
        let x0 = (r[0] - 3.0).max(0.0) as u32;
        let y0 = (r[1] - 3.0).max(0.0) as u32;
        let x1 = ((r[0] + r[2] + 3.0) as u32).min(w);
        let y1 = ((r[1] + r[3] + 3.0) as u32).min(h);
        let (mut off, mut all) = (0u32, 0u32);
        let mut extent: Option<[u32; 4]> = None;
        for y in y0..y1 {
            for x in x0..x1 {
                if mask.at(x, y) < 128 {
                    continue;
                }
                let px = pixels.get_pixel(x, y).0;
                all += 1;
                let rgb = [px[0], px[1], px[2]];
                // The letters themselves set the size; their soft edges are erased too.
                if far(rgb, bg, 60) {
                    off += 1;
                    let e = extent.get_or_insert([x, y, x, y]);
                    *e = [e[0].min(x), e[1].min(y), e[2].max(x), e[3].max(y)];
                }
                if far(rgb, bg, 18) {
                    for yy in y.saturating_sub(2)..(y + 3).min(h) {
                        for xx in x.saturating_sub(2)..(x + 3).min(w) {
                            if mask.at(xx, yy) >= 128 {
                                letters.alpha[(yy * w + xx) as usize] = 255;
                            }
                        }
                    }
                }
            }
        }
        // Much more than letters differs: a gradient or a photo below.
        if all > 0 && off * 2 > all {
            rough = true;
        }
        extents.push(extent);
    }

    // A Text object per paragraph, where its first line was, with its size and color.
    let dpi = xs_editor::editor(core)
        .store
        .with(&id, |d| d.document.dpi)
        .unwrap_or(96.0);
    let mut texts: Vec<Map<String, Value>> = Vec::new();
    let mut blocks: Vec<usize> = touched.iter().map(|(b, _)| *b).collect();
    blocks.dedup();
    for b in blocks {
        let lines: Vec<&xs_ocr::Line> = touched
            .iter()
            .filter(|(bb, _)| *bb == b)
            .map(|(_, l)| &text.blocks[b].lines[*l])
            .collect();
        let first_at = touched.iter().position(|(bb, _)| *bb == b).unwrap_or(0);
        let first = lines[0];
        // Where the first line's letters are; the recognized box if none were seen.
        let glyphs = extents[first_at]
            .map(|e| {
                [
                    e[0] as f64,
                    e[1] as f64,
                    (e[2] - e[0] + 1) as f64,
                    (e[3] - e[1] + 1) as f64,
                ]
            })
            .unwrap_or(first.rect);
        let content: Vec<String> = lines
            .iter()
            .map(|l| {
                if fit {
                    l.text.clone()
                } else {
                    l.words
                        .iter()
                        .filter(|w| held(&mask, w.rect).0)
                        .map(|w| w.text.as_str())
                        .collect::<Vec<_>>()
                        .join(" ")
                }
            })
            .collect();
        let first_text = &content[0];
        let px = font_px(first_text, glyphs[3]);
        let left = touched
            .iter()
            .zip(&extents)
            .filter(|((bb, _), _)| *bb == b)
            .filter_map(|(_, e)| e.map(|e| e[0] as f64))
            .fold(glyphs[0], f64::min);
        let size = (px * 72.0 / dpi.max(1.0) * 2.0).round() / 2.0;
        texts.push(
            json!({
                "at": [(left - 0.05 * px - cx).round(), (glyphs[1] - top_offset(first_text, px) - cy).round()],
                "width": null,
                "text": content.join("\n"),
                "rotation": 0,
                "style": {"font": "Segoe UI", "size": size.clamp(4.0, 400.0), "bold": false, "italic": false,
                          "color": first.color, "align": "left", "opacity": 100}
            })
            .as_object()
            .cloned()
            .unwrap_or_default(),
        );
    }
    let ids = xs_editor::replace_text(core, ctx, &id, &letters, &texts)?;
    tracing::info!(lines = touched.len(), objects = ids.len(), "edit text");
    Ok(json!({"documentId": id, "ids": ids, "lines": touched.len(), "rough": rough}))
}

pub fn register(reg: &mut xs_core::command::Registry, weak: &std::sync::Weak<Core>) {
    let w = weak.clone();
    reg.handle("smart.edit_text", move |ctx, p| {
        let core = w
            .upgrade()
            .ok_or_else(|| CommandError::internal("shutting down"))?;
        command(&core, ctx, &p)
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use xs_core::document::{Document, Object};
    use xs_core::Source;

    /// A blue button with the white text "Enviar" and a label beside it.
    fn button() -> image::RgbaImage {
        let (w, h) = (600u32, 200u32);
        let mut doc = Document::new(w, h, 96.0);
        doc.objects.push(Object {
            id: "b".into(),
            kind: "shape".into(),
            props: json!({"rect": {"x": 100, "y": 60, "w": 200, "h": 70}, "rotation": 0,
                "style": {"shapeType": "rectangle", "fill": "#2D6CC4", "outline": "#2D6CC4", "outlineWidth": 0}})
            .as_object()
            .unwrap()
            .clone(),
        });
        doc.objects.push(Object {
            id: "t".into(),
            kind: "text".into(),
            props: json!({"at": [150, 74], "width": null, "text": "Enviar", "rotation": 0,
                "style": {"font": "Segoe UI", "size": 22, "bold": false, "italic": false,
                          "color": "#FFFFFF", "align": "left", "opacity": 100}})
            .as_object()
            .unwrap()
            .clone(),
        });
        let base = xs_image::render::Base {
            width: w,
            height: h,
            rgba: Arc::new(vec![255; (w * h * 4) as usize]),
            images: Arc::new(Default::default()),
        };
        xs_image::render::render(&doc, &base)
    }

    fn delta_e(a: [u8; 3], b: [u8; 3]) -> f64 {
        // CIE76 over Lab.
        fn lab(c: [u8; 3]) -> [f64; 3] {
            let lin = |v: u8| {
                let v = v as f64 / 255.0;
                if v <= 0.04045 {
                    v / 12.92
                } else {
                    ((v + 0.055) / 1.055).powf(2.4)
                }
            };
            let (r, g, b) = (lin(c[0]), lin(c[1]), lin(c[2]));
            let x = (0.4124 * r + 0.3576 * g + 0.1805 * b) / 0.95047;
            let y = 0.2126 * r + 0.7152 * g + 0.0722 * b;
            let z = (0.0193 * r + 0.1192 * g + 0.9505 * b) / 1.08883;
            let f = |t: f64| {
                if t > 0.008856 {
                    t.cbrt()
                } else {
                    7.787 * t + 16.0 / 116.0
                }
            };
            [116.0 * f(y) - 16.0, 500.0 * (f(x) - f(y)), 200.0 * (f(y) - f(z))]
        }
        let (p, q) = (lab(a), lab(b));
        ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt()
    }

    /// SMT-009: editing "Enviar" on a blue button leaves the button blue with no trace
    /// of the old text, a white Text object in its place; one undo gives back the very
    /// same pixels; a selection cutting a word asks first; none: no text.
    #[test]
    fn edit_the_text_of_a_button() {
        if xs_ocr::languages().is_empty() {
            eprintln!("no recognition language on this computer: skipped");
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let core = Core::for_tests_with(
            xs_core::platform::Platform::Windows,
            vec![crate::extension(), xs_editor::extension()],
        );
        xs_editor::editor(&core).set_folders(dir.path().join("lib"), dir.path().join("rec"));
        let (id, _) = xs_editor::add_capture(&core, &button(), json!({}), 1.0, None).unwrap();
        let call = |cmd: &str, p: Value| core.call(Source::Cli, cmd, p, Some(xs_core::Lang::Es));
        let before = xs_editor::editor(&core).store.snapshot(&id).unwrap().render();

        // Half the word: it asks first, and says how to fit.
        let r = call(
            "smart.edit_text",
            json!({"documentId": id, "rect": [140, 60, 40, 70]}),
        )
        .unwrap();
        assert_eq!(r["cut"], true, "{r}");
        // Nothing there.
        let e = call(
            "smart.edit_text",
            json!({"documentId": id, "rect": [400, 150, 100, 40]}),
        )
        .unwrap_err();
        assert_eq!(e.code, ErrorCode::NotFound);

        let r = call(
            "smart.edit_text",
            json!({"documentId": id, "rect": [100, 60, 200, 70]}),
        )
        .unwrap();
        assert_eq!(r["ids"].as_array().unwrap().len(), 1, "{r}");
        let oid = r["ids"][0].as_str().unwrap().to_string();
        let object = xs_editor::editor(&core)
            .store
            .with(&id, |d| d.document.object(&oid).cloned())
            .flatten()
            .unwrap();
        assert_eq!(object.props["text"], "Enviar");
        let color = rgb(object.props["style"]["color"].as_str().unwrap());
        assert!(delta_e(color, [255, 255, 255]) <= 5.0, "{color:?}");
        let size = object.props["style"]["size"].as_f64().unwrap();
        assert!((19.0..=25.0).contains(&size), "size {size}");

        // Without the new text: the button is all blue, no trace of the letters.
        let mut snap = xs_editor::editor(&core).store.snapshot(&id).unwrap();
        snap.document.objects.retain(|o| o.id != oid);
        let after = snap.render();
        let (dx, dy) = (snap.document.crop.x as u32, snap.document.crop.y as u32);
        for y in 66..124 {
            for x in 106..294 {
                let p = after.get_pixel(x - dx, y - dy).0;
                assert!(
                    !far([p[0], p[1], p[2]], [0x2D, 0x6C, 0xC4], 30),
                    "a trace at {x},{y}: {p:?}"
                );
            }
        }
        // Retyped, it is a normal text.
        call(
            "document.update_object",
            json!({"documentId": id, "objectIds": [oid], "set": {"text": "Guardar"}}),
        )
        .unwrap();
        // Two undos: the new text, then the edit; the pixels are the very same.
        call("document.undo", json!({"documentId": id})).unwrap();
        call("document.undo", json!({"documentId": id})).unwrap();
        let undone = xs_editor::editor(&core).store.snapshot(&id).unwrap().render();
        assert!(undone.as_raw() == before.as_raw(), "undo gives the same pixels");
    }

    #[test]
    fn letter_size_from_the_letters() {
        assert!((font_px("Enviar", 24.0) - 32.0).abs() < 1e-9);
        assert!((font_px("Pagaba", 31.04) - 32.0).abs() < 1e-9);
        assert!((font_px("acero", 10.0) - 20.0).abs() < 1e-9);
    }
}
