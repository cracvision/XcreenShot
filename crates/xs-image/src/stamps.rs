//! Stamps (EDT-031): a collection of vector pictures of our own design, found by name
//! and keywords in English and Spanish. A placed stamp carries its drawing (`art`), so a
//! `.xshot` shows it the same on a computer whose collection differs.
//!
//! Drawings use a small format: shapes in a box of `w` × `h` units, painted with `ink`
//! (the stamp's color, which monochrome stamps let the user change), `paper` (white) or
//! a fixed color.

use serde_json::{json, Map, Value};
use tiny_skia::{FillRule, LineCap, LineJoin, PathBuilder, Pixmap, PixmapPaint, Stroke};

use crate::color::Rgba;
use crate::render::{blend_layer, paint, turned};
use crate::shapes::{rect, Rect};
use crate::text::{self, Text, TextAlign, TextStyle};

/// A stamp of the collection.
pub struct Stamp {
    pub id: &'static str,
    pub category: &'static str,
    pub en: &'static str,
    pub es: &'static str,
    /// Extra words to find it by, both languages.
    pub keywords: &'static str,
    /// Its own color (`ink`), when it has one to change.
    pub ink: Option<&'static str>,
    w: f64,
    h: f64,
    shapes: fn(Lang) -> Vec<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    En,
    Es,
}

pub const CATEGORIES: [&str; 7] = [
    "indicators",
    "cursors",
    "keys",
    "devices",
    "interface",
    "labels",
    "arrows",
];

// --- shape helpers -----------------------------------------------------------

fn circle(cx: f64, cy: f64, r: f64, fill: &str) -> Value {
    json!({"k": "circle", "cx": cx, "cy": cy, "r": r, "fill": fill})
}
fn ring(cx: f64, cy: f64, r: f64, stroke: &str, sw: f64) -> Value {
    json!({"k": "circle", "cx": cx, "cy": cy, "r": r, "stroke": stroke, "sw": sw})
}
fn rrect(x: f64, y: f64, w: f64, h: f64, r: f64, fill: &str) -> Value {
    json!({"k": "rect", "x": x, "y": y, "w": w, "h": h, "r": r, "fill": fill})
}
fn rframe(x: f64, y: f64, w: f64, h: f64, r: f64, stroke: &str, sw: f64) -> Value {
    json!({"k": "rect", "x": x, "y": y, "w": w, "h": h, "r": r, "stroke": stroke, "sw": sw})
}
fn fill(d: &str, color: &str) -> Value {
    json!({"k": "path", "d": d, "fill": color})
}
fn line(d: &str, color: &str, sw: f64) -> Value {
    json!({"k": "path", "d": d, "stroke": color, "sw": sw})
}
fn label(x: f64, y: f64, size: f64, s: &str, color: &str) -> Value {
    json!({"k": "text", "x": x, "y": y, "size": size, "text": s, "fill": color})
}
fn tr(lang: Lang, en: &str, es: &str) -> String {
    match lang {
        Lang::En => en.to_string(),
        Lang::Es => es.to_string(),
    }
}

/// A key cap with a label.
fn key(text: &'static str) -> Vec<Value> {
    vec![
        rrect(4.0, 6.0, 112.0, 88.0, 14.0, "ink"),
        rrect(8.0, 8.0, 104.0, 76.0, 11.0, "paper"),
        label(60.0, 46.0, 30.0, text, "ink"),
    ]
}
fn arrow_key(d: &'static str) -> Vec<Value> {
    vec![
        rrect(4.0, 6.0, 92.0, 88.0, 14.0, "ink"),
        rrect(8.0, 8.0, 84.0, 76.0, 11.0, "paper"),
        fill(d, "ink"),
    ]
}
fn tag(lang: Lang, en: &str, es: &str) -> Vec<Value> {
    vec![
        rrect(2.0, 2.0, 196.0, 76.0, 38.0, "ink"),
        label(100.0, 40.0, 34.0, &tr(lang, en, es), "paper"),
    ]
}

macro_rules! stamp {
    ($id:expr, $cat:expr, $en:expr, $es:expr, $kw:expr, $ink:expr, $w:expr, $h:expr, $shapes:expr) => {
        Stamp {
            id: $id,
            category: $cat,
            en: $en,
            es: $es,
            keywords: $kw,
            ink: $ink,
            w: $w,
            h: $h,
            shapes: $shapes,
        }
    };
}

/// The collection, in the order it is shown.
pub fn collection() -> Vec<Stamp> {
    const GREEN: Option<&str> = Some("#2BA84A");
    const RED: Option<&str> = Some("#E5484D");
    const AMBER: Option<&str> = Some("#F5A623");
    const BLUE: Option<&str> = Some("#2D7FF9");
    const CORAL: Option<&str> = Some("#F2545B");
    const GREY: Option<&str> = Some("#3A3F4B");
    vec![
        // Indicators
        stamp!(
            "check",
            "indicators",
            "Check",
            "Marca",
            "ok yes done correct tick sí bien hecho correcto visto",
            GREEN,
            100.0,
            100.0,
            |_| vec![
                circle(50.0, 50.0, 48.0, "ink"),
                line("M27 52 L43 68 L74 34", "paper", 10.0),
            ]
        ),
        stamp!(
            "cross",
            "indicators",
            "Cross",
            "Cruz",
            "no wrong error close x mal error cerrar incorrecto",
            RED,
            100.0,
            100.0,
            |_| vec![
                circle(50.0, 50.0, 48.0, "ink"),
                line("M33 33 L67 67 M67 33 L33 67", "paper", 10.0),
            ]
        ),
        stamp!(
            "warning",
            "indicators",
            "Warning",
            "Aviso",
            "alert caution attention alerta cuidado atención advertencia peligro",
            AMBER,
            100.0,
            90.0,
            |_| vec![
                fill(
                    "M50 4 Q54 4 57 9 L97 78 Q99 86 91 86 L9 86 Q1 86 3 78 L43 9 Q46 4 50 4 Z",
                    "ink"
                ),
                line("M50 30 L50 56", "paper", 10.0),
                circle(50.0, 71.0, 5.5, "paper"),
            ]
        ),
        stamp!(
            "info",
            "indicators",
            "Information",
            "Información",
            "info help note ayuda nota",
            BLUE,
            100.0,
            100.0,
            |_| vec![
                circle(50.0, 50.0, 48.0, "ink"),
                circle(50.0, 28.0, 6.5, "paper"),
                line("M50 44 L50 74", "paper", 11.0),
            ]
        ),
        stamp!(
            "question",
            "indicators",
            "Question",
            "Pregunta",
            "help ask doubt ayuda duda interrogación",
            BLUE,
            100.0,
            100.0,
            |_| vec![
                circle(50.0, 50.0, 48.0, "ink"),
                line(
                    "M36 38 Q36 24 50 24 Q64 24 64 37 Q64 47 50 52 L50 60",
                    "paper",
                    9.0
                ),
                circle(50.0, 73.0, 5.5, "paper"),
            ]
        ),
        stamp!(
            "prohibited",
            "indicators",
            "Prohibited",
            "Prohibido",
            "forbidden not allowed stop no permitido",
            RED,
            100.0,
            100.0,
            |_| vec![
                ring(50.0, 50.0, 42.0, "ink", 10.0),
                line("M21 79 L79 21", "ink", 10.0),
            ]
        ),
        stamp!(
            "star",
            "indicators",
            "Star",
            "Estrella",
            "favorite best rating favorito mejor valoración",
            AMBER,
            100.0,
            96.0,
            |_| vec![fill(
                "M50 3 L62 36 L97 37 L69 58 L79 92 L50 72 L21 92 L31 58 L3 37 L38 36 Z",
                "ink"
            ),]
        ),
        stamp!(
            "heart",
            "indicators",
            "Heart",
            "Corazón",
            "love like amor me gusta",
            CORAL,
            100.0,
            90.0,
            |_| vec![fill(
                "M50 86 L12 48 Q-4 28 12 12 Q30 -2 50 20 Q70 -2 88 12 Q104 28 88 48 Z",
                "ink"
            ),]
        ),
        stamp!(
            "plus",
            "indicators",
            "Plus",
            "Más",
            "add new añadir nuevo sumar",
            GREEN,
            100.0,
            100.0,
            |_| vec![
                circle(50.0, 50.0, 48.0, "ink"),
                line("M50 26 L50 74 M26 50 L74 50", "paper", 10.0),
            ]
        ),
        stamp!(
            "minus",
            "indicators",
            "Minus",
            "Menos",
            "remove less quitar restar",
            RED,
            100.0,
            100.0,
            |_| vec![
                circle(50.0, 50.0, 48.0, "ink"),
                line("M26 50 L74 50", "paper", 10.0),
            ]
        ),
        stamp!(
            "idea",
            "indicators",
            "Idea",
            "Idea",
            "bulb light tip bombilla consejo luz",
            AMBER,
            80.0,
            100.0,
            |_| vec![
                fill(
                    "M40 4 Q74 4 74 38 Q74 54 60 66 L58 74 L22 74 L20 66 Q6 54 6 38 Q6 4 40 4 Z",
                    "ink"
                ),
                rrect(24.0, 78.0, 32.0, 8.0, 3.0, "ink"),
                rrect(28.0, 89.0, 24.0, 8.0, 3.0, "ink"),
            ]
        ),
        stamp!(
            "flag",
            "indicators",
            "Flag",
            "Bandera",
            "mark milestone goal meta hito marcar",
            CORAL,
            90.0,
            100.0,
            |_| vec![
                line("M12 6 L12 96", "ink", 8.0),
                fill("M16 8 L84 8 L68 30 L84 52 L16 52 Z", "ink"),
            ]
        ),
        stamp!(
            "pin",
            "indicators",
            "Location",
            "Ubicación",
            "pin place map lugar mapa chincheta",
            CORAL,
            80.0,
            100.0,
            |_| vec![
                fill("M40 98 Q8 60 8 36 Q8 4 40 4 Q72 4 72 36 Q72 60 40 98 Z", "ink"),
                circle(40.0, 36.0, 13.0, "paper"),
            ]
        ),
        stamp!(
            "clock",
            "indicators",
            "Clock",
            "Reloj",
            "time wait hour tiempo esperar hora",
            GREY,
            100.0,
            100.0,
            |_| vec![
                ring(50.0, 50.0, 43.0, "ink", 9.0),
                line("M50 24 L50 52 L68 62", "ink", 8.0),
            ]
        ),
        stamp!(
            "lock",
            "indicators",
            "Lock",
            "Candado",
            "locked secure private bloqueado seguro privado",
            GREY,
            80.0,
            100.0,
            |_| vec![
                line("M20 46 L20 30 Q20 8 40 8 Q60 8 60 30 L60 46", "ink", 9.0),
                rrect(6.0, 44.0, 68.0, 52.0, 8.0, "ink"),
                circle(40.0, 66.0, 7.0, "paper"),
            ]
        ),
        // Cursors and gestures
        stamp!(
            "pointer",
            "cursors",
            "Pointer",
            "Puntero",
            "cursor mouse arrow click flecha ratón",
            None,
            64.0,
            100.0,
            |_| vec![
                json!({"k": "path", "d": "M6 4 L6 80 L24 62 L36 92 L50 86 L38 57 L62 57 Z", "fill": "#FFFFFF", "stroke": "#000000", "sw": 5.0}),
            ]
        ),
        stamp!(
            "hand",
            "cursors",
            "Hand",
            "Mano",
            "link click finger enlace dedo pulsar",
            None,
            84.0,
            100.0,
            |_| vec![
                json!({"k": "path", "d": "M30 50 L30 12 Q30 4 37 4 Q44 4 44 12 L44 44 L44 36 Q44 30 51 30 Q58 30 58 36 L58 46 L58 40 Q58 34 65 34 Q72 34 72 40 L72 52 L72 46 Q72 42 77 42 Q82 42 82 48 L82 72 Q82 96 58 96 L44 96 Q30 96 22 82 L6 58 Q2 50 10 48 Q16 46 22 54 Z", "fill": "#FFFFFF", "stroke": "#000000", "sw": 5.0}),
            ]
        ),
        stamp!(
            "click",
            "cursors",
            "Click",
            "Clic",
            "tap press pulsar tocar",
            BLUE,
            100.0,
            100.0,
            |_| vec![
                circle(50.0, 50.0, 12.0, "ink"),
                ring(50.0, 50.0, 26.0, "ink", 6.0),
                ring(50.0, 50.0, 42.0, "ink", 4.0),
            ]
        ),
        stamp!(
            "drag",
            "cursors",
            "Move",
            "Mover",
            "drag arrows arrastrar flechas",
            GREY,
            100.0,
            100.0,
            |_| {
                vec![
            line("M50 10 L50 90 M10 50 L90 50", "ink", 8.0),
            fill("M50 2 L64 18 L36 18 Z M50 98 L64 82 L36 82 Z M2 50 L18 36 L18 64 Z M98 50 L82 36 L82 64 Z", "ink"),
        ]
            }
        ),
        stamp!(
            "textcursor",
            "cursors",
            "Text cursor",
            "Cursor de texto",
            "i-beam type escribir texto",
            GREY,
            50.0,
            100.0,
            |_| vec![line(
                "M10 6 Q25 6 25 18 L25 82 Q25 94 10 94 M40 6 Q25 6 25 18 M25 82 Q25 94 40 94",
                "ink",
                7.0
            ),]
        ),
        stamp!(
            "wait",
            "cursors",
            "Wait",
            "Espera",
            "hourglass loading busy reloj de arena cargando",
            GREY,
            70.0,
            100.0,
            |_| vec![
                line("M8 6 L62 6 M8 94 L62 94", "ink", 8.0),
                fill(
                    "M14 10 L56 10 Q56 34 35 50 Q14 34 14 10 Z M35 50 Q56 66 56 90 L14 90 Q14 66 35 50 Z",
                    "ink"
                ),
            ]
        ),
        // Keys
        stamp!(
            "key-ctrl",
            "keys",
            "Ctrl key",
            "Tecla Ctrl",
            "control keyboard teclado",
            GREY,
            120.0,
            100.0,
            |_| key("Ctrl")
        ),
        stamp!(
            "key-shift",
            "keys",
            "Shift key",
            "Tecla Mayús",
            "shift mayúsculas keyboard teclado",
            GREY,
            120.0,
            100.0,
            |l| {
                let mut k = key(if l == Lang::Es { "Mayús" } else { "Shift" });
                if let Some(t) = k.get_mut(2) {
                    t["size"] = json!(26.0);
                }
                k
            }
        ),
        stamp!(
            "key-alt",
            "keys",
            "Alt key",
            "Tecla Alt",
            "alt option keyboard teclado",
            GREY,
            120.0,
            100.0,
            |_| key("Alt")
        ),
        stamp!(
            "key-enter",
            "keys",
            "Enter key",
            "Tecla Intro",
            "enter return intro keyboard teclado",
            GREY,
            120.0,
            100.0,
            |l| key(if l == Lang::Es { "Intro" } else { "Enter" })
        ),
        stamp!(
            "key-esc",
            "keys",
            "Esc key",
            "Tecla Esc",
            "escape keyboard teclado",
            GREY,
            120.0,
            100.0,
            |_| key("Esc")
        ),
        stamp!(
            "key-tab",
            "keys",
            "Tab key",
            "Tecla Tab",
            "tab tabulador keyboard teclado",
            GREY,
            120.0,
            100.0,
            |_| key("Tab")
        ),
        stamp!(
            "key-del",
            "keys",
            "Delete key",
            "Tecla Supr",
            "delete supr borrar keyboard teclado",
            GREY,
            120.0,
            100.0,
            |l| key(if l == Lang::Es { "Supr" } else { "Del" })
        ),
        stamp!(
            "key-cmd",
            "keys",
            "Cmd key",
            "Tecla Cmd",
            "command mac keyboard teclado",
            GREY,
            120.0,
            100.0,
            |_| key("Cmd")
        ),
        stamp!(
            "key-space",
            "keys",
            "Space bar",
            "Barra espaciadora",
            "space espacio keyboard teclado",
            GREY,
            240.0,
            100.0,
            |l| vec![
                rrect(4.0, 6.0, 232.0, 88.0, 14.0, "ink"),
                rrect(8.0, 8.0, 224.0, 76.0, 11.0, "paper"),
                label(120.0, 46.0, 26.0, &tr(l, "Space", "Espacio"), "ink"),
            ]
        ),
        stamp!(
            "key-up",
            "keys",
            "Up arrow key",
            "Tecla flecha arriba",
            "arrow up flecha arriba keyboard teclado",
            GREY,
            100.0,
            100.0,
            |_| arrow_key("M50 22 L72 52 L58 52 L58 70 L42 70 L42 52 L28 52 Z")
        ),
        stamp!(
            "key-down",
            "keys",
            "Down arrow key",
            "Tecla flecha abajo",
            "arrow down flecha abajo keyboard teclado",
            GREY,
            100.0,
            100.0,
            |_| arrow_key("M50 72 L72 42 L58 42 L58 24 L42 24 L42 42 L28 42 Z")
        ),
        stamp!(
            "key-left",
            "keys",
            "Left arrow key",
            "Tecla flecha izquierda",
            "arrow left flecha izquierda keyboard teclado",
            GREY,
            100.0,
            100.0,
            |_| arrow_key("M24 46 L54 24 L54 38 L72 38 L72 54 L54 54 L54 68 Z")
        ),
        stamp!(
            "key-right",
            "keys",
            "Right arrow key",
            "Tecla flecha derecha",
            "arrow right flecha derecha keyboard teclado",
            GREY,
            100.0,
            100.0,
            |_| arrow_key("M76 46 L46 24 L46 38 L28 38 L28 54 L46 54 L46 68 Z")
        ),
        // Devices
        stamp!(
            "monitor",
            "devices",
            "Monitor",
            "Monitor",
            "screen display computer pantalla ordenador computadora",
            GREY,
            120.0,
            100.0,
            |_| vec![
                rrect(4.0, 4.0, 112.0, 72.0, 8.0, "ink"),
                rrect(12.0, 12.0, 96.0, 56.0, 3.0, "paper"),
                fill("M50 76 L70 76 L74 90 L46 90 Z", "ink"),
                rrect(34.0, 88.0, 52.0, 8.0, 4.0, "ink"),
            ]
        ),
        stamp!(
            "laptop",
            "devices",
            "Laptop",
            "Portátil",
            "notebook computer ordenador computadora",
            GREY,
            140.0,
            90.0,
            |_| vec![
                rrect(18.0, 4.0, 104.0, 66.0, 8.0, "ink"),
                rrect(26.0, 12.0, 88.0, 50.0, 3.0, "paper"),
                rrect(4.0, 72.0, 132.0, 12.0, 6.0, "ink"),
            ]
        ),
        stamp!(
            "phone",
            "devices",
            "Phone",
            "Teléfono",
            "mobile smartphone móvil celular",
            GREY,
            60.0,
            100.0,
            |_| vec![
                rrect(4.0, 2.0, 52.0, 96.0, 10.0, "ink"),
                rrect(9.0, 12.0, 42.0, 72.0, 3.0, "paper"),
                circle(30.0, 91.0, 3.5, "paper"),
            ]
        ),
        stamp!(
            "tablet",
            "devices",
            "Tablet",
            "Tableta",
            "ipad tablet tableta",
            GREY,
            90.0,
            110.0,
            |_| vec![
                rrect(4.0, 4.0, 82.0, 102.0, 10.0, "ink"),
                rrect(11.0, 12.0, 68.0, 82.0, 3.0, "paper"),
                circle(45.0, 100.0, 3.5, "paper"),
            ]
        ),
        stamp!(
            "mouse",
            "devices",
            "Mouse",
            "Ratón",
            "mouse click ratón clic",
            GREY,
            70.0,
            100.0,
            |_| vec![
                fill(
                    "M35 4 Q66 4 66 40 L66 64 Q66 96 35 96 Q4 96 4 64 L4 40 Q4 4 35 4 Z",
                    "ink"
                ),
                line("M35 10 L35 36 M8 40 L62 40", "paper", 4.0),
            ]
        ),
        stamp!(
            "keyboard",
            "devices",
            "Keyboard",
            "Teclado",
            "keys typing teclas escribir",
            GREY,
            140.0,
            70.0,
            |_| {
                let mut v = vec![rrect(2.0, 2.0, 136.0, 66.0, 10.0, "ink")];
                for row in 0..3 {
                    for col in 0..10 {
                        v.push(rrect(
                            10.0 + col as f64 * 12.4,
                            10.0 + row as f64 * 14.0,
                            9.0,
                            10.0,
                            2.0,
                            "paper",
                        ));
                    }
                }
                v.push(rrect(34.0, 52.0, 72.0, 9.0, 2.0, "paper"));
                v
            }
        ),
        // Interface
        stamp!(
            "button",
            "interface",
            "Button",
            "Botón",
            "ok click button aceptar botón",
            BLUE,
            160.0,
            64.0,
            |l| vec![
                rrect(2.0, 2.0, 156.0, 60.0, 14.0, "ink"),
                label(80.0, 32.0, 28.0, &tr(l, "OK", "Aceptar"), "paper"),
            ]
        ),
        stamp!(
            "toggle-on",
            "interface",
            "Switch on",
            "Interruptor activado",
            "toggle on enabled interruptor activado encendido",
            BLUE,
            120.0,
            64.0,
            |_| vec![
                rrect(2.0, 2.0, 116.0, 60.0, 30.0, "ink"),
                circle(86.0, 32.0, 22.0, "paper"),
            ]
        ),
        stamp!(
            "toggle-off",
            "interface",
            "Switch off",
            "Interruptor desactivado",
            "toggle off disabled interruptor desactivado apagado",
            None,
            120.0,
            64.0,
            |_| vec![
                rrect(2.0, 2.0, 116.0, 60.0, 30.0, "#B9BEC8"),
                circle(34.0, 32.0, 22.0, "#FFFFFF"),
            ]
        ),
        stamp!(
            "checkbox-on",
            "interface",
            "Checked box",
            "Casilla marcada",
            "checkbox checked tick casilla marcada",
            BLUE,
            100.0,
            100.0,
            |_| vec![
                rrect(4.0, 4.0, 92.0, 92.0, 16.0, "ink"),
                line("M26 52 L42 68 L74 34", "paper", 10.0),
            ]
        ),
        stamp!(
            "checkbox-off",
            "interface",
            "Empty box",
            "Casilla vacía",
            "checkbox unchecked casilla vacía",
            GREY,
            100.0,
            100.0,
            |_| vec![rframe(8.0, 8.0, 84.0, 84.0, 14.0, "ink", 8.0),]
        ),
        stamp!(
            "radio-on",
            "interface",
            "Selected option",
            "Opción elegida",
            "radio button selected opción elegida",
            BLUE,
            100.0,
            100.0,
            |_| vec![
                ring(50.0, 50.0, 42.0, "ink", 8.0),
                circle(50.0, 50.0, 22.0, "ink"),
            ]
        ),
        stamp!(
            "radio-off",
            "interface",
            "Option",
            "Opción",
            "radio button option opción",
            GREY,
            100.0,
            100.0,
            |_| vec![ring(50.0, 50.0, 42.0, "ink", 8.0),]
        ),
        stamp!(
            "slider",
            "interface",
            "Slider",
            "Control deslizante",
            "slider range deslizador volumen",
            BLUE,
            160.0,
            50.0,
            |_| vec![
                rrect(4.0, 20.0, 152.0, 10.0, 5.0, "#C9CDD6"),
                rrect(4.0, 20.0, 96.0, 10.0, 5.0, "ink"),
                circle(100.0, 25.0, 20.0, "ink"),
                circle(100.0, 25.0, 9.0, "paper"),
            ]
        ),
        // Labels
        stamp!(
            "label-new",
            "labels",
            "New",
            "Nuevo",
            "new label etiqueta nuevo",
            BLUE,
            200.0,
            80.0,
            |l| tag(l, "New", "Nuevo")
        ),
        stamp!(
            "label-important",
            "labels",
            "Important",
            "Importante",
            "important label etiqueta importante",
            RED,
            200.0,
            80.0,
            |l| tag(l, "Important", "Importante")
        ),
        stamp!(
            "label-draft",
            "labels",
            "Draft",
            "Borrador",
            "draft label etiqueta borrador",
            GREY,
            200.0,
            80.0,
            |l| tag(l, "Draft", "Borrador")
        ),
        stamp!(
            "label-approved",
            "labels",
            "Approved",
            "Aprobado",
            "approved ok label etiqueta aprobado",
            GREEN,
            200.0,
            80.0,
            |l| tag(l, "Approved", "Aprobado")
        ),
        stamp!(
            "label-beta",
            "labels",
            "Beta",
            "Beta",
            "beta test label etiqueta prueba",
            Some("#8B5CF6"),
            200.0,
            80.0,
            |_| tag(Lang::En, "Beta", "Beta")
        ),
        stamp!(
            "label-todo",
            "labels",
            "To do",
            "Pendiente",
            "todo pending label etiqueta pendiente",
            AMBER,
            200.0,
            80.0,
            |l| tag(l, "To do", "Pendiente")
        ),
        stamp!(
            "label-done",
            "labels",
            "Done",
            "Hecho",
            "done finished label etiqueta hecho terminado",
            GREEN,
            200.0,
            80.0,
            |l| tag(l, "Done", "Hecho")
        ),
        // Decorative arrows
        stamp!(
            "arrow-right",
            "arrows",
            "Arrow",
            "Flecha",
            "arrow right next flecha derecha siguiente",
            CORAL,
            120.0,
            80.0,
            |_| vec![fill("M4 28 L70 28 L70 6 L116 40 L70 74 L70 52 L4 52 Z", "ink"),]
        ),
        stamp!(
            "arrow-double",
            "arrows",
            "Double arrow",
            "Flecha doble",
            "both ways swap ambos sentidos intercambiar",
            CORAL,
            140.0,
            70.0,
            |_| vec![fill(
                "M4 35 L36 6 L36 24 L104 24 L104 6 L136 35 L104 64 L104 46 L36 46 L36 64 Z",
                "ink"
            ),]
        ),
        stamp!(
            "arrow-curved",
            "arrows",
            "Curved arrow",
            "Flecha curva",
            "curve turn curva girar",
            CORAL,
            110.0,
            100.0,
            |_| vec![
                line("M12 90 Q12 30 70 30", "ink", 14.0),
                fill("M66 6 L106 30 L66 54 Z", "ink"),
            ]
        ),
        stamp!(
            "arrow-refresh",
            "arrows",
            "Refresh",
            "Actualizar",
            "reload repeat cycle recargar repetir ciclo",
            CORAL,
            100.0,
            100.0,
            |_| vec![
                line(
                    "M84 50 Q84 84 50 84 Q16 84 16 50 Q16 16 50 16 L62 16",
                    "ink",
                    11.0
                ),
                fill("M58 0 L84 16 L58 32 Z", "ink"),
            ]
        ),
        stamp!(
            "arrow-undo",
            "arrows",
            "Back arrow",
            "Flecha atrás",
            "undo back return deshacer volver atrás",
            CORAL,
            110.0,
            90.0,
            |_| vec![
                line("M30 30 L70 30 Q100 30 100 58 Q100 86 70 86 L40 86", "ink", 12.0),
                fill("M4 30 L36 6 L36 54 Z", "ink"),
            ]
        ),
    ]
}

/// A stamp of the collection by id.
pub fn find(id: &str) -> Option<Stamp> {
    collection().into_iter().find(|s| s.id == id)
}

impl Stamp {
    /// Its drawing in `lang`, to place in a document.
    pub fn art(&self, lang: Lang) -> Value {
        json!({"w": self.w, "h": self.h, "shapes": (self.shapes)(lang)})
    }

    pub fn aspect(&self) -> f64 {
        self.w / self.h
    }

    /// Whether `query` finds it: every word in its names or keywords.
    pub fn matches(&self, query: &str) -> bool {
        let hay = format!("{} {} {} {}", self.id, self.en, self.es, self.keywords).to_lowercase();
        let hay = fold(&hay);
        fold(&query.to_lowercase())
            .split_whitespace()
            .all(|w| hay.contains(w))
    }
}

/// Without accents, so "informacion" finds "información".
fn fold(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'á' | 'à' | 'ä' => 'a',
            'é' | 'è' | 'ë' => 'e',
            'í' | 'ì' | 'ï' => 'i',
            'ó' | 'ò' | 'ö' => 'o',
            'ú' | 'ù' | 'ü' => 'u',
            'ñ' => 'n',
            c => c,
        })
        .collect()
}

// --- drawing -------------------------------------------------------------------

/// A path in a small SVG-like syntax: `M x y`, `L x y`, `Q cx cy x y`, `Z`.
fn parse_path(d: &str) -> Option<tiny_skia::Path> {
    let mut pb = PathBuilder::new();
    let mut it = d.split_whitespace().peekable();
    let mut cmd = 'M';
    let num = |s: Option<&str>| s.and_then(|v| v.parse::<f32>().ok());
    while it.peek().is_some() {
        let tok = *it.peek()?;
        if let Some(c) = tok.chars().next().filter(|c| c.is_ascii_alphabetic()) {
            cmd = c;
            let rest = &tok[1..];
            it.next();
            if cmd == 'Z' {
                pb.close();
                continue;
            }
            if !rest.is_empty() {
                // "M27" style: the number follows the letter.
                let x = rest.parse::<f32>().ok()?;
                let y = num(it.next())?;
                match cmd {
                    'M' => pb.move_to(x, y),
                    'L' => pb.line_to(x, y),
                    'Q' => {
                        let (ex, ey) = (num(it.next())?, num(it.next())?);
                        pb.quad_to(x, y, ex, ey)
                    }
                    _ => return None,
                }
                continue;
            }
        }
        let x = num(it.next())?;
        let y = num(it.next())?;
        match cmd {
            'M' => {
                pb.move_to(x, y);
                cmd = 'L';
            }
            'L' => pb.line_to(x, y),
            'Q' => {
                let (ex, ey) = (num(it.next())?, num(it.next())?);
                pb.quad_to(x, y, ex, ey);
            }
            _ => return None,
        }
    }
    pb.finish()
}

fn color_of(v: Option<&Value>, ink: Rgba) -> Option<Rgba> {
    match v?.as_str()? {
        "ink" => Some(ink),
        "paper" => Some(Rgba::WHITE),
        c => Rgba::parse(c),
    }
}

/// A placed stamp (EDT-031): its box, turn, drawing and style.
#[derive(Debug, Clone, PartialEq)]
pub struct StampObject {
    pub rect: Rect,
    pub rotation: f64,
    pub art: Value,
    /// The stamp's own color when `None`.
    pub color: Option<String>,
    pub ink: String,
    pub opacity: f64,
}

impl StampObject {
    pub fn from_props(props: &Map<String, Value>) -> Result<StampObject, String> {
        let art = props.get("art").cloned().ok_or("art is required")?;
        let size_ok =
            art["w"].as_f64().is_some_and(|w| w > 0.0) && art["h"].as_f64().is_some_and(|h| h > 0.0);
        if !size_ok || !art["shapes"].is_array() {
            return Err("art must have w, h and shapes".to_string());
        }
        let style = props.get("style").cloned().unwrap_or(json!({}));
        let color = style.get("color").and_then(Value::as_str).map(str::to_string);
        if let Some(c) = &color {
            Rgba::parse(c).ok_or("color must be a color like #RRGGBB")?;
        }
        let opacity = style.get("opacity").and_then(Value::as_f64).unwrap_or(100.0);
        if !(0.0..=100.0).contains(&opacity) {
            return Err("opacity must be between 0 and 100".to_string());
        }
        Ok(StampObject {
            rect: rect(props.get("rect"))?,
            rotation: props.get("rotation").and_then(Value::as_f64).unwrap_or(0.0),
            ink: props
                .get("ink")
                .and_then(Value::as_str)
                .unwrap_or("#3A3F4B")
                .to_string(),
            art,
            color,
            opacity,
        })
    }
}

/// Draws a stamp's drawing stretched over its box and turned.
pub fn draw(pm: &mut Pixmap, s: &StampObject, dpi: f64) {
    let ink = s
        .color
        .as_deref()
        .and_then(Rgba::parse)
        .or_else(|| Rgba::parse(&s.ink))
        .unwrap_or(Rgba::WHITE);
    let (aw, ah) = (
        s.art["w"].as_f64().unwrap_or(1.0),
        s.art["h"].as_f64().unwrap_or(1.0),
    );
    let (kx, ky) = (s.rect.w / aw, s.rect.h / ah);
    let to = turned(s.rect, s.rotation)
        .pre_translate(s.rect.x as f32, s.rect.y as f32)
        .pre_scale(kx as f32, ky as f32);
    let shapes = s.art["shapes"].as_array().cloned().unwrap_or_default();
    blend_layer(pm, s.opacity, |pm| {
        for sh in &shapes {
            let n = |k: &str| sh.get(k).and_then(Value::as_f64).unwrap_or(0.0) as f32;
            let fill = color_of(sh.get("fill"), ink);
            let stroke = color_of(sh.get("stroke"), ink);
            let sw = n("sw");
            let path = match sh["k"].as_str() {
                Some("circle") => PathBuilder::from_circle(n("cx"), n("cy"), n("r").max(0.01)),
                Some("rect") => {
                    let (x, y, w, h) = (n("x"), n("y"), n("w").max(0.01), n("h").max(0.01));
                    let q = n("r").clamp(0.0, w.min(h) / 2.0);
                    if q <= 0.0 {
                        tiny_skia::Rect::from_xywh(x, y, w, h).map(PathBuilder::from_rect)
                    } else {
                        let k = 0.552_284_8 * q;
                        let mut pb = PathBuilder::new();
                        pb.move_to(x + q, y);
                        pb.line_to(x + w - q, y);
                        pb.cubic_to(x + w - q + k, y, x + w, y + q - k, x + w, y + q);
                        pb.line_to(x + w, y + h - q);
                        pb.cubic_to(x + w, y + h - q + k, x + w - q + k, y + h, x + w - q, y + h);
                        pb.line_to(x + q, y + h);
                        pb.cubic_to(x + q - k, y + h, x, y + h - q + k, x, y + h - q);
                        pb.line_to(x, y + q);
                        pb.cubic_to(x, y + q - k, x + q - k, y, x + q, y);
                        pb.close();
                        pb.finish()
                    }
                }
                Some("path") => sh["d"].as_str().and_then(parse_path),
                Some("text") => {
                    // Letters at the size they will show, centered on (x, y).
                    let (Some(color), Some(t)) = (fill, sh["text"].as_str()) else {
                        continue;
                    };
                    let k = (kx * ky).sqrt();
                    let px = n("size") as f64 * k;
                    let text = Text {
                        at: [0.0, 0.0],
                        width: None,
                        text: t.to_string(),
                        rotation: 0.0,
                        style: TextStyle {
                            shadow: None,
                            font: text::default_font().to_string(),
                            size: (px * 72.0 / dpi.max(1.0)).clamp(1.0, 1000.0),
                            bold: true,
                            italic: false,
                            color: format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b),
                            align: TextAlign::Center,
                            opacity: 100.0,
                            markup: false,
                        },
                    };
                    if let Some((raster, _)) = text::rasterize(&text, dpi) {
                        let c = (n("x") as f64 * kx + s.rect.x, n("y") as f64 * ky + s.rect.y);
                        let at = turned(s.rect, s.rotation).pre_translate(
                            (c.0 - raster.width() as f64 / 2.0) as f32,
                            (c.1 - raster.height() as f64 / 2.0) as f32,
                        );
                        let p = PixmapPaint {
                            quality: tiny_skia::FilterQuality::Bilinear,
                            ..PixmapPaint::default()
                        };
                        pm.draw_pixmap(0, 0, raster.as_ref(), &p, at, None);
                    }
                    continue;
                }
                _ => None,
            };
            let Some(path) = path else {
                continue;
            };
            if let Some(c) = fill {
                pm.fill_path(&path, &paint(c), FillRule::Winding, to, None);
            }
            if let (Some(c), true) = (stroke, sw > 0.0) {
                let st = Stroke {
                    width: sw,
                    line_cap: LineCap::Round,
                    line_join: LineJoin::Round,
                    ..Stroke::default()
                };
                pm.stroke_path(&path, &paint(c), &st, to, None);
            }
        }
    });
}

/// A stamp of the collection drawn in a `size` px square, for the collection's grid.
pub fn thumbnail(id: &str, lang: Lang, size: u32) -> Option<image::RgbaImage> {
    let st = find(id)?;
    let k = (size as f64 - 4.0) / st.w.max(st.h);
    let (w, h) = (st.w * k, st.h * k);
    let obj = StampObject {
        rect: Rect {
            x: (size as f64 - w) / 2.0,
            y: (size as f64 - h) / 2.0,
            w,
            h,
        },
        rotation: 0.0,
        art: st.art(lang),
        color: None,
        ink: st.ink.unwrap_or("#3A3F4B").to_string(),
        opacity: 100.0,
    };
    let mut pm = Pixmap::new(size, size)?;
    draw(&mut pm, &obj, 96.0);
    let mut out = Vec::with_capacity(pm.data().len());
    for px in pm.pixels() {
        let c = px.demultiply();
        out.extend_from_slice(&[c.red(), c.green(), c.blue(), c.alpha()]);
    }
    image::RgbaImage::from_raw(size, size, out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn found_by_words_in_either_language() {
        let all = collection();
        let warning: Vec<&str> = all
            .iter()
            .filter(|s| s.matches("warning"))
            .map(|s| s.id)
            .collect();
        let aviso: Vec<&str> = all.iter().filter(|s| s.matches("aviso")).map(|s| s.id).collect();
        assert_eq!(warning, ["warning"]);
        assert_eq!(aviso, ["warning"]);
        assert!(find("info").unwrap().matches("informacion"), "without the accent");
    }

    #[test]
    fn every_stamp_draws_and_ids_are_unique() {
        let all = collection();
        let mut ids: Vec<&str> = all.iter().map(|s| s.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), all.len());
        for s in &all {
            assert!(CATEGORIES.contains(&s.category), "{}", s.id);
            let img = thumbnail(s.id, Lang::Es, 48).unwrap();
            assert!(img.pixels().any(|p| p.0[3] > 0), "{} draws nothing", s.id);
            for sh in s.art(Lang::En)["shapes"].as_array().unwrap() {
                if let Some(d) = sh["d"].as_str() {
                    assert!(parse_path(d).is_some(), "{}: {d}", s.id);
                }
            }
        }
    }
}
