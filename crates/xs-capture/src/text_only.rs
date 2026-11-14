//! "Only the text" (SMT-021): a highlight, a blur or a spotlight whose style asks for
//! the text only covers just the lines of text inside its area. After every change to
//! a document, the objects whose bands are stale (new, moved, resized, or just switched
//! to the text only) get them again, in the background: the text is read from the
//! document without the objects that tint or cover it, and the bands go into the
//! objects without an undo step of their own (undoing the change takes them back too).

use std::sync::Arc;

use serde_json::Value;
use xs_core::{CommandResult, Context, Core, Lang};
use xs_image::shapes::rect;
use xs_image::text_bands;

use crate::redact;

/// The objects that can cover only the text.
pub const KINDS: [&str; 3] = ["highlight", "blur", "spotlight"];

/// Objects left out when reading the text below: they tint, hide or darken it.
const COVERING: [&str; 4] = ["highlight", "blur", "spotlight", "simplify"];

/// Room around the text of a line, in pixels: a highlighter's band is a little taller
/// than the letters; a spotlight leaves more around them.
fn pad(kind: &str) -> f64 {
    match kind {
        "spotlight" => 4.0,
        _ => 2.0,
    }
}

fn stale_in(core: &Core, id: &str) -> bool {
    xs_editor::editor(core)
        .store
        .with(id, |d| {
            d.document
                .objects
                .iter()
                .any(|o| KINDS.contains(&o.kind.as_str()) && text_bands::stale(&o.props))
        })
        .unwrap_or(false)
}

/// Finds the stale bands of a document again. Returns how many objects changed.
pub fn refresh(core: &Core, lang: Lang, id: &str) -> Result<usize, String> {
    let ed = xs_editor::editor(core);
    let mut snap = ed.store.snapshot(id)?;
    snap.document
        .objects
        .retain(|o| !COVERING.contains(&o.kind.as_str()));
    let (image, origin) = snap.render_with_origin();
    let languages = crate::extract::wanted(core, None);
    let reading = redact::read(&image, &languages, lang).map_err(|e| e.message)?;
    let (ox, oy) = (origin[0] as f64, origin[1] as f64);
    let lines: Vec<Vec<[f64; 4]>> = reading
        .text
        .iter()
        .flat_map(|r| r.blocks.iter().flat_map(|b| &b.lines))
        .map(|l| {
            l.words
                .iter()
                .map(|w| [w.rect[0] - ox, w.rect[1] - oy, w.rect[2], w.rect[3]])
                .collect()
        })
        .collect();
    let changed = ed
        .store
        .amend(id, |doc| {
            let mut n = 0;
            for o in doc
                .objects
                .iter_mut()
                .filter(|o| KINDS.contains(&o.kind.as_str()) && text_bands::stale(&o.props))
            {
                let Ok(r) = rect(o.props.get("rect")) else {
                    continue;
                };
                let bands = text_bands::find(r, &lines, pad(&o.kind));
                text_bands::set(&mut o.props, &bands);
                n += 1;
            }
            n
        })
        .ok_or("document closed")?;
    if changed > 0 {
        xs_editor::document_changed(core, id);
        tracing::info!(objects = changed, "text bands found");
    }
    Ok(changed)
}

/// After every change that may move such an object: the documents with stale bands get
/// them again, in the background.
pub fn register(reg: &mut xs_core::command::Registry, weak: &std::sync::Weak<Core>) {
    let w = weak.clone();
    reg.add_observer(Box::new(
        move |ctx: &Context, spec: &xs_core::command::CommandSpec, result: &CommandResult| {
            let relevant = spec.mutates || spec.id.starts_with("document.") || spec.id.starts_with("editor.");
            if result.is_err() || !relevant {
                return;
            }
            let Some(core) = w.upgrade() else {
                return;
            };
            // Switched off (SMT-002): the bands found stay as they are.
            if !core.settings.get_bool("smart.textOnly.enabled") {
                return;
            }
            let docs: Vec<String> = xs_editor::editor(&core)
                .store
                .ids()
                .into_iter()
                .filter(|id| stale_in(&core, id))
                .collect();
            if docs.is_empty() {
                return;
            }
            let lang = ctx.lang;
            let core: Arc<Core> = core;
            let _ = std::thread::Builder::new()
                .name("text-bands".into())
                .spawn(move || {
                    for id in docs {
                        if let Err(e) = refresh(&core, lang, &id) {
                            tracing::warn!(error = %e, "text bands not found");
                        }
                    }
                });
        },
    ));
}

/// The bands of an object as the editor shows them, for tests and tools.
pub fn bands_of(core: &Core, id: &str, object: &str) -> Option<Value> {
    xs_editor::editor(core).store.with(id, |d| {
        d.document
            .object(object)
            .and_then(|o| o.props.get("bands").cloned())
    })?
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use xs_core::Source;

    /// SMT-021: a highlight with only the text over a paragraph of three lines of
    /// different length gets three bands, each as wide as its line; moved over another
    /// paragraph, its bands fit the new text; with no text, it covers its area.
    #[test]
    fn bands_follow_the_text() {
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
        let lines = [
            "Una linea bastante larga de texto",
            "Corta",
            "Otra linea mediana",
            "",
            "",
            "Segundo parrafo de prueba aqui",
        ];
        let (id, _) =
            xs_editor::add_capture(&core, &redact::tests::form(&lines), json!({}), 1.0, None).unwrap();
        let call = |cmd: &str, p: Value| core.call(Source::Cli, cmd, p, Some(Lang::Es)).unwrap();
        let r = call(
            "document.annotate",
            json!({"documentId": id, "objects": [{"type": "highlight", "rect": {"x": 10, "y": 10, "w": 700, "h": 180}, "style": {"textOnly": true}}]}),
        );
        let oid = r["ids"][0].as_str().unwrap().to_string();
        refresh(&core, Lang::Es, &id).unwrap();
        let bands = bands_of(&core, &id, &oid).unwrap();
        let bands = bands.as_array().unwrap();
        assert_eq!(bands.len(), 3, "{bands:?}");
        let width = |i: usize| bands[i][2].as_f64().unwrap();
        assert!(width(0) > width(2) && width(2) > width(1), "{bands:?}");

        // Over the second paragraph: one band, of its line.
        call(
            "document.update_object",
            json!({"documentId": id, "objectIds": [oid], "set": {"rect": {"x": 10, "y": 320, "w": 700, "h": 50}}}),
        );
        refresh(&core, Lang::Es, &id).unwrap();
        assert_eq!(bands_of(&core, &id, &oid).unwrap().as_array().unwrap().len(), 1);

        // Where there is no text: no bands, the whole area.
        call(
            "document.update_object",
            json!({"documentId": id, "objectIds": [oid], "set": {"rect": {"x": 10, "y": 200, "w": 700, "h": 50}}}),
        );
        refresh(&core, Lang::Es, &id).unwrap();
        assert_eq!(bands_of(&core, &id, &oid).unwrap(), json!([]));
        // One undo puts the object back where it was, with its bands.
        call("document.undo", json!({"documentId": id}));
        assert_eq!(bands_of(&core, &id, &oid).unwrap().as_array().unwrap().len(), 1);
    }
}
