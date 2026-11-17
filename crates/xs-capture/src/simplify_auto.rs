//! Simplify automatically (SMT-019): text lines become rounded bars of their height
//! (with little detail, paragraphs), and buttons, fields and icons become simple shapes
//! of the active palette's nearest color, as normal simplify objects (EDT-041).
//!
//! The generated objects carry `autoSimplify: {fp, set, total, detail}`: `fp` tells an
//! object the user changed, `total` one the user deleted. Generating again with changes
//! asks first (`needsConfirm`); objects drawn by hand are never touched. Switching off
//! removes the generated objects the user did not change. Everything is one undo step.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{json, Map, Value};
use xs_core::document::Object;
use xs_core::i18n::tr;
use xs_core::{CommandError, CommandResult, Context, Core, ErrorCode};
use xs_editor::objects;
use xs_image::ui_detect::{self, ElementShape};

use crate::redact;

pub const MARK: &str = "autoSimplify";

/// Objects left out of the picture the text and the elements are read from.
const COVERING: [&str; 4] = ["simplify", "highlight", "blur", "spotlight"];

static SETS: AtomicU64 = AtomicU64::new(1);

/// A fingerprint of an object's props as generated, without the mark.
fn fingerprint(props: &Map<String, Value>) -> String {
    let mut p = props.clone();
    p.remove(MARK);
    let mut h = DefaultHasher::new();
    serde_json::to_string(&p).unwrap_or_default().hash(&mut h);
    format!("{:016x}", h.finish())
}

/// The generated objects of a document: changed by the user or not.
struct Generated {
    ids: Vec<String>,
    unchanged: Vec<String>,
    /// Changed or deleted since they were generated.
    edited: usize,
    detail: Option<u64>,
}

fn generated(doc: &xs_core::document::Document) -> Generated {
    let mut g = Generated {
        ids: Vec::new(),
        unchanged: Vec::new(),
        edited: 0,
        detail: None,
    };
    let mut sets: std::collections::HashMap<String, (u64, u64)> = Default::default();
    for o in doc.objects.iter().filter(|o| o.kind == "simplify") {
        let Some(mark) = o.props.get(MARK) else {
            continue;
        };
        g.ids.push(o.id.clone());
        g.detail = mark.get("detail").and_then(Value::as_u64).or(g.detail);
        if mark.get("fp").and_then(Value::as_str) == Some(fingerprint(&o.props).as_str()) {
            g.unchanged.push(o.id.clone());
        } else {
            g.edited += 1;
        }
        let set = mark.get("set").map(|v| v.to_string()).unwrap_or_default();
        let total = mark.get("total").and_then(Value::as_u64).unwrap_or(0);
        let e = sets.entry(set).or_insert((0, total));
        e.0 += 1;
    }
    // Deleted ones count as changed.
    g.edited += sets
        .values()
        .map(|(present, total)| total.saturating_sub(*present) as usize)
        .sum::<usize>();
    g
}

fn kind_of(shape: ElementShape) -> &'static str {
    match shape {
        ElementShape::Rect => "block",
        ElementShape::Rounded => "roundedBlock",
        ElementShape::Circle => "circle",
        ElementShape::Icon => "icon",
    }
}

fn rgb_of(hex: &str) -> [u8; 3] {
    xs_image::color::Rgba::parse(hex)
        .map(|c| [c.r, c.g, c.b])
        .unwrap_or([60, 60, 60])
}

/// `smart.simplify_auto` (SMT-019).
pub fn command(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let id = xs_editor::resolve_document(core, lang, p)?;
    xs_editor::video::refuse_for_video(core, lang, &json!({"documentId": id}))?;
    let internal = |e: String| CommandError::internal(e);
    let ed = xs_editor::editor(core);
    let detail = p.get("detail").and_then(Value::as_u64).unwrap_or(5).clamp(1, 10);
    let force = p.get("force").and_then(Value::as_bool) == Some(true);
    let now = ed
        .store
        .with(&id, |d| generated(&d.document))
        .ok_or_else(|| internal("document closed".into()))?;

    // Switched off: the generated objects the user did not change go.
    if p.get("remove").and_then(Value::as_bool) == Some(true) {
        let gone = now.unchanged.clone();
        if !gone.is_empty() {
            ed.store
                .mutate(&id, "history.auto_simplify", |doc| {
                    doc.objects.retain(|o| !gone.contains(&o.id));
                    Ok::<_, String>(())
                })
                .ok_or_else(|| internal("document closed".into()))?
                .map_err(internal)?;
            xs_editor::document_changed(core, &id);
        }
        return Ok(json!({"documentId": id, "removed": gone.len(), "kept": now.ids.len() - gone.len()}));
    }
    if now.edited > 0 && !force {
        return Ok(
            json!({"documentId": id, "needsConfirm": true, "edited": now.edited, "detail": now.detail}),
        );
    }

    // What is below the simplified shapes.
    let mut snap = ed.store.snapshot(&id).map_err(internal)?;
    snap.document
        .objects
        .retain(|o| !COVERING.contains(&o.kind.as_str()));
    let (image, origin) = snap.render_with_origin();
    let languages = crate::extract::wanted(core, None);
    let reading = redact::read(&image, &languages, lang)?;
    let lines: Vec<&xs_ocr::Line> = reading
        .text
        .iter()
        .flat_map(|r| r.blocks.iter().flat_map(|b| &b.lines))
        .collect();
    let line_rects: Vec<[f64; 4]> = lines.iter().map(|l| l.rect).collect();
    let elements = ui_detect::detect(&image, &line_rects, detail as u8);
    let blocks = ui_detect::text_blocks(&line_rects, detail as u8);

    let tool = ed.styles().simplify;
    let palette: Vec<String> = tool
        .palettes
        .iter()
        .find(|pl| {
            Some(pl.name.as_str())
                == p.get("palette")
                    .and_then(Value::as_str)
                    .or(Some(tool.palette.as_str()))
        })
        .or_else(|| tool.palettes.first())
        .map(|pl| pl.colors.clone())
        .unwrap_or_default();
    let color_for = |c: [u8; 3]| -> String {
        ui_detect::nearest(c, &palette)
            .cloned()
            .unwrap_or_else(|| tool.shape.fill.clone())
    };
    // The text's color: the darkest of the line's, or of all.
    let text_color = |r: [f64; 4]| -> [u8; 3] {
        lines
            .iter()
            .find(|l| l.rect[1] >= r[1] - 1.0 && l.rect[1] <= r[1] + r[3])
            .map(|l| rgb_of(&l.color))
            .unwrap_or([40, 40, 40])
    };
    let (ox, oy) = (origin[0] as f64, origin[1] as f64);
    let mut wanted: Vec<Map<String, Value>> = Vec::new();
    // The shapes first; the text bars over them (a button's label).
    for e in &elements {
        let [x, y, w, h] = e.rect;
        wanted.push(
            json!({
                "rect": {"x": x as f64 - ox, "y": y as f64 - oy, "w": w, "h": h},
                "rotation": 0,
                "style": {"kind": kind_of(e.shape), "fill": color_for(e.color)}
            })
            .as_object()
            .cloned()
            .unwrap_or_default(),
        );
    }
    for b in &blocks {
        let [x, y, w, h] = b.rect;
        let kind = if b.lines > 1 { "paragraph" } else { "textLine" };
        // As tall as the line: it covers the text.
        let bar = b.line_height.clamp(2.0, 200.0);
        wanted.push(
            json!({
                "rect": {"x": x - ox, "y": y - oy, "w": w.max(2.0), "h": h.max(bar)},
                "rotation": 0,
                "style": {"kind": kind, "fill": color_for(text_color(b.rect)), "textHeight": bar.round(),
                          "lineSpacing": (b.spacing * 10.0).round() / 10.0}
            })
            .as_object()
            .cloned()
            .unwrap_or_default(),
        );
    }
    if wanted.is_empty() && now.ids.is_empty() {
        return Err(CommandError::new(
            ErrorCode::InvalidParams,
            tr(lang, "errors.simplify_nothing", &[]),
        ));
    }
    let set = SETS.fetch_add(1, Ordering::SeqCst);
    let total = wanted.len();
    let old = now.ids.clone();
    let added = ed
        .store
        .mutate(&id, "history.auto_simplify", |doc| {
            doc.objects.retain(|o| !old.contains(&o.id));
            let mut n = 0;
            for props in &wanted {
                let mut checked = objects::checked("simplify", props)?;
                let fp = fingerprint(&checked);
                checked.insert(
                    MARK.into(),
                    json!({"fp": fp, "set": set, "total": total, "detail": detail}),
                );
                let oid = doc.next_object_id();
                doc.objects.push(Object {
                    id: oid,
                    kind: "simplify".into(),
                    props: checked,
                });
                n += 1;
            }
            Ok::<_, String>(n)
        })
        .ok_or_else(|| internal("document closed".into()))?
        .map_err(internal)?;
    xs_editor::document_changed(core, &id);
    tracing::info!(
        text = blocks.len(),
        elements = elements.len(),
        detail,
        "auto simplify"
    );
    Ok(json!({
        "documentId": id,
        "added": added,
        "removed": old.len(),
        "text": blocks.len(),
        "elements": elements.len(),
        "detail": detail,
    }))
}

pub fn register(reg: &mut xs_core::command::Registry, weak: &std::sync::Weak<Core>) {
    let w = weak.clone();
    reg.handle("smart.simplify_auto", move |ctx, p| {
        let core = w
            .upgrade()
            .ok_or_else(|| CommandError::internal("shutting down"))?;
        command(&core, ctx, &p)
    });
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::sync::Arc;
    use xs_core::Source;

    /// A settings window: lines of text and three buttons.
    pub(crate) fn settings_window() -> image::RgbaImage {
        use xs_core::document::{Document, Object};
        let (w, h) = (900u32, 420u32);
        let mut doc = Document::new(w, h, 96.0);
        let mut add = |kind: &str, props: Value| {
            let id = format!("o{}", doc.objects.len());
            doc.objects.push(Object {
                id,
                kind: kind.into(),
                props: props.as_object().unwrap().clone(),
            });
        };
        for (i, text) in [
            "Opciones de la cuenta",
            "Mostrar avisos al iniciar",
            "Guardar copia cada hora",
        ]
        .iter()
        .enumerate()
        {
            add(
                "text",
                json!({"at": [30, 30 + 60 * i], "width": null, "text": text, "rotation": 0,
                       "style": {"font": "Segoe UI", "size": 24, "bold": false, "italic": false,
                                 "color": "#202020", "align": "left", "opacity": 100}}),
            );
        }
        for i in 0..3 {
            add(
                "shape",
                json!({"rect": {"x": 420 + i * 150, "y": 330, "w": 120, "h": 44}, "rotation": 0,
                       "style": {"shapeType": "rectangle", "fill": "#2D6CC4", "outline": "#2D6CC4", "outlineWidth": 0}}),
            );
        }
        let base = xs_image::render::Base {
            width: w,
            height: h,
            rgba: Arc::new(vec![255; (w * h * 4) as usize]),
            images: Arc::new(Default::default()),
        };
        xs_image::render::render(&doc, &base)
    }

    fn core_with(dir: &std::path::Path) -> (Arc<Core>, String) {
        let core = Core::for_tests_with(
            xs_core::platform::Platform::Windows,
            vec![crate::extension(), xs_editor::extension()],
        );
        xs_editor::editor(&core).set_folders(dir.join("lib"), dir.join("rec"));
        let (id, _) = xs_editor::add_capture(&core, &settings_window(), json!({}), 1.0, None).unwrap();
        (core, id)
    }

    fn simplified(core: &Core, id: &str) -> Vec<Object> {
        xs_editor::editor(core)
            .store
            .with(id, |d| {
                d.document
                    .objects
                    .iter()
                    .filter(|o| o.kind == "simplify")
                    .cloned()
                    .collect()
            })
            .unwrap()
    }

    /// SMT-019: each text line gets a bar and each button a shape of the palette; after
    /// moving one, regenerating asks first and Cancel leaves everything; a shape drawn
    /// by hand stays; switching off removes only what the user did not change.
    #[test]
    fn auto_simplify_a_settings_window() {
        if xs_ocr::languages().is_empty() {
            eprintln!("no recognition language on this computer: skipped");
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let (core, id) = core_with(dir.path());
        let call = |cmd: &str, p: Value| core.call(Source::Cli, cmd, p, Some(xs_core::Lang::Es)).unwrap();
        // One drawn by hand first.
        call(
            "document.annotate",
            json!({"documentId": id, "objects": [{"type": "simplify", "rect": {"x": 700, "y": 20, "w": 80, "h": 40}, "style": {"kind": "block"}}]}),
        );
        let r = call("smart.simplify_auto", json!({"documentId": id, "detail": 8}));
        assert_eq!(
            (r["text"].as_u64(), r["elements"].as_u64()),
            (Some(3), Some(3)),
            "{r}"
        );
        let all = simplified(&core, &id);
        let bars = all
            .iter()
            .filter(|o| o.props["style"]["kind"] == "textLine")
            .count();
        let buttons: Vec<&Object> = all
            .iter()
            .filter(|o| o.props.contains_key(MARK) && o.props["style"]["kind"] != "textLine")
            .collect();
        assert_eq!((bars, buttons.len()), (3, 3), "{all:?}");
        // The palette's blue, nearest to the buttons' color.
        assert!(
            buttons.iter().all(|o| o.props["style"]["fill"] == "#2D7FF9"),
            "{buttons:?}"
        );

        // Move one: changing the detail asks first; without force nothing changes.
        let moved = buttons[0].id.clone();
        call(
            "document.update_object",
            json!({"documentId": id, "objectIds": [moved], "move": [5, 5]}),
        );
        let before = simplified(&core, &id);
        let r = call("smart.simplify_auto", json!({"documentId": id, "detail": 2}));
        assert_eq!(
            (r["needsConfirm"].as_bool(), r["edited"].as_u64()),
            (Some(true), Some(1)),
            "{r}"
        );
        assert_eq!(simplified(&core, &id), before);
        // Regenerate: with little detail the three buttons are one shape; the one drawn
        // by hand is still there.
        let r = call(
            "smart.simplify_auto",
            json!({"documentId": id, "detail": 1, "force": true}),
        );
        assert_eq!(r["elements"].as_u64(), Some(1), "{r}");
        let all = simplified(&core, &id);
        assert!(
            all.iter()
                .any(|o| !o.props.contains_key(MARK) && o.props["rect"]["x"] == 700.0),
            "{all:?}"
        );
        // One undo goes back to before regenerating.
        call("document.undo", json!({"documentId": id}));
        assert_eq!(simplified(&core, &id), before);

        // Switched off: the generated ones go, except the one moved; the hand-drawn stays.
        let r = call("smart.simplify_auto", json!({"documentId": id, "remove": true}));
        assert_eq!(r["kept"].as_u64(), Some(1), "{r}");
        let left = simplified(&core, &id);
        assert_eq!(left.len(), 2, "{left:?}");
    }
}
