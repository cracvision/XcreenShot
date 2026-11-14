//! The step recorder's model (SMT-022 to SMT-027, ADR 0009), without the system: which
//! clicks make a step, the sentence of each step and the framing of its image. The
//! Windows side (`steps_win`) feeds it the clicks it hears and the accessibility
//! information of what was clicked.

use serde::{Deserialize, Serialize};
use xs_core::i18n::tr;
use xs_core::Lang;

/// A press farther than this from where it started is a drag (SMT-024), px.
pub const DRAG_PX: f64 = 5.0;
/// Names longer than this are cut with "…" (SMT-025).
pub const NAME_CHARS: usize = 60;
/// A framed zone smaller than this on either side gives the whole monitor (SMT-026).
pub const MIN_FRAME: f64 = 200.0;
/// The margin around the clicked zone (SMT-026), px.
pub const FRAME_MARGIN: f64 = 24.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Button {
    Left,
    Right,
}

/// What a step was.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Click,
    Right,
    Double,
    Drag,
    /// A keyboard shortcut, with "Note keyboard use" (SMT-024).
    Keys,
}

/// A mouse event as the hook hears it, in desktop pixels, `t` in milliseconds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    Down { button: Button, x: f64, y: f64, t: u64 },
    Move { x: f64, y: f64 },
    Up { button: Button, x: f64, y: f64, t: u64 },
}

/// What the classifier asks for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Out {
    /// A press that may become a step: take its picture now (SMT-024: the screen as it
    /// was just before the click).
    Begin { id: u64, button: Button, x: f64, y: f64 },
    /// It is a step of this kind; `to` is where a drag ended.
    Finish {
        id: u64,
        kind: Kind,
        to: Option<(f64, f64)>,
    },
    /// The second press of a double click: the step `id` becomes a double click.
    Double { id: u64 },
}

/// Turns presses and releases into steps (SMT-024): a click, a right click, a double
/// click (one step) or a drag (from the press to the release).
#[derive(Debug, Clone)]
pub struct Classifier {
    /// The system's double-click time, ms, and its rectangle, px.
    double_ms: u64,
    double_px: f64,
    next: u64,
    down: Option<Press>,
    /// The last click's release, for a double click.
    last: Option<Press>,
    /// The second press of a double click is going on.
    second: Option<u64>,
}

#[derive(Debug, Clone, Copy)]
struct Press {
    id: u64,
    button: Button,
    x: f64,
    y: f64,
    t: u64,
    /// Farthest from the press, px.
    far: f64,
}

impl Classifier {
    pub fn new(double_ms: u64, double_px: f64) -> Classifier {
        Classifier {
            double_ms,
            double_px,
            next: 0,
            down: None,
            last: None,
            second: None,
        }
    }

    pub fn feed(&mut self, e: Event) -> Option<Out> {
        match e {
            Event::Down { button, x, y, t } => {
                if self.down.is_some() {
                    // A second button while one is held: ignored.
                    return None;
                }
                let double = self.last.filter(|l| {
                    l.button == Button::Left
                        && button == Button::Left
                        && t.saturating_sub(l.t) <= self.double_ms
                        && (x - l.x).abs() <= self.double_px / 2.0
                        && (y - l.y).abs() <= self.double_px / 2.0
                });
                self.last = None;
                if let Some(first) = double {
                    self.second = Some(first.id);
                    self.down = Some(Press {
                        id: first.id,
                        button,
                        x,
                        y,
                        t,
                        far: 0.0,
                    });
                    return None;
                }
                self.next += 1;
                let id = self.next;
                self.down = Some(Press {
                    id,
                    button,
                    x,
                    y,
                    t,
                    far: 0.0,
                });
                Some(Out::Begin { id, button, x, y })
            }
            Event::Move { x, y } => {
                if let Some(d) = &mut self.down {
                    d.far = d.far.max((x - d.x).hypot(y - d.y));
                }
                None
            }
            Event::Up { button, x, y, t } => {
                let d = self.down.filter(|d| d.button == button)?;
                self.down = None;
                let far = d.far.max((x - d.x).hypot(y - d.y));
                if let Some(first) = self.second.take() {
                    return Some(Out::Double { id: first });
                }
                if far > DRAG_PX {
                    return Some(Out::Finish {
                        id: d.id,
                        kind: Kind::Drag,
                        to: Some((x, y)),
                    });
                }
                self.last = Some(Press { t, ..d });
                let kind = match button {
                    Button::Left => Kind::Click,
                    Button::Right => Kind::Right,
                };
                Some(Out::Finish {
                    id: d.id,
                    kind,
                    to: None,
                })
            }
        }
    }
}

/// What the accessibility information says the click was on (SMT-025).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Role {
    Button,
    Tab,
    /// A menu of the menu bar.
    Menu,
    MenuItem,
    CheckBox,
    Radio,
    Field,
    Link,
    List,
    ListItem,
    Other,
}

/// What was clicked, as well as it is known.
#[derive(Debug, Clone, PartialEq)]
pub enum Target {
    /// From the accessibility information; `checked` is a check box's state before the
    /// click.
    Element {
        role: Role,
        name: String,
        checked: Option<bool>,
    },
    /// Only recognized text near the click.
    Text(String),
    Nothing,
}

/// A name cut to [`NAME_CHARS`] with "…", in bold.
fn bold(name: &str) -> String {
    let name = name.split_whitespace().collect::<Vec<_>>().join(" ");
    let cut: String = if name.chars().count() > NAME_CHARS {
        let mut s: String = name.chars().take(NAME_CHARS - 1).collect();
        s = s.trim_end().to_string();
        s.push('…');
        s
    } else {
        name
    };
    // A name with asterisks would break the bold marks.
    format!("**{}**", cut.replace("**", "*"))
}

fn quoted(lang: Lang, text: &str) -> String {
    let text: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let cut: String = if text.chars().count() > NAME_CHARS {
        let mut s: String = text.chars().take(NAME_CHARS - 1).collect();
        s.push('…');
        s
    } else {
        text
    };
    match lang {
        Lang::Es => format!("«{cut}»"),
        _ => format!("“{cut}”"),
    }
}

/// The sentence of a step (SMT-025), with the names in bold (`**…**`). `app` is the
/// application when it changed from the step before (and in the first step).
pub fn phrase(lang: Lang, kind: Kind, target: &Target, drop: Option<&Target>, app: Option<&str>) -> String {
    let t = |key: &str, name: &str| tr(lang, &format!("steps.phrase.{key}"), &[("name", name)]);
    let named = |target: &Target| match target {
        Target::Element { name, .. } if !name.trim().is_empty() => Some(bold(name)),
        Target::Text(s) if !s.trim().is_empty() => Some(quoted(lang, s)),
        _ => None,
    };
    let sentence = match kind {
        Kind::Drag => match (named(target), drop.and_then(named)) {
            (Some(from), Some(to)) => tr(lang, "steps.phrase.drag", &[("name", &from), ("to", &to)]),
            (Some(from), None) => t("drag_from", &from),
            _ => tr(lang, "steps.phrase.drag_here", &[]),
        },
        Kind::Right => match named(target) {
            Some(n) => t("right", &n),
            None => tr(lang, "steps.phrase.right_here", &[]),
        },
        Kind::Double => match named(target) {
            Some(n) => t("double", &n),
            None => tr(lang, "steps.phrase.double_here", &[]),
        },
        Kind::Keys => tr(lang, "steps.phrase.keys", &[]),
        Kind::Click => match target {
            Target::Element { role, name, checked } if !name.trim().is_empty() => {
                let n = bold(name);
                match role {
                    Role::Button => t("button", &n),
                    Role::Tab => t("tab", &n),
                    Role::Menu => t("menu", &n),
                    Role::MenuItem => t("menu_item", &n),
                    // Its state after the click: the opposite of before.
                    Role::CheckBox if *checked == Some(true) => t("uncheck", &n),
                    Role::CheckBox => t("check", &n),
                    Role::Radio => t("radio", &n),
                    Role::Field => t("field", &n),
                    Role::Link => t("link", &n),
                    Role::List => t("list", &n),
                    Role::ListItem => t("item", &n),
                    Role::Other => t("other", &n),
                }
            }
            Target::Text(s) if !s.trim().is_empty() => t("text", &quoted(lang, s)),
            _ => tr(lang, "steps.phrase.here", &[]),
        },
    };
    match app.filter(|a| !a.trim().is_empty()) {
        Some(a) => tr(
            lang,
            "steps.phrase.in_app",
            &[("app", &bold(a)), ("rest", &lower_first(&sentence))],
        ),
        None => sentence,
    }
}

/// The sentence of a keyboard shortcut step (SMT-024).
pub fn keys_phrase(lang: Lang, keys: &str, app: Option<&str>) -> String {
    let sentence = tr(lang, "steps.phrase.press", &[("keys", &format!("**{keys}**"))]);
    match app.filter(|a| !a.trim().is_empty()) {
        Some(a) => tr(
            lang,
            "steps.phrase.in_app",
            &[("app", &bold(a)), ("rest", &lower_first(&sentence))],
        ),
        None => sentence,
    }
}

/// What typing adds to the step before (SMT-024): never the text typed.
pub fn typed_phrase(lang: Lang, field: Option<&str>) -> String {
    match field.filter(|f| !f.trim().is_empty()) {
        Some(f) => tr(lang, "steps.phrase.typed_in", &[("name", &quoted(lang, f))]),
        None => tr(lang, "steps.phrase.typed", &[]),
    }
}

fn lower_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        // A name in bold or quotes keeps its case.
        Some(c) if c.is_uppercase() => c.to_lowercase().chain(chars).collect(),
        _ => s.to_string(),
    }
}

/// How a step's image is framed (SMT-026).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Framing {
    Zone,
    Window,
    Screen,
}

impl Framing {
    pub fn parse(s: &str) -> Framing {
        match s {
            "window" => Framing::Window,
            "screen" => Framing::Screen,
            _ => Framing::Zone,
        }
    }
}

type R = [f64; 4];

fn clamp_to(r: R, monitor: R) -> Option<R> {
    let x0 = r[0].max(monitor[0]);
    let y0 = r[1].max(monitor[1]);
    let x1 = (r[0] + r[2]).min(monitor[0] + monitor[2]);
    let y1 = (r[1] + r[3]).min(monitor[1] + monitor[3]);
    (x1 > x0 && y1 > y0).then_some([x0, y0, x1 - x0, y1 - y0])
}

/// The framing of a step's image on its monitor (SMT-026), both in desktop pixels:
/// the clicked element's container (the first of `containers`, innermost first, of at
/// least [`MIN_FRAME`] on each side) with a margin; else the window; else the monitor.
/// Returns monitor-local pixels.
pub fn frame(framing: Framing, containers: &[R], window: Option<R>, monitor: R) -> R {
    let local = |r: R| [r[0] - monitor[0], r[1] - monitor[1], r[2], r[3]];
    let whole = [0.0, 0.0, monitor[2], monitor[3]];
    let pick = match framing {
        Framing::Screen => None,
        Framing::Window => window.and_then(|w| clamp_to(w, monitor)),
        Framing::Zone => containers
            .iter()
            .filter_map(|c| clamp_to(*c, monitor))
            .find(|c| c[2] >= MIN_FRAME && c[3] >= MIN_FRAME)
            .map(|c| {
                [
                    c[0] - FRAME_MARGIN,
                    c[1] - FRAME_MARGIN,
                    c[2] + 2.0 * FRAME_MARGIN,
                    c[3] + 2.0 * FRAME_MARGIN,
                ]
            })
            .and_then(|c| clamp_to(c, monitor)),
    };
    match pick {
        Some(r) if r[2] >= MIN_FRAME && r[3] >= MIN_FRAME => local(r).map(f64::round),
        _ => whole,
    }
}

/// The pointer of a step (CAP-047): its picture's file and where its top left is, in
/// monitor pixels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StepPointer {
    pub file: String,
    pub at: [f64; 2],
    pub size: [u32; 2],
}

/// One step as the recorder keeps it on disk (`steps.jsonl`, ADR 0009). Positions in
/// pixels of its monitor's picture (`image`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Step {
    pub n: u32,
    pub kind: Kind,
    pub image: String,
    pub at: [f64; 2],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<[f64; 2]>,
    pub frame: [f64; 4],
    pub text: String,
    #[serde(default)]
    pub app: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pointer: Option<StepPointer>,
    pub time: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn down(x: f64, t: u64) -> Event {
        Event::Down {
            button: Button::Left,
            x,
            y: 10.0,
            t,
        }
    }
    fn up(x: f64, t: u64) -> Event {
        Event::Up {
            button: Button::Left,
            x,
            y: 10.0,
            t,
        }
    }

    /// SMT-024: a click, a right click, a drag; a tremor of 3 px is a click; a double
    /// click is one step.
    #[test]
    fn what_makes_a_step() {
        let mut c = Classifier::new(500, 4.0);
        assert!(matches!(c.feed(down(10.0, 0)), Some(Out::Begin { id: 1, .. })));
        c.feed(Event::Move { x: 12.0, y: 12.0 });
        assert_eq!(
            c.feed(up(13.0, 80)),
            Some(Out::Finish {
                id: 1,
                kind: Kind::Click,
                to: None
            })
        );
        // Long after: a right click.
        let r = c.feed(Event::Down {
            button: Button::Right,
            x: 50.0,
            y: 50.0,
            t: 2000,
        });
        assert!(matches!(
            r,
            Some(Out::Begin {
                id: 2,
                button: Button::Right,
                ..
            })
        ));
        let r = c.feed(Event::Up {
            button: Button::Right,
            x: 50.0,
            y: 50.0,
            t: 2100,
        });
        assert_eq!(
            r,
            Some(Out::Finish {
                id: 2,
                kind: Kind::Right,
                to: None
            })
        );
        // A drag.
        assert!(matches!(
            c.feed(down(100.0, 4000)),
            Some(Out::Begin { id: 3, .. })
        ));
        c.feed(Event::Move { x: 160.0, y: 10.0 });
        let r = c.feed(up(200.0, 4300));
        assert_eq!(
            r,
            Some(Out::Finish {
                id: 3,
                kind: Kind::Drag,
                to: Some((200.0, 10.0))
            })
        );
        // A double click: one step.
        assert!(matches!(
            c.feed(down(300.0, 6000)),
            Some(Out::Begin { id: 4, .. })
        ));
        c.feed(up(300.0, 6050));
        assert_eq!(c.feed(down(301.0, 6200)), None);
        assert_eq!(c.feed(up(301.0, 6250)), Some(Out::Double { id: 4 }));
        // Too slow for a double click: two steps.
        c.feed(down(400.0, 9000));
        c.feed(up(400.0, 9050));
        assert!(matches!(
            c.feed(down(400.0, 9800)),
            Some(Out::Begin { id: 6, .. })
        ));
    }

    /// SMT-025: the sentences, in Spanish and English, with the app when it changes.
    #[test]
    fn sentences() {
        let el = |role, name: &str| Target::Element {
            role,
            name: name.into(),
            checked: None,
        };
        assert_eq!(
            phrase(
                Lang::Es,
                Kind::Click,
                &el(Role::Menu, "Archivo"),
                None,
                Some("Bloc de notas")
            ),
            "En **Bloc de notas**, abrir el menú **Archivo**"
        );
        assert_eq!(
            phrase(
                Lang::Es,
                Kind::Click,
                &el(Role::MenuItem, "Guardar como..."),
                None,
                None
            ),
            "Elegir **Guardar como...** en el menú"
        );
        assert_eq!(
            phrase(Lang::En, Kind::Click, &el(Role::Button, "Save"), None, None),
            "Click the **Save** button"
        );
        assert_eq!(
            phrase(Lang::Es, Kind::Click, &Target::Text("Aceptar".into()), None, None),
            "Clic en «Aceptar»"
        );
        assert_eq!(
            phrase(Lang::Es, Kind::Click, &Target::Nothing, None, None),
            "Clic aquí"
        );
        assert_eq!(
            phrase(
                Lang::Es,
                Kind::Right,
                &el(Role::ListItem, "informe.docx"),
                None,
                None
            ),
            "Clic derecho en **informe.docx**"
        );
        assert_eq!(
            phrase(
                Lang::Es,
                Kind::Drag,
                &el(Role::ListItem, "informe.docx"),
                Some(&el(Role::ListItem, "Documentos")),
                None
            ),
            "Arrastrar **informe.docx** hasta **Documentos**"
        );
        let checked = Target::Element {
            role: Role::CheckBox,
            name: "Recordarme".into(),
            checked: Some(false),
        };
        assert_eq!(
            phrase(Lang::Es, Kind::Click, &checked, None, None),
            "Activar la casilla **Recordarme**"
        );
        let long = "x".repeat(80);
        let s = phrase(Lang::En, Kind::Click, &el(Role::Link, &long), None, None);
        assert!(s.contains(&format!("**{}…**", "x".repeat(59))), "{s}");
        assert_eq!(keys_phrase(Lang::Es, "Ctrl+S", None), "Pulsar **Ctrl+S**");
        assert_eq!(
            typed_phrase(Lang::Es, Some("Nombre")),
            " y escribir en el campo «Nombre»"
        );
    }

    /// SMT-026: the dialog clicked, with a margin; a small zone gives the whole monitor.
    #[test]
    fn framing() {
        let monitor = [1920.0, 0.0, 1920.0, 1080.0];
        let button = [2400.0, 500.0, 80.0, 30.0];
        let dialog = [2200.0, 300.0, 500.0, 300.0];
        let f = frame(
            Framing::Zone,
            &[button, dialog],
            Some([2000.0, 100.0, 1200.0, 900.0]),
            monitor,
        );
        assert_eq!(f, [256.0, 276.0, 548.0, 348.0]);
        // A taskbar icon: nothing big enough, the whole monitor.
        let icon = [2000.0, 1040.0, 48.0, 40.0];
        let bar = [1920.0, 1032.0, 1920.0, 48.0];
        assert_eq!(
            frame(Framing::Zone, &[icon, bar], None, monitor),
            [0.0, 0.0, 1920.0, 1080.0]
        );
        assert_eq!(
            frame(
                Framing::Window,
                &[],
                Some([2000.0, 100.0, 1200.0, 900.0]),
                monitor
            ),
            [80.0, 100.0, 1200.0, 900.0]
        );
        assert_eq!(
            frame(Framing::Screen, &[dialog], None, monitor),
            [0.0, 0.0, 1920.0, 1080.0]
        );
    }
}
