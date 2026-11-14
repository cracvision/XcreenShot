//! Auto redact (SMT-010 to SMT-013, SMT-017): reads a document, finds the sensitive data
//! (`xs_ocr::sensitive`) and the faces, and covers each with a redaction, a normal Blur
//! object (EDT-040) that `xs_editor::auto_redact` manages. The whole analysis is one undo
//! step. Nothing leaves the computer; the text found is never kept in the document nor
//! written to the logs.
//!
//! The reading of an image is kept (by the hash of its pixels), so that switching a
//! category or a term, or changing the style, does not read it again (SMT-017).

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use serde_json::{json, Map, Value};
use xs_core::i18n::tr;
use xs_core::{CommandError, CommandResult, Context, Core, ErrorCode};
use xs_editor::auto_redact::{self, Wanted};
use xs_ocr::sensitive::{self, Term, TermKind};
use xs_ocr::{Recognized, Rect};

/// The categories on when nothing says otherwise (SMT-011): URLs and dates off.
pub const DEFAULT_ON: [&str; 7] = ["email", "phone", "card", "id", "address", "ip", "face"];

fn strings(v: Option<&Value>) -> Option<Vec<String>> {
    Some(
        v?.as_array()?
            .iter()
            .filter_map(|s| s.as_str().map(str::to_string))
            .collect(),
    )
}

/// The user's terms (SMT-012) from `[{name, text, exact, on} | {name, pattern, on}]`,
/// with whether each is on; invalid patterns are left out and named.
fn terms(v: &Value) -> (Vec<(Term, bool)>, Vec<String>) {
    let mut ok = Vec::new();
    let mut bad = Vec::new();
    for t in v.as_array().into_iter().flatten() {
        let on = t.get("on").and_then(Value::as_bool).unwrap_or(true);
        let name = t.get("name").and_then(Value::as_str).unwrap_or_default().trim();
        let named = |fallback: &str| {
            if name.is_empty() {
                fallback.to_string()
            } else {
                name.to_string()
            }
        };
        if let Some(p) = t.get("pattern").and_then(Value::as_str) {
            match sensitive::check_pattern(p) {
                Ok(()) => ok.push((
                    Term {
                        name: named(p),
                        kind: TermKind::Pattern(p.to_string()),
                    },
                    on,
                )),
                Err(_) => bad.push(named(p)),
            }
        } else if let Some(text) = t
            .get("text")
            .and_then(Value::as_str)
            .filter(|s| !s.trim().is_empty())
        {
            ok.push((
                Term {
                    name: named(text),
                    kind: TermKind::Words {
                        text: text.to_string(),
                        exact: t.get("exact").and_then(Value::as_bool).unwrap_or(false),
                    },
                },
                on,
            ));
        }
    }
    (ok, bad)
}

/// The style of the redactions (SMT-013): solid (default, black), blur or pixelate, and
/// the margin around the data (0–10 px, default 2).
fn style(v: &Value) -> (Map<String, Value>, f64) {
    let kind = match v.get("type").and_then(Value::as_str).unwrap_or("solid") {
        "blur" => "blur",
        "pixelate" => "pixelate",
        _ => "solid",
    };
    let color = v
        .get("color")
        .and_then(Value::as_str)
        .filter(|c| c.len() == 7 && c.starts_with('#') && c[1..].chars().all(|h| h.is_ascii_hexdigit()))
        .unwrap_or("#000000");
    let intensity = v
        .get("intensity")
        .and_then(Value::as_f64)
        .unwrap_or(60.0)
        .clamp(1.0, 100.0);
    let margin = v
        .get("margin")
        .and_then(Value::as_f64)
        .unwrap_or(2.0)
        .clamp(0.0, 10.0);
    let style = json!({"blurType": kind, "color": color, "intensity": intensity});
    (style.as_object().cloned().unwrap_or_default(), margin)
}

/// One reading of an image: its text (none without a recognition language) and faces.
struct Reading {
    key: u64,
    text: Option<Recognized>,
    faces: Vec<Rect>,
}

/// The last readings, newest last.
static READINGS: Mutex<Vec<Arc<Reading>>> = Mutex::new(Vec::new());
const KEPT_READINGS: usize = 4;

fn key_of(image: &image::RgbaImage, languages: &[String]) -> u64 {
    let mut h = DefaultHasher::new();
    (image.width(), image.height()).hash(&mut h);
    image.as_raw().hash(&mut h);
    languages.hash(&mut h);
    h.finish()
}

fn read(
    image: &image::RgbaImage,
    languages: &[String],
    lang: xs_core::Lang,
) -> Result<Arc<Reading>, CommandError> {
    let key = key_of(image, languages);
    if let Some(r) = READINGS.lock().unwrap().iter().find(|r| r.key == key) {
        return Ok(r.clone());
    }
    let text = match xs_ocr::recognize(image, languages, &AtomicBool::new(false)) {
        Ok(r) => Some(r),
        // Without a recognition language, the faces alone.
        Err(xs_ocr::OcrError::NoLanguage) | Err(xs_ocr::OcrError::NotSupported) => None,
        Err(e) => return Err(crate::extract::ocr_error(lang, e)),
    };
    let faces = xs_ocr::faces(image).unwrap_or_default();
    let reading = Arc::new(Reading { key, text, faces });
    let mut kept = READINGS.lock().unwrap();
    kept.push(reading.clone());
    if kept.len() > KEPT_READINGS {
        kept.remove(0);
    }
    Ok(reading)
}

/// The settings asked for, remembered for the next time; else those of the last time.
fn asked_or_kept(core: &Core, ctx: &Context, p: &Value, param: &str, key: &str, default: Value) -> Value {
    match p.get(param) {
        Some(v) => {
            if core
                .settings
                .set(key, v.clone(), ctx.source, ctx.lang)
                .unwrap_or(false)
            {
                core.setting_changed(key, v);
            }
            v.clone()
        }
        None => core.settings.get(key).unwrap_or(default),
    }
}

/// `smart.redact` (SMT-010): covers the sensitive data of an open document (or of an
/// image file, written to `output`). Without `categories`, `terms` or `style`, those of
/// the last time; with them, they are remembered. Redactions of an earlier analysis are
/// kept, never doubled; those no longer wanted go. `clear` removes them all. Returns how
/// many per category and where each managed redaction is.
pub fn command(core: &Arc<Core>, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    if let Some(path) = p.get("path").and_then(Value::as_str) {
        return on_file(core, ctx, p, PathBuf::from(path));
    }
    xs_editor::video::refuse_for_video(core, lang, p)?;
    let id = xs_editor::resolve_document(core, lang, p)?;
    redact(core, ctx, p, &id)
}

/// Redactions removed by hand in a document, by category and place: an analysis does
/// not put them back while the app runs (SMT-014, provisional).
static IGNORED: Mutex<Vec<(String, String, [f64; 4])>> = Mutex::new(Vec::new());

fn ignored(id: &str, w: &Wanted) -> bool {
    IGNORED.lock().unwrap().iter().any(|(doc, c, s)| {
        doc == id && *c == w.category && s.iter().zip(w.source).all(|(a, b)| (a - b).abs() < 0.5)
    })
}

/// A rectangle of the canvas with the redactions' margin.
fn padded(s: [f64; 4], margin: f64) -> [f64; 4] {
    let (x, y) = ((s[0] - margin).floor(), (s[1] - margin).floor());
    let (right, bottom) = ((s[0] + s[2] + margin).ceil(), (s[1] + s[3] + margin).ceil());
    [x, y, right - x, bottom - y]
}

fn contains(outer: [f64; 4], inner: [f64; 4]) -> bool {
    inner[0] >= outer[0] - 1.0
        && inner[1] >= outer[1] - 1.0
        && inner[0] + inner[2] <= outer[0] + outer[2] + 1.0
        && inner[1] + inner[3] <= outer[1] + outer[3] + 1.0
}

fn overlaps(a: [f64; 4], b: [f64; 4]) -> bool {
    a[0] < b[0] + b[2] && b[0] < a[0] + a[2] && a[1] < b[1] + b[3] && b[1] < a[1] + a[3]
}

fn redact(core: &Core, ctx: &Context, p: &Value, id: &str) -> CommandResult {
    let lang = ctx.lang;
    let internal = CommandError::internal;
    if p.get("clear").and_then(Value::as_bool) == Some(true) {
        let out = auto_redact::clear(core, ctx, id).map_err(internal)?;
        IGNORED.lock().unwrap().retain(|(doc, _, _)| doc != id);
        return Ok(
            json!({"documentId": id, "redactions": 0, "removed": out.removed, "counts": {}, "items": []}),
        );
    }
    // Changes by hand (SMT-014, SMT-015): only those, then the panel's state.
    let by_hand = p.get("add").is_some() || p.get("remove").is_some() || p.get("matches").is_some();
    let default_on = json!(DEFAULT_ON);
    let categories = strings(Some(&asked_or_kept(
        core,
        ctx,
        p,
        "categories",
        "smart.autoRedact.categories",
        default_on,
    )))
    .unwrap_or_else(|| DEFAULT_ON.iter().map(|s| s.to_string()).collect());
    let terms_value = asked_or_kept(core, ctx, p, "terms", "smart.autoRedact.customTerms", json!([]));
    let restyle = p.get("style").is_some();
    let style_value = asked_or_kept(core, ctx, p, "style", "smart.autoRedact.style", json!({}));
    let (user_terms, invalid) = terms(&terms_value);
    let (blur_style, margin) = style(&style_value);

    // The document as it would be without the managed redactions.
    let (image, origin) = auto_redact::image_without(core, id).map_err(internal)?;
    let started = std::time::Instant::now();
    let languages = crate::extract::wanted(core, None);
    let reading = read(&image, &languages, lang)?;

    // What is covered: the categories and terms that are on. The others are counted, so
    // that the panel can say what switching them on would cover.
    let on: Vec<&str> = categories
        .iter()
        .map(String::as_str)
        .filter(|c| sensitive::CATEGORIES.contains(c))
        .collect();
    let terms_on: Vec<Term> = user_terms
        .iter()
        .filter(|(_, on)| *on)
        .map(|(t, _)| t.clone())
        .collect();
    let all_terms: Vec<Term> = user_terms.iter().map(|(t, _)| t.clone()).collect();
    let (found, all) = match &reading.text {
        Some(r) => (
            sensitive::find(r, &on, &terms_on),
            sensitive::find(r, &sensitive::CATEGORIES, &all_terms),
        ),
        None => (Vec::new(), Vec::new()),
    };
    let faces_on = categories.iter().any(|c| c == "face");
    let mut counts: Map<String, Value> = Map::new();
    let mut count = |c: &str| {
        let n = counts.get(c).and_then(Value::as_u64).unwrap_or(0);
        counts.insert(c.to_string(), json!(n + 1));
    };
    for f in &found {
        count(&f.category);
    }
    for f in all
        .iter()
        .filter(|f| !on.contains(&f.category.as_str()) && !found.iter().any(|g| g.category == f.category))
    {
        count(&f.category);
    }
    for _ in &reading.faces {
        count("face");
    }
    let to_canvas = |r: &Rect| -> [f64; 4] { [r[0] - origin[0] as f64, r[1] - origin[1] as f64, r[2], r[3]] };
    let mut wanted: Vec<Wanted> = Vec::new();
    let mut push = |category: &str, r: &Rect| {
        let s = to_canvas(r);
        wanted.push(Wanted {
            category: category.to_string(),
            source: s.map(|v| (v * 10.0).round() / 10.0),
            rect: padded(s, margin),
        });
    };
    for f in &found {
        for r in &f.rects {
            push(&f.category, r);
        }
    }
    if faces_on {
        for r in &reading.faces {
            push("face", r);
        }
    }
    // Those removed by hand stay away, unless they are back in the document (Undo).
    let existing = auto_redact::managed(core, id).map_err(internal)?;
    IGNORED.lock().unwrap().retain(|(doc, c, s)| {
        doc != id
            || !existing
                .iter()
                .any(|m| m.category == *c && m.source.iter().zip(s).all(|(a, b)| (a - b).abs() < 0.5))
    });
    let mut dropped: Vec<String> = Vec::new();
    wanted.retain(|w| {
        let keep = !ignored(id, w);
        if !keep {
            dropped.push(w.category.clone());
        }
        keep
    });
    // Its category counts one less (SMT-014).
    for c in dropped {
        match counts.get(&c).and_then(Value::as_u64) {
            Some(n) if n > 1 => {
                counts.insert(c, json!(n - 1));
            }
            Some(_) => {
                counts.remove(&c);
            }
            None => {}
        }
    }
    let mut added_by_hand = 0;
    let outcome = if by_hand {
        let mut add: Vec<Wanted> = Vec::new();
        let mut remove: Vec<String> = p
            .get("remove")
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(Value::as_str).map(str::to_string).collect())
            .unwrap_or_default();
        // A rectangle of the canvas each (a word, or words of one line).
        for r in p.get("add").and_then(Value::as_array).into_iter().flatten() {
            let four = |k: usize| r.get(k).and_then(Value::as_f64);
            if let (Some(x), Some(y), Some(w), Some(h)) = (four(0), four(1), four(2), four(3)) {
                let s = [x, y, w, h].map(|v| (v * 10.0).round() / 10.0);
                add.push(Wanted {
                    category: auto_redact::MANUAL.into(),
                    source: s,
                    rect: padded(s, margin),
                });
            }
        }
        // Every place where a text appears (SMT-015), or none of them.
        if let (Some(m), Some(rec)) = (p.get("matches"), reading.text.as_ref()) {
            let text = m.get("text").and_then(Value::as_str).unwrap_or("").trim();
            let term = Term {
                name: "match".into(),
                kind: TermKind::Words {
                    text: text.to_string(),
                    exact: m.get("exact").and_then(Value::as_bool).unwrap_or(false),
                },
            };
            let places: Vec<[f64; 4]> = if text.is_empty() {
                Vec::new()
            } else {
                sensitive::find(rec, &[], &[term])
                    .iter()
                    .flat_map(|f| f.rects.iter().map(&to_canvas))
                    .collect()
            };
            if m.get("remove").and_then(Value::as_bool) == Some(true) {
                for e in existing
                    .iter()
                    .filter(|e| places.iter().any(|pl| overlaps(*pl, e.source)))
                {
                    remove.push(e.object_id.clone());
                }
            } else {
                for pl in places {
                    // A place already covered whole needs nothing more.
                    if !existing.iter().any(|e| contains(e.source, pl)) {
                        let s = pl.map(|v| (v * 10.0).round() / 10.0);
                        add.push(Wanted {
                            category: auto_redact::MANUAL.into(),
                            source: s,
                            rect: padded(s, margin),
                        });
                    }
                }
            }
        }
        added_by_hand = add.len();
        // One by hand that covers others by hand takes their place (a word, then Shift+click).
        for e in existing.iter().filter(|e| e.category == auto_redact::MANUAL) {
            if add.iter().any(|a| contains(a.source, e.source)) && !remove.contains(&e.object_id) {
                remove.push(e.object_id.clone());
            }
        }
        // An automatic redaction removed by hand is not put back (SMT-014).
        {
            let mut ign = IGNORED.lock().unwrap();
            for e in existing
                .iter()
                .filter(|e| remove.contains(&e.object_id) && e.category != auto_redact::MANUAL)
            {
                ign.push((id.to_string(), e.category.clone(), e.source));
            }
        }
        auto_redact::edit(core, ctx, id, &add, &remove, &blur_style).map_err(internal)?
    } else {
        auto_redact::reconcile(core, ctx, id, &wanted, &blur_style, restyle).map_err(internal)?
    };
    // Counts only, never the text (SMT-017).
    tracing::info!(
        redactions = wanted.len(),
        added = outcome.added,
        removed = outcome.removed,
        elapsed_ms = started.elapsed().as_millis() as u64,
        "auto redact"
    );
    let managed = auto_redact::managed(core, id).map_err(internal)?;
    let manual = managed
        .iter()
        .filter(|m| m.category == auto_redact::MANUAL)
        .count();
    if manual > 0 {
        counts.insert(auto_redact::MANUAL.into(), json!(manual));
    }
    let items: Vec<Value> = managed
        .into_iter()
        .map(|m| json!({"objectId": m.object_id, "category": m.category, "rect": m.rect, "source": m.source}))
        .collect();
    let mut result = json!({
        "documentId": id,
        "redactions": items.len(),
        "added": outcome.added,
        "removed": outcome.removed,
        "restyled": outcome.restyled,
        "addedByHand": added_by_hand,
        "counts": counts,
        "categories": categories,
        "terms": user_terms.iter().map(|(t, _)| format!("term:{}", t.name)).collect::<Vec<_>>(),
        "invalidPatterns": invalid,
        "textRead": reading.text.is_some(),
        "items": items,
    });
    if let Some(l) = reading.text.as_ref().map(|r| r.language.clone()) {
        result["language"] = json!(l);
    }
    if p.get("flatten").and_then(Value::as_bool) == Some(true) && !items.is_empty() {
        let ids: Vec<Value> = result["items"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|i| i["objectId"].clone())
            .collect();
        core.call(
            ctx.source,
            "document.flatten",
            json!({"documentId": id, "objectIds": ids}),
            Some(lang),
        )?;
        result["flattened"] = json!(true);
    }
    Ok(result)
}

const VIDEO_EXTENSIONS: [&str; 8] = ["mp4", "m4v", "mov", "avi", "wmv", "mkv", "webm", "gif"];

/// `smart.redact` on an image file (SMT-010): opened apart, redacted and written, flat,
/// to `output` (default: next to it, "<name> (redacted).<ext>"). The file itself never
/// changes.
fn on_file(core: &Arc<Core>, ctx: &Context, p: &Value, path: PathBuf) -> CommandResult {
    let lang = ctx.lang;
    // A video has no image to redact; opening it would import it into the library.
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    if VIDEO_EXTENSIONS.contains(&ext.as_str()) {
        return Err(CommandError::new(
            ErrorCode::InvalidParams,
            tr(lang, "errors.video_no_image_tools", &[]),
        ));
    }
    let was_open = xs_editor::editor(core).store.find_by_path(&path);
    let id = match &was_open {
        Some(id) => id.clone(),
        None => xs_editor::open(core, lang, &path, false)?,
    };
    let done = (|| -> CommandResult {
        xs_editor::video::refuse_for_video(core, lang, &json!({"documentId": id}))?;
        let mut asked = p.clone();
        if let Some(m) = asked.as_object_mut() {
            m.remove("path");
            m.remove("flatten");
        }
        let mut result = redact(core, ctx, &asked, &id)?;
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .filter(|e| ["png", "jpg", "jpeg", "gif", "bmp", "tif", "tiff"].contains(&e.as_str()))
            .unwrap_or_else(|| "png".into());
        let format = match ext.as_str() {
            "jpeg" => "jpg",
            "tif" => "tiff",
            e => e,
        }
        .to_string();
        let output = match p.get("output").and_then(Value::as_str) {
            Some(o) => PathBuf::from(o),
            None => {
                let stem = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default();
                path.with_file_name(format!("{stem} (redacted).{ext}"))
            }
        };
        let written = core.call(
            ctx.source,
            "document.export",
            json!({"documentId": id, "format": format, "path": output, "onConflict": "keepBoth"}),
            Some(lang),
        )?;
        result["output"] = written["path"].clone();
        Ok(result)
    })();
    if was_open.is_none() {
        let _ = core.call(
            ctx.source,
            "document.close",
            json!({"documentId": id, "discard": [id]}),
            Some(lang),
        );
    }
    done
}

/// `smart.redact_words` (SMT-014): the words recognized in an open document, by line,
/// in canvas pixels, so that the panel can highlight them and redact one with a click.
/// The text goes back to the panel only; it is never kept.
pub fn words(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let id = xs_editor::resolve_document(core, lang, p)?;
    let (image, origin) = auto_redact::image_without(core, &id).map_err(CommandError::internal)?;
    let languages = crate::extract::wanted(core, None);
    let reading = read(&image, &languages, lang)?;
    let (ox, oy) = (origin[0] as f64, origin[1] as f64);
    let lines: Vec<Value> = reading
        .text
        .iter()
        .flat_map(|r| r.blocks.iter().flat_map(|b| &b.lines))
        .map(|l| {
            let words: Vec<Value> = l
                .words
                .iter()
                .map(|w| json!({"text": w.text, "rect": [w.rect[0] - ox, w.rect[1] - oy, w.rect[2], w.rect[3]]}))
                .collect();
            json!({"words": words})
        })
        .collect();
    Ok(json!({"documentId": id, "lines": lines}))
}

/// Handlers of auto redact.
pub fn register(reg: &mut xs_core::command::Registry, weak: &std::sync::Weak<Core>) {
    let w = weak.clone();
    reg.handle("smart.redact", move |ctx, p| {
        let core = w
            .upgrade()
            .ok_or_else(|| CommandError::internal("shutting down"))?;
        command(&core, ctx, &p)
    });
    let w = weak.clone();
    reg.handle("smart.redact_words", move |ctx, p| {
        let core = w
            .upgrade()
            .ok_or_else(|| CommandError::internal("shutting down"))?;
        words(&core, ctx, &p)
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use xs_core::Source;

    #[test]
    fn styles_and_terms() {
        let (s, m) = style(&json!({"type": "pixelate", "margin": 30}));
        assert_eq!(s["blurType"], "pixelate");
        assert_eq!(m, 10.0);
        let (s, m) = style(&json!({"color": "#12345g"}));
        assert_eq!(
            (s["blurType"].as_str(), s["color"].as_str(), m),
            (Some("solid"), Some("#000000"), 2.0)
        );
        let (ok, bad) = terms(&json!([
            {"name": "Pedidos", "pattern": "PO-\\d{6}"},
            {"pattern": "PO-("},
            {"text": "Proyecto Orión", "exact": true},
            {"text": "  "},
            {"text": "Apagado", "on": false}
        ]));
        assert_eq!(
            ok.iter()
                .map(|(t, on)| (t.name.as_str(), *on))
                .collect::<Vec<_>>(),
            vec![("Pedidos", true), ("Proyecto Orión", true), ("Apagado", false)]
        );
        assert_eq!(bad, vec!["PO-(".to_string()]);
    }

    /// A white form with a label and a value on each line, as an image.
    fn form(lines: &[&str]) -> image::RgbaImage {
        use xs_core::document::{Document, Object};
        let (w, h) = (900u32, 60 + 60 * lines.len() as u32);
        let mut doc = Document::new(w, h, 96.0);
        for (i, text) in lines.iter().enumerate() {
            let props = json!({"at": [30, 30 + 60 * i], "width": null, "text": text, "rotation": 0,
                "style": {"font": "Segoe UI", "size": 24, "bold": false, "italic": false,
                          "color": "#000000", "align": "left", "opacity": 100}});
            doc.objects.push(Object {
                id: format!("o{i}"),
                kind: "text".into(),
                props: props.as_object().unwrap().clone(),
            });
        }
        let base = xs_image::render::Base {
            width: w,
            height: h,
            rgba: Arc::new(vec![255; (w * h * 4) as usize]),
            images: Arc::new(Default::default()),
        };
        xs_image::render::render(&doc, &base)
    }

    fn core_with_form(dir: &std::path::Path, lines: &[&str]) -> (Arc<Core>, String) {
        let core = Core::for_tests_with(
            xs_core::platform::Platform::Windows,
            vec![crate::extension(), xs_editor::extension()],
        );
        xs_editor::editor(&core).set_folders(dir.join("lib"), dir.join("rec"));
        let (id, _) = xs_editor::add_capture(&core, &form(lines), json!({}), 1.0, None).unwrap();
        (core, id)
    }

    fn redactions(core: &Core, id: &str) -> Vec<Value> {
        xs_editor::editor(core)
            .store
            .with(id, |d| {
                d.document
                    .objects
                    .iter()
                    .filter(|o| o.props.contains_key(auto_redact::MARK))
                    .map(|o| serde_json::to_value(o).unwrap())
                    .collect()
            })
            .unwrap()
    }

    /// SMT-010, SMT-011, SMT-013, SMT-017: a form with an email, a phone and a card has
    /// the three covered, one per category, in one undo step; again, nothing doubles;
    /// a category off takes its redaction away; a new style changes them all at once;
    /// the text is kept nowhere and the exported image no longer has it.
    #[test]
    fn a_form_redacted() {
        if xs_ocr::languages().is_empty() {
            eprintln!("no recognition language on this computer: skipped");
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let (core, id) = core_with_form(
            dir.path(),
            &[
                "Correo: ana.ruiz@example.com",
                "Teléfono: +34 612 345 678",
                // Windows' recognition leaves out a line of "4111 1111 1111 1111".
                "Tarjeta: 4539 1488 0343 6467",
                "Gracias por su pedido",
            ],
        );
        let call = |cmd: &str, p: Value| core.call(Source::Cli, cmd, p, Some(xs_core::Lang::Es));
        let r = call("smart.redact", json!({"documentId": id})).unwrap();
        assert_eq!(r["counts"]["email"], 1, "{r}");
        assert_eq!(r["counts"]["phone"], 1, "{r}");
        assert_eq!(r["counts"]["card"], 1, "{r}");
        assert_eq!(r["redactions"], 3, "{r}");
        // Only the data: the label stays visible.
        let objs = redactions(&core, &id);
        let email = objs
            .iter()
            .find(|o| o["autoRedact"]["category"] == "email")
            .unwrap();
        assert!(email["rect"]["x"].as_f64().unwrap() > 100.0, "{email}");
        assert_eq!(email["style"]["blurType"], "solid");
        // Nothing of the text in the document (SMT-017).
        let doc = serde_json::to_string(&objs).unwrap();
        assert!(!doc.contains("ana.ruiz") && !doc.contains("4539"), "{doc}");

        // Again: nothing doubled, nothing to undo.
        let again = call("smart.redact", json!({"documentId": id})).unwrap();
        assert_eq!(
            (again["redactions"].as_u64(), again["added"].as_u64()),
            (Some(3), Some(0))
        );
        call("document.undo", json!({"documentId": id})).unwrap();
        assert!(
            redactions(&core, &id).is_empty(),
            "one undo takes the whole analysis away"
        );

        call("smart.redact", json!({"documentId": id})).unwrap();
        // Phones off: its redaction goes, the others stay; counted all the same.
        let off = call(
            "smart.redact",
            json!({"documentId": id, "categories": ["email", "card", "face"]}),
        )
        .unwrap();
        assert_eq!(
            (off["removed"].as_u64(), off["redactions"].as_u64()),
            (Some(1), Some(2))
        );
        assert_eq!(off["counts"]["phone"], 1, "{off}");
        assert_eq!(
            core.settings.get("smart.autoRedact.categories"),
            Some(json!(["email", "card", "face"]))
        );
        // A new style for all of them, in one step.
        let styled = call(
            "smart.redact",
            json!({"documentId": id, "style": {"type": "pixelate"}}),
        )
        .unwrap();
        assert_eq!(styled["restyled"], 2);
        assert!(redactions(&core, &id)
            .iter()
            .all(|o| o["style"]["blurType"] == "pixelate"));
        call("document.undo", json!({"documentId": id})).unwrap();
        assert!(redactions(&core, &id)
            .iter()
            .all(|o| o["style"]["blurType"] == "solid"));

        // The final image no longer has the data (SMT-017).
        let text = call(
            "smart.extract_text",
            json!({"source": "document", "documentId": id, "copy": false}),
        )
        .unwrap();
        let text = text["text"].as_str().unwrap();
        assert!(!text.contains("ana.ruiz") && !text.contains("4539"), "{text}");
        assert!(text.contains("Gracias"), "{text}");

        let cleared = call("smart.redact", json!({"documentId": id, "clear": true})).unwrap();
        assert_eq!(cleared["removed"], 2);
        assert!(redactions(&core, &id).is_empty());
    }

    /// SMT-014, SMT-015: a word redacted by hand stays through a new analysis; all the
    /// places of a name are redacted at once, and uncovered at once; an automatic
    /// redaction removed by hand is not put back.
    #[test]
    fn redactions_by_hand() {
        if xs_ocr::languages().is_empty() {
            eprintln!("no recognition language on this computer: skipped");
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let (core, id) = core_with_form(
            dir.path(),
            &[
                "Cliente: Marta Ruiz",
                "Contacto: Marta Ruiz",
                "Firma de Marta Ruiz",
                "Correo: ana.ruiz@example.com",
            ],
        );
        let call = |cmd: &str, p: Value| core.call(Source::Cli, cmd, p, Some(xs_core::Lang::Es)).unwrap();
        let r = call("smart.redact", json!({"documentId": id, "categories": ["email"]}));
        assert_eq!(r["redactions"], 1, "{r}");
        // Click on a word: it is redacted, in Manual.
        let words = call("smart.redact_words", json!({"documentId": id}));
        let marta = words["lines"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|l| l["words"].as_array().unwrap().iter())
            .find(|w| w["text"] == "Marta")
            .expect("the word Marta")
            .clone();
        let r = call("smart.redact", json!({"documentId": id, "add": [marta["rect"]]}));
        assert_eq!(
            (r["addedByHand"].as_u64(), r["counts"]["manual"].as_u64()),
            (Some(1), Some(1)),
            "{r}"
        );
        // A new analysis keeps it.
        let r = call("smart.redact", json!({"documentId": id}));
        assert_eq!(r["counts"]["manual"], 1, "{r}");
        // Every place of the name: 3, the first one taking the place of the word's.
        let r = call(
            "smart.redact",
            json!({"documentId": id, "matches": {"text": "Marta Ruiz"}}),
        );
        assert_eq!(
            (r["addedByHand"].as_u64(), r["counts"]["manual"].as_u64()),
            (Some(3), Some(3)),
            "{r}"
        );
        // Uncovered at once.
        let r = call(
            "smart.redact",
            json!({"documentId": id, "matches": {"text": "Marta Ruiz", "remove": true}}),
        );
        assert_eq!(r["counts"].get("manual"), None, "{r}");
        call("document.undo", json!({"documentId": id}));
        assert_eq!(
            call("smart.redact", json!({"documentId": id}))["counts"]["manual"],
            3
        );
        // The email's redaction removed by hand: not put back by an analysis.
        let email = r["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["category"] == "email")
            .unwrap()["objectId"]
            .clone();
        call("smart.redact", json!({"documentId": id, "remove": [email]}));
        let r = call("smart.redact", json!({"documentId": id}));
        assert!(
            r["items"]
                .as_array()
                .unwrap()
                .iter()
                .all(|i| i["category"] != "email"),
            "{r}"
        );
        assert_eq!(
            r["counts"].get("email"),
            None,
            "its category counts one less: {r}"
        );
        // Undo puts it back, and the next analysis keeps it.
        call("document.undo", json!({"documentId": id}));
        let r = call("smart.redact", json!({"documentId": id}));
        assert_eq!(r["counts"]["email"], 1, "{r}");
        assert!(
            r["items"]
                .as_array()
                .unwrap()
                .iter()
                .any(|i| i["category"] == "email"),
            "{r}"
        );
    }

    /// SMT-012: a word of the user's and a pattern count as categories of their own; an
    /// invalid pattern is named and left out.
    #[test]
    fn my_terms() {
        if xs_ocr::languages().is_empty() {
            eprintln!("no recognition language on this computer: skipped");
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let (core, id) = core_with_form(
            dir.path(),
            &["Pedido PO-123456 y PO-12345", "proyecto orion listo"],
        );
        let r = core
            .call(
                Source::Cli,
                "smart.redact",
                json!({"documentId": id, "categories": [], "terms": [
                    {"name": "Pedidos", "pattern": "PO-\\d{6}\\b"},
                    {"text": "Proyecto Orión"},
                    {"name": "Roto", "pattern": "("}
                ]}),
                None,
            )
            .unwrap();
        assert_eq!(r["counts"]["term:Pedidos"], 1, "{r}");
        assert_eq!(r["counts"]["term:Proyecto Orión"], 1, "{r}");
        assert_eq!(r["invalidPatterns"], json!(["Roto"]));
        assert_eq!(r["redactions"], 2);
    }

    /// SMT-010: an image file is redacted apart and written next to it; the file itself
    /// never changes and no document stays open.
    #[test]
    fn a_file_redacted() {
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
        let path = dir.path().join("form.png");
        form(&["Correo: ana.ruiz@example.com"]).save(&path).unwrap();
        let before = std::fs::read(&path).unwrap();
        let r = core
            .call(Source::Cli, "smart.redact", json!({"path": path}), None)
            .unwrap();
        assert_eq!(r["counts"]["email"], 1, "{r}");
        let out = std::path::PathBuf::from(r["output"].as_str().unwrap());
        assert_eq!(out.file_name().unwrap(), "form (redacted).png");
        assert_eq!(std::fs::read(&path).unwrap(), before);
        let text = xs_ocr::recognize(
            &image::open(&out).unwrap().to_rgba8(),
            &["auto".to_string()],
            &AtomicBool::new(false),
        )
        .unwrap();
        let words: String = text
            .blocks
            .iter()
            .flat_map(|b| &b.lines)
            .flat_map(|l| &l.words)
            .map(|w| w.text.clone())
            .collect();
        assert!(words.contains("Correo") && !words.contains("ana.ruiz"), "{words}");
        assert!(xs_editor::editor(&core).store.ids().is_empty());
        // A video is refused before anything is opened.
        let refused = core
            .call(
                Source::Cli,
                "smart.redact",
                json!({"path": dir.path().join("clip.mp4")}),
                None,
            )
            .unwrap_err();
        assert_eq!(refused.code, xs_core::ErrorCode::InvalidParams);
    }
}
