//! Live drawing (VID-048 to VID-051): shapes drawn over the screen while recording,
//! which fade by themselves. This is the model, the same on every platform: the tools,
//! their keys, the numbered steps, undo and the fading. The input comes from the
//! platform's hooks (`livedraw_win`); the overlay window shows what [`Model::view`]
//! returns. Coordinates are physical pixels of the monitor drawn on, from its top left.

use std::sync::{Arc, Mutex};
use std::time::Instant;

use serde_json::{json, Value};
use xs_core::i18n::tr;
use xs_core::{CommandError, CommandResult, Context, Core, ErrorCode, Source};

use crate::Rect;

/// How long a shape takes to fade out (VID-050).
pub const FADE_OUT: f64 = 0.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Arrow,
    Rect,
    Oval,
    Pen,
    Step,
}

impl Tool {
    pub fn name(self) -> &'static str {
        match self {
            Tool::Arrow => "arrow",
            Tool::Rect => "rect",
            Tool::Oval => "oval",
            Tool::Pen => "pen",
            Tool::Step => "step",
        }
    }

    pub fn from_name(name: &str) -> Option<Tool> {
        Some(match name {
            "arrow" => Tool::Arrow,
            "rect" => Tool::Rect,
            "oval" => Tool::Oval,
            "pen" => Tool::Pen,
            "step" => Tool::Step,
            _ => return None,
        })
    }
}

/// Thin, medium or thick, as pixels at 100 % (VID-050).
pub fn width_px(name: &str) -> f64 {
    match name {
        "thin" => 3.0,
        "thick" => 10.0,
        _ => 6.0,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Shape {
    pub id: u64,
    pub tool: Tool,
    pub color: String,
    pub width: f64,
    pub points: Vec<[f64; 2]>,
    /// Rect and oval: square or circle (Shift).
    pub even: bool,
    /// Step: its number.
    pub number: Option<u32>,
    /// When it was finished (seconds of the model's clock); `None` while drawn.
    pub done: Option<f64>,
    /// When it starts to fade; `None`: not before the mode ends.
    pub fade_at: Option<f64>,
}

/// A key the hooks pass on to the model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Escape,
    Delete,
    Undo,
}

/// What a key did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyResult {
    /// Used by live drawing: kept from the apps.
    Used,
    /// `Esc`: the mode ends.
    Exit,
    /// Not one of live drawing's keys.
    Ignored,
}

#[derive(Debug, Clone)]
pub struct Model {
    pub tool: Tool,
    pub color: String,
    pub width: String,
    /// Seconds before a finished shape fades; `None`: never while the mode is on.
    pub fade: Option<f64>,
    /// The six colors of keys 2 to 7 (VID-050).
    pub palette: Vec<String>,
    /// The monitor's scale: widths and steps grow with it.
    pub scale: f64,
    shapes: Vec<Shape>,
    drawing: Option<Shape>,
    next_step: u32,
    next_id: u64,
}

/// The colors when the editor's theme gives none: red, orange, yellow, green, blue,
/// black.
pub const DEFAULT_PALETTE: [&str; 6] = ["#E53935", "#FB8C00", "#FDD835", "#43A047", "#1E88E5", "#212121"];

impl Default for Model {
    fn default() -> Self {
        Model {
            tool: Tool::Arrow,
            color: DEFAULT_PALETTE[0].to_string(),
            width: "medium".into(),
            fade: Some(3.0),
            palette: DEFAULT_PALETTE.iter().map(|s| s.to_string()).collect(),
            scale: 1.0,
            shapes: Vec::new(),
            drawing: None,
            next_step: 1,
            next_id: 1,
        }
    }
}

impl Model {
    fn new_shape(&mut self, x: f64, y: f64) -> Shape {
        let id = self.next_id;
        self.next_id += 1;
        Shape {
            id,
            tool: self.tool,
            color: self.color.clone(),
            width: width_px(&self.width) * self.scale,
            points: vec![[x, y]],
            even: false,
            number: None,
            done: None,
            fade_at: None,
        }
    }

    fn finish(&mut self, mut shape: Shape, now: f64) {
        shape.done = Some(now);
        shape.fade_at = self.fade.map(|f| now + f);
        self.shapes.push(shape);
    }

    /// The left button went down: a step is placed at once; the rest start.
    pub fn press(&mut self, x: f64, y: f64, now: f64) {
        let mut shape = self.new_shape(x, y);
        if self.tool == Tool::Step {
            shape.number = Some(self.next_step);
            self.next_step += 1;
            self.finish(shape, now);
        } else {
            self.drawing = Some(shape);
        }
    }

    /// The pointer moved with the button down. `even`: Shift is held.
    pub fn drag(&mut self, x: f64, y: f64, even: bool) {
        let Some(s) = self.drawing.as_mut() else {
            return;
        };
        s.even = even;
        match s.tool {
            // A pen keeps every point a little away from the last one.
            Tool::Pen => {
                let last = s.points.last().copied().unwrap_or([x, y]);
                if (last[0] - x).hypot(last[1] - y) >= 2.0 * self.scale.max(1.0) {
                    s.points.push([x, y]);
                }
            }
            _ => {
                s.points.truncate(1);
                s.points.push([x, y]);
            }
        }
    }

    /// The button came up: the shape is finished, unless it is a mere click.
    pub fn release(&mut self, x: f64, y: f64, even: bool, now: f64) {
        self.drag(x, y, even);
        let Some(s) = self.drawing.take() else {
            return;
        };
        let [a, b] = [s.points[0], *s.points.last().unwrap_or(&s.points[0])];
        let moved = (a[0] - b[0]).hypot(a[1] - b[1]) >= 3.0 || s.points.len() > 2;
        if moved {
            self.finish(s, now);
        }
    }

    /// A key while the mode is on (VID-049, VID-050).
    pub fn key(&mut self, key: Key, now: f64) -> KeyResult {
        match key {
            Key::Escape => KeyResult::Exit,
            Key::Undo => {
                self.shapes.pop();
                KeyResult::Used
            }
            Key::Delete => {
                self.shapes.clear();
                KeyResult::Used
            }
            Key::Char(c) => {
                match c.to_ascii_lowercase() {
                    'a' => self.tool = Tool::Arrow,
                    'r' | 's' => self.tool = Tool::Rect,
                    'o' => self.tool = Tool::Oval,
                    'p' => self.tool = Tool::Pen,
                    // `1` picks Step and counts from 1 again.
                    '1' => {
                        self.tool = Tool::Step;
                        self.next_step = 1;
                    }
                    'z' => {
                        self.shapes.pop();
                    }
                    d @ '2'..='7' => {
                        let i = d as usize - '2' as usize;
                        if let Some(c) = self.palette.get(i) {
                            self.color = c.clone();
                        }
                    }
                    _ => return KeyResult::Ignored,
                }
                let _ = now;
                KeyResult::Used
            }
        }
    }

    /// The mode ends: every shape fades now, even with Never (VID-050).
    pub fn fade_all(&mut self, now: f64) {
        self.drawing = None;
        for s in &mut self.shapes {
            s.fade_at = Some(s.fade_at.map_or(now, |f| f.min(now)));
        }
    }

    /// Forgets the shapes already gone.
    pub fn prune(&mut self, now: f64) {
        self.shapes
            .retain(|s| s.fade_at.is_none_or(|f| now < f + FADE_OUT));
    }

    /// Whether something is still to fade (the overlay needs updates).
    pub fn fading(&self, now: f64) -> bool {
        self.shapes
            .iter()
            .any(|s| s.fade_at.is_some_and(|f| now < f + FADE_OUT))
    }

    /// How opaque a shape is at `now`: 1 until it fades, then down to 0 in 0.5 s.
    pub fn opacity(shape: &Shape, now: f64) -> f64 {
        match shape.fade_at {
            Some(f) if now >= f => (1.0 - (now - f) / FADE_OUT).clamp(0.0, 1.0),
            _ => 1.0,
        }
    }

    pub fn shapes(&self) -> &[Shape] {
        &self.shapes
    }

    /// What the overlay shows, with each shape's fade as seconds from now (the overlay
    /// has a clock of its own).
    pub fn view(&self, now: f64) -> Value {
        let shape = |s: &Shape| {
            json!({
                "id": s.id,
                "tool": s.tool.name(),
                "color": s.color,
                "width": s.width,
                "points": s.points,
                "even": s.even,
                "number": s.number,
                "fadeIn": s.fade_at.map(|f| f - now),
            })
        };
        json!({
            "tool": self.tool.name(),
            "color": self.color,
            "width": self.width,
            "fade": self.fade,
            "palette": self.palette,
            "scale": self.scale,
            "fadeOut": FADE_OUT,
            "shapes": self.shapes.iter().chain(self.drawing.as_ref()).map(shape).collect::<Vec<_>>(),
        })
    }
}

// ---------------------------------------------------------------------------
// The mode: on and off, its options and the overlay (VID-048 to VID-051)

/// Live drawing on, over one monitor.
struct Active {
    model: Arc<Mutex<Model>>,
    clock: Instant,
    #[cfg(windows)]
    session: crate::livedraw_win::Session,
    /// Started without a recording (VID-051): it does not end with one.
    standalone: bool,
}

#[derive(Default)]
pub struct LiveDraw {
    active: Mutex<Option<Active>>,
}

fn service(core: &Core) -> Arc<LiveDraw> {
    core.service(LiveDraw::default)
}

pub fn is_on(core: &Core) -> bool {
    service(core).active.lock().unwrap().is_some()
}

/// The colors of the editor's active theme (VID-050), or the default ones.
fn palette(core: &Core) -> Vec<String> {
    let themes = core
        .call(Source::Ui, "editor.themes", json!({}), None)
        .unwrap_or_default();
    let active = themes["active"].as_str().unwrap_or_default();
    let colors: Vec<String> = themes["themes"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|t| t["id"] == active)
        .and_then(|t| t["colors"].as_array())
        .map(|a| {
            a.iter()
                .filter_map(|c| c.as_str().map(str::to_string))
                .take(6)
                .collect()
        })
        .unwrap_or_default();
    if colors.len() >= 6 {
        colors
    } else {
        DEFAULT_PALETTE.iter().map(|s| s.to_string()).collect()
    }
}

/// The options as the settings remember them (VID-050).
fn model_from_settings(core: &Core, scale: f64) -> Model {
    let s = &core.settings;
    let palette = palette(core);
    let color = Some(s.get_str("video.drawColor"))
        .filter(|c| c.starts_with('#'))
        .unwrap_or_else(|| palette[0].clone());
    let fade = s.get("video.drawFade").and_then(|v| v.as_f64()).unwrap_or(3.0);
    Model {
        tool: Tool::from_name(&s.get_str("video.drawTool")).unwrap_or(Tool::Arrow),
        color,
        width: Some(s.get_str("video.drawWidth"))
            .filter(|w| !w.is_empty())
            .unwrap_or_else(|| "medium".into()),
        fade: (fade > 0.0).then_some(fade),
        palette,
        scale,
        ..Default::default()
    }
}

fn store(core: &Core, ctx: &Context, key: &str, value: Value) -> Result<(), CommandError> {
    if core.settings.set(key, value.clone(), ctx.source, ctx.lang)? {
        core.setting_changed(key, &value);
    }
    Ok(())
}

fn bad(ctx: &Context, param: &str, reason: &str) -> CommandError {
    CommandError::new(
        ErrorCode::InvalidParams,
        tr(
            ctx.lang,
            "errors.invalid_param",
            &[("param", param), ("reason", reason)],
        ),
    )
}

/// Options in `p` (`tool`, `color`, `width`, `fade`): remembered, and taken by the
/// mode at once when it is on.
fn apply_options(core: &Core, ctx: &Context, p: &Value) -> Result<bool, CommandError> {
    let mut any = false;
    if let Some(t) = p.get("tool").and_then(Value::as_str) {
        if Tool::from_name(t).is_none() {
            return Err(bad(ctx, "tool", "arrow, rect, oval, pen or step"));
        }
        store(core, ctx, "video.drawTool", json!(t))?;
        any = true;
    }
    for (param, key) in [
        ("color", "video.drawColor"),
        ("width", "video.drawWidth"),
        ("fade", "video.drawFade"),
    ] {
        if let Some(v) = p.get(param) {
            store(core, ctx, key, v.clone())?;
            any = true;
        }
    }
    if any {
        if let Some(a) = service(core).active.lock().unwrap().as_ref() {
            let scale = a.model.lock().unwrap().scale;
            let fresh = model_from_settings(core, scale);
            {
                let mut m = a.model.lock().unwrap();
                // A step picked again counts from 1 (VID-049).
                m.tool = fresh.tool;
                m.color = fresh.color;
                m.width = fresh.width;
                m.fade = fresh.fade;
            }
            #[cfg(windows)]
            a.session.changed();
        }
    }
    Ok(any)
}

/// The monitor live drawing covers: the recording's (or the area waiting to be
/// recorded), else the one under the pointer when it may run without a recording.
fn monitor_for(core: &Core) -> Option<(Rect, f64, bool)> {
    if let Some((r, s)) = crate::record::drawing_monitor(core) {
        return Some((r, s, false));
    }
    crate::record::pointer_monitor().map(|(r, s)| (r, s, true))
}

/// `record.draw` (VID-048, VID-051): live drawing on or off (`on`; without it and
/// without options, the other way), and its options (`tool`, `color`, `width`,
/// `fade`), which are remembered.
pub fn command(core: &Arc<Core>, ctx: &Context, p: &Value) -> CommandResult {
    // Only what it shows now (its windows, when they open).
    if p.get("query").and_then(Value::as_bool) == Some(true) {
        return Ok(state(core));
    }
    let mut options = apply_options(core, ctx, p)?;
    // Undo and Clear all from the toolbar (VID-049).
    for (param, key) in [("undo", Key::Undo), ("clear", Key::Delete)] {
        if p.get(param).and_then(Value::as_bool) == Some(true) {
            options = true;
            if let Some(a) = service(core).active.lock().unwrap().as_ref() {
                let now = a.clock.elapsed().as_secs_f64();
                a.model.lock().unwrap().key(key, now);
                #[cfg(windows)]
                a.session.changed();
            }
        }
    }
    let on = match p.get("on").and_then(Value::as_bool) {
        Some(on) => on,
        None if options => is_on(core),
        None => !is_on(core),
    };
    if on && !is_on(core) {
        start(core, ctx)?;
    } else if !on && is_on(core) {
        stop(core);
    }
    Ok(state(core))
}

fn state(core: &Core) -> Value {
    let svc = service(core);
    let active = svc.active.lock().unwrap();
    match active.as_ref() {
        Some(a) => {
            let now = a.clock.elapsed().as_secs_f64();
            let mut v = a.model.lock().unwrap().view(now);
            v["on"] = json!(true);
            v
        }
        None => json!({ "on": false }),
    }
}

fn start(core: &Arc<Core>, ctx: &Context) -> Result<(), CommandError> {
    let recording = crate::record::drawing_monitor(core).is_some();
    // VID-051: the Video tab's switch, and Live drawing at any time.
    if recording && !core.settings.get_bool("video.liveDraw") {
        return Err(CommandError::new(
            ErrorCode::FeatureDisabled,
            tr(ctx.lang, "errors.livedraw_off", &[]),
        ));
    }
    if !recording && !core.settings.get_bool("video.liveDrawAnytime") {
        return Err(CommandError::new(
            ErrorCode::FeatureDisabled,
            tr(ctx.lang, "errors.livedraw_not_recording", &[]),
        ));
    }
    let Some((monitor, scale, standalone)) = monitor_for(core) else {
        return Err(CommandError::internal("no monitor"));
    };
    let model = Arc::new(Mutex::new(model_from_settings(core, scale)));
    let clock = Instant::now();
    core.host()
        .live_draw(Some((monitor.x, monitor.y, monitor.width, monitor.height)));
    #[cfg(windows)]
    let session = {
        let push_core = Arc::downgrade(core);
        let exit_core = Arc::downgrade(core);
        crate::livedraw_win::Session::start(
            model.clone(),
            (monitor.x, monitor.y),
            (monitor.width, monitor.height),
            clock,
            Box::new(move |view| {
                if let Some(c) = push_core.upgrade() {
                    let mut v = view;
                    v["on"] = json!(true);
                    c.host().live_draw_state(v);
                }
            }),
            Box::new(move || {
                if let Some(c) = exit_core.upgrade() {
                    // Off the hook's worker: stopping waits for it.
                    std::thread::spawn(move || stop(&c));
                }
            }),
        )
        .map_err(CommandError::internal)?
    };
    *service(core).active.lock().unwrap() = Some(Active {
        model,
        clock,
        #[cfg(windows)]
        session,
        standalone,
    });
    core.host().live_draw_state(state(core));
    tell(core, true);
    Ok(())
}

/// Live drawing off: the mouse and keyboard go back to the apps at once; the shapes
/// fade in 0.5 s, then its windows close (VID-050).
pub fn stop(core: &Core) {
    let Some(a) = service(core).active.lock().unwrap().take() else {
        return;
    };
    let now = a.clock.elapsed().as_secs_f64();
    let view = {
        let mut m = a.model.lock().unwrap();
        m.fade_all(now);
        let mut v = m.view(now);
        v["on"] = json!(false);
        v
    };
    core.host().live_draw_state(view);
    #[cfg(windows)]
    a.session.stop();
    tell(core, false);
    let host = core.host();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(650));
        host.live_draw(None);
    });
}

/// A recording ended: live drawing started with it ends too.
pub fn recording_ended(core: &Core) {
    let standalone = service(core)
        .active
        .lock()
        .unwrap()
        .as_ref()
        .map(|a| a.standalone);
    if standalone == Some(false) {
        stop(core);
    }
}

fn tell(core: &Core, on: bool) {
    core.events
        .publish("record.draw", Source::Ui, json!({ "on": on }));
}

/// `record.draw_shape` (VID-049): a shape drawn by automation, as if with the mouse:
/// `tool` and `points` (physical pixels of the monitor drawn on).
pub fn draw_shape(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let svc = service(core);
    let active = svc.active.lock().unwrap();
    let a = active.as_ref().ok_or_else(|| {
        CommandError::new(ErrorCode::NotFound, tr(ctx.lang, "errors.livedraw_inactive", &[]))
    })?;
    let points: Vec<[f64; 2]> = p
        .get("points")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|pt| Some([pt.get(0)?.as_f64()?, pt.get(1)?.as_f64()?]))
        .collect();
    let Some(first) = points.first().copied() else {
        return Err(bad(ctx, "points", "[[x, y], ...]"));
    };
    let now = a.clock.elapsed().as_secs_f64();
    {
        let mut m = a.model.lock().unwrap();
        if let Some(t) = p.get("tool").and_then(Value::as_str) {
            m.tool = Tool::from_name(t).ok_or_else(|| bad(ctx, "tool", "arrow, rect, oval, pen or step"))?;
        }
        if let Some(c) = p.get("color").and_then(Value::as_str) {
            m.color = c.to_string();
        }
        m.press(first[0], first[1], now);
        for pt in points.iter().skip(1) {
            m.drag(pt[0], pt[1], false);
        }
        let last = *points.last().unwrap_or(&first);
        m.release(last[0], last[1], false, now);
    }
    #[cfg(windows)]
    a.session.changed();
    drop(active);
    Ok(state(core))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arrow(m: &mut Model, now: f64) {
        m.press(10.0, 10.0, now);
        m.drag(50.0, 40.0, false);
        m.release(100.0, 80.0, false, now);
    }

    /// VID-049: `1` and three clicks make 1, 2, 3; `1` again starts at 1.
    #[test]
    fn steps_count_and_start_again() {
        let mut m = Model::default();
        assert_eq!(m.key(Key::Char('1'), 0.0), KeyResult::Used);
        for i in 0..3 {
            m.press(10.0 * i as f64, 0.0, 0.0);
        }
        let numbers: Vec<_> = m.shapes().iter().map(|s| s.number).collect();
        assert_eq!(numbers, vec![Some(1), Some(2), Some(3)]);
        m.key(Key::Char('1'), 0.0);
        m.press(0.0, 0.0, 0.0);
        assert_eq!(m.shapes().last().unwrap().number, Some(1));
    }

    /// VID-049: each tool by its key; a click without a drag draws nothing.
    #[test]
    fn tools_by_key() {
        let mut m = Model::default();
        for (k, t) in [
            ('a', Tool::Arrow),
            ('r', Tool::Rect),
            ('s', Tool::Rect),
            ('o', Tool::Oval),
            ('p', Tool::Pen),
        ] {
            m.key(Key::Char(k), 0.0);
            assert_eq!(m.tool, t);
        }
        assert_eq!(m.key(Key::Char('q'), 0.0), KeyResult::Ignored);
        m.key(Key::Char('a'), 0.0);
        m.press(5.0, 5.0, 0.0);
        m.release(6.0, 5.0, false, 0.0);
        assert!(m.shapes().is_empty());
        arrow(&mut m, 0.0);
        assert_eq!(m.shapes()[0].points, vec![[10.0, 10.0], [100.0, 80.0]]);
    }

    /// VID-049: undo takes only the last shape; Delete takes them all.
    #[test]
    fn undo_and_delete() {
        let mut m = Model::default();
        arrow(&mut m, 0.0);
        arrow(&mut m, 0.0);
        m.key(Key::Undo, 0.0);
        assert_eq!(m.shapes().len(), 1);
        arrow(&mut m, 0.0);
        m.key(Key::Delete, 0.0);
        assert!(m.shapes().is_empty());
        assert_eq!(m.key(Key::Escape, 0.0), KeyResult::Exit);
    }

    /// VID-050: with 5 s a shape starts to fade at 5 s and is gone at 5.5 s; with
    /// Never it stays until the mode ends.
    #[test]
    fn shapes_fade() {
        let mut m = Model {
            fade: Some(5.0),
            ..Default::default()
        };
        arrow(&mut m, 10.0);
        let s = m.shapes()[0].clone();
        assert_eq!(Model::opacity(&s, 14.9), 1.0);
        assert!((Model::opacity(&s, 15.25) - 0.5).abs() < 1e-9);
        assert_eq!(Model::opacity(&s, 15.5), 0.0);
        m.prune(15.6);
        assert!(m.shapes().is_empty());

        let mut m = Model {
            fade: None,
            ..Default::default()
        };
        arrow(&mut m, 0.0);
        m.prune(1000.0);
        assert_eq!(m.shapes().len(), 1);
        m.fade_all(1000.0);
        assert!(m.fading(1000.2));
        m.prune(1000.6);
        assert!(m.shapes().is_empty());
    }

    /// VID-050: `3` picks the palette's third color for the next shape.
    #[test]
    fn colors_by_key() {
        let mut m = Model::default();
        m.key(Key::Char('4'), 0.0);
        arrow(&mut m, 0.0);
        assert_eq!(m.shapes()[0].color, DEFAULT_PALETTE[2]);
    }

    #[test]
    fn a_pen_keeps_its_path() {
        let mut m = Model::default();
        m.key(Key::Char('p'), 0.0);
        m.press(0.0, 0.0, 0.0);
        for i in 1..=10 {
            m.drag(i as f64 * 5.0, (i % 2) as f64 * 5.0, false);
        }
        m.release(55.0, 0.0, false, 0.0);
        assert!(m.shapes()[0].points.len() >= 10);
        let v = m.view(0.0);
        assert_eq!(v["shapes"][0]["tool"], "pen");
    }
}
