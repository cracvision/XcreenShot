//! Templates (EDT-095 to EDT-101, ADR 0007): a template is an ordinary document with a
//! `template` field (its kind, settings and stacked sections) and objects the layout
//! put in named places (`slot`). Laying it out again keeps what the user changed: the
//! text of each place, each zone's picture and framing, and the annotations without a
//! place, which move with their section.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use xs_core::document::{Document, Object, PixelRect};
use xs_core::i18n::tr;
use xs_core::Lang;

use crate::objects::{self, Styles};

/// Most images a template takes (EDT-095).
pub const MAX_IMAGES: usize = 20;
/// The width of every built-in template, px.
pub const WIDTH: f64 = 1200.0;
const MARGIN: f64 = 60.0;

/// Template settings (EDT-097).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// "fit" (whole picture) or "fill" (the whole zone).
    pub scale: String,
    pub title: bool,
    pub captions: bool,
    pub footer: bool,
    /// "system" (the account's name) or "custom".
    pub author: String,
    pub author_text: String,
    pub font: Option<String>,
    pub text_color: Option<String>,
    /// The canvas: a color, a gradient or null.
    pub background: Value,
    pub numbering: bool,
    pub number_color: Option<String>,
    pub number_shape: String,
    pub reversed: bool,
    /// The theme of its colors and font (EDT-096); none: the active one.
    pub theme: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            scale: "fit".into(),
            title: true,
            captions: true,
            footer: false,
            author: "system".into(),
            author_text: String::new(),
            font: None,
            text_color: None,
            background: json!("#FFFFFF"),
            numbering: true,
            number_color: None,
            number_shape: "circle".into(),
            reversed: false,
            theme: None,
        }
    }
}

/// A band of the template (EDT-099).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Section {
    pub id: String,
    pub kind: String,
    pub height: f64,
    /// Zones side by side (rows, timelines, columns).
    #[serde(default)]
    pub zones: usize,
}

/// The `template` field of a document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Template {
    pub id: String,
    pub settings: Settings,
    pub sections: Vec<Section>,
    #[serde(default = "width")]
    pub width: f64,
    /// Places the user removed (a caption), not laid out again.
    #[serde(default)]
    pub hidden: Vec<String>,
    #[serde(default)]
    pub next_section: u32,
}

fn width() -> f64 {
    WIDTH
}

impl Template {
    pub fn of(doc: &Document) -> Option<Template> {
        doc.template.clone().and_then(|v| serde_json::from_value(v).ok())
    }

    pub fn new_section(&mut self, kind: &str, zones: usize) -> Section {
        self.next_section += 1;
        Section {
            id: format!("s{}", self.next_section),
            kind: kind.to_string(),
            height: default_height(kind, zones),
            zones,
        }
    }
}

/// Section kinds a user can add (EDT-099).
pub const ADDABLE: [&str; 6] = ["step", "imageText", "text", "twoImages", "title", "footer"];

fn default_height(kind: &str, zones: usize) -> f64 {
    match kind {
        "title" => 110.0,
        "step" => 720.0,
        "imageText" => 380.0,
        "text" => 140.0,
        "twoImages" => 400.0,
        "gridRow" if zones >= 3 => 320.0,
        "gridRow" => 400.0,
        "footer" => 70.0,
        "timeline" => 460.0,
        "milestone" => 360.0,
        "compare" | "beforeAfter" => 520.0,
        "columns3" => 420.0,
        "featured" => 640.0,
        _ => 300.0,
    }
}

/// Zones a section of `kind` holds.
fn zones_of(kind: &str, zones: usize) -> usize {
    match kind {
        "step" | "imageText" | "milestone" => 1,
        "twoImages" | "compare" | "beforeAfter" => 2,
        "columns3" => 3,
        "featured" => 4,
        "gridRow" | "timeline" => zones.max(1),
        _ => 0,
    }
}

/// A built-in template (EDT-096).
pub struct Builtin {
    pub id: &'static str,
    pub family: &'static str,
    pub min: usize,
    pub max: usize,
    /// Whether it starts with a title.
    title: bool,
}

pub const BUILTINS: [Builtin; 11] = [
    Builtin {
        id: "steps-column",
        family: "steps",
        min: 1,
        max: MAX_IMAGES,
        title: true,
    },
    Builtin {
        id: "steps-grid",
        family: "steps",
        min: 2,
        max: MAX_IMAGES,
        title: true,
    },
    Builtin {
        id: "steps-side",
        family: "steps",
        min: 1,
        max: MAX_IMAGES,
        title: true,
    },
    Builtin {
        id: "steps-custom",
        family: "steps",
        min: 1,
        max: MAX_IMAGES,
        title: false,
    },
    Builtin {
        id: "timeline-horizontal",
        family: "timeline",
        min: 3,
        max: 6,
        title: true,
    },
    Builtin {
        id: "timeline-vertical",
        family: "timeline",
        min: 2,
        max: MAX_IMAGES,
        title: true,
    },
    Builtin {
        id: "compare-side",
        family: "compare",
        min: 2,
        max: 2,
        title: true,
    },
    Builtin {
        id: "before-after",
        family: "compare",
        min: 2,
        max: 2,
        title: true,
    },
    Builtin {
        id: "three-options",
        family: "compare",
        min: 3,
        max: 3,
        title: true,
    },
    Builtin {
        id: "collage-featured",
        family: "collage",
        min: 4,
        max: 4,
        title: false,
    },
    Builtin {
        id: "collage-grid",
        family: "collage",
        min: 4,
        max: 9,
        title: false,
    },
];

pub fn builtin(id: &str) -> Option<&'static Builtin> {
    BUILTINS.iter().find(|b| b.id == id)
}

/// A built-in template for `n` images: its sections (EDT-096).
pub fn create(b: &Builtin, n: usize) -> Template {
    let n = n.clamp(b.min, b.max);
    let mut t = Template {
        id: b.id.to_string(),
        settings: Settings {
            title: b.title,
            numbering: b.family == "steps" || b.id == "timeline-vertical",
            captions: b.family != "collage",
            ..Settings::default()
        },
        sections: Vec::new(),
        width: WIDTH,
        hidden: Vec::new(),
        next_section: 0,
    };
    let push = |t: &mut Template, kind: &str, zones: usize| {
        let s = t.new_section(kind, zones);
        t.sections.push(s);
    };
    // Title and footer are always there, shown as the settings say (EDT-097).
    push(&mut t, "title", 0);
    match b.id {
        "steps-column" | "steps-custom" => (0..n).for_each(|_| push(&mut t, "step", 1)),
        "steps-grid" => {
            let mut left = n;
            while left > 0 {
                push(&mut t, "gridRow", left.min(2));
                left = left.saturating_sub(2);
            }
        }
        "steps-side" => (0..n).for_each(|_| push(&mut t, "imageText", 1)),
        "timeline-horizontal" => push(&mut t, "timeline", n),
        "timeline-vertical" => (0..n).for_each(|_| push(&mut t, "milestone", 1)),
        "compare-side" => push(&mut t, "compare", 2),
        "before-after" => push(&mut t, "beforeAfter", 2),
        "three-options" => push(&mut t, "columns3", 3),
        "collage-featured" => push(&mut t, "featured", 4),
        _ => {
            let side = if n <= 4 { 2 } else { 3 };
            (0..side).for_each(|_| push(&mut t, "gridRow", side));
        }
    }
    push(&mut t, "footer", 0);
    t
}

/// The colors and font a template takes from the theme (EDT-096).
pub struct Look {
    pub accent: String,
    pub text: String,
    pub font: Option<String>,
}

impl Look {
    pub fn of(lib: &crate::themes::Library, s: &Settings) -> Look {
        let theme = s
            .theme
            .as_deref()
            .and_then(|id| lib.themes.iter().find(|t| t.id == id))
            .unwrap_or_else(|| lib.active());
        let colors = &theme.colors;
        let darkest = colors
            .iter()
            .min_by(|a, b| crate::themes::luminance(a).total_cmp(&crate::themes::luminance(b)))
            .cloned()
            .unwrap_or_else(|| "#1F2430".into());
        // The first color neither too dark nor too light marks numbers and lines.
        let middle = colors
            .iter()
            .find(|c| (40.0..200.0).contains(&crate::themes::luminance(c)))
            .or(colors.first())
            .cloned()
            .unwrap_or_else(|| "#2D7FF9".into());
        Look {
            accent: s.number_color.clone().unwrap_or(middle),
            text: s.text_color.clone().unwrap_or(darkest),
            font: s.font.clone().or_else(|| theme.font.clone()),
        }
    }
}

/// One object the layout places, before ids.
struct Placed {
    slot: String,
    section: String,
    kind: String,
    props: Map<String, Value>,
}

struct Layout<'a> {
    lang: Lang,
    styles: &'a Styles,
    look: &'a Look,
    settings: &'a Settings,
    width: f64,
    hidden: &'a [String],
    out: Vec<Placed>,
    /// Zones laid out so far, for the numbers.
    zone_count: usize,
    /// Which zones have a picture, by slot: only those get a number (EDT-097).
    filled: &'a std::collections::HashSet<String>,
    numbered: usize,
}

impl Layout<'_> {
    fn push(&mut self, slot: String, section: &str, spec: Value) {
        if self.hidden.contains(&slot) {
            return;
        }
        if let Ok((kind, props)) = objects::new_object(&spec, self.styles) {
            self.out.push(Placed {
                slot,
                section: section.to_string(),
                kind,
                props,
            });
        }
    }

    /// A text of the layout: `place` is `[x, y, width]`, `font` is (size, bold, align).
    fn text(&mut self, slot: String, section: &str, place: [f64; 3], text: &str, font: (f64, bool, &str)) {
        let ([x, y, width], (size, bold, align)) = (place, font);
        let at = [x, y];
        let mut style = json!({"size": size, "bold": bold, "color": self.look.text, "align": align});
        if let Some(f) = &self.look.font {
            style["font"] = json!(f);
        }
        self.push(
            slot,
            section,
            json!({"type": "text", "at": at, "width": width, "text": text, "style": style}),
        );
    }

    fn zone(&mut self, sid: &str, i: usize, r: [f64; 4]) {
        let slot = format!("zone:{sid}:{i}");
        self.zone_count += 1;
        let fill = self.settings.scale == "fill";
        if let Ok(props) = objects::zone_props(r, None, fill) {
            self.out.push(Placed {
                slot: slot.clone(),
                section: sid.to_string(),
                kind: "zone".into(),
                props,
            });
        }
        // Its number, on the zone's corner (EDT-097).
        if self.settings.numbering && self.filled.contains(&slot) {
            self.numbered += 1;
            let style = json!({"fill": self.look.accent, "shape": self.settings.number_shape, "size": 40});
            self.push(
                format!("number:{sid}:{i}"),
                sid,
                json!({"type": "step", "at": [r[0] + 28.0, r[1] + 28.0], "value": self.numbered.to_string(), "style": style}),
            );
        }
    }

    fn caption(&mut self, sid: &str, i: usize, at: [f64; 2], width: f64, align: &str) {
        if self.settings.captions {
            let text = tr(self.lang, "editor.template.caption", &[]);
            self.text(
                format!("caption:{sid}:{i}"),
                sid,
                [at[0], at[1], width],
                &text,
                (18.0, false, align),
            );
        }
    }

    /// Lays out `s` from `top`.
    fn section(&mut self, s: &Section, top: f64, author: &str) {
        let (w, m, h, sid) = (self.width, MARGIN, s.height, s.id.as_str());
        let inner = w - 2.0 * m;
        let caption_room = if self.settings.captions { 70.0 } else { 20.0 };
        match s.kind.as_str() {
            "title" => {
                let text = tr(self.lang, "editor.template.title", &[]);
                self.text(
                    "title".into(),
                    sid,
                    [m, top + 30.0, inner],
                    &text,
                    (34.0, true, "center"),
                );
            }
            "step" => {
                let zh = (h - 20.0 - caption_room).max(40.0);
                self.zone(sid, 0, [m, top + 20.0, inner, zh]);
                self.caption(sid, 0, [m, top + 30.0 + zh], inner, "left");
            }
            "imageText" => {
                let zw = inner * 0.55;
                self.zone(sid, 0, [m, top + 20.0, zw, (h - 40.0).max(40.0)]);
                self.caption(sid, 0, [m + zw + 30.0, top + 30.0], inner - zw - 30.0, "left");
            }
            "text" => {
                let text = tr(self.lang, "editor.template.text", &[]);
                self.text(
                    format!("text:{sid}"),
                    sid,
                    [m, top + 20.0, inner],
                    &text,
                    (18.0, false, "left"),
                );
            }
            "twoImages" | "gridRow" => {
                let k = zones_of(&s.kind, s.zones).max(1);
                let gap = 30.0;
                let zw = (inner - gap * (k as f64 - 1.0)) / k as f64;
                let zh = (h - 20.0 - caption_room).max(40.0);
                for i in 0..k {
                    let x = m + i as f64 * (zw + gap);
                    self.zone(sid, i, [x, top + 20.0, zw, zh]);
                    self.caption(sid, i, [x, top + 30.0 + zh], zw, "left");
                }
            }
            "footer" => {
                let date = chrono::Local::now().format("%Y-%m-%d").to_string();
                let text = if author.is_empty() {
                    date
                } else {
                    format!("{author} · {date}")
                };
                let mut style = json!({"size": 14.0, "color": self.look.text, "align": "right"});
                if let Some(f) = &self.look.font {
                    style["font"] = json!(f);
                }
                self.push(
                    "footer".into(),
                    sid,
                    json!({"type": "text", "at": [m, top + 22.0], "width": inner, "text": text, "style": style}),
                );
            }
            "timeline" => {
                let k = zones_of("timeline", s.zones);
                let line_y = top + h * 0.58;
                self.push(
                    format!("line:{sid}"),
                    sid,
                    json!({"type": "line", "from": [m, line_y], "to": [w - m, line_y], "style": {"color": self.look.accent, "width": 4}}),
                );
                let slot_w = inner / k as f64;
                for i in 0..k {
                    let cx = m + (i as f64 + 0.5) * slot_w;
                    let zw = slot_w - 24.0;
                    self.zone(
                        sid,
                        i,
                        [cx - zw / 2.0, top + 20.0, zw, (line_y - top - 50.0).max(40.0)],
                    );
                    self.push(
                        format!("dot:{sid}:{i}"),
                        sid,
                        json!({"type": "shape", "rect": {"x": cx - 10.0, "y": line_y - 10.0, "w": 20.0, "h": 20.0},
                               "style": {"shapeType": "ellipse", "fill": self.look.accent, "outline": self.look.accent, "outlineWidth": 0}}),
                    );
                    let date = tr(self.lang, "editor.template.date", &[]);
                    self.text(
                        format!("date:{sid}:{i}"),
                        sid,
                        [cx - zw / 2.0, line_y + 20.0, zw],
                        &date,
                        (16.0, true, "center"),
                    );
                    self.caption(sid, i, [cx - zw / 2.0, line_y + 50.0], zw, "center");
                }
            }
            "milestone" => {
                let x = m + 30.0;
                self.push(
                    format!("line:{sid}"),
                    sid,
                    json!({"type": "line", "from": [x, top], "to": [x, top + h], "style": {"color": self.look.accent, "width": 4}}),
                );
                self.push(
                    format!("dot:{sid}:0"),
                    sid,
                    json!({"type": "shape", "rect": {"x": x - 12.0, "y": top + 28.0, "w": 24.0, "h": 24.0},
                           "style": {"shapeType": "ellipse", "fill": self.look.accent, "outline": self.look.accent, "outlineWidth": 0}}),
                );
                let date = tr(self.lang, "editor.template.date", &[]);
                self.text(
                    format!("date:{sid}:0"),
                    sid,
                    [x + 40.0, top + 26.0, 300.0],
                    &date,
                    (18.0, true, "left"),
                );
                let zx = x + 40.0;
                let zh = (h - 80.0 - caption_room).max(40.0);
                self.zone(sid, 0, [zx, top + 64.0, w - m - zx, zh]);
                self.caption(sid, 0, [zx, top + 74.0 + zh], w - m - zx, "left");
            }
            "compare" | "columns3" | "beforeAfter" => {
                let k = zones_of(&s.kind, 0);
                let gap = if s.kind == "beforeAfter" { 90.0 } else { 30.0 };
                let zw = (inner - gap * (k as f64 - 1.0)) / k as f64;
                let zh = (h - 80.0 - caption_room).max(40.0);
                for i in 0..k {
                    let x = m + i as f64 * (zw + gap);
                    let header = match s.kind.as_str() {
                        "beforeAfter" if i == 0 => tr(self.lang, "editor.template.before", &[]),
                        "beforeAfter" => tr(self.lang, "editor.template.after", &[]),
                        _ => tr(
                            self.lang,
                            "editor.template.option",
                            &[("n", &(i + 1).to_string())],
                        ),
                    };
                    self.text(
                        format!("header:{sid}:{i}"),
                        sid,
                        [x, top + 20.0, zw],
                        &header,
                        (24.0, true, "center"),
                    );
                    self.zone(sid, i, [x, top + 70.0, zw, zh]);
                    self.caption(sid, i, [x, top + 80.0 + zh], zw, "center");
                }
                if s.kind == "beforeAfter" {
                    let y = top + 70.0 + zh / 2.0;
                    self.push(
                        format!("arrow:{sid}"),
                        sid,
                        json!({"type": "arrow", "from": [m + zw + 14.0, y], "to": [m + zw + gap - 14.0, y],
                               "style": {"color": self.look.accent, "width": 8}}),
                    );
                }
            }
            "featured" => {
                let gap = 20.0;
                let big = inner * 0.62;
                let small_w = inner - big - gap;
                let small_h = (h - 40.0 - 2.0 * gap) / 3.0;
                self.zone(sid, 0, [m, top + 20.0, big, h - 40.0]);
                for i in 0..3 {
                    self.zone(
                        sid,
                        i + 1,
                        [
                            m + big + gap,
                            top + 20.0 + i as f64 * (small_h + gap),
                            small_w,
                            small_h,
                        ],
                    );
                }
            }
            _ => {}
        }
    }
}

/// What a layout keeps of the objects already there, by place.
fn kept(old: &[Object]) -> BTreeMap<String, &Object> {
    old.iter()
        .filter_map(|o| Some((o.props.get("slot")?.as_str()?.to_string(), o)))
        .collect()
}

/// Lays `t` out on `doc` again (ADR 0007): the canvas, its color and the objects in
/// their places, keeping texts, pictures, framing and the user's annotations.
/// `restyle` gives texts and numbers the look again (after a change of theme, font or
/// colors); otherwise they keep the style the user gave them.
pub fn layout(
    doc: &mut Document,
    t: &Template,
    styles: &Styles,
    look: &Look,
    lang: Lang,
    author: &str,
    restyle: bool,
) {
    let old = std::mem::take(&mut doc.objects);
    let old_template = Template::of(doc);
    let by_slot = kept(&old);
    let filled: std::collections::HashSet<String> = old
        .iter()
        .filter(|o| o.kind == "zone" && o.props.get("image").is_some_and(|v| v.is_string()))
        .filter_map(|o| o.props.get("slot").and_then(Value::as_str).map(str::to_string))
        .collect();
    let mut lay = Layout {
        lang,
        styles,
        look,
        settings: &t.settings,
        width: t.width,
        hidden: &t.hidden,
        out: Vec::new(),
        zone_count: 0,
        filled: &filled,
        numbered: 0,
    };
    // Where each section was and now is, for the annotations.
    let mut tops: BTreeMap<String, (f64, f64)> = BTreeMap::new();
    let mut y = 0.0;
    for s in &t.sections {
        if (s.kind == "title" && !t.settings.title) || (s.kind == "footer" && !t.settings.footer) {
            continue;
        }
        lay.section(s, y, author);
        tops.insert(s.id.clone(), (y, s.height));
        y += s.height;
    }
    let height = y.max(1.0);
    let placed = lay.out;
    let mut next_id = old
        .iter()
        .filter_map(|o| o.id.strip_prefix('o').and_then(|n| n.parse::<u64>().ok()))
        .max()
        .unwrap_or(0);
    let mut objects = Vec::new();
    for p in placed {
        let mut props = p.props;
        let previous = by_slot.get(&p.slot);
        if let Some(prev) = previous {
            // The user's text, picture and framing stay.
            for key in ["text", "image", "zoom", "offset"] {
                if let Some(v) = prev.props.get(key) {
                    props.insert(key.into(), v.clone());
                }
            }
            if !restyle && p.kind != "zone" {
                if let Some(style) = prev.props.get("style") {
                    props.insert("style".into(), style.clone());
                }
            }
            if let Some(lock) = prev.props.get("locked") {
                props.insert("locked".into(), lock.clone());
            }
        }
        props.insert("slot".into(), json!(p.slot));
        props.insert("section".into(), json!(p.section));
        let id = match previous {
            Some(prev) => prev.id.clone(),
            None => {
                next_id += 1;
                format!("o{next_id}")
            }
        };
        objects.push(Object {
            id,
            kind: p.kind,
            props,
        });
    }
    // The user's annotations move with their section, or go with it.
    let bands: Vec<(String, f64, f64)> = old_template
        .as_ref()
        .map(|ot| {
            let mut y = 0.0;
            ot.sections
                .iter()
                .filter(|s| {
                    !((s.kind == "title" && !ot.settings.title)
                        || (s.kind == "footer" && !ot.settings.footer))
                })
                .map(|s| {
                    let band = (s.id.clone(), y, s.height);
                    y += s.height;
                    band
                })
                .collect()
        })
        .unwrap_or_default();
    for mut o in old.into_iter().filter(|o| o.props.get("slot").is_none()) {
        let Some(b) = xs_image::object_bounds(&o, doc.dpi) else {
            objects.push(o);
            continue;
        };
        let cy = (b[1] + b[3]) / 2.0;
        match bands.iter().find(|(_, top, h)| cy >= *top && cy < top + h) {
            // In a section that went, it goes too (EDT-099).
            Some((id, top, h)) => {
                if let Some((new_top, new_h)) = tops.get(id) {
                    let ny = new_top + (cy - top) * (new_h / h.max(1.0));
                    o.translate(0.0, ny - cy);
                    objects.push(o);
                }
            }
            None => objects.push(o),
        }
    }
    doc.objects = objects;
    let (w, h) = (t.width.round() as u32, height.round() as u32);
    doc.crop = PixelRect {
        x: 0,
        y: 0,
        width: w,
        height: h,
    };
    doc.canvas.width = w;
    doc.canvas.height = h;
    doc.frame = None;
    doc.crop_history.clear();
    doc.outside.clear();
    match &t.settings.background {
        Value::Object(_) => {
            doc.canvas.gradient = Some(t.settings.background.clone());
            doc.canvas.color = serde_json::from_value::<xs_image::paint::Fill>(t.settings.background.clone())
                .ok()
                .and_then(|f| f.main_color())
                .map(|c| c.to_hex());
        }
        Value::String(c) => {
            doc.canvas.gradient = None;
            doc.canvas.color = Some(c.clone());
        }
        _ => {
            doc.canvas.gradient = None;
            doc.canvas.color = None;
        }
    }
    doc.template = serde_json::to_value(t).ok();
}

/// Whether a section shows: a title or a footer turned off does not (EDT-097).
pub fn shown(t: &Template, s: &Section) -> bool {
    !((s.kind == "title" && !t.settings.title) || (s.kind == "footer" && !t.settings.footer))
}

/// The sections shown, each with where it starts.
pub fn bands(t: &Template) -> Vec<(&Section, f64)> {
    let mut y = 0.0;
    t.sections
        .iter()
        .filter(|s| shown(t, s))
        .map(|s| {
            let band = (s, y);
            y += s.height;
            band
        })
        .collect()
}

/// The zones of `doc` in order (the layout's order).
pub fn zones(doc: &Document) -> Vec<String> {
    doc.objects
        .iter()
        .filter(|o| o.kind == "zone")
        .filter_map(|o| o.props.get("slot").and_then(Value::as_str).map(str::to_string))
        .collect()
}

/// Reverses the pictures (and framing) over the zones (EDT-097: Reverse the order).
pub fn reverse_pictures(doc: &mut Document) {
    let contents: Vec<(Value, Value, Value)> = doc
        .objects
        .iter()
        .filter(|o| o.kind == "zone")
        .map(|o| {
            (
                o.props.get("image").cloned().unwrap_or(Value::Null),
                o.props.get("zoom").cloned().unwrap_or(json!(1.0)),
                o.props.get("offset").cloned().unwrap_or(json!([0.0, 0.0])),
            )
        })
        .collect();
    let mut reversed = contents.into_iter().rev();
    for o in doc.objects.iter_mut().filter(|o| o.kind == "zone") {
        if let Some((image, zoom, offset)) = reversed.next() {
            o.props.insert("image".into(), image);
            o.props.insert("zoom".into(), zoom);
            o.props.insert("offset".into(), offset);
        }
    }
}

/// The account's name for the footer (EDT-097: Author = system name).
pub fn system_author() -> String {
    xs_core::platform::account_name()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn look() -> Look {
        Look {
            accent: "#0064FB".into(),
            text: "#1F2430".into(),
            font: None,
        }
    }

    fn doc_for(t: &Template) -> Document {
        let mut d = Document::new(1, 1, 96.0);
        layout(&mut d, t, &Styles::default(), &look(), Lang::En, "", true);
        d
    }

    #[test]
    fn every_family_has_two_templates_or_more() {
        for family in ["steps", "timeline", "compare", "collage"] {
            assert!(
                BUILTINS.iter().filter(|b| b.family == family).count() >= 2,
                "{family}"
            );
        }
    }

    #[test]
    fn steps_have_one_zone_each_and_numbers_only_on_pictures() {
        let t = create(builtin("steps-column").unwrap(), 4);
        let mut d = doc_for(&t);
        assert_eq!(zones(&d).len(), 4);
        assert!(
            !d.objects.iter().any(|o| o.kind == "step"),
            "no pictures, no numbers"
        );
        // Pictures in zones 1 and 3: numbers 1 and 2 there.
        for (i, o) in d.objects.iter_mut().filter(|o| o.kind == "zone").enumerate() {
            if i % 2 == 0 {
                o.props.insert("image".into(), json!("ins-a.png"));
            }
        }
        let t = Template::of(&d).unwrap();
        layout(&mut d, &t, &Styles::default(), &look(), Lang::En, "", false);
        let values: Vec<&str> = d
            .objects
            .iter()
            .filter(|o| o.kind == "step")
            .filter_map(|o| o.props.get("value").and_then(Value::as_str))
            .collect();
        assert_eq!(values, ["1", "2"]);
        assert!(d.canvas.height > 2000, "four steps make it tall");
    }

    #[test]
    fn layout_keeps_texts_and_moves_annotations_with_their_section() {
        let t = create(builtin("steps-column").unwrap(), 2);
        let mut d = doc_for(&t);
        let caption = d
            .objects
            .iter_mut()
            .find(|o| o.props.get("slot").and_then(Value::as_str) == Some("caption:s3:0"))
            .unwrap();
        caption.props.insert("text".into(), json!("Click Save"));
        // An arrow drawn in the second step.
        let second = &t.sections[2];
        let top: f64 = t.sections[..2].iter().map(|s| s.height).sum();
        d.objects.push(Object {
            id: "o99".into(),
            kind: "arrow".into(),
            props: serde_json::from_value(json!({"from": [100, top + 100.0], "to": [200, top + 100.0]}))
                .unwrap(),
        });
        // The title goes: everything moves up by its height.
        let mut t = Template::of(&d).unwrap();
        t.settings.title = false;
        layout(&mut d, &t, &Styles::default(), &look(), Lang::En, "", false);
        let caption = d
            .objects
            .iter()
            .find(|o| o.props.get("slot").and_then(Value::as_str) == Some("caption:s3:0"))
            .unwrap();
        assert_eq!(caption.props["text"], "Click Save");
        let arrow = d.objects.iter().find(|o| o.id == "o99").unwrap();
        let title_h = t.sections[0].height;
        assert_eq!(arrow.props["from"][1].as_f64().unwrap(), top + 100.0 - title_h);
        assert_eq!(second.kind, "step");
    }

    #[test]
    fn before_after_puts_the_first_under_before() {
        let t = create(builtin("before-after").unwrap(), 2);
        let d = doc_for(&t);
        let header0 = d
            .objects
            .iter()
            .find(|o| {
                o.props
                    .get("slot")
                    .and_then(Value::as_str)
                    .is_some_and(|s| s.starts_with("header:") && s.ends_with(":0"))
            })
            .unwrap();
        assert_eq!(header0.props["text"], "Before");
        assert!(d.objects.iter().any(|o| o.kind == "arrow"));
    }
}
