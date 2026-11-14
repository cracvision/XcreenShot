//! Narrated slides (VID-071 to VID-074): images of the editor shown one after another
//! while the user talks, recorded from these pictures and not from the screen. This is
//! the mode's state (which slides, which one shows, the video's size and background,
//! each slide's zoom) and its commands; the recording itself is `record.start` with
//! `source: "slides"`.

use std::sync::{Arc, Mutex};

use serde_json::{json, Value};
use xs_core::i18n::tr;
use xs_core::{CommandError, CommandResult, Context, Core, ErrorCode};
use xs_video::slides::{custom_size, Slide, SlideFeed, View};

struct Item {
    document: String,
    name: String,
    width: u32,
    height: u32,
    /// Its final image, as the engine draws it (premultiplied BGRA).
    bgra: Arc<Vec<u8>>,
    /// Its own number, for the engine: a new one is a cut.
    id: u64,
    view: View,
}

struct Mode {
    items: Vec<Item>,
    index: usize,
    size: (u32, u32),
    background: [u8; 3],
    enlarge: bool,
}

#[derive(Default)]
pub struct SlidesService {
    mode: Mutex<Option<Mode>>,
    next_id: std::sync::atomic::AtomicU64,
    /// Where the editor shows the video's frame, in screen pixels (for the camera's
    /// preview over it).
    frame: Mutex<Option<(i32, i32, i32, i32)>>,
    /// The annotations added while narrating, in order: (document, object) (VID-073).
    /// They stay listed after the mode closes, until they are kept or removed.
    narration: Mutex<Vec<(String, String)>>,
}

fn service(core: &Core) -> Arc<SlidesService> {
    core.service(SlidesService::default)
}

fn bad(lang: xs_core::Lang, key: &str) -> CommandError {
    CommandError::new(ErrorCode::InvalidParams, tr(lang, key, &[]))
}

/// The slides mode is open.
pub fn active(core: &Core) -> bool {
    service(core).mode.lock().unwrap().is_some()
}

/// The video's size, while the mode is open.
pub fn size(core: &Core) -> Option<(u32, u32)> {
    service(core).mode.lock().unwrap().as_ref().map(|m| m.size)
}

/// Where the editor shows the video's frame on the screen, and the video's width: the
/// camera's preview goes over it at that scale (VID-072).
pub fn screen_frame(core: &Core) -> Option<(i32, i32, i32, i32, u32)> {
    let svc = service(core);
    let video = svc.mode.lock().unwrap().as_ref().map(|m| m.size.0)?;
    let f = (*svc.frame.lock().unwrap())?;
    (f.2 > 1 && f.3 > 1).then_some((f.0, f.1, f.2, f.3, video))
}

/// The camera's preview follows the mode (VID-072): while it waits, with the state
/// a recording would have; while recording, the recording's bar does it.
fn camera(core: &Core) {
    if crate::record::running(core) {
        crate::record::refresh(core);
        return;
    }
    match size(core) {
        Some((w, h)) => crate::rec_camera::sync(
            core,
            Some(&json!({
                "status": "ready",
                "source": { "type": "slides" },
                "area": { "x": 0, "y": 0, "width": w, "height": h },
            })),
        ),
        None => crate::rec_camera::sync(core, None),
    }
}

/// `slides.frame` (VID-072): where the editor shows the video's frame now, in screen
/// pixels; the camera's preview goes there.
pub fn frame(core: &Core, p: &Value) -> CommandResult {
    let n = |k: &str| p.get(k).and_then(Value::as_f64).unwrap_or(0.0).round() as i32;
    let f = (n("x"), n("y"), n("width"), n("height"));
    let svc = service(core);
    let changed = *svc.frame.lock().unwrap() != Some(f);
    if changed {
        *svc.frame.lock().unwrap() = Some(f);
        camera(core);
    }
    Ok(json!({ "frame": [f.0, f.1, f.2, f.3] }))
}

/// Where a recording gets the slide showing now.
pub fn feed(core: &Core) -> Option<SlideFeed> {
    let svc = service(core);
    if svc.mode.lock().unwrap().is_none() {
        return None;
    }
    let s = svc.clone();
    Some(Arc::new(move || {
        let mode = s.mode.lock().unwrap();
        let m = mode.as_ref()?;
        let item = m.items.get(m.index)?;
        Some(Slide {
            id: item.id,
            width: item.width,
            height: item.height,
            bgra: item.bgra.clone(),
            background: m.background,
            enlarge: m.enlarge,
            view: item.view,
        })
    }))
}

/// A document's final image as a slide, if it is one: an image, not a video or a
/// guide of steps (VID-071).
fn item(core: &Core, svc: &SlidesService, document: &str) -> Option<Item> {
    let name = xs_editor::slide_name(core, document)?;
    let img = xs_editor::final_image(core, document).ok()?;
    let (width, height) = img.dimensions();
    let mut bgra = img.into_raw();
    for px in bgra.as_chunks_mut::<4>().0 {
        let a = px[3] as u32;
        let (r, g, b) = (px[0] as u32, px[1] as u32, px[2] as u32);
        px[0] = (b * a / 255) as u8;
        px[1] = (g * a / 255) as u8;
        px[2] = (r * a / 255) as u8;
    }
    Some(Item {
        document: document.to_string(),
        name,
        width,
        height,
        bgra: Arc::new(bgra),
        id: svc.next_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1,
        view: View::default(),
    })
}

fn hex([r, g, b]: [u8; 3]) -> String {
    format!("#{r:02X}{g:02X}{b:02X}")
}

/// The mode as the editor shows it.
pub fn state(core: &Core) -> Value {
    let svc = service(core);
    let mode = svc.mode.lock().unwrap();
    let Some(m) = mode.as_ref() else {
        return json!({ "active": false });
    };
    json!({
        "active": true,
        "index": m.index,
        "count": m.items.len(),
        "size": [m.size.0, m.size.1],
        "background": hex(m.background),
        "enlarge": m.enlarge,
        // Where the editor shows the video on the screen (for the camera's preview).
        "frame": *svc.frame.lock().unwrap(),
        "annotations": svc.narration.lock().unwrap().len(),
        "slides": m.items.iter().map(|i| json!({
            "documentId": i.document,
            "name": i.name,
            "width": i.width,
            "height": i.height,
            "version": i.id,
            "zoom": i.view.zoom,
            "center": [i.view.center.0, i.view.center.1],
        })).collect::<Vec<_>>(),
    })
}

/// Tells the editor what changed, and gives it back.
fn changed(core: &Core) -> CommandResult {
    let s = state(core);
    core.host().editor_event("slides", s.clone());
    camera(core);
    Ok(s)
}

fn documents(p: &Value) -> Vec<String> {
    p.get("documentIds")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
        .unwrap_or_default()
}

/// `slides.start` (VID-071): the mode with the images given, in their order (by
/// default the one in the editor); videos and guides are left out.
pub fn start(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    if crate::record::running(core) {
        return Err(CommandError::new(
            ErrorCode::Busy,
            tr(lang, "errors.record_busy", &[]),
        ));
    }
    let mut ids = documents(p);
    if ids.is_empty() {
        ids = xs_editor::active_document(core).into_iter().collect();
    }
    let svc = service(core);
    let items: Vec<Item> = ids.iter().filter_map(|d| item(core, &svc, d)).collect();
    if items.is_empty() {
        return Err(bad(lang, "errors.slides_none"));
    }
    let s = &core.settings;
    let size = parse_size(&s.get_str("video.slidesSize")).unwrap_or((1920, 1080));
    let background =
        xs_video::effects::parse_color(&s.get_str("video.slidesBackground")).unwrap_or([0, 0, 0]);
    *svc.mode.lock().unwrap() = Some(Mode {
        items,
        index: 0,
        size,
        background,
        enlarge: s.get_bool("video.slidesEnlarge"),
    });
    changed(core)
}

/// "1920x1080" as a size, even and held (VID-074).
fn parse_size(s: &str) -> Option<(u32, u32)> {
    let (w, h) = s.split_once(['x', '×'])?;
    Some(custom_size(w.trim().parse().ok()?, h.trim().parse().ok()?))
}

/// Runs `f` on the open mode, or says it is not open.
fn with_mode<R>(core: &Core, lang: xs_core::Lang, f: impl FnOnce(&mut Mode) -> R) -> Result<R, CommandError> {
    let svc = service(core);
    let mut mode = svc.mode.lock().unwrap();
    let m = mode
        .as_mut()
        .ok_or_else(|| CommandError::new(ErrorCode::NotFound, tr(lang, "errors.slides_off", &[])))?;
    Ok(f(m))
}

/// `slides.go` (VID-072): to a slide by number (from 0), or `next`, `prev`, `first`
/// or `last`. The last one's Next does nothing.
pub fn go(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let to = p.get("to").cloned().unwrap_or(json!("next"));
    with_mode(core, ctx.lang, |m| {
        let last = m.items.len().saturating_sub(1);
        m.index = match (to.as_u64(), to.as_str()) {
            (Some(n), _) => (n as usize).min(last),
            (_, Some("prev")) => m.index.saturating_sub(1),
            (_, Some("first")) => 0,
            (_, Some("last")) => last,
            _ => (m.index + 1).min(last),
        };
    })?;
    changed(core)
}

/// `slides.move` (VID-071): a slide to another place in the order.
pub fn reorder(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let n = |k: &str| p.get(k).and_then(Value::as_u64).map(|v| v as usize);
    let (Some(from), Some(to)) = (n("from"), n("to")) else {
        return Err(bad(ctx.lang, "errors.slides_move"));
    };
    with_mode(core, ctx.lang, |m| {
        if from < m.items.len() {
            let shown = m.items.get(m.index).map(|i| i.id);
            let it = m.items.remove(from);
            m.items.insert(to.min(m.items.len()), it);
            // The slide showing stays the one showing.
            m.index = m.items.iter().position(|i| Some(i.id) == shown).unwrap_or(0);
        }
    })?;
    changed(core)
}

/// `slides.add` (VID-071): images at the end; `slides.remove`: one out (never the
/// last one left).
pub fn add(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let svc = service(core);
    let new: Vec<Item> = documents(p).iter().filter_map(|d| item(core, &svc, d)).collect();
    with_mode(core, ctx.lang, |m| m.items.extend(new))?;
    changed(core)
}

pub fn remove(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let index = p.get("index").and_then(Value::as_u64).map(|v| v as usize);
    with_mode(core, ctx.lang, |m| {
        if let Some(i) = index.filter(|i| *i < m.items.len() && m.items.len() > 1) {
            m.items.remove(i);
            m.index = m.index.min(m.items.len() - 1);
        }
    })?;
    changed(core)
}

/// `slides.set` (VID-074): the video's size (not while recording), its background and
/// whether small images are enlarged. They stay as the settings.
pub fn set(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let size = match p.get("size") {
        Some(v) => {
            if crate::record::running(core) {
                return Err(CommandError::new(
                    ErrorCode::Busy,
                    tr(lang, "errors.slides_size_recording", &[]),
                ));
            }
            Some(
                v.as_str()
                    .and_then(parse_size)
                    .or_else(|| {
                        let a = v.as_array()?;
                        Some(custom_size(
                            a.first()?.as_u64()? as u32,
                            a.get(1)?.as_u64()? as u32,
                        ))
                    })
                    .ok_or_else(|| bad(lang, "errors.slides_size"))?,
            )
        }
        None => None,
    };
    let background = match p.get("background").and_then(Value::as_str) {
        Some(c) => Some(xs_video::effects::parse_color(c).ok_or_else(|| bad(lang, "errors.slides_color"))?),
        None => None,
    };
    let enlarge = p.get("enlarge").and_then(Value::as_bool);
    with_mode(core, lang, |m| {
        if let Some(s) = size {
            m.size = s;
        }
        if let Some(b) = background {
            m.background = b;
        }
        if let Some(e) = enlarge {
            m.enlarge = e;
        }
    })?;
    let put = |k: &str, v: Value| {
        if core.settings.set(k, v.clone(), ctx.source, lang).unwrap_or(false) {
            core.setting_changed(k, &v);
        }
    };
    if let Some((w, h)) = size {
        put("video.slidesSize", json!(format!("{w}x{h}")));
    }
    if let Some(b) = background {
        put("video.slidesBackground", json!(hex(b)));
    }
    if let Some(e) = enlarge {
        put("video.slidesEnlarge", json!(e));
    }
    changed(core)
}

/// `slides.view` (VID-074): the slide showing, zoomed (1 to 8) and moved so that
/// `center` (0 to 1 across and down the image) is in the middle; `reset` goes back to
/// how it started. In the video it glides there.
pub fn view(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    with_mode(core, ctx.lang, |m| {
        let Some(item) = m.items.get_mut(m.index) else {
            return;
        };
        if p.get("reset") == Some(&json!(true)) {
            item.view = View::default();
            return;
        }
        if let Some(z) = p.get("zoom").and_then(Value::as_f64) {
            item.view.zoom = (z as f32).clamp(1.0, 8.0);
        }
        if let Some(c) = p.get("center").and_then(Value::as_array) {
            let v = |i: usize| {
                c.get(i)
                    .and_then(Value::as_f64)
                    .map(|v| (v as f32).clamp(0.0, 1.0))
            };
            if let (Some(x), Some(y)) = (v(0), v(1)) {
                item.view.center = (x, y);
            }
        }
    })?;
    changed(core)
}

/// `slides.exit`: the mode closes; the images stay as they were (VID-071). Not while
/// recording.
pub fn exit(core: &Core, ctx: &Context) -> CommandResult {
    if crate::record::running(core) && active(core) {
        return Err(CommandError::new(
            ErrorCode::Busy,
            tr(ctx.lang, "errors.slides_recording", &[]),
        ));
    }
    close(core);
    Ok(json!({ "active": false }))
}

/// Closes the mode (and tells the editor): after Exit, or when its video is saved.
pub fn close(core: &Core) {
    let svc = service(core);
    if svc.mode.lock().unwrap().take().is_some() {
        svc.frame.lock().unwrap().take();
        // Annotations added while narrating: the editor asks whether to keep them.
        let annotations = svc.narration.lock().unwrap().len();
        core.host()
            .editor_event("slides", json!({ "active": false, "annotations": annotations }));
        // The camera goes with the mode, unless a recording still holds it.
        if !crate::record::running(core) {
            crate::rec_camera::sync(core, None);
        }
    }
}

/// The slide of `document` drawn again (its annotations changed): a cut in the video.
fn redraw(core: &Core, svc: &SlidesService, document: &str) {
    let Some(fresh) = item(core, svc, document) else {
        return;
    };
    let mut mode = svc.mode.lock().unwrap();
    if let Some(m) = mode.as_mut() {
        for it in m.items.iter_mut().filter(|i| i.document == document) {
            it.bgra = fresh.bgra.clone();
            it.width = fresh.width;
            it.height = fresh.height;
            it.id = svc.next_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
        }
    }
}

/// `slides.annotate` (VID-073): annotations drawn on the slide showing, in its image's
/// pixels, as `document.annotate` takes them. They show in the video at once and stay
/// in the image, listed as added while narrating.
pub fn annotate(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let svc = service(core);
    let document = with_mode(core, ctx.lang, |m| {
        m.items.get(m.index).map(|i| i.document.clone())
    })?
    .ok_or_else(|| bad(ctx.lang, "errors.slides_off"))?;
    let objects = p.get("objects").cloned().unwrap_or(json!([]));
    let added = core.call(
        ctx.source,
        "document.annotate",
        json!({ "documentId": document, "objects": objects }),
        Some(ctx.lang),
    )?;
    let ids: Vec<String> = added["ids"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
        .unwrap_or_default();
    svc.narration
        .lock()
        .unwrap()
        .extend(ids.iter().map(|id| (document.clone(), id.clone())));
    redraw(core, &svc, &document);
    changed(core)?;
    Ok(json!({ "ids": ids }))
}

/// `slides.undo` (VID-073): the last annotation added while narrating goes, also in
/// the video.
pub fn undo(core: &Core, ctx: &Context) -> CommandResult {
    let svc = service(core);
    let Some((document, id)) = svc.narration.lock().unwrap().pop() else {
        return state_now(core);
    };
    core.call(
        ctx.source,
        "document.delete_object",
        json!({ "documentId": document, "objectId": id }),
        Some(ctx.lang),
    )?;
    redraw(core, &svc, &document);
    changed(core)
}

fn state_now(core: &Core) -> CommandResult {
    Ok(state(core))
}

/// `slides.annotations` (VID-073): after narrating, the annotations added are kept, or
/// removed from all the images at once (`keep: false`).
pub fn annotations(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let keep = p.get("keep").and_then(Value::as_bool).unwrap_or(true);
    let list = std::mem::take(&mut *service(core).narration.lock().unwrap());
    let count = list.len();
    if !keep {
        let mut by_doc: std::collections::BTreeMap<String, Vec<String>> = Default::default();
        for (d, id) in list {
            by_doc.entry(d).or_default().push(id);
        }
        for (d, ids) in by_doc {
            // A document closed meanwhile keeps them: nothing to do there.
            let _ = core.call(
                ctx.source,
                "document.delete_object",
                json!({ "documentId": d, "objectIds": ids }),
                Some(ctx.lang),
            );
        }
    }
    Ok(json!({ "kept": keep, "count": count }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_are_read_even_and_held() {
        assert_eq!(parse_size("1920x1080"), Some((1920, 1080)));
        assert_eq!(parse_size("1081 × 1921"), Some((1080, 1920)));
        assert_eq!(parse_size("big"), None);
        assert_eq!(hex([255, 8, 0]), "#FF0800");
    }
}

#[cfg(test)]
mod mode_tests {
    use serde_json::json;
    use xs_core::{Core, Lang, Source};

    /// VID-071, VID-072, VID-074: two new images become slides in their order; they are
    /// gone through, reordered, sized, zoomed and left; the recording's feed gives the
    /// one showing, with its background.
    #[test]
    fn slides_are_made_gone_through_and_left() {
        let core = Core::for_tests_with(
            xs_core::platform::Platform::Windows,
            vec![crate::extension(), xs_editor::extension()],
        );
        let call = |id: &str, p: serde_json::Value| core.call(Source::Cli, id, p, Some(Lang::En));
        let new = |w: u32, color: &str| {
            call("document.new", json!({"width": w, "height": 300, "color": color})).unwrap()["documentId"]
                .as_str()
                .unwrap()
                .to_string()
        };
        let (red, blue) = (new(400, "#FF0000"), new(600, "#0000FF"));
        assert!(call("slides.go", json!({})).is_err(), "not open yet");
        let s = call("slides.start", json!({"documentIds": [red, blue]})).unwrap();
        assert_eq!((s["count"].clone(), s["index"].clone()), (json!(2), json!(0)));
        assert_eq!(s["slides"][1]["width"], 600);
        let feed = super::feed(&core).unwrap();
        let first = feed().unwrap();
        assert_eq!((first.width, first.height), (400, 300));
        // Red, as the engine wants it: BGRA.
        assert_eq!(&first.bgra[..4], &[0, 0, 255, 255]);
        // Next, past the last: it stays on the last.
        call("slides.go", json!({"to": "next"})).unwrap();
        let s = call("slides.go", json!({"to": "next"})).unwrap();
        assert_eq!(s["index"], 1);
        assert_eq!(feed().unwrap().width, 600);
        // Reordered: the blue one first, still the one showing.
        let s = call("slides.move", json!({"from": 1, "to": 0})).unwrap();
        assert_eq!(
            (s["slides"][0]["documentId"].clone(), s["index"].clone()),
            (json!(blue), json!(0))
        );
        // Size, background and zoom.
        let s = call("slides.set", json!({"size": "1281x721", "background": "#FFFFFF"})).unwrap();
        assert_eq!(s["size"], json!([1280, 720]));
        assert_eq!(feed().unwrap().background, [255, 255, 255]);
        assert_eq!(core.settings.get_str("video.slidesSize"), "1280x720");
        call("slides.view", json!({"zoom": 2, "center": [0.25, 0.75]})).unwrap();
        assert_eq!(feed().unwrap().view.zoom, 2.0);
        call("slides.view", json!({"reset": true})).unwrap();
        assert_eq!(feed().unwrap().view.zoom, 1.0);
        // Annotated while narrating (VID-073): on the slide showing, in the video at
        // once; undone; and after leaving, removed from all the images.
        let before = feed().unwrap().id;
        let r = call(
            "slides.annotate",
            json!({"objects": [{"type": "shape", "rect": {"x": 10, "y": 10, "w": 100, "h": 50}}]}),
        )
        .unwrap();
        assert_eq!(r["ids"].as_array().unwrap().len(), 1);
        assert_ne!(feed().unwrap().id, before, "a new picture for the video");
        let objects = |d: &str| {
            call("document.info", json!({"documentId": d})).unwrap()["objects"]
                .as_array()
                .map_or(0, |a| a.len())
        };
        assert_eq!(objects(&blue), 1);
        call("slides.undo", json!({})).unwrap();
        assert_eq!(objects(&blue), 0);
        call(
            "slides.annotate",
            json!({"objects": [{"type": "arrow", "from": [0, 0], "to": [50, 50]}]}),
        )
        .unwrap();
        // Left: closed, the images as they were.
        call("slides.exit", json!({})).unwrap();
        assert_eq!(
            call("slides.annotations", json!({"keep": false})).unwrap()["count"],
            1
        );
        assert_eq!(objects(&blue), 0);
        assert_eq!(call("slides.state", json!({})).unwrap()["active"], false);
        assert!(super::feed(&core).is_none());
    }
}
