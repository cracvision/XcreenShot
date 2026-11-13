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

use crate::store::{DocStore, Format};
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
    // A GIF plays, but is not cut or converted again (VID-068).
    if editor(core)
        .store
        .with(&id, |d| d.format == Format::Gif)
        .unwrap_or(false)
    {
        return Err(CommandError::new(
            ErrorCode::InvalidParams,
            tr(lang, "errors.gif_no_edit", &[]),
        ));
    }
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

/// The file of an open video as it is now (VID-066): its own file without cuts at
/// once, or, with cuts not saved, a cut copy written into `dir`.
pub fn current_file(core: &Core, id: &str, dir: &Path) -> Result<PathBuf, String> {
    let o = open_video(core, Lang::En, &json!({ "documentId": id })).map_err(|e| e.message)?;
    let cut = editor(core)
        .store
        .with(id, |d| !d.document.cuts.is_empty())
        .unwrap_or(false);
    if !cut {
        return Ok(o.path);
    }
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let ext = "mp4";
    let target = xs_library::unique(dir, &o.name, ext);
    write(core, id, "save", &o.path, &target, &o.kept, o.facts.sound)?;
    Ok(target)
}

/// A small picture of a video's first frame, as PNG (for dragging it out, TRY-015).
pub fn poster_png(path: &Path, max: u32) -> Option<Vec<u8>> {
    let (w, h, mut px) = xs_video::frame_at(path, 0.0).ok()?;
    for p in px.as_chunks_mut::<4>().0 {
        p.swap(0, 2);
    }
    let image = image::RgbaImage::from_raw(w, h, px)?;
    let small = xs_image::encode::thumbnail(&image, max);
    xs_image::encode::png(&small, 96.0, true).ok()
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

// ---------------------------------------------------------------------------
// Animated GIF (VID-068 to VID-070)

/// Long jobs that can be cancelled, by document (`video.cancel`).
static JOBS: std::sync::Mutex<Option<std::collections::HashMap<String, Arc<AtomicBool>>>> =
    std::sync::Mutex::new(None);

fn job(id: &str) -> Arc<AtomicBool> {
    let flag = Arc::new(AtomicBool::new(false));
    JOBS.lock()
        .unwrap()
        .get_or_insert_with(Default::default)
        .insert(id.to_string(), flag.clone());
    flag
}

fn job_done(id: &str) {
    if let Some(map) = JOBS.lock().unwrap().as_mut() {
        map.remove(id);
    }
}

/// `video.cancel`: stops the GIF being made from a video; nothing is left behind.
pub fn cancel(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let id = resolve(core, ctx.lang, p)?;
    let flag = JOBS.lock().unwrap().as_ref().and_then(|m| m.get(&id).cloned());
    if let Some(f) = &flag {
        f.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    Ok(json!({ "cancelled": flag.is_some() }))
}

/// What a GIF will be (VID-069, VID-070).
#[derive(Debug, Clone, PartialEq)]
pub struct GifSettings {
    pub fps: f64,
    pub width: u32,
    pub height: u32,
    pub dither: bool,
    pub dynamic: bool,
    pub loops: xs_image::gif_anim::Loops,
    /// Seconds of fade to black at the end; 0: none.
    pub fade: f64,
}

/// The presets of Optimize for (VID-069): frames per second, longest side, dithering
/// and dynamic colors.
fn preset(name: &str) -> Option<(u32, Option<u32>, bool, bool)> {
    Some(match name {
        "screen" => (15, Some(1280), false, true),
        "motion" => (20, Some(800), true, true),
        "small" => (8, Some(200), false, false),
        "best" => (30, None, false, true),
        _ => return None,
    })
}

/// The settings asked for, never more frames or pixels than the video (VID-069).
pub fn gif_settings(p: &Value, video: (u32, u32, u32)) -> Result<GifSettings, String> {
    let (vw, vh, vfps) = video;
    let vfps = vfps.clamp(1, 240);
    let optimize = p.get("optimize").and_then(Value::as_str).unwrap_or("screen");
    let num = |k: &str| p.get(k).and_then(Value::as_f64);
    let flag = |k: &str, d: bool| p.get(k).and_then(Value::as_bool).unwrap_or(d);
    let fit = |longest: Option<u32>| -> (u32, u32) {
        let k = longest
            .map(|l| (l as f64 / vw.max(vh) as f64).min(1.0))
            .unwrap_or(1.0);
        (
            ((vw as f64 * k).round() as u32).max(1),
            ((vh as f64 * k).round() as u32).max(1),
        )
    };
    let (fps, (width, height), dither, dynamic) = match preset(optimize) {
        Some((fps, longest, dither, dynamic)) => (fps as f64, fit(longest), dither, dynamic),
        None if optimize == "custom" => {
            let fps = num("fps").unwrap_or(15.0).clamp(1.0, 30.0);
            let size = match (num("percent"), num("width"), num("height")) {
                (Some(pc), _, _) => {
                    let k = (pc / 100.0).clamp(0.01, 1.0);
                    ((vw as f64 * k).round() as u32, (vh as f64 * k).round() as u32)
                }
                (None, Some(w), Some(h)) => (w as u32, h as u32),
                // One side: the other keeps the proportion.
                (None, Some(w), None) => (w as u32, (w * vh as f64 / vw as f64).round() as u32),
                (None, None, Some(h)) => ((h * vw as f64 / vh as f64).round() as u32, h as u32),
                (None, None, None) => (vw, vh),
            };
            (fps, size, flag("dither", false), flag("dynamic", true))
        }
        None => return Err(format!("optimize: {optimize}")),
    };
    let loops = match p.get("loops") {
        None => xs_image::gif_anim::Loops::Forever,
        Some(v) if v == "forever" => xs_image::gif_anim::Loops::Forever,
        Some(v) if v == "once" => xs_image::gif_anim::Loops::Times(1),
        Some(v) => xs_image::gif_anim::Loops::Times(
            v.as_u64()
                .filter(|n| (1..=100).contains(n))
                .ok_or("loops: forever, once or 1 to 100")? as u16,
        ),
    };
    Ok(GifSettings {
        fps: fps.min(vfps as f64),
        width: width.clamp(16.min(vw), vw),
        height: height.clamp(16.min(vh), vh),
        dither: if optimize == "custom" {
            flag("dither", dither)
        } else {
            dither
        },
        dynamic: if optimize == "custom" {
            flag("dynamic", dynamic)
        } else {
            dynamic
        },
        loops,
        fade: num("fade").unwrap_or(0.0).clamp(0.0, 3.0),
    })
}

/// The parts of the file a GIF takes: the selection or the whole video (VID-068).
fn gif_parts(o: &Open, p: &Value) -> Vec<Span> {
    match (
        p.get("start").and_then(Value::as_f64),
        p.get("end").and_then(Value::as_f64),
    ) {
        (Some(a), Some(b)) if b > a => timeline::source_spans(&o.kept, a, b),
        _ => o.kept.clone(),
    }
}

/// Encodes `parts` of `path` as a GIF; `each` gets the progress and says whether to go
/// on. `until`: stop after that many seconds (the estimate's sample).
fn encode_gif(
    path: &Path,
    parts: &[Span],
    s: &GifSettings,
    until: Option<f64>,
    each: &mut dyn FnMut(f64) -> bool,
) -> Result<Option<Vec<u8>>, String> {
    let total = timeline::length(parts);
    let shared = if s.dynamic {
        None
    } else {
        // One palette from frames across the GIF.
        let mut samples = Vec::new();
        for k in 0..6 {
            let at = timeline::to_source(parts, total * (k as f64 + 0.5) / 6.0);
            if let Ok((w, h, mut px)) = xs_video::frame_at(path, at) {
                for p in px.as_chunks_mut::<4>().0 {
                    p.swap(0, 2);
                }
                if let Some(img) = image::RgbaImage::from_raw(w, h, px) {
                    samples.push(image::imageops::thumbnail(
                        &img,
                        s.width.min(320),
                        s.height.min(320),
                    ));
                }
            }
        }
        Some(xs_image::gif_anim::shared_palette(&samples))
    };
    let mut gif = xs_image::gif_anim::Animation::new(s.width, s.height, s.loops, shared, s.dither);
    let frames_total = (total * s.fps).ceil().max(1.0);
    let mut k = 0u64;
    let mut stopped = false;
    let mut failure: Option<String> = None;
    let fade_from = total - s.fade;
    let step = 1.0 / s.fps;
    xs_video::frames(path, parts, s.fps, &mut |t, w, h, bgra| {
        if until.is_some_and(|u| t >= u) {
            return false;
        }
        let mut px = bgra.to_vec();
        for p in px.as_chunks_mut::<4>().0 {
            p.swap(0, 2);
            p[3] = 255;
        }
        let Some(img) = image::RgbaImage::from_raw(w, h, px) else {
            return false;
        };
        let mut img = if (w, h) == (s.width, s.height) {
            img
        } else {
            image::imageops::resize(&img, s.width, s.height, image::imageops::FilterType::Triangle)
        };
        // Fade to black (VID-070): darker each frame, black on the last.
        if s.fade > 0.0 && t >= fade_from {
            let k = (1.0 - (t - fade_from) / (s.fade - step).max(step)).clamp(0.0, 1.0);
            for p in img.pixels_mut() {
                for c in 0..3 {
                    p.0[c] = (p.0[c] as f64 * k).round() as u8;
                }
            }
        }
        // Hundredths, rounded so the total stays right.
        let delay = ((k + 1) as f64 * 100.0 / s.fps).round() - (k as f64 * 100.0 / s.fps).round();
        k += 1;
        if let Err(e) = gif.push(img, delay.max(1.0) as u32) {
            failure = Some(e);
            return false;
        }
        if !each((k as f64 / frames_total).min(1.0)) {
            stopped = true;
            return false;
        }
        true
    })?;
    if let Some(e) = failure {
        return Err(e);
    }
    if stopped {
        return Ok(None);
    }
    gif.finish().map(Some)
}

/// `video.gif_estimate` (VID-068): the GIF's size, length and frames before making
/// it, from a sample of its first seconds, which is also its preview.
pub fn gif_estimate(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let o = open_video(core, lang, p)?;
    let s = gif_settings(p, (o.facts.width, o.facts.height, o.facts.fps)).map_err(|e| {
        invalid(tr(
            lang,
            "errors.invalid_param",
            &[("param", "optimize"), ("reason", &e)],
        ))
    })?;
    let parts = gif_parts(&o, p);
    let length = timeline::length(&parts);
    let sample = length.min(2.0);
    let bytes = encode_gif(&o.path, &parts, &s, Some(sample), &mut |_| true)
        .map_err(|e| failed(lang, &o.name, e))?
        .unwrap_or_default();
    let estimate = if sample > 0.0 {
        (bytes.len() as f64 * length / sample) as u64
    } else {
        bytes.len() as u64
    };
    use base64::Engine as _;
    Ok(json!({
        "width": s.width,
        "height": s.height,
        "fps": s.fps,
        "duration": (length * 10.0).round() / 10.0,
        "bytes": estimate,
        "dither": s.dither,
        "dynamic": s.dynamic,
        // The size guide (VID-068): a warning, never a block.
        "long": length > 30.0,
        "heavy": estimate > 10 * 1024 * 1024,
        "preview": format!("data:image/gif;base64,{}", base64::engine::general_purpose::STANDARD.encode(&bytes)),
    }))
}

/// `video.gif` (VID-068): the selection or the whole video as an animated GIF in the
/// library, opened in the editor; with progress, and `video.cancel` leaves no file.
pub fn gif(core: &Arc<Core>, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let o = open_video(core, lang, p)?;
    let s = gif_settings(p, (o.facts.width, o.facts.height, o.facts.fps)).map_err(|e| {
        invalid(tr(
            lang,
            "errors.invalid_param",
            &[("param", "optimize"), ("reason", &e)],
        ))
    })?;
    let parts = gif_parts(&o, p);
    let dir = library_target(core).join(chrono::Local::now().format("%Y-%m").to_string());
    let dst = xs_library::unique(&dir, &o.name, "gif");
    let flag = job(&o.id);
    let mut report = progress(core, &o.id, "gif");
    let result = encode_gif(&o.path, &parts, &s, None, &mut |done| {
        report(done.min(0.99));
        !flag.load(std::sync::atomic::Ordering::Relaxed)
    });
    job_done(&o.id);
    report(1.0);
    let bytes = match result.map_err(|e| failed(lang, &o.name, e))? {
        Some(b) => b,
        None => {
            return Err(CommandError::new(
                ErrorCode::Cancelled,
                tr(lang, "errors.gif_cancelled", &[]),
            ))
        }
    };
    std::fs::create_dir_all(&dir).map_err(|e| failed(lang, &o.name, e))?;
    crate::write_file(&dst, &bytes).map_err(|e| failed(lang, &o.name, e))?;
    xs_library::library(core).scan();
    let open = p.get("open").and_then(Value::as_str).unwrap_or("editor") == "editor";
    let id = if open {
        crate::open(core, lang, &dst, true)?
    } else {
        new_video(core, &dst).map_err(|e| failed(lang, &o.name, e))?
    };
    Ok(json!({
        "documentId": id,
        "path": dst,
        "width": s.width,
        "height": s.height,
        "fps": s.fps,
        "duration": (timeline::length(&parts) * 10.0).round() / 10.0,
        "bytes": bytes.len(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// VID-069: the presets never go beyond the video.
    #[test]
    fn gif_presets() {
        let s = gif_settings(&json!({"optimize": "small"}), (1920, 1080, 30)).unwrap();
        assert_eq!((s.width, s.height, s.fps), (200, 113, 8.0));
        let s = gif_settings(&json!({}), (1280, 720, 30)).unwrap();
        assert_eq!((s.width, s.height, s.fps, s.dynamic), (1280, 720, 15.0, true));
        let s = gif_settings(&json!({"optimize": "best"}), (640, 360, 10)).unwrap();
        assert_eq!(s.fps, 10.0);
        let s = gif_settings(&json!({"optimize": "custom", "percent": 50}), (1600, 900, 30)).unwrap();
        assert_eq!((s.width, s.height), (800, 450));
        let s = gif_settings(
            &json!({"optimize": "custom", "loops": 3, "fade": 1}),
            (100, 100, 30),
        )
        .unwrap();
        assert_eq!(s.loops, xs_image::gif_anim::Loops::Times(3));
        assert_eq!(s.fade, 1.0);
        assert!(gif_settings(&json!({"optimize": "huge"}), (100, 100, 30)).is_err());
    }

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
