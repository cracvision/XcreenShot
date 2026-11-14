//! Guides of steps (SMT-028, ADR 0009): what the step recorder hands over becomes a
//! guide, a template document (`template.id = "guide"`) with a header, one numbered
//! section per step (its text above its picture) and a footer. Each step's picture is
//! the whole monitor; its zone frames the part that matters, so that a wider framing
//! later loses nothing (SMT-026). The click's ring, the drag's arrow and the pointer
//! are ordinary objects of the step's section.

use super::*;
use crate::template_cmds::{active_theme, create_document, relayout, transparent_pixel};
use crate::templates::{self, Template};

/// One step as the recorder hands it over. Positions in pixels of its picture.
pub struct GuideStep {
    /// With the names in bold (`**…**`).
    pub text: String,
    /// The whole monitor, PNG.
    pub png: Vec<u8>,
    pub size: [u32; 2],
    /// The part shown (SMT-026).
    pub frame: [f64; 4],
    pub at: [f64; 2],
    /// Where a drag ended.
    pub to: Option<[f64; 2]>,
    pub right: bool,
    /// The click's ring (not on a keyboard shortcut's step).
    pub ring: bool,
    /// The pointer (CAP-047): its PNG, its top left and its size.
    pub pointer: Option<(Vec<u8>, [f64; 2], [u32; 2])>,
}

/// A guide to create: its title, steps and metadata (CAP-056).
pub struct GuideIn {
    pub title: String,
    pub steps: Vec<GuideStep>,
    pub meta: Value,
}

/// The ring's colors (SMT-026): orange for a click, violet for a right click.
const RING_LEFT: &str = "#F28C28";
const RING_RIGHT: &str = "#8E5BD9";
const RING_SIZE: f64 = 44.0;

/// Zoom and offset of a zone of `zone` (filling) so that it shows `frame` of a picture
/// of `size` (SMT-026).
pub fn framing(zone: [f64; 4], size: [u32; 2], frame: [f64; 4]) -> (f64, [f64; 2]) {
    let (iw, ih) = (size[0].max(1) as f64, size[1].max(1) as f64);
    let k = zone[2] / frame[2].max(1.0);
    let k0 = (zone[2] / iw).max(zone[3] / ih);
    let zoom = (k / k0).clamp(0.1, 100.0);
    let k = k0 * zoom;
    let off_x = -frame[0] * k - (zone[2] - iw * k) / 2.0;
    let off_y = -frame[1] * k - (zone[3] - ih * k) / 2.0;
    (zoom, [off_x, off_y])
}

fn rect_of(o: &Object) -> Option<[f64; 4]> {
    let r = o.props.get("rect")?;
    Some([
        r.get("x")?.as_f64()?,
        r.get("y")?.as_f64()?,
        r.get("w")?.as_f64()?,
        r.get("h")?.as_f64()?,
    ])
}

fn slot_of(o: &Object) -> Option<&str> {
    o.props.get("slot").and_then(Value::as_str)
}

/// Creates the guide (SMT-028), writes it to the library and opens it in the store (not
/// shown). Returns its id and file.
pub fn create_guide(core: &Arc<Core>, lang: Lang, g: GuideIn) -> Result<(String, PathBuf), CommandError> {
    let (doc, meta, images) = build(core, lang, g);
    create_document(core, lang, doc, meta, transparent_pixel(), images)
}

/// Writes the guide as a recovery item (SMT-027, APP-047): a recording cut short by a
/// crash is offered in Recover work like an unsaved document. Returns its id.
pub fn recover_guide(core: &Arc<Core>, lang: Lang, g: GuideIn) -> Result<String, String> {
    use xs_core::document::{Manifest, Xshot, THUMB_SIZE};
    let name = g.title.clone();
    let (document, meta, images) = build(core, lang, g);
    let base_png = transparent_pixel();
    let mut base = Base::from_image(xs_image::encode::decode(&base_png)?);
    for (n, bytes) in &images {
        base.add_image(n, bytes)?;
    }
    let thumb = xs_image::encode::thumbnail(
        &crate::guide_export::thumb_picture(&document, &base, &meta),
        THUMB_SIZE,
    );
    let file = Xshot {
        manifest: Manifest::current(),
        document,
        base_png,
        meta,
        thumb_png: xs_image::encode::png(&thumb, 0.0, true)?,
        images,
    };
    let dir = editor(core).recovery_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let id = recovery::new_id();
    file.write(&recovery::document_path(&dir, &id))
        .map_err(|e| e.to_string())?;
    recovery::write_entry(
        &dir,
        &recovery::Entry {
            id: id.clone(),
            name,
            original_path: None,
            kind: "library".into(),
            format: "xshot".into(),
            saved_at: chrono::Utc::now(),
            ..Default::default()
        },
    )
    .map_err(|e| e.to_string())?;
    Ok(id)
}

/// The guide's document, metadata and pictures (SMT-028).
fn build(core: &Arc<Core>, lang: Lang, g: GuideIn) -> (Document, Value, BTreeMap<String, Vec<u8>>) {
    let pictures: Vec<[f64; 2]> = g
        .steps
        .iter()
        .map(|s| templates::guide_picture([s.frame[2], s.frame[3]]))
        .collect();
    let mut t = templates::guide(&pictures);
    t.settings.theme = Some(active_theme(core));
    // "Created by": the guides' author, else the account's name (SMT-028).
    let author = core.settings.get_str("smart.steps.author");
    if !author.trim().is_empty() {
        t.settings.author = "custom".into();
        t.settings.author_text = author.trim().to_string();
    }
    let mut doc = Document::new(1, 1, 96.0);
    relayout(core, lang, &mut doc, &t, true);
    let step_sections: Vec<String> = t
        .sections
        .iter()
        .filter(|s| s.kind == "guideStep")
        .map(|s| s.id.clone())
        .collect();
    if let Some(o) = doc.objects.iter_mut().find(|o| slot_of(o) == Some("title")) {
        o.props.insert("text".into(), json!(g.title));
    }
    let mut images = BTreeMap::new();
    fill(core, lang, &mut doc, &step_sections, &g.steps, 0, &mut images);
    let mut meta = g.meta;
    meta["name"] = json!(g.title);
    (doc, meta, images)
}

/// Puts each step in its section (`sids`, laid out already): its picture, named after
/// `first` (`step-NNN.png`), its text, the framing, the click's ring, the drag's arrow
/// and the pointer (SMT-026, SMT-028).
fn fill(
    core: &Arc<Core>,
    lang: Lang,
    doc: &mut Document,
    sids: &[String],
    steps: &[GuideStep],
    first: usize,
    images: &mut BTreeMap<String, Vec<u8>>,
) {
    for (i, (step, sid)) in steps.iter().zip(sids).enumerate() {
        let name = format!("step-{:03}.png", first + i + 1);
        images.insert(name.clone(), step.png.clone());
        for o in doc.objects.iter_mut() {
            match slot_of(o) {
                Some(s) if s == format!("zone:{sid}:0") => {
                    o.props.insert("image".into(), json!(name));
                }
                Some(s) if s == format!("caption:{sid}:0") => {
                    o.props.insert("text".into(), json!(step.text));
                }
                _ => {}
            }
        }
    }
    // Laid out again: each section as tall as its text, the numbers on the steps.
    if let Some(t) = Template::of(doc) {
        relayout(core, lang, doc, &t, false);
    }
    // Each zone frames its step; the ring, the arrow and the pointer go on it.
    for (i, (step, sid)) in steps.iter().zip(sids).enumerate() {
        let Some(zone) = doc
            .objects
            .iter_mut()
            .find(|o| slot_of(o) == Some(&format!("zone:{sid}:0")))
        else {
            continue;
        };
        let Some(r) = rect_of(zone) else { continue };
        let (zoom, offset) = framing(r, step.size, step.frame);
        zone.props.insert("zoom".into(), json!(zoom));
        zone.props.insert("offset".into(), json!(offset));
        let k = r[2] / step.frame[2].max(1.0);
        let place = |p: [f64; 2]| {
            [
                r[0] + (p[0] - step.frame[0]) * k,
                r[1] + (p[1] - step.frame[1]) * k,
            ]
        };
        let at = place(step.at);
        let add = |kind: &str, props: Map<String, Value>, doc: &mut Document| {
            let id = doc.next_object_id();
            doc.objects.push(Object {
                id,
                kind: kind.to_string(),
                props,
            });
        };
        if let Some(to) = step.to {
            let to = place(to);
            if let Ok(props) = objects::checked(
                "arrow",
                json!({"from": at, "to": to, "style": {"color": RING_LEFT, "width": 6}})
                    .as_object()
                    .expect("object"),
            ) {
                add("arrow", props, doc);
            }
        }
        if step.ring {
            let color = if step.right { RING_RIGHT } else { RING_LEFT };
            let ring = json!({
                "rect": {"x": at[0] - RING_SIZE / 2.0, "y": at[1] - RING_SIZE / 2.0, "w": RING_SIZE, "h": RING_SIZE},
                "rotation": 0,
                "style": {"shapeType": "ellipse", "fill": null, "outline": color, "outlineWidth": 5, "opacity": 75},
            });
            if let Ok(props) = objects::checked("shape", ring.as_object().expect("object")) {
                add("shape", props, doc);
            }
        }
        if let Some((png, top_left, size)) = &step.pointer {
            let name = format!("pointer-{:03}.png", first + i + 1);
            let p = place(*top_left);
            let (w, h) = (size[0] as f64 * k, size[1] as f64 * k);
            if let Ok(mut props) = objects::image_object(&name, [p[0], p[1], w.max(1.0), h.max(1.0)]) {
                if let Some(style) = props.get_mut("style").and_then(Value::as_object_mut) {
                    style.insert(
                        "cursor".into(),
                        json!({"shape": "captured", "highlight": "none", "natural": [size[0], size[1]]}),
                    );
                }
                images.insert(name, png.clone());
                add("image", props, doc);
            }
        }
    }
}

/// "Record more steps here" (SMT-029): steps go into an open guide, after the section
/// `after` (else at the end), in one undo step. Returns how many.
pub fn add_steps(
    core: &Arc<Core>,
    ctx: &Context,
    id: &str,
    after: Option<&str>,
    steps: Vec<GuideStep>,
) -> Result<usize, CommandError> {
    let lang = ctx.lang;
    let ed = editor(core);
    let doc = ed
        .store
        .with(id, |d| d.document.clone())
        .ok_or_else(|| CommandError::internal("document closed"))?;
    let used = ed.store.image_names(id);
    let mut t = Template::of(&doc)
        .filter(|t| t.is_guide())
        .ok_or_else(|| invalid(tr(lang, "errors.not_a_guide", &[])))?;
    let at = after
        .and_then(|a| t.sections.iter().position(|s| s.id == a))
        .map(|i| i + 1)
        .unwrap_or_else(|| {
            t.sections
                .iter()
                .rposition(|s| s.kind != "footer")
                .map_or(0, |i| i + 1)
        });
    let mut sids = Vec::new();
    for (k, st) in steps.iter().enumerate() {
        let mut s = t.new_section("guideStep", 1);
        s.natural = Some([st.frame[2], st.frame[3]]);
        s.picture = Some(templates::guide_picture([st.frame[2], st.frame[3]]));
        sids.push(s.id.clone());
        t.sections.insert(at + k, s);
    }
    let mut next = doc;
    relayout(core, lang, &mut next, &t, false);
    // New names after the pictures already there.
    let first = used
        .iter()
        .filter_map(|n| {
            n.strip_prefix("step-")?
                .strip_suffix(".png")?
                .parse::<usize>()
                .ok()
        })
        .max()
        .unwrap_or(0);
    let mut images = BTreeMap::new();
    fill(core, lang, &mut next, &sids, &steps, first, &mut images);
    ed.store.add_images(id, &images).map_err(CommandError::internal)?;
    ed.store
        .mutate(id, "history.add_steps", |d| {
            *d = next;
            Ok::<_, CommandError>(())
        })
        .ok_or_else(|| CommandError::internal("document closed"))??;
    changed(core, ctx, id);
    Ok(steps.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png(w: u32, h: u32) -> Vec<u8> {
        let img = image::RgbaImage::from_pixel(w, h, image::Rgba([200, 210, 230, 255]));
        xs_image::encode::png(&img, 96.0, true).unwrap()
    }

    /// SMT-028: a header, the steps numbered 1 to N with their text and picture, a long
    /// text seen whole above its picture, and the footer "Created by".
    #[test]
    fn a_guide_of_steps() {
        let dir = tempfile::tempdir().unwrap();
        let core = Core::for_tests_with(xs_core::platform::Platform::Windows, vec![crate::extension()]);
        editor(&core).set_folders(dir.path().join("lib"), dir.path().join("rec"));
        let long = format!("Clic en el botón **Guardar** {}", "y sigue ".repeat(50));
        let steps = vec![
            GuideStep {
                text: "En **Bloc de notas**, abrir el menú **Archivo**".into(),
                png: png(800, 600),
                size: [800, 600],
                frame: [0.0, 0.0, 800.0, 600.0],
                at: [40.0, 20.0],
                to: None,
                right: false,
                ring: true,
                pointer: Some((png(32, 32), [40.0, 20.0], [32, 32])),
            },
            GuideStep {
                text: long.clone(),
                png: png(1920, 1080),
                size: [1920, 1080],
                frame: [200.0, 100.0, 600.0, 400.0],
                at: [500.0, 300.0],
                to: Some([700.0, 450.0]),
                right: true,
                ring: true,
                pointer: None,
            },
        ];
        let (id, path) = create_guide(
            &core,
            Lang::Es,
            GuideIn {
                title: "Pasos en Bloc de notas".into(),
                steps,
                meta: json!({"source": "steps"}),
            },
        )
        .unwrap();
        assert!(path.is_file());
        let doc = editor(&core).store.with(&id, |d| d.document.clone()).unwrap();
        let t = Template::of(&doc).unwrap();
        assert!(t.is_guide());
        let text_of = |slot: &str| {
            doc.objects
                .iter()
                .find(|o| slot_of(o) == Some(slot))
                .and_then(|o| o.props.get("text").and_then(Value::as_str))
                .map(str::to_string)
        };
        assert_eq!(text_of("title").as_deref(), Some("Pasos en Bloc de notas"));
        let numbers: Vec<String> = doc
            .objects
            .iter()
            .filter(|o| o.kind == "step")
            .filter_map(|o| o.props.get("value").and_then(Value::as_str).map(str::to_string))
            .collect();
        assert_eq!(numbers, ["1", "2"]);
        assert!(text_of("footer").unwrap().starts_with("Creado por"));
        // The long text is whole, and the second section is taller for it.
        let steps: Vec<_> = t.sections.iter().filter(|s| s.kind == "guideStep").collect();
        let s2 = steps[1];
        assert_eq!(
            text_of(&format!("caption:{}:0", s2.id)).as_deref(),
            Some(long.as_str())
        );
        let text = doc
            .objects
            .iter()
            .find(|o| slot_of(o) == Some(&format!("caption:{}:0", s2.id)))
            .unwrap();
        let laid = xs_image::object_bounds(text, doc.dpi).unwrap();
        let zone = doc
            .objects
            .iter()
            .find(|o| slot_of(o) == Some(&format!("zone:{}:0", s2.id)))
            .unwrap();
        assert!(
            laid[3] <= rect_of(zone).unwrap()[1],
            "the text ends above its picture"
        );
        // The rings (orange, then violet for the right click), the arrow and the pointer.
        let rings: Vec<&str> = doc
            .objects
            .iter()
            .filter(|o| o.kind == "shape")
            .filter_map(|o| o.props.get("style")?.get("outline")?.as_str())
            .collect();
        assert_eq!(rings, [RING_LEFT, RING_RIGHT]);
        assert_eq!(doc.objects.iter().filter(|o| o.kind == "arrow").count(), 1);
        assert_eq!(doc.objects.iter().filter(|o| o.kind == "image").count(), 1);
        // It renders.
        let img = crate::final_image(&core, &id).unwrap();
        assert_eq!(img.width(), 1200);
    }

    /// SMT-029: editing a guide. Moving step 4 before step 2 numbers them 1 to 4 and its
    /// arrow stays on the same place of its picture; a longer text pushes its picture
    /// down (and the ring with it); duplicating, adding a text and a picture, the
    /// pictures at 50 %, and deleting several steps at once.
    #[test]
    fn editing_a_guide() {
        let dir = tempfile::tempdir().unwrap();
        let core = Core::for_tests_with(xs_core::platform::Platform::Windows, vec![crate::extension()]);
        editor(&core).set_folders(dir.path().join("lib"), dir.path().join("rec"));
        let step = |n: u32, drag: bool| GuideStep {
            text: format!("Paso **{n}**"),
            png: png(800, 500),
            size: [800, 500],
            frame: [0.0, 0.0, 800.0, 500.0],
            at: [100.0 * n as f64, 100.0],
            to: drag.then_some([400.0, 300.0]),
            right: false,
            ring: true,
            pointer: None,
        };
        let (id, _) = create_guide(
            &core,
            Lang::Es,
            GuideIn {
                title: "Guía".into(),
                steps: vec![step(1, false), step(2, false), step(3, false), step(4, true)],
                meta: json!({}),
            },
        )
        .unwrap();
        let call = |cmd: &str, p: Value| core.call(xs_core::Source::Cli, cmd, p, Some(Lang::Es)).unwrap();
        let doc = || editor(&core).store.with(&id, |d| d.document.clone()).unwrap();
        let steps_of = |d: &Document| -> Vec<String> {
            Template::of(d)
                .unwrap()
                .sections
                .iter()
                .filter(|s| s.kind == "guideStep")
                .map(|s| s.id.clone())
                .collect()
        };
        let zone = |d: &Document, sid: &str| {
            d.objects
                .iter()
                .find(|o| o.kind == "zone" && o.props.get("section").and_then(Value::as_str) == Some(sid))
                .and_then(rect_of)
                .unwrap()
        };
        let arrow_on_picture = |d: &Document, sid: &str| {
            let a = d.objects.iter().find(|o| o.kind == "arrow").unwrap();
            let from = a.props["from"].as_array().unwrap();
            let z = zone(d, sid);
            (from[0].as_f64().unwrap() - z[0], from[1].as_f64().unwrap() - z[1])
        };
        let numbers = |d: &Document| -> Vec<String> {
            d.objects
                .iter()
                .filter(|o| o.kind == "step")
                .map(|o| o.props["value"].as_str().unwrap().to_string())
                .collect()
        };
        let d = doc();
        let ids = steps_of(&d);
        let fourth = ids[3].clone();
        let before = arrow_on_picture(&d, &fourth);
        // Step 4 before step 2: header is section 0, so step 2 is at index 2.
        call(
            "document.template",
            json!({"documentId": id, "action": "move_section", "sectionId": fourth, "index": 2}),
        );
        let d = doc();
        assert_eq!(steps_of(&d)[1], fourth);
        assert_eq!(numbers(&d), ["1", "2", "3", "4"]);
        let after = arrow_on_picture(&d, &fourth);
        assert!(
            (after.0 - before.0).abs() < 1.0 && (after.1 - before.1).abs() < 1.0,
            "{before:?} {after:?}"
        );
        // A long text pushes its picture down; the arrow goes with the picture.
        let caption = d
            .objects
            .iter()
            .find(|o| o.props.get("slot").and_then(Value::as_str) == Some(&format!("caption:{fourth}:0")))
            .unwrap()
            .id
            .clone();
        let z0 = zone(&d, &fourth);
        let long = "y sigue ".repeat(80);
        call(
            "document.update_object",
            json!({"documentId": id, "objectId": caption, "set": {"text": long}}),
        );
        let d = doc();
        assert!(zone(&d, &fourth)[1] > z0[1] + 40.0, "the picture went down");
        let moved = arrow_on_picture(&d, &fourth);
        assert!((moved.0 - before.0).abs() < 1.0 && (moved.1 - before.1).abs() < 1.0);
        // Duplicate: one more step, with its arrow.
        call(
            "document.template",
            json!({"documentId": id, "action": "duplicate_section", "sectionId": fourth}),
        );
        let d = doc();
        assert_eq!(steps_of(&d).len(), 5);
        assert_eq!(d.objects.iter().filter(|o| o.kind == "arrow").count(), 2);
        assert_eq!(numbers(&d), ["1", "2", "3", "4", "5"]);
        // A text and an empty picture; an empty picture is not in the final image.
        call(
            "document.template",
            json!({"documentId": id, "action": "add_section", "kind": "guideText"}),
        );
        call(
            "document.template",
            json!({"documentId": id, "action": "add_section", "kind": "guideImage"}),
        );
        let d = doc();
        let t = Template::of(&d).unwrap();
        let kinds: Vec<&str> = t.sections.iter().map(|s| s.kind.as_str()).collect();
        assert_eq!(&kinds[kinds.len() - 3..], ["guideText", "guideImage", "footer"]);
        let printed = xs_image::render::printed_objects(&d.objects);
        assert!(
            printed.iter().filter(|o| o.kind == "zone").count()
                < d.objects.iter().filter(|o| o.kind == "zone").count()
        );
        // Pictures at 50 %: half as wide, framing kept.
        let w0 = zone(&d, &fourth)[2];
        call(
            "document.template",
            json!({"documentId": id, "action": "settings", "settings": {"imageScale": 50}}),
        );
        let d = doc();
        assert!(
            (zone(&d, &fourth)[2] - w0 / 2.0).abs() < 1.0,
            "{} {}",
            zone(&d, &fourth)[2],
            w0
        );
        // Its arrow shrinks with it, on the same place of the picture.
        let half = arrow_on_picture(&d, &fourth);
        assert!(
            (half.0 - before.0 / 2.0).abs() < 1.0 && (half.1 - before.1 / 2.0).abs() < 1.0,
            "{half:?} {before:?}"
        );
        // Several steps deleted at once.
        let ids = steps_of(&d);
        call(
            "document.template",
            json!({"documentId": id, "action": "remove_section", "sectionIds": [ids[0], ids[1]]}),
        );
        assert_eq!(steps_of(&doc()).len(), 3);
        assert_eq!(numbers(&doc()), ["1", "2", "3"]);
    }

    /// SMT-029: "Record more steps here" puts the new steps after the chosen one, with
    /// pictures of their own names, numbered with the rest, in one undo step.
    #[test]
    fn more_steps_go_where_asked() {
        let dir = tempfile::tempdir().unwrap();
        let core = Core::for_tests_with(xs_core::platform::Platform::Windows, vec![crate::extension()]);
        editor(&core).set_folders(dir.path().join("lib"), dir.path().join("rec"));
        let step = |n: u32| GuideStep {
            text: format!("Paso **{n}**"),
            png: png(600, 400),
            size: [600, 400],
            frame: [0.0, 0.0, 600.0, 400.0],
            at: [50.0, 50.0],
            to: None,
            right: false,
            ring: true,
            pointer: None,
        };
        let (id, _) = create_guide(
            &core,
            Lang::Es,
            GuideIn {
                title: "Guía".into(),
                steps: vec![step(1), step(2)],
                meta: json!({}),
            },
        )
        .unwrap();
        let ed = editor(&core);
        let first = Template::of(&ed.store.with(&id, |d| d.document.clone()).unwrap())
            .unwrap()
            .sections
            .iter()
            .find(|s| s.kind == "guideStep")
            .unwrap()
            .id
            .clone();
        let ctx = Context {
            source: xs_core::Source::Cli,
            lang: Lang::Es,
        };
        assert_eq!(
            add_steps(&core, &ctx, &id, Some(&first), vec![step(9)]).unwrap(),
            1
        );
        let doc = ed.store.with(&id, |d| d.document.clone()).unwrap();
        let texts: Vec<String> = Template::of(&doc)
            .unwrap()
            .sections
            .iter()
            .filter(|s| s.kind == "guideStep")
            .filter_map(|s| {
                doc.objects
                    .iter()
                    .find(|o| slot_of(o) == Some(&format!("caption:{}:0", s.id)))
                    .and_then(|o| o.props.get("text")?.as_str().map(str::to_string))
            })
            .collect();
        assert_eq!(texts, ["Paso **1**", "Paso **9**", "Paso **2**"]);
        assert!(ed.store.image_names(&id).contains(&"step-003.png".to_string()));
        let numbers: Vec<&str> = doc
            .objects
            .iter()
            .filter(|o| o.kind == "step")
            .filter_map(|o| o.props.get("value")?.as_str())
            .collect();
        assert_eq!(numbers, ["1", "2", "3"]);
        // One undo takes them away.
        core.call(
            xs_core::Source::Cli,
            "document.undo",
            json!({"documentId": id}),
            None,
        )
        .unwrap();
        let doc = ed.store.with(&id, |d| d.document.clone()).unwrap();
        assert_eq!(
            Template::of(&doc)
                .unwrap()
                .sections
                .iter()
                .filter(|s| s.kind == "guideStep")
                .count(),
            2
        );
    }

    /// SMT-030, SMT-032 to SMT-034: a guide of 3 steps in every format, from the editor.
    #[test]
    fn a_guide_exported() {
        let dir = tempfile::tempdir().unwrap();
        let core = Core::for_tests_with(xs_core::platform::Platform::Windows, vec![crate::extension()]);
        editor(&core).set_folders(dir.path().join("lib"), dir.path().join("rec"));
        let step = |n: u32| GuideStep {
            text: format!("Clic en **Botón {n}**"),
            png: png(640, 400),
            size: [640, 400],
            frame: [0.0, 0.0, 640.0, 400.0],
            at: [100.0, 100.0],
            to: None,
            right: false,
            ring: true,
            pointer: None,
        };
        let (id, _) = create_guide(
            &core,
            Lang::Es,
            GuideIn {
                title: "Alta de cliente".into(),
                steps: vec![step(1), step(2), step(3)],
                meta: json!({}),
            },
        )
        .unwrap();
        let out = dir.path().join("out");
        let export = |p: Value| {
            let mut p = p;
            p["documentId"] = json!(id);
            core.call(xs_core::Source::Cli, "smart.guide_export", p, Some(Lang::Es))
                .unwrap()
        };
        let docx = out.join("guia.docx");
        export(json!({"format": "docx", "out": docx}));
        let bytes = std::fs::read(&docx).unwrap();
        assert!(bytes.starts_with(b"PK"));
        let pdf = out.join("guia.pdf");
        export(json!({"format": "pdf", "out": pdf, "paper": "a4"}));
        let text = String::from_utf8_lossy(&std::fs::read(&pdf).unwrap()).into_owned();
        assert_eq!(text.matches("/Dest [").count(), 3);
        assert!(text.contains("Clic en "));
        export(json!({"format": "pptx", "out": out.join("guia.pptx")}));
        let r = export(json!({"format": "md", "out": out}));
        let md = std::path::PathBuf::from(r["paths"][0].as_str().unwrap());
        assert!(
            md.file_name().unwrap() == "Alta de cliente.md"
                && md.parent().unwrap().file_name().unwrap() == "Alta de cliente",
            "{md:?}"
        );
        assert!(md.parent().unwrap().join("images/paso-03.png").is_file());
        // Again: another folder, -2 (SMT-032).
        let again = export(json!({"format": "md", "out": out}));
        assert!(again["paths"][0].as_str().unwrap().contains("Alta de cliente-2"));
        let r = export(json!({"format": "html", "out": out, "singleFile": true}));
        let page = std::fs::read_to_string(r["paths"][0].as_str().unwrap()).unwrap();
        assert_eq!(page.matches("src=\"data:image/png;base64,").count(), 3);
        let r = export(json!({"format": "images", "out": out.join("sueltas"), "withText": true}));
        let paths: Vec<&str> = r["paths"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(paths.len(), 4);
        assert!(paths[0].ends_with("Alta_de_cliente-01.png"), "{paths:?}");
        assert!(paths[3].ends_with("pasos.txt"));
    }

    /// SMT-035: a guide in the library is of the type Guide, with its steps, and the
    /// search finds it by the text of one of its steps; it exports from its file.
    #[test]
    fn a_guide_in_the_library() {
        let library = tempfile::tempdir().unwrap();
        let recovery = tempfile::tempdir().unwrap();
        let core = Core::for_tests_with(
            xs_core::platform::Platform::Windows,
            vec![
                crate::extension(),
                xs_library::extension_with(Some(library.path().to_path_buf())),
            ],
        );
        editor(&core).set_folders(library.path().to_path_buf(), recovery.path().to_path_buf());
        let step = |n: u32| GuideStep {
            text: format!("Clic en **Botón {n}**"),
            png: png(300, 200),
            size: [300, 200],
            frame: [0.0, 0.0, 300.0, 200.0],
            at: [10.0, 10.0],
            to: None,
            right: false,
            ring: true,
            pointer: None,
        };
        let steps: Vec<GuideStep> = (1..=7).map(step).collect();
        let (_, path) = create_guide(
            &core,
            Lang::Es,
            GuideIn {
                title: "Pedido nuevo".into(),
                steps,
                meta: json!({"source": "steps"}),
            },
        )
        .unwrap();
        let lib = xs_library::library(&core);
        lib.scan();
        let call = |id: &str, p: Value| core.call(xs_core::Source::Cli, id, p, Some(Lang::Es)).unwrap();
        let found = call("library.search", json!({"type": "guide"}));
        let items = found["items"].as_array().unwrap();
        assert_eq!(items.len(), 1, "{found}");
        assert_eq!(items[0]["kind"], "guide");
        assert_eq!(items[0]["steps"], 7);
        // A word only in step 5.
        let by_text = call("library.search", json!({"query": "Botón 5"}));
        assert_eq!(by_text["items"].as_array().unwrap().len(), 1, "{by_text}");
        // Export from the file, without the editor (SMT-035).
        let out = library.path().join("export");
        let r = call(
            "smart.guide_export",
            json!({"path": path, "format": "md", "out": out}),
        );
        assert!(std::path::Path::new(r["paths"][0].as_str().unwrap()).is_file());

        // LIB-019: its thumbnail is its first step (as wide as tall as the step, 3:2),
        // and the quick view walks through its 7 steps.
        let id = items[0]["id"].as_str().unwrap();
        let thumb = image::load_from_memory(&lib.thumbnail(id).unwrap()).unwrap();
        let ratio = thumb.width() as f64 / thumb.height() as f64;
        assert!(
            (ratio - 1.5).abs() < 0.05,
            "thumbnail {}×{}",
            thumb.width(),
            thumb.height()
        );
        assert!(crate::quick_view::library_step(&core, id, 1).is_some());
        assert!(crate::quick_view::library_step(&core, id, 7).is_some());
        assert!(crate::quick_view::library_step(&core, id, 8).is_none());
    }

    /// SMT-026: the zone shows exactly the frame, whatever the monitor.
    #[test]
    fn a_zone_frames_its_step() {
        for (size, frame) in [
            ([1920, 1080], [256.0, 276.0, 548.0, 348.0]),
            ([3840, 2160], [0.0, 0.0, 3840.0, 2160.0]),
            ([5120, 2880], [100.0, 200.0, 248.0, 248.0]),
        ] {
            let p = templates::guide_picture([frame[2], frame[3]]);
            let zone = [60.0, 300.0, p[0], p[1]];
            let (zoom, offset) = framing(zone, size, frame);
            let z = xs_image::shapes::ZoneObject::from_props(
                json!({"rect": {"x": zone[0], "y": zone[1], "w": zone[2], "h": zone[3]}, "image": "a.png",
                       "fit": "fill", "zoom": zoom, "offset": offset})
                .as_object()
                .unwrap(),
            )
            .unwrap();
            let (x, y, k) = z.placement((size[0] as f64, size[1] as f64));
            // The frame's corners land on the zone's.
            assert!((x + frame[0] * k - zone[0]).abs() < 0.01, "{size:?}");
            assert!((y + frame[1] * k - zone[1]).abs() < 0.01, "{size:?}");
            assert!((frame[2] * k - zone[2]).abs() < 0.01, "{size:?}");
        }
    }
}
