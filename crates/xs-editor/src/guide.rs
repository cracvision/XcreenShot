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
    let thumb = xs_image::encode::thumbnail(&xs_image::render(&document, &base), THUMB_SIZE);
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
    // The steps' pictures and texts, and the title.
    let mut images = BTreeMap::new();
    let step_sections: Vec<String> = t
        .sections
        .iter()
        .filter(|s| s.kind == "guideStep")
        .map(|s| s.id.clone())
        .collect();
    for (i, (step, sid)) in g.steps.iter().zip(&step_sections).enumerate() {
        let name = format!("step-{:03}.png", i + 1);
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
    if let Some(o) = doc.objects.iter_mut().find(|o| slot_of(o) == Some("title")) {
        o.props.insert("text".into(), json!(g.title));
    }
    // Laid out again: each section as tall as its text, the numbers on the steps.
    let t = Template::of(&doc).unwrap_or(t);
    relayout(core, lang, &mut doc, &t, false);
    // Each zone frames its step; the ring, the arrow and the pointer go on it.
    for (i, (step, sid)) in g.steps.iter().zip(&step_sections).enumerate() {
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
                add("arrow", props, &mut doc);
            }
        }
        let color = if step.right { RING_RIGHT } else { RING_LEFT };
        let ring = json!({
            "rect": {"x": at[0] - RING_SIZE / 2.0, "y": at[1] - RING_SIZE / 2.0, "w": RING_SIZE, "h": RING_SIZE},
            "rotation": 0,
            "style": {"shapeType": "ellipse", "fill": null, "outline": color, "outlineWidth": 5, "opacity": 75},
        });
        if let Ok(props) = objects::checked("shape", ring.as_object().expect("object")) {
            add("shape", props, &mut doc);
        }
        if let Some((png, top_left, size)) = &step.pointer {
            let name = format!("pointer-{:03}.png", i + 1);
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
                add("image", props, &mut doc);
            }
        }
    }
    let mut meta = g.meta;
    meta["name"] = json!(g.title);
    (doc, meta, images)
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
