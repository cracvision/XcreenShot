//! Objects that come from commands: validation, the tool's current style (EDT-014)
//! and the name of the undo step (EDT-092).
//!
//! Every object is stored in a canonical form: its geometry and its whole style, so a
//! `.xshot` looks the same on any machine whatever the tools' styles there.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use xs_core::document::{translate_props, Object};
use xs_image::callouts::{CalloutObject, CalloutStyle};
use xs_image::geometry::{Arrow, ArrowStyle, Cap};
use xs_image::highlights::{HighlightObject, HighlightStyle};
use xs_image::magnify::{MagnifyObject, MagnifyStyle};
use xs_image::pens::{self, PenObject, PenStyle};
use xs_image::shapes::{BlurObject, BlurStyle, ImageObject, ShapeObject, ShapeStyle};
use xs_image::spotlights::{SpotlightObject, SpotlightStyle, SHARED};
use xs_image::steps::{self, StepObject, StepStyle};
use xs_image::text::{Text, TextStyle};

/// Current style of each tool: what the next object will look like (EDT-014).
/// Kept between sessions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Styles {
    pub arrow: ArrowStyle,
    /// A line: an arrow without ends (EDT-025).
    pub line: ArrowStyle,
    pub text: TextStyle,
    pub shape: ShapeStyle,
    pub blur: BlurStyle,
    pub step: StepStyle,
    pub callout: CalloutStyle,
    pub highlight: HighlightStyle,
    pub pen: PenStyle,
    pub magnify: MagnifyStyle,
    pub spotlight: SpotlightStyle,
    pub stamp: StampStyle,
    pub simplify: xs_image::simplify::SimplifyToolStyle,
    pub fill: FillStyle,
    pub eraser: EraserStyle,
    pub select: SelectStyle,
    pub wand: WandStyle,
    pub cutout: CutOutStyle,
}

/// Cut out (EDT-052).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct CutOutStyle {
    /// "horizontal" (a strip of the whole width) or "vertical".
    pub direction: String,
    /// "remove" or "add".
    pub mode: String,
    /// "straight", "torn", "wavy" or "zigzag".
    pub join: String,
    /// The gap of the other joins, 8–60 px.
    pub gap: f64,
}

impl Default for CutOutStyle {
    fn default() -> Self {
        CutOutStyle {
            direction: "horizontal".to_string(),
            mode: "remove".to_string(),
            join: "straight".to_string(),
            gap: 16.0,
        }
    }
}

impl CutOutStyle {
    fn merged(base: &CutOutStyle, changes: &Map<String, Value>) -> Result<CutOutStyle, String> {
        let s: CutOutStyle = merged(base, changes)?;
        if !["horizontal", "vertical"].contains(&s.direction.as_str()) {
            return Err("direction is horizontal or vertical".into());
        }
        if !["remove", "add"].contains(&s.mode.as_str()) {
            return Err("mode is remove or add".into());
        }
        if !["straight", "torn", "wavy", "zigzag"].contains(&s.join.as_str()) {
            return Err("join is straight, torn, wavy or zigzag".into());
        }
        within("gap", s.gap, 8.0, 60.0)?;
        Ok(s)
    }
}

/// The Selection tool (EDT-043, EDT-045).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct SelectStyle {
    /// "rectangle", "ellipse", "polygon" or "freehand".
    pub shape: String,
    pub antialias: bool,
    /// 0–50 px.
    pub feather: f64,
    /// Snap to object, and how far it groups (0–100).
    pub snap: bool,
    pub level: f64,
    /// "smart", "transparent" or "canvas".
    pub gap_fill: String,
    /// Smart fill: "fast" or "photo".
    pub quality: String,
}

impl Default for SelectStyle {
    fn default() -> Self {
        SelectStyle {
            shape: "rectangle".to_string(),
            antialias: true,
            feather: 0.0,
            snap: false,
            level: 50.0,
            gap_fill: "smart".to_string(),
            quality: "fast".to_string(),
        }
    }
}

/// The Magic wand (EDT-046, EDT-045).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct WandStyle {
    pub tolerance: f64,
    pub global: bool,
    pub gap_fill: String,
    pub quality: String,
}

impl Default for WandStyle {
    fn default() -> Self {
        WandStyle {
            tolerance: 15.0,
            global: false,
            gap_fill: "smart".to_string(),
            quality: "fast".to_string(),
        }
    }
}

fn gap_fill_ok(v: &str, quality: &str) -> Result<(), String> {
    if !["smart", "transparent", "canvas"].contains(&v) {
        return Err("gapFill is smart, transparent or canvas".into());
    }
    if !["fast", "photo"].contains(&quality) {
        return Err("quality is fast or photo".into());
    }
    Ok(())
}

impl SelectStyle {
    fn merged(base: &SelectStyle, changes: &Map<String, Value>) -> Result<SelectStyle, String> {
        let s: SelectStyle = merged(base, changes)?;
        if !["rectangle", "ellipse", "polygon", "freehand"].contains(&s.shape.as_str()) {
            return Err("shape is rectangle, ellipse, polygon or freehand".into());
        }
        within("feather", s.feather, 0.0, 50.0)?;
        within("level", s.level, 0.0, 100.0)?;
        gap_fill_ok(&s.gap_fill, &s.quality)?;
        Ok(s)
    }
}

impl WandStyle {
    fn merged(base: &WandStyle, changes: &Map<String, Value>) -> Result<WandStyle, String> {
        let s: WandStyle = merged(base, changes)?;
        within("tolerance", s.tolerance, 0.0, 100.0)?;
        gap_fill_ok(&s.gap_fill, &s.quality)?;
        Ok(s)
    }
}

/// The Fill tool's style (EDT-047). `color` `None` is Transparent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct FillStyle {
    /// A color or a gradient (EDT-020); `None` is Transparent.
    pub color: Option<xs_image::paint::Fill>,
    /// 0–100 %: how different a color may be and still fill.
    pub tolerance: f64,
    pub opacity: f64,
    /// Every pixel of that color, touching or not.
    pub global: bool,
}

impl Default for FillStyle {
    fn default() -> Self {
        FillStyle {
            color: Some("#F2545B".into()),
            tolerance: 10.0,
            opacity: 100.0,
            global: false,
        }
    }
}

/// The Eraser's style (EDT-048).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct EraserStyle {
    /// 1–200 px.
    pub width: f64,
    /// "round" or "square".
    pub shape: String,
    pub opacity: f64,
    /// Smooth edges.
    pub antialias: bool,
}

impl Default for EraserStyle {
    fn default() -> Self {
        EraserStyle {
            width: 20.0,
            shape: "round".to_string(),
            opacity: 100.0,
            antialias: true,
        }
    }
}

/// `base` with `changes`, checked as a `T`.
fn merged<T: Serialize + serde::de::DeserializeOwned>(
    base: &T,
    changes: &Map<String, Value>,
) -> Result<T, String> {
    let mut v = serde_json::to_value(base).map_err(|e| e.to_string())?;
    if let Some(m) = v.as_object_mut() {
        for (k, val) in changes {
            m.insert(k.clone(), val.clone());
        }
    }
    serde_json::from_value(v).map_err(|e| e.to_string())
}

fn within(name: &str, v: f64, min: f64, max: f64) -> Result<(), String> {
    if (min..=max).contains(&v) {
        Ok(())
    } else {
        Err(format!("{name} must be between {min} and {max}"))
    }
}

impl FillStyle {
    fn merged(base: &FillStyle, changes: &Map<String, Value>) -> Result<FillStyle, String> {
        let s: FillStyle = merged(base, changes)?;
        if let Some(c) = &s.color {
            c.check("color")?;
        }
        within("tolerance", s.tolerance, 0.0, 100.0)?;
        within("opacity", s.opacity, 0.0, 100.0)?;
        Ok(s)
    }
}

impl EraserStyle {
    fn merged(base: &EraserStyle, changes: &Map<String, Value>) -> Result<EraserStyle, String> {
        let s: EraserStyle = merged(base, changes)?;
        if s.shape != "round" && s.shape != "square" {
            return Err("shape is round or square".into());
        }
        within("width", s.width, 1.0, 200.0)?;
        within("opacity", s.opacity, 0.0, 100.0)?;
        Ok(s)
    }
}

/// The Stamp tool's style (EDT-031): which stamp it places, and its color (`None`: the
/// stamp's own) and opacity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct StampStyle {
    pub stamp: String,
    pub color: Option<String>,
    pub opacity: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shadow: Option<xs_image::shadows::Shadow>,
}

impl Default for StampStyle {
    fn default() -> Self {
        StampStyle {
            stamp: "check".to_string(),
            color: None,
            opacity: 100.0,
            shadow: None,
        }
    }
}

impl StampStyle {
    fn merged(base: &StampStyle, changes: &Map<String, Value>) -> Result<StampStyle, String> {
        let mut v = serde_json::to_value(base).map_err(|e| e.to_string())?;
        if let Some(m) = v.as_object_mut() {
            for (k, val) in changes {
                m.insert(k.clone(), val.clone());
            }
        }
        let s: StampStyle = serde_json::from_value(v).map_err(|e| e.to_string())?;
        if xs_image::stamps::find(&s.stamp).is_none() {
            return Err(format!("there is no stamp \"{}\"", s.stamp));
        }
        Ok(s)
    }
}

impl Default for Styles {
    fn default() -> Self {
        Styles {
            arrow: ArrowStyle::default(),
            line: line_style(),
            text: TextStyle::default(),
            shape: ShapeStyle::default(),
            blur: BlurStyle::default(),
            step: StepStyle::default(),
            callout: CalloutStyle::default(),
            highlight: HighlightStyle::default(),
            pen: PenStyle::default(),
            magnify: MagnifyStyle::default(),
            spotlight: SpotlightStyle::default(),
            stamp: StampStyle::default(),
            simplify: xs_image::simplify::SimplifyToolStyle::default(),
            fill: FillStyle::default(),
            eraser: EraserStyle::default(),
            select: SelectStyle::default(),
            wand: WandStyle::default(),
            cutout: CutOutStyle::default(),
        }
    }
}

/// The Line tool's own default: no ends, thinner (EDT-025).
pub fn line_style() -> ArrowStyle {
    ArrowStyle {
        width: 4.0,
        start_cap: Cap::None,
        end_cap: Cap::None,
        ..ArrowStyle::default()
    }
}

impl Styles {
    /// The current style of the tool that draws `kind`.
    pub fn of(&self, kind: &str) -> Option<Value> {
        match kind {
            "arrow" => serde_json::to_value(&self.arrow).ok(),
            "line" => serde_json::to_value(&self.line).ok(),
            "text" => serde_json::to_value(&self.text).ok(),
            "shape" => serde_json::to_value(&self.shape).ok(),
            "blur" => serde_json::to_value(&self.blur).ok(),
            "step" => serde_json::to_value(&self.step).ok(),
            "callout" => serde_json::to_value(&self.callout).ok(),
            "highlight" => serde_json::to_value(&self.highlight).ok(),
            "pen" => serde_json::to_value(&self.pen).ok(),
            "magnify" => serde_json::to_value(&self.magnify).ok(),
            "spotlight" => serde_json::to_value(&self.spotlight).ok(),
            "stamp" => serde_json::to_value(&self.stamp).ok(),
            "simplify" => serde_json::to_value(&self.simplify).ok(),
            "fill" => serde_json::to_value(&self.fill).ok(),
            "eraser" => serde_json::to_value(&self.eraser).ok(),
            "select" => serde_json::to_value(&self.select).ok(),
            "wand" => serde_json::to_value(&self.wand).ok(),
            "cutout" => serde_json::to_value(&self.cutout).ok(),
            _ => None,
        }
    }

    /// Changes the current style of the tool that draws `kind`.
    pub fn set(&mut self, kind: &str, changes: &Map<String, Value>) -> Result<Value, String> {
        let mut next = self.clone();
        next.set_unchecked(kind, changes)?;
        let style = next.of(kind).ok_or_else(|| "style".to_string())?;
        check_shadow(Some(&style))?;
        *self = next;
        Ok(style)
    }

    fn set_unchecked(&mut self, kind: &str, changes: &Map<String, Value>) -> Result<(), String> {
        match kind {
            "arrow" => self.arrow = ArrowStyle::merged(&self.arrow, changes)?,
            "line" => self.line = ArrowStyle::merged(&self.line, changes)?,
            "text" => self.text = TextStyle::merged(&self.text, changes)?,
            "shape" => self.shape = ShapeStyle::merged(&self.shape, changes)?,
            "blur" => self.blur = BlurStyle::merged(&self.blur, changes)?,
            "step" => self.step = StepStyle::merged(&self.step, changes)?,
            "callout" => self.callout = CalloutStyle::merged(&self.callout, changes)?,
            "highlight" => self.highlight = HighlightStyle::merged(&self.highlight, changes)?,
            "pen" => self.pen = PenStyle::merged(&self.pen, changes)?,
            "magnify" => self.magnify = MagnifyStyle::merged(&self.magnify, changes)?,
            "spotlight" => self.spotlight = SpotlightStyle::merged(&self.spotlight, changes)?,
            "stamp" => self.stamp = StampStyle::merged(&self.stamp, changes)?,
            "simplify" => {
                self.simplify = xs_image::simplify::SimplifyToolStyle::merged(&self.simplify, changes)?
            }
            "fill" => self.fill = FillStyle::merged(&self.fill, changes)?,
            "eraser" => self.eraser = EraserStyle::merged(&self.eraser, changes)?,
            "select" => self.select = SelectStyle::merged(&self.select, changes)?,
            "wand" => self.wand = WandStyle::merged(&self.wand, changes)?,
            "cutout" => self.cutout = CutOutStyle::merged(&self.cutout, changes)?,
            k => return Err(format!("the {k} tool has no style")),
        }
        Ok(())
    }
}

/// Object types `document.annotate` knows (EDT-119) that arrive in later builds.
const LATER_TYPES: &[&str] = &[];

/// Geometry properties of each type (everything but `style`).
fn geometry_keys(kind: &str) -> Option<&'static [&'static str]> {
    match kind {
        "arrow" | "line" => Some(&["from", "to"]),
        "text" => Some(&["at", "width", "text", "rotation"]),
        "shape" => Some(&["rect", "rotation", "points"]),
        "blur" | "highlight" | "spotlight" => Some(&["rect", "rotation"]),
        // An image can carry a name: a window's app (CAP-029).
        "image" => Some(&["rect", "rotation", "name"]),
        "step" => Some(&["rect", "rotation", "value"]),
        "pen" => Some(&["points"]),
        "stamp" => Some(&["rect", "rotation", "stamp", "art", "ink"]),
        "simplify" => Some(&["rect", "rotation"]),
        // A QR code (SMT-036): its content too.
        "qr" => Some(&["rect", "rotation", "content"]),
        "magnify" => Some(&["source", "at"]),
        "callout" => Some(&["rect", "rotation", "text", "tails"]),
        // A template's image zone (EDT-098): its frame, picture and framing.
        "zone" => Some(&["rect", "image", "fit", "zoom", "offset"]),
        _ => None,
    }
}

fn only_keys(input: &Map<String, Value>, allowed: &[&str]) -> Result<(), String> {
    match input
        .keys()
        .find(|k| k.as_str() != "style" && !allowed.contains(&k.as_str()))
    {
        Some(k) => Err(format!("unknown property \"{k}\"")),
        None => Ok(()),
    }
}

fn style_changes(v: Option<&Value>) -> Result<Map<String, Value>, String> {
    match v {
        None => Ok(Map::new()),
        Some(Value::Object(m)) => Ok(m.clone()),
        // A saved style by name (EDT-015) arrives in M2.
        Some(Value::String(_)) => Err("saved styles by name arrive in M2; pass the properties".to_string()),
        Some(_) => Err("style must be an object".to_string()),
    }
}

fn merge_style(current: Option<&Value>, changes: &Map<String, Value>) -> Value {
    let mut style = current.and_then(Value::as_object).cloned().unwrap_or_default();
    for (k, v) in changes {
        style.insert(k.clone(), v.clone());
    }
    Value::Object(style)
}

/// Validates `props` for `kind` and returns them in canonical form.
fn canonical(kind: &str, props: &Map<String, Value>) -> Result<Map<String, Value>, String> {
    let value = match kind {
        "arrow" | "line" => {
            let a = Arrow::from_props(props)?;
            if a.length() < 1.0 {
                return Err("from and to must be at least 1 pixel apart".to_string());
            }
            json!({"from": a.from, "to": a.to, "style": a.style})
        }
        "text" => {
            let t = Text::from_props(props)?;
            if t.text.trim().is_empty() {
                return Err("text must not be empty".to_string());
            }
            json!({"at": t.at, "width": t.width, "text": t.text, "rotation": t.rotation, "style": t.style})
        }
        "shape" => {
            let s = ShapeObject::from_props(props)?;
            let mut v = json!({"rect": s.rect, "rotation": s.rotation, "style": s.style});
            if !s.points.is_empty() {
                v["points"] = json!(s.points);
            }
            v
        }
        "blur" => {
            let b = BlurObject::from_props(props)?;
            json!({"rect": b.rect, "rotation": b.rotation, "style": b.style})
        }
        "step" => {
            // Square, of the size of its style (EDT-035).
            let s = StepObject::from_props(props)?;
            let r = steps::square(s.rect, s.style.size);
            let mut style = serde_json::to_value(&s.style).map_err(|e| e.to_string())?;
            style["size"] = json!(r.w);
            json!({"rect": r, "rotation": s.rotation, "value": s.value, "style": style})
        }
        "stamp" => {
            let s = xs_image::stamps::StampObject::from_props(props)?;
            json!({
                "rect": s.rect,
                "rotation": s.rotation,
                "stamp": props.get("stamp").cloned().unwrap_or(Value::Null),
                "art": s.art,
                "ink": s.ink,
                "style": stamp_style(&s, props)
            })
        }
        "simplify" => {
            let s = xs_image::simplify::SimpleObject::from_props(props)?;
            json!({"rect": s.rect, "rotation": s.rotation, "style": s.style})
        }
        "spotlight" => {
            let s = SpotlightObject::from_props(props)?;
            json!({"rect": s.rect, "rotation": s.rotation, "style": s.style})
        }
        "qr" => {
            let q = xs_image::qr::QrObject::from_props(props)?;
            json!({"rect": q.rect, "rotation": q.rotation, "content": q.content, "style": q.style})
        }
        "magnify" => {
            let m = MagnifyObject::from_props(props)?;
            json!({"source": m.source, "at": m.at, "style": m.style})
        }
        "pen" => {
            let p = PenObject::from_props(props)?;
            json!({"points": p.points, "style": p.style})
        }
        "highlight" => {
            let h = HighlightObject::from_props(props)?;
            json!({"rect": h.rect, "rotation": h.rotation, "style": h.style})
        }
        "callout" => {
            let c = CalloutObject::from_props(props)?;
            json!({"rect": c.rect, "rotation": c.rotation, "text": c.text, "tails": c.tails, "style": c.style})
        }
        "image" => {
            let i = ImageObject::from_props(props)?;
            let mut v = json!({"image": i.image, "rect": i.rect, "rotation": i.rotation, "style": i.style});
            if let Some(name) = props
                .get("name")
                .and_then(Value::as_str)
                .filter(|n| !n.is_empty())
            {
                v["name"] = json!(name);
            }
            v
        }
        "zone" => {
            let z = xs_image::shapes::ZoneObject::from_props(props)?;
            json!({"image": z.image, "rect": z.rect, "fit": if z.fill { "fill" } else { "fit" }, "zoom": z.zoom, "offset": z.offset})
        }
        k => return Err(format!("objects of type \"{k}\" cannot be changed in this build")),
    };
    check_shadow(value.get("style"))?;
    Ok(value.as_object().cloned().unwrap_or_default())
}

/// `props` of an object of `kind` checked and in canonical form; unknown properties
/// are dropped (auto redact, SMT-010; auto simplify, SMT-019).
pub fn checked(kind: &str, props: &Map<String, Value>) -> Result<Map<String, Value>, String> {
    canonical(kind, props)
}

/// The props of a template's image zone (EDT-098), checked.
pub fn zone_props(rect: [f64; 4], image: Option<&str>, fill: bool) -> Result<Map<String, Value>, String> {
    let [x, y, w, h] = rect;
    canonical(
        "zone",
        json!({"rect": {"x": x, "y": y, "w": w, "h": h}, "image": image, "fit": if fill { "fill" } else { "fit" }})
            .as_object()
            .expect("object"),
    )
}

/// A stamp's style: its color, opacity and shadow (EDT-031, EDT-021).
fn stamp_style(s: &xs_image::stamps::StampObject, props: &Map<String, Value>) -> Value {
    let mut style = json!({"color": s.color, "opacity": s.opacity});
    if let Some(shadow) = props
        .get("style")
        .and_then(|st| st.get("shadow"))
        .filter(|v| !v.is_null())
    {
        style["shadow"] = shadow.clone();
    }
    style
}

/// The values of a style's shadow, if it has one (EDT-021).
fn check_shadow(style: Option<&Value>) -> Result<(), String> {
    match style.and_then(|s| s.get("shadow")).filter(|v| !v.is_null()) {
        Some(v) => serde_json::from_value::<xs_image::shadows::Shadow>(v.clone())
            .map_err(|e| format!("shadow: {e}"))?
            .check(),
        None => Ok(()),
    }
}

/// Validates one object of `document.annotate` and fills what is missing from the
/// current style. Returns its type and stored properties.
pub fn new_object(input: &Value, styles: &Styles) -> Result<(String, Map<String, Value>), String> {
    let obj = input.as_object().ok_or("must be an object")?;
    let kind = obj
        .get("type")
        .and_then(Value::as_str)
        .ok_or("type is required")?;
    if kind == "image" {
        // Its picture has to travel with it: document.insert_images (EDT-078).
        return Err("images are added with document.insert_images".to_string());
    }
    let Some(keys) = geometry_keys(kind) else {
        return Err(if LATER_TYPES.contains(&kind) {
            format!("type \"{kind}\" arrives in a later build")
        } else {
            format!("unknown type \"{kind}\"")
        });
    };
    let mut props = obj.clone();
    props.remove("type");
    // A step can be placed by its center, `at`, with the size of the style (EDT-035).
    let at = (kind == "step").then(|| props.remove("at")).flatten();
    only_keys(&props, keys)?;
    let mut style = merge_style(styles.of(kind).as_ref(), &style_changes(obj.get("style"))?);
    if kind == "step" {
        if let Some(size) = side(props.get("rect")) {
            style["size"] = json!(size);
        }
        if let Some(c) = at {
            let c = c
                .as_array()
                .and_then(|a| Some([a.first()?.as_f64()?, a.get(1)?.as_f64()?]))
                .ok_or("at must be [x, y]")?;
            let s = style["size"].as_f64().unwrap_or(32.0);
            props.insert(
                "rect".into(),
                json!({"x": c[0] - s / 2.0, "y": c[1] - s / 2.0, "w": s, "h": s}),
            );
        }
    }
    // A new stroke is smoothed as its style says (EDT-037).
    if kind == "pen" {
        let amount = style["smooth"].as_f64().unwrap_or(30.0);
        let points: Vec<[f64; 2]> = props
            .get("points")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(|p| Some([p.get(0)?.as_f64()?, p.get(1)?.as_f64()?]))
                    .collect()
            })
            .unwrap_or_default();
        if !points.is_empty() {
            props.insert("points".into(), json!(pens::smoothed(&points, amount)));
        }
    }
    props.insert("style".into(), style);
    Ok((kind.to_string(), canonical(kind, &props)?))
}

/// Gives a callout drawn without tails one (EDT-028): below its body, once it fits its
/// text, or above it when there is no room below on a `height` px canvas.
pub fn default_tail(object: &mut Object, height: f64) {
    if object.kind != "callout" {
        return;
    }
    let Some(r) = object.props.get("rect") else {
        return;
    };
    let n = |k: &str| r.get(k).and_then(Value::as_f64).unwrap_or(0.0);
    let (x, y, w, h) = (n("x"), n("y"), n("w"), n("h"));
    let gap = (h * 0.5).clamp(24.0, 60.0);
    let below = y + h + gap;
    let tip_y = if below <= height - 2.0 || y - gap < 2.0 {
        below
    } else {
        y - gap
    };
    object
        .props
        .insert("tails".into(), json!([[x + w * 0.25, tip_y]]));
}

/// A stamp of the collection, ready for `document.annotate` or a replacement (EDT-031):
/// its drawing in `lang` and its color. For a new one without a box, `at` (its center)
/// or the canvas middle and the usual size: the smaller of 96 px and a quarter of the
/// canvas's shorter side. A box keeps its height; its width follows the stamp.
pub fn stamp_props(
    input: &Map<String, Value>,
    id: &str,
    lang: xs_core::Lang,
    canvas: (f64, f64),
) -> Result<Map<String, Value>, String> {
    let st = xs_image::stamps::find(id).ok_or_else(|| format!("there is no stamp \"{id}\""))?;
    let l = if lang == xs_core::Lang::Es {
        xs_image::stamps::Lang::Es
    } else {
        xs_image::stamps::Lang::En
    };
    let mut out = input.clone();
    out.insert("stamp".into(), json!(id));
    out.insert("art".into(), st.art(l));
    out.insert("ink".into(), json!(st.ink.unwrap_or("#3A3F4B")));
    let at = input
        .get("at")
        .and_then(Value::as_array)
        .and_then(|a| Some([a.first()?.as_f64()?, a.get(1)?.as_f64()?]));
    out.remove("at");
    let rect = match input.get("rect") {
        Some(r) => {
            let n = |k: &str| r.get(k).and_then(Value::as_f64).unwrap_or(0.0);
            let h = n("h").max(1.0);
            json!({"x": n("x"), "y": n("y"), "w": (h * st.aspect()).max(1.0), "h": h})
        }
        None => {
            let h = 96f64.min(canvas.0.min(canvas.1) * 0.25).max(16.0);
            let w = h * st.aspect();
            let c = at.unwrap_or([canvas.0 / 2.0, canvas.1 / 2.0]);
            json!({"x": c[0] - w / 2.0, "y": c[1] - h / 2.0, "w": w, "h": h})
        }
    };
    out.insert("rect".into(), rect);
    Ok(out)
}

/// The spotlights of a document share their outside (EDT-039): the settings of the one
/// that changed (`from`) pass to all of them.
pub fn share_spotlights(doc: &mut xs_core::document::Document, from: &str) {
    let Some(source) = doc.object(from).filter(|o| o.kind == "spotlight") else {
        return;
    };
    let shared: Map<String, Value> = SHARED
        .iter()
        .filter_map(|k| Some((k.to_string(), source.props.get("style")?.get(*k)?.clone())))
        .collect();
    for o in doc
        .objects
        .iter_mut()
        .filter(|o| o.kind == "spotlight" && o.id != from)
    {
        if let Some(style) = o.props.get_mut("style").and_then(Value::as_object_mut) {
            for (k, v) in &shared {
                style.insert(k.clone(), v.clone());
            }
        }
    }
}

/// A callout that fits its text grows to show all of it (EDT-028). Other objects stay.
pub fn fit(object: &mut Object, dpi: f64) {
    if object.kind != "callout" {
        return;
    }
    let Ok(c) = CalloutObject::from_props(&object.props) else {
        return;
    };
    let Some(h) = c.fitted_height(dpi) else {
        return;
    };
    if let Some(r) = object.props.get_mut("rect").and_then(Value::as_object_mut) {
        r.insert("h".into(), json!(h));
    }
    // A tip the grown body now covers moves down with it, so its tail still shows.
    if c.rotation == 0.0 {
        let r = c.rect;
        let grown = h - r.h;
        let tails: Vec<[f64; 2]> = c
            .tails
            .iter()
            .map(|t| {
                let covered = t[0] >= r.x && t[0] <= r.x + r.w && t[1] > r.y + r.h && t[1] <= r.y + h;
                if covered {
                    [t[0], t[1] + grown]
                } else {
                    *t
                }
            })
            .collect();
        object.props.insert("tails".into(), json!(tails));
    }
}

/// The side of a step dragged or resized to `rect`: the mean of its width and height,
/// between 16 and 256 (EDT-035).
fn side(rect: Option<&Value>) -> Option<f64> {
    let r = rect?;
    let (w, h) = (r.get("w")?.as_f64()?, r.get("h")?.as_f64()?);
    Some(((w + h) / 2.0).clamp(steps::MIN_SIZE, steps::MAX_SIZE).round())
}

/// A floating image showing `image` (a file of `images/`) in `rect` = `[x, y, w, h]`
/// (EDT-078).
pub fn image_object(image: &str, rect: [f64; 4]) -> Result<Map<String, Value>, String> {
    let [x, y, w, h] = rect;
    let props = json!({"image": image, "rect": {"x": x, "y": y, "w": w, "h": h}, "rotation": 0.0});
    canonical("image", props.as_object().expect("object"))
}

/// What an update changed, for the undo step name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    /// Only its position.
    Move,
    /// Anything else: size, style, text…
    Change,
}

/// Where an object is, minus where it would be after `translate`: equal when the
/// object only moved.
fn shape_of(kind: &str, props: &Map<String, Value>) -> Value {
    let mut p = props.clone();
    if let Some(origin) = origin(kind, &p) {
        translate_props(&mut p, -origin[0], -origin[1]);
    }
    Value::Object(p)
}

fn origin(kind: &str, p: &Map<String, Value>) -> Option<[f64; 2]> {
    let xy = |v: &Value| Some([v.get(0)?.as_f64()?, v.get(1)?.as_f64()?]);
    match kind {
        "arrow" | "line" => xy(p.get("from")?),
        "pen" => xy(p.get("points")?.get(0)?),
        "magnify" => {
            let r = p.get("source")?;
            Some([r.get("x")?.as_f64()?, r.get("y")?.as_f64()?])
        }
        "text" => xy(p.get("at")?),
        _ => {
            let r = p.get("rect")?;
            Some([r.get("x")?.as_f64()?, r.get("y")?.as_f64()?])
        }
    }
}

/// Applies `set` (geometry to replace, style properties to change) to `object`.
/// Fields of the stored object this build does not know are kept.
pub fn update(object: &mut Object, set: &Map<String, Value>) -> Result<Change, String> {
    // Locked (EDT-085): kept apart from the geometry and the style, saved in the file.
    let mut set = set.clone();
    if let Some(lock) = set.remove("locked") {
        match lock.as_bool() {
            Some(true) => {
                object.props.insert("locked".into(), Value::Bool(true));
            }
            Some(false) => {
                object.props.remove("locked");
            }
            None => return Err("locked must be true or false".to_string()),
        }
        if set.is_empty() {
            return Ok(Change::Change);
        }
    }
    let set = &set;
    let keys = geometry_keys(&object.kind).ok_or_else(|| {
        format!(
            "objects of type \"{}\" cannot be changed in this build",
            object.kind
        )
    })?;
    only_keys(set, keys)?;
    let mut props = object.props.clone();
    for (k, v) in set {
        if k == "style" {
            let style = merge_style(props.get("style"), &style_changes(Some(v))?);
            props.insert("style".into(), style);
        } else {
            props.insert(k.clone(), v.clone());
        }
    }
    // A step's size follows a new rect; otherwise its rect follows a new size.
    if object.kind == "step" && set.contains_key("rect") {
        if let (Some(size), Some(style)) = (
            side(props.get("rect")),
            props.get_mut("style").and_then(Value::as_object_mut),
        ) {
            style.insert("size".into(), json!(size));
        }
    }
    let before = shape_of(
        &object.kind,
        &canonical(&object.kind, &object.props).unwrap_or_default(),
    );
    let next = canonical(&object.kind, &props)?;
    let change = if shape_of(&object.kind, &next) == before {
        Change::Move
    } else {
        Change::Change
    };
    for (k, v) in next {
        object.props.insert(k, v);
    }
    Ok(change)
}

/// Moves an object by (`dx`, `dy`) canvas pixels, whatever its type: its points
/// (`from`, `to`, `at`, `points`) and its `rect`. Used when cropping (EDT-049).
pub fn translate(object: &mut Object, dx: f64, dy: f64) {
    object.translate(dx, dy);
}

/// i18n key of the undo step for objects of one `kind` (`history.*`).
/// The stacking order after moving `ids` (EDT-081): to the front or the back of all,
/// or one place up or down past the next object that is not moving. Moved objects keep
/// their order among themselves.
pub fn arranged(objects: &mut Vec<Object>, ids: &[String], to: &str) {
    let moving = |o: &Object| ids.contains(&o.id);
    match to {
        "front" | "back" => {
            let (picked, rest): (Vec<Object>, Vec<Object>) = objects.drain(..).partition(moving);
            if to == "front" {
                objects.extend(rest);
                objects.extend(picked);
            } else {
                objects.extend(picked);
                objects.extend(rest);
            }
        }
        "forward" => {
            for i in (0..objects.len().saturating_sub(1)).rev() {
                if moving(&objects[i]) && !moving(&objects[i + 1]) {
                    objects.swap(i, i + 1);
                }
            }
        }
        "backward" => {
            for i in 1..objects.len() {
                if moving(&objects[i]) && !moving(&objects[i - 1]) {
                    objects.swap(i - 1, i);
                }
            }
        }
        _ => {}
    }
}

/// The groups an object belongs to, outermost first (EDT-082).
pub fn groups_of(o: &Object) -> Vec<String> {
    o.props
        .get("groups")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).map(String::from).collect())
        .unwrap_or_default()
}

fn set_groups(o: &mut Object, groups: Vec<String>) {
    if groups.is_empty() {
        o.props.remove("groups");
    } else {
        o.props.insert("groups".into(), json!(groups));
    }
}

/// A group id no object uses yet: `g1`, `g2`…
pub fn new_group_id(objects: &[Object]) -> String {
    let used: Vec<String> = objects.iter().flat_map(groups_of).collect();
    (1..)
        .map(|n| format!("g{n}"))
        .find(|g| !used.contains(g))
        .unwrap_or_default()
}

/// Puts `ids` in a new group around whatever groups they were in (EDT-082). Returns it.
pub fn group(objects: &mut [Object], ids: &[String]) -> String {
    let g = new_group_id(objects);
    for o in objects.iter_mut().filter(|o| ids.contains(&o.id)) {
        let mut groups = groups_of(o);
        groups.insert(0, g.clone());
        set_groups(o, groups);
    }
    g
}

/// Takes `ids` out of their outermost group; the groups inside it stay.
pub fn ungroup(objects: &mut [Object], ids: &[String]) {
    for o in objects.iter_mut().filter(|o| ids.contains(&o.id)) {
        let mut groups = groups_of(o);
        if !groups.is_empty() {
            groups.remove(0);
            set_groups(o, groups);
        }
    }
}

/// Copies keep being a group among themselves, not with the originals (EDT-088).
pub fn regroup_copies(objects: &[Object], copies: &mut [Object]) {
    let mut taken: Vec<String> = objects.iter().flat_map(groups_of).collect();
    let mut renamed: std::collections::BTreeMap<String, String> = Default::default();
    for c in copies.iter_mut() {
        let groups: Vec<String> = groups_of(c)
            .into_iter()
            .map(|g| {
                renamed
                    .entry(g)
                    .or_insert_with(|| {
                        let fresh = (1..)
                            .map(|n| format!("g{n}"))
                            .find(|x| !taken.contains(x))
                            .unwrap_or_default();
                        taken.push(fresh.clone());
                        fresh
                    })
                    .clone()
            })
            .collect();
        set_groups(c, groups);
    }
}

pub fn label(action: &str, kinds: &[&str]) -> String {
    // One kind (even several objects of it) names it; a mix is "objects".
    let what = match kinds.first() {
        Some(k) if kinds.iter().all(|x| x == k) => *k,
        _ => "objects",
    };
    format!("history.{action}_{what}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn named(ids: &[&str]) -> Vec<Object> {
        ids.iter()
            .map(|id| Object {
                id: id.to_string(),
                kind: "shape".into(),
                props: Map::new(),
            })
            .collect()
    }

    fn order(objects: &[Object]) -> Vec<&str> {
        objects.iter().map(|o| o.id.as_str()).collect()
    }

    #[test]
    fn groups_nest_and_copies_get_their_own() {
        let mut o = named(&["a", "b", "c"]);
        let g1 = group(&mut o, &["a".into(), "b".into()]);
        let g2 = group(&mut o, &["a".into(), "b".into(), "c".into()]);
        assert_eq!(groups_of(&o[0]), [g2.clone(), g1.clone()]);
        assert_eq!(groups_of(&o[2]), std::slice::from_ref(&g2));
        ungroup(&mut o, &["a".into(), "b".into(), "c".into()]);
        assert_eq!(
            groups_of(&o[0]),
            std::slice::from_ref(&g1),
            "the inner group stays"
        );
        assert!(o[2].props.get("groups").is_none());
        let mut copies = vec![o[0].clone(), o[1].clone()];
        regroup_copies(&o, &mut copies);
        let new = groups_of(&copies[0]);
        assert_eq!(new, groups_of(&copies[1]), "the copies are one group");
        assert_ne!(new, [g1]);
    }

    #[test]
    fn stacking_order() {
        // Bottom first: a, b, c.
        let mut o = named(&["a", "b", "c"]);
        arranged(&mut o, &["c".into()], "back");
        assert_eq!(order(&o), ["c", "a", "b"], "the top one goes under the other two");
        arranged(&mut o, &["c".into()], "forward");
        assert_eq!(order(&o), ["a", "c", "b"], "one place up, not to the top");
        arranged(&mut o, &["a".into(), "c".into()], "front");
        assert_eq!(order(&o), ["b", "a", "c"], "several keep their order");
        arranged(&mut o, &["c".into()], "backward");
        assert_eq!(order(&o), ["b", "c", "a"]);
    }

    #[test]
    fn missing_style_comes_from_the_tool() {
        let mut styles = Styles::default();
        styles.arrow.color = "#00FF00".into();
        let (kind, props) =
            new_object(&json!({"type": "arrow", "from": [0, 0], "to": [10, 0]}), &styles).unwrap();
        assert_eq!(kind, "arrow");
        assert_eq!(props["style"]["color"], "#00FF00");
        assert_eq!(props["style"]["endCap"], "filled");
        let (_, shape) = new_object(
            &json!({"type": "shape", "rect": {"x": 1, "y": 2, "w": 30, "h": 40}, "style": {"shapeType": "ellipse"}}),
            &styles,
        )
        .unwrap();
        assert_eq!(shape["style"]["shapeType"], "ellipse");
        assert_eq!(shape["style"]["outlineWidth"], json!(4.0));
        assert_eq!(shape["rotation"], json!(0.0));
    }

    #[test]
    fn explains_what_is_wrong() {
        let s = Styles::default();
        let err = |v: Value| new_object(&v, &s).unwrap_err();
        assert!(err(json!({"type": "spiral"})).contains("unknown type"));
        assert!(err(json!({"type": "spiral"})).contains("unknown type"));
        assert!(err(json!({"type": "arrow", "from": [0, 0]})).contains("to"));
        assert!(err(json!({"type": "arrow", "from": [0, 0], "to": [0, 0]})).contains("1 pixel"));
        assert!(err(json!({"type": "arrow", "from": [0, 0], "to": [9, 9], "colour": 1})).contains("colour"));
        assert!(err(json!({"type": "arrow", "from": [0, 0], "to": [9, 9], "style": "Bold"})).contains("M2"));
        assert!(err(json!({"type": "text", "at": [0, 0], "text": "  "})).contains("empty"));
        assert!(err(json!({"type": "shape", "rect": {"x": 0, "y": 0, "w": 0, "h": 5}})).contains("1 pixel"));
        assert!(err(
            json!({"type": "blur", "rect": {"x": 0, "y": 0, "w": 9, "h": 5}, "style": {"blurType": "x"}})
        )
        .contains("style"));
    }

    #[test]
    fn updates_tell_moves_from_changes_and_keep_unknown_fields() {
        let (_, mut props) = new_object(
            &json!({"type": "arrow", "from": [0, 0], "to": [10, 0]}),
            &Styles::default(),
        )
        .unwrap();
        props.insert("future".into(), json!(1));
        let mut o = Object {
            id: "o1".into(),
            kind: "arrow".into(),
            props,
        };
        let moved = update(
            &mut o,
            json!({"from": [5, 5], "to": [15, 5]}).as_object().unwrap(),
        )
        .unwrap();
        assert_eq!(moved, Change::Move);
        let bent = update(&mut o, json!({"to": [20, 20]}).as_object().unwrap()).unwrap();
        assert_eq!(bent, Change::Change);
        let styled = update(&mut o, json!({"style": {"width": 12}}).as_object().unwrap()).unwrap();
        assert_eq!(styled, Change::Change);
        assert_eq!(o.props["style"]["width"], json!(12.0));
        assert_eq!(o.props["future"], json!(1));
        assert!(update(&mut o, json!({"style": {"width": 0}}).as_object().unwrap()).is_err());
        assert!(update(&mut o, json!({"rect": {}}).as_object().unwrap()).is_err());
    }

    #[test]
    fn translate_moves_every_kind() {
        let s = Styles::default();
        for input in [
            json!({"type": "arrow", "from": [0, 0], "to": [10, 0]}),
            json!({"type": "text", "at": [3, 4], "text": "Hi"}),
            json!({"type": "blur", "rect": {"x": 3, "y": 4, "w": 5, "h": 6}}),
        ] {
            let (kind, props) = new_object(&input, &s).unwrap();
            let mut o = Object {
                id: "o1".into(),
                kind: kind.clone(),
                props,
            };
            let before = origin(&kind, &o.props).unwrap();
            translate(&mut o, -10.0, 20.0);
            assert_eq!(
                origin(&kind, &o.props).unwrap(),
                [before[0] - 10.0, before[1] + 20.0]
            );
        }
    }

    #[test]
    fn tool_styles_change_and_validate() {
        let mut s = Styles::default();
        let v = s
            .set("text", json!({"size": 20, "bold": true}).as_object().unwrap())
            .unwrap();
        assert_eq!(v["size"], json!(20.0));
        assert!(s
            .set("blur", json!({"intensity": 500}).as_object().unwrap())
            .is_err());
        assert!(s.set("crop", &Map::new()).is_err());
    }
}
