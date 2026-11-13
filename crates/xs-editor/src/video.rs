//! Videos in the editor (ADR 0008): the timeline after the cuts and the `video.*`
//! commands. Cutting is an undo step on the document (VID-059); saving, a part as a
//! new video (VID-060) and the silent copy (VID-063) write new files with
//! `xs_video::export`; a frame becomes an image (VID-061) or its text (VID-062).

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use serde_json::{json, Value};
use xs_core::i18n::tr;
use xs_core::{CommandError, CommandResult, Context, Core, ErrorCode, Lang};
use xs_video::timeline::{self, Span};

use crate::store::DocStore;
use crate::{activate, editor, emit_document, emit_list, invalid, library_target, resolve};

/// Commands that only make sense for an image: with a video they say why not
/// (VID-065).
pub const IMAGE_ONLY: &[&str] = &[
    "document.annotate",
    "document.simplify_view",
    "document.steps",
    "document.update_object",
    "document.transform",
    "document.flatten",
    "document.unflatten",
    "document.fill",
    "document.erase",
    "document.select_pixels",
    "document.selection_action",
    "document.resize",
    "document.rotate",
    "document.flip",
    "document.trim",
    "document.cut_out",
    "document.fit_canvas",
    "document.crop_to_canvas",
    "document.canvas",
    "document.canvas_size",
    "document.effects",
    "document.arrange",
    "document.group",
    "document.duplicate",
    "document.delete_object",
    "document.crop",
    "document.copy_objects",
    "document.template",
    "document.save_as_template",
    "document.apply_theme",
    "document.print_pages",
    "document.print",
    "document.paste",
    "document.insert_images",
];

/// With a video, an image-only command fails saying what to do instead (VID-065).
pub fn refuse_for_video(core: &Core, lang: Lang, p: &Value) -> Result<(), CommandError> {
    let Ok(id) = resolve(core, lang, p) else {
        return Ok(());
    };
    if editor(core).store.with(&id, |d| d.is_video()).unwrap_or(false) {
        return Err(CommandError::new(
            ErrorCode::InvalidParams,
            tr(lang, "errors.video_no_image_tools", &[]),
        ));
    }
    Ok(())
}

fn spans(cuts: &[[f64; 2]]) -> Vec<Span> {
    cuts.iter().map(|c| Span::new(c[0], c[1])).collect()
}

/// A video's facts and its timeline after the cuts, for `document.info`.
pub fn timeline_json(f: &xs_video::Facts, cuts: &[[f64; 2]]) -> Value {
    let cut = spans(cuts);
    let kept = timeline::kept(f.duration, &cut);
    json!({
        "width": f.width,
        "height": f.height,
        "duration": f.duration,
        "fps": f.fps,
        "codec": f.codec,
        "sound": f.sound,
        "length": timeline::length(&kept),
        "kept": kept.iter().map(|k| [k.start, k.end]).collect::<Vec<_>>(),
        "marks": timeline::marks(f.duration, &cut),
    })
}

/// The open video a command acts on: its id, file, facts and the parts kept.
struct Open {
    id: String,
    path: PathBuf,
    name: String,
    facts: xs_video::Facts,
    kept: Vec<Span>,
}

fn open_video(core: &Core, lang: Lang, p: &Value) -> Result<Open, CommandError> {
    let id = resolve(core, lang, p)?;
    editor(core)
        .store
        .with(&id, |d| {
            let facts = d.video.clone()?;
            let kept = timeline::kept(facts.duration, &spans(&d.document.cuts));
            Some(Open {
                id: id.clone(),
                path: d.path.clone()?,
                name: d.name(),
                facts,
                kept,
            })
        })
        .flatten()
        .ok_or_else(|| CommandError::new(ErrorCode::InvalidParams, tr(lang, "errors.not_a_video", &[])))
}

fn seconds(lang: Lang, p: &Value, key: &str) -> Result<f64, CommandError> {
    p.get(key)
        .and_then(Value::as_f64)
        .filter(|v| v.is_finite() && *v >= 0.0)
        .ok_or_else(|| {
            invalid(tr(
                lang,
                "errors.invalid_param",
                &[("param", key), ("reason", "seconds >= 0")],
            ))
        })
}

fn failed(lang: Lang, name: &str, reason: impl std::fmt::Display) -> CommandError {
    CommandError::new(
        ErrorCode::Internal,
        tr(
            lang,
            "errors.save_failed",
            &[("name", name), ("reason", &reason.to_string())],
        ),
    )
}

/// Reports how far a long job is, for the editor's progress bar.
fn progress<'a>(core: &'a Core, id: &str, task: &'static str) -> impl FnMut(f64) + 'a {
    let id = id.to_string();
    let mut last = -1.0;
    move |p: f64| {
        if p - last >= 0.02 || p >= 1.0 {
            last = p;
            core.host().editor_event(
                "video_progress",
                json!({"documentId": id, "task": task, "progress": p}),
            );
        }
    }
}

/// Writes `keep` of `src` to `dst` through a partial file, so a failure leaves nothing.
fn write(
    core: &Core,
    id: &str,
    task: &'static str,
    src: &Path,
    dst: &Path,
    keep: &[Span],
    sound: bool,
) -> Result<(), String> {
    if let Some(dir) = dst.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let partial = xs_core::document::partial_path(dst);
    let mut report = progress(core, id, task);
    xs_video::export(src, &partial, keep, sound, &mut report, &AtomicBool::new(false))?;
    std::fs::rename(&partial, dst).map_err(|e| {
        let _ = std::fs::remove_file(&partial);
        e.to_string()
    })
}

/// Times as "m.ss" for file names (no ":" in them on Windows).
fn stamp(t: f64) -> String {
    let s = t.round() as u64;
    format!("{}.{:02}", s / 60, s % 60)
}

/// A new video next to the original, in the library: open in the tray, not shown
/// (VID-060, VID-063).
fn new_video(core: &Arc<Core>, path: &Path) -> Result<String, String> {
    let ed = editor(core);
    let doc = DocStore::load(path, &ed.library()).map_err(|e| format!("{e:?}"))?;
    let (id, _) = ed.store.insert(doc, true);
    emit_list(core);
    Ok(id)
}

/// Where a new video made from `o` goes: the library's folder of this month.
fn new_path(core: &Core, o: &Open, suffix: &str) -> PathBuf {
    let dir = library_target(core).join(chrono::Local::now().format("%Y-%m").to_string());
    xs_library::unique(&dir, &format!("{} ({suffix})", o.name), "mp4")
}

/// `video.cut` (VID-059): cuts `start..end` of the video as it is now (its cut
/// timeline), in one undo step.
pub fn cut(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let o = open_video(core, lang, p)?;
    let (start, end) = (seconds(lang, p, "start")?, seconds(lang, p, "end")?);
    if end <= start {
        return Err(invalid(tr(
            lang,
            "errors.invalid_param",
            &[("param", "end"), ("reason", "after start")],
        )));
    }
    let duration = o.facts.duration;
    let frame = 1.0 / o.facts.fps.max(1) as f64;
    let result = editor(core)
        .store
        .mutate(&o.id, "history.cut_video", |doc| {
            let cuts = timeline::cut(duration, &spans(&doc.cuts), start, end, frame).ok_or_else(|| {
                CommandError::new(ErrorCode::InvalidParams, tr(lang, "errors.video_cut_all", &[]))
            })?;
            doc.cuts = cuts.iter().map(|c| [c.start, c.end]).collect();
            Ok::<_, CommandError>(())
        })
        .ok_or_else(|| CommandError::internal("document closed"))?;
    result?;
    emit_document(core, &o.id);
    emit_list(core);
    let info = editor(core)
        .store
        .with(&o.id, |d| {
            d.video.as_ref().map(|f| timeline_json(f, &d.document.cuts))
        })
        .flatten()
        .unwrap_or_default();
    Ok(info)
}

/// Saves a video's cuts in its own file (VID-059) or writes it, cut, to `target`
/// (Save as, VID-066). Saving its own file forgets the undo steps before it.
pub fn save(core: &Arc<Core>, ctx: &Context, id: &str, target: Option<&Path>) -> CommandResult {
    let lang = ctx.lang;
    let o = open_video(core, lang, &json!({ "documentId": id }))?;
    let sound = o.facts.sound;
    match target {
        Some(dst) => {
            write(core, id, "save", &o.path, dst, &o.kept, sound).map_err(|e| failed(lang, &o.name, e))?;
            Ok(json!({ "status": "saved", "path": dst, "linked": false }))
        }
        None => {
            let ed = editor(core);
            let (dirty, version) = ed
                .store
                .with(id, |d| (d.dirty(), d.version))
                .unwrap_or((false, 0));
            if !dirty {
                return Ok(json!({ "status": "saved", "path": o.path, "linked": true }));
            }
            write(core, id, "save", &o.path.clone(), &o.path, &o.kept, sound)
                .map_err(|e| failed(lang, &o.name, e))?;
            let fresh = DocStore::load(&o.path, &ed.library())
                .map_err(|e| failed(lang, &o.name, format!("{e:?}")))?;
            ed.store.replace_video(id, fresh, version);
            emit_document(core, id);
            emit_list(core);
            Ok(json!({ "status": "saved", "path": o.path, "linked": true }))
        }
    }
}

/// `video.save_selection` (VID-060): `start..end` of the video as it is now, as a new
/// video in the library and the tray; the original does not change.
pub fn save_selection(core: &Arc<Core>, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let o = open_video(core, lang, p)?;
    let (start, end) = (seconds(lang, p, "start")?, seconds(lang, p, "end")?);
    let part = timeline::source_spans(&o.kept, start, end);
    if part.is_empty() {
        return Err(invalid(tr(
            lang,
            "errors.invalid_param",
            &[("param", "end"), ("reason", "after start")],
        )));
    }
    let dst = new_path(core, &o, &format!("{}-{}", stamp(start), stamp(end)));
    write(core, &o.id, "selection", &o.path, &dst, &part, o.facts.sound)
        .map_err(|e| failed(lang, &o.name, e))?;
    let id = new_video(core, &dst).map_err(|e| failed(lang, &o.name, e))?;
    Ok(json!({ "documentId": id, "path": dst, "duration": timeline::length(&part) }))
}

/// `video.silent_copy` (VID-063): the video as it is now, without any sound, as a new
/// video in the library and the tray.
pub fn silent_copy(core: &Arc<Core>, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let o = open_video(core, lang, p)?;
    if !o.facts.sound {
        return Err(CommandError::new(
            ErrorCode::InvalidParams,
            tr(lang, "errors.video_no_sound", &[]),
        ));
    }
    let dst = new_path(core, &o, &tr(lang, "ui.video.silent_suffix", &[]));
    write(core, &o.id, "silent", &o.path, &dst, &o.kept, false).map_err(|e| failed(lang, &o.name, e))?;
    let id = new_video(core, &dst).map_err(|e| failed(lang, &o.name, e))?;
    Ok(json!({ "documentId": id, "path": dst }))
}

/// The frame at `time` of the video as it is now, as RGBA.
fn frame(o: &Open, time: f64) -> Result<image::RgbaImage, String> {
    let at = timeline::to_source(&o.kept, time);
    let (w, h, mut px) = xs_video::frame_at(&o.path, at)?;
    for p in px.as_chunks_mut::<4>().0 {
        p.swap(0, 2);
    }
    image::RgbaImage::from_raw(w, h, px).ok_or_else(|| "frame".to_string())
}

/// `video.save_frame` (VID-061): the frame at `time` (of the video as it is now) as
/// a new image in the library, with the video's details, shown in the editor.
pub fn save_frame(core: &Arc<Core>, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let o = open_video(core, lang, p)?;
    let time = seconds(lang, p, "time")?;
    let image = frame(&o, time).map_err(|e| failed(lang, &o.name, e))?;
    // The video's capture details (app, address) and where the frame comes from.
    let source = editor(core)
        .store
        .with(&o.id, |d| d.meta.clone())
        .unwrap_or_default();
    let mut meta = json!({
        "source": "video",
        "video": { "path": o.path, "time": time },
    });
    for key in ["app", "url", "title"] {
        if let Some(v) = source.get(key) {
            meta[key] = v.clone();
        }
    }
    let app = source.get("app").and_then(Value::as_str).map(str::to_string);
    let (id, path) =
        crate::add_image(core, &image, meta, 1.0, app.as_deref()).map_err(|e| failed(lang, &o.name, e))?;
    editor(core).store.show_in_tray(&id);
    activate(core, &id);
    let _ = core.host().show_main_window();
    Ok(json!({ "documentId": id, "path": path, "width": image.width(), "height": image.height() }))
}

/// `video.extract_text` (VID-062): Extract text (SMT-005) of the frame at `time`,
/// without adding an image to the library.
pub fn extract_text(core: &Arc<Core>, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let o = open_video(core, lang, p)?;
    let time = seconds(lang, p, "time")?;
    let image = frame(&o, time).map_err(|e| failed(lang, &o.name, e))?;
    // The frame goes through a file of the app's own temporary folder, gone at once.
    let dir = core.paths.local_dir.join("tmp");
    std::fs::create_dir_all(&dir).map_err(|e| failed(lang, &o.name, e))?;
    let file = dir.join(format!("frame-{}.png", std::process::id()));
    image.save(&file).map_err(|e| failed(lang, &o.name, e))?;
    let mut params = json!({ "source": "file", "path": file });
    if let Some(copy) = p.get("copy") {
        params["copy"] = copy.clone();
    }
    if let Some(show) = p.get("show") {
        params["show"] = show.clone();
    }
    let result = core.call(ctx.source, "smart.extract_text", params, Some(lang));
    let _ = std::fs::remove_file(&file);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stamps_for_file_names() {
        assert_eq!(stamp(0.0), "0.00");
        assert_eq!(stamp(75.4), "1.15");
        assert_eq!(stamp(3600.0), "60.00");
    }

    #[test]
    fn the_timeline_for_the_editor() {
        let f = xs_video::Facts {
            width: 640,
            height: 360,
            duration: 60.0,
            fps: 30,
            codec: "H.264".into(),
            sound: true,
        };
        let t = timeline_json(&f, &[[10.0, 20.0]]);
        assert_eq!(t["length"], 50.0);
        assert_eq!(t["kept"], json!([[0.0, 10.0], [20.0, 60.0]]));
        assert_eq!(t["marks"], json!([10.0]));
    }
}
