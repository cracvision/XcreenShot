//! Cursor and keyboard effects in the video (VID-045 to VID-047): what to draw, and
//! how it changes over time. The drawing itself is per platform; this part only
//! decides, so it can be tested without a screen.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// A colour, 0–255 per channel.
pub type Rgb = [u8; 3];

/// Highlight cursor (VID-045): a circle or ring under the pointer.
#[derive(Debug, Clone, PartialEq)]
pub struct Highlight {
    pub color: Rgb,
    /// Diameter in logical pixels (20 to 120).
    pub diameter: f32,
    /// 0.1 to 0.9.
    pub opacity: f32,
    /// Only the outline, 4 px wide.
    pub ring: bool,
}

/// Click animation (VID-046).
#[derive(Debug, Clone, PartialEq)]
pub struct Clicks {
    pub primary: Rgb,
    pub secondary: Rgb,
    /// Final diameter of the wave, logical pixels (40, 64 or 96).
    pub diameter: f32,
}

/// Where the keys are shown, inside the area (VID-047).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyPlace {
    Center,
    Left,
    Right,
}

/// Show keys pressed (VID-047).
#[derive(Debug, Clone, PartialEq)]
pub struct Keys {
    /// Every key; otherwise only combinations and special keys.
    pub all: bool,
    pub place: KeyPlace,
    /// Text height in logical pixels.
    pub size: f32,
    /// Combinations never shown: the recording's own shortcuts, as
    /// [`normalize`] writes them.
    pub hidden: Vec<String>,
}

/// The effects of a recording; each can change while it records.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Effects {
    pub highlight: Option<Highlight>,
    pub clicks: Option<Clicks>,
    pub keys: Option<Keys>,
}

impl Effects {
    pub fn any(&self) -> bool {
        self.highlight.is_some() || self.clicks.is_some() || self.keys.is_some()
    }
}

/// `#RRGGBB` as a colour.
pub fn parse_color(s: &str) -> Option<Rgb> {
    let h = s.trim().strip_prefix('#')?;
    if h.len() != 6 {
        return None;
    }
    let n = u32::from_str_radix(h, 16).ok()?;
    Some([(n >> 16) as u8, (n >> 8) as u8, n as u8])
}

/// A combination written the same way whatever its spelling: lower case, no spaces,
/// `+` between keys ("Ctrl + Shift + F9" and "ctrl+shift+f9" are the same).
pub fn normalize(combo: &str) -> String {
    combo
        .split('+')
        .map(|k| k.trim().to_ascii_lowercase())
        .filter(|k| !k.is_empty())
        .collect::<Vec<_>>()
        .join("+")
}

/// Modifier keys held down.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Mods {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub win: bool,
}

/// Windows virtual-key codes of the modifiers.
pub fn is_modifier(vk: u32) -> bool {
    matches!(vk, 0x10..=0x12 | 0x5B | 0x5C | 0xA0..=0xA5)
}

/// The name of a key on its label, and whether it is "special" (shown even without a
/// modifier): Enter, Esc, Tab, arrows and function keys (VID-047).
pub fn key_name(vk: u32) -> Option<(String, bool)> {
    let special = |s: &str| Some((s.to_string(), true));
    match vk {
        0x0D => special("Enter"),
        0x1B => special("Esc"),
        0x09 => special("Tab"),
        0x25 => special("←"),
        0x26 => special("↑"),
        0x27 => special("→"),
        0x28 => special("↓"),
        0x70..=0x87 => Some((format!("F{}", vk - 0x6F), true)),
        0x08 => Some(("Backspace".into(), false)),
        0x2E => Some(("Delete".into(), false)),
        0x20 => Some(("Space".into(), false)),
        0x24 => Some(("Home".into(), false)),
        0x23 => Some(("End".into(), false)),
        0x21 => Some(("PgUp".into(), false)),
        0x22 => Some(("PgDn".into(), false)),
        0x2D => Some(("Insert".into(), false)),
        0x2C => Some(("PrtScn".into(), false)),
        0x30..=0x39 | 0x41..=0x5A => Some(((vk as u8 as char).to_string(), false)),
        0x60..=0x69 => Some(((b'0' + (vk - 0x60) as u8) as char).to_string()).map(|s| (s, false)),
        0xBA => Some((";".into(), false)),
        0xBB => Some(("=".into(), false)),
        0xBC => Some((",".into(), false)),
        0xBD => Some(("-".into(), false)),
        0xBE => Some((".".into(), false)),
        0xBF => Some(("/".into(), false)),
        0xC0 => Some(("`".into(), false)),
        0xDB => Some(("[".into(), false)),
        0xDC => Some(("\\".into(), false)),
        0xDD => Some(("]".into(), false)),
        0xDE => Some(("'".into(), false)),
        _ => None,
    }
}

/// The label of a key pressed with `mods`, or `None` when it is not shown: with
/// "only combinations", plain typing (letters, digits, Shift + letter) is left out
/// (VID-047).
pub fn label(vk: u32, mods: Mods, all: bool) -> Option<String> {
    if is_modifier(vk) {
        return None;
    }
    let (name, special) = key_name(vk)?;
    let combo = mods.ctrl || mods.alt || mods.win;
    if combo || special {
        let mut parts: Vec<&str> = Vec::new();
        if mods.ctrl {
            parts.push("Ctrl");
        }
        if mods.alt {
            parts.push("Alt");
        }
        if mods.shift {
            parts.push("Shift");
        }
        if mods.win {
            parts.push("Win");
        }
        parts.push(&name);
        return Some(parts.join(" + "));
    }
    if !all {
        return None;
    }
    // Plain typing: the character, a capital one with Shift.
    Some(if name.len() == 1 && !mods.shift {
        name.to_lowercase()
    } else {
        name
    })
}

/// How long a click's wave lasts (VID-046).
pub const WAVE: Duration = Duration::from_millis(400);
/// How long a key label stays (VID-047).
pub const LABEL: Duration = Duration::from_millis(1500);
/// A button held this long is a drag: a dot instead of waves.
pub const HOLD: Duration = Duration::from_millis(250);

/// A click's wave: where, when, which button.
#[derive(Debug, Clone, PartialEq)]
pub struct Wave {
    pub x: i32,
    pub y: i32,
    pub at: Instant,
    pub secondary: bool,
}

/// What the effects show over time: waves, a held button, key labels.
#[derive(Debug, Default)]
pub struct Live {
    pub waves: VecDeque<Wave>,
    /// A button down since then (secondary?), for the drag dot.
    pub held: Option<(Instant, bool)>,
    /// Labels and when each goes away; at most 3 (VID-047).
    pub labels: VecDeque<(String, Instant)>,
    pub mods: Mods,
}

impl Live {
    /// A button went down or up at (`x`, `y`).
    pub fn button(&mut self, down: bool, secondary: bool, x: i32, y: i32, at: Instant) {
        if down {
            self.waves.push_back(Wave { x, y, at, secondary });
            self.held = Some((at, secondary));
        } else {
            self.held = None;
        }
    }

    /// A key went down or up; `shown` decides whether its label appears (passwords,
    /// the recording's own shortcuts).
    pub fn key(&mut self, vk: u32, down: bool, keys: &Keys, at: Instant, shown: &dyn Fn() -> bool) {
        let m = &mut self.mods;
        let set = |flag: &mut bool| *flag = down;
        match vk {
            0x11 | 0xA2 | 0xA3 => set(&mut m.ctrl),
            0x12 | 0xA4 | 0xA5 => set(&mut m.alt),
            0x10 | 0xA0 | 0xA1 => set(&mut m.shift),
            0x5B | 0x5C => set(&mut m.win),
            _ => {}
        }
        if !down {
            return;
        }
        let Some(text) = label(vk, self.mods, keys.all) else {
            return;
        };
        if keys.hidden.contains(&normalize(&text)) || !shown() {
            return;
        }
        // The same keys again: the label stays longer.
        if let Some(last) = self.labels.back_mut() {
            if last.0 == text {
                last.1 = at + LABEL;
                return;
            }
        }
        self.labels.push_back((text, at + LABEL));
        while self.labels.len() > 3 {
            self.labels.pop_front();
        }
    }

    /// Forgets what is over at `now`; true while something still shows (frames must
    /// keep coming).
    pub fn tick(&mut self, now: Instant) -> bool {
        while self
            .waves
            .front()
            .is_some_and(|w| now.duration_since(w.at) >= WAVE)
        {
            self.waves.pop_front();
        }
        self.labels.retain(|(_, until)| *until > now);
        !self.waves.is_empty() || !self.labels.is_empty() || self.dragging(now).is_some()
    }

    /// A button held long enough to be a drag: its colour (secondary?).
    pub fn dragging(&self, now: Instant) -> Option<bool> {
        self.held
            .filter(|(since, _)| now.duration_since(*since) >= HOLD)
            .map(|(_, secondary)| secondary)
    }
}

/// How far a wave is through, 0 to 1, eased out (fast first, then slow).
pub fn wave_progress(age: Duration) -> f32 {
    let t = (age.as_secs_f32() / WAVE.as_secs_f32()).clamp(0.0, 1.0);
    1.0 - (1.0 - t) * (1.0 - t)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(all: bool) -> Keys {
        Keys {
            all,
            place: KeyPlace::Center,
            size: 18.0,
            hidden: vec![normalize("Ctrl+Shift+F9")],
        }
    }

    #[test]
    fn only_combinations_leaves_typing_out() {
        let none = Mods::default();
        let ctrl = Mods { ctrl: true, ..none };
        let shift = Mods { shift: true, ..none };
        assert_eq!(label(0x43, ctrl, false).as_deref(), Some("Ctrl + C"));
        assert_eq!(label(0x48, none, false), None, "typing a letter");
        assert_eq!(label(0x48, shift, false), None, "a capital letter");
        assert_eq!(label(0x0D, none, false).as_deref(), Some("Enter"));
        assert_eq!(label(0x73, none, false).as_deref(), Some("F4"));
        assert_eq!(
            label(
                0x53,
                Mods {
                    ctrl: true,
                    shift: true,
                    ..none
                },
                false
            )
            .as_deref(),
            Some("Ctrl + Shift + S")
        );
        assert_eq!(label(0x11, ctrl, false), None, "a modifier alone");
    }

    #[test]
    fn all_keys_shows_typing() {
        let none = Mods::default();
        let shift = Mods { shift: true, ..none };
        assert_eq!(label(0x48, none, true).as_deref(), Some("h"));
        assert_eq!(label(0x48, shift, true).as_deref(), Some("H"));
        assert_eq!(label(0x31, none, true).as_deref(), Some("1"));
    }

    #[test]
    fn labels_stack_up_to_three_and_fade() {
        let t0 = Instant::now();
        let k = keys(true);
        let mut live = Live::default();
        for (i, vk) in [0x48u32, 0x4F, 0x4C, 0x41].iter().enumerate() {
            live.key(*vk, true, &k, t0 + Duration::from_millis(100 * i as u64), &|| {
                true
            });
            live.key(
                *vk,
                false,
                &k,
                t0 + Duration::from_millis(100 * i as u64 + 50),
                &|| true,
            );
        }
        let shown: Vec<&str> = live.labels.iter().map(|(t, _)| t.as_str()).collect();
        assert_eq!(shown, ["o", "l", "a"]);
        assert!(live.tick(t0 + Duration::from_millis(1000)));
        assert!(
            !live.tick(t0 + Duration::from_millis(2000)),
            "all gone after 1.5 s"
        );
    }

    #[test]
    fn passwords_and_own_shortcuts_show_nothing() {
        let t0 = Instant::now();
        let k = keys(true);
        let mut live = Live::default();
        live.key(0x48, true, &k, t0, &|| false);
        assert!(live.labels.is_empty(), "a password field");
        live.key(0x11, true, &k, t0, &|| true);
        live.key(0x10, true, &k, t0, &|| true);
        live.key(0x78, true, &k, t0, &|| true);
        assert!(live.labels.is_empty(), "Ctrl+Shift+F9 is the recording's own");
        live.key(0x10, false, &k, t0, &|| true);
        live.key(0x43, true, &k, t0, &|| true);
        assert_eq!(live.labels.back().unwrap().0, "Ctrl + C");
    }

    #[test]
    fn waves_fade_and_a_hold_is_a_drag() {
        let t0 = Instant::now();
        let mut live = Live::default();
        live.button(true, false, 10, 20, t0);
        live.button(false, false, 10, 20, t0 + Duration::from_millis(80));
        live.button(true, true, 10, 20, t0 + Duration::from_millis(100));
        assert_eq!(live.waves.len(), 2);
        assert!(live.tick(t0 + Duration::from_millis(300)));
        assert_eq!(live.dragging(t0 + Duration::from_millis(300)), None);
        assert_eq!(live.dragging(t0 + Duration::from_millis(400)), Some(true));
        live.button(false, true, 10, 20, t0 + Duration::from_millis(450));
        assert!(!live.tick(t0 + Duration::from_millis(600)), "the waves are over");
        assert!((wave_progress(Duration::ZERO)).abs() < 1e-6);
        assert!((wave_progress(WAVE) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn colors_and_combinations_read() {
        assert_eq!(parse_color("#FFD000"), Some([255, 208, 0]));
        assert_eq!(parse_color("red"), None);
        assert_eq!(normalize("Ctrl + Shift + F9"), "ctrl+shift+f9");
    }
}
