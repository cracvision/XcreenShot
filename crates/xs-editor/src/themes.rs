//! Themes (EDT-017), saved styles (EDT-015), favorites (EDT-016) and `.xtheme` files
//! (EDT-018). A theme is a few colors, a font and a shadow; each theme has its own
//! saved styles, generated from it for each tool and then changed by the user.
//! Favorites keep a copy of their style, so changing themes never touches them.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::objects::Styles;

/// Where the themes, saved styles and favorites live between sessions.
pub const KEY: &str = "editor.themes";

/// Tools whose saved styles come from the theme (EDT-017), and Blur, which has fixed ones.
pub const STYLED_TOOLS: &[&str] = &[
    "arrow",
    "line",
    "text",
    "callout",
    "shape",
    "step",
    "highlight",
    "pen",
    "magnify",
    "spotlight",
    "blur",
];

/// Tools whose generated styles carry the theme's shadow (EDT-021).
const SHADOWED: &[&str] = &["arrow", "line", "text", "callout", "shape", "step", "pen"];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Theme {
    pub id: String,
    pub name: String,
    /// 2 to 8 colors, `#RRGGBB`.
    pub colors: Vec<String>,
    /// `None`: the font the tools already use (the system's).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font: Option<String>,
    pub shadow: bool,
    /// Tools whose styles have no shadow even with `shadow`.
    #[serde(default)]
    pub shadow_off: Vec<String>,
    #[serde(default)]
    pub built_in: bool,
    /// Saved styles of each tool, in their order.
    #[serde(default)]
    pub styles: BTreeMap<String, Vec<Saved>>,
}

/// A saved style (EDT-015): a whole tool style, never position, size or text.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Saved {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub style: Value,
}

/// A favorite (EDT-016): a copy of a style of some tool.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Favorite {
    pub id: String,
    pub tool: String,
    pub style: Value,
}

/// Everything the Styles tab knows, kept between sessions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Library {
    pub themes: Vec<Theme>,
    pub active: String,
    #[serde(default)]
    pub favorites: Vec<Favorite>,
    #[serde(default)]
    pub next_id: u64,
}

impl Default for Library {
    fn default() -> Self {
        let mut lib = Library {
            themes: vec![built_in("base"), built_in("sober")],
            active: "base".to_string(),
            favorites: Vec::new(),
            next_id: 1,
        };
        for i in 0..lib.themes.len() {
            lib.generate(i, None);
        }
        lib
    }
}

/// The themes that come with the app (EDT-017, design decision).
fn built_in(id: &str) -> Theme {
    let (name, colors) = match id {
        "sober" => (
            "Sobrio",
            vec!["#1F1F1F", "#6B6B6B", "#B5B5B5", "#FFFFFF", "#2D7FF9"],
        ),
        _ => (
            "Base",
            vec!["#1F2430", "#F2545B", "#2D7FF9", "#F5A524", "#FFFFFF"],
        ),
    };
    Theme {
        id: id.to_string(),
        name: name.to_string(),
        colors: colors.into_iter().map(str::to_string).collect(),
        font: None,
        shadow: true,
        // Base: shadow on arrows, callouts, shapes and steps; off on text, stamps and
        // pen strokes (EDT-021).
        shadow_off: vec!["text".into(), "stamp".into(), "pen".into()],
        built_in: true,
        styles: BTreeMap::new(),
    }
}

/// How light a color looks, 0–255.
pub(crate) fn luminance(hex: &str) -> f64 {
    xs_image::color::Rgba::parse(hex)
        .map(|c| 0.2126 * c.r as f64 + 0.7152 * c.g as f64 + 0.0722 * c.b as f64)
        .unwrap_or(0.0)
}

/// Dark enough for white text on it.
fn dark(hex: &str) -> bool {
    luminance(hex) < 150.0
}

/// The styles a theme gives `tool`, as changes to the tool's own default style.
pub fn generated(theme: &Theme, tool: &str) -> Vec<Map<String, Value>> {
    let shadow = (theme.shadow && SHADOWED.contains(&tool) && !theme.shadow_off.iter().any(|t| t == tool))
        .then(|| json!(xs_image::shadows::Shadow::default()));
    let font = theme.font.clone();
    let colors = &theme.colors;
    let by_light = |a: &&String, b: &&String| luminance(a).total_cmp(&luminance(b));
    let darkest = colors
        .iter()
        .min_by(by_light)
        .filter(|c| dark(c))
        .cloned()
        .unwrap_or("#1F2430".into());
    let lightest = colors
        .iter()
        .max_by(by_light)
        .filter(|c| !dark(c))
        .cloned()
        .unwrap_or("#FFFFFF".into());
    let on = |c: &str| {
        if dark(c) {
            lightest.clone()
        } else {
            darkest.clone()
        }
    };
    let mut out: Vec<Value> = match tool {
        "arrow" => colors.iter().map(|c| json!({"color": c, "width": 6})).collect(),
        "line" => colors.iter().map(|c| json!({"color": c, "width": 4})).collect(),
        "text" => colors.iter().map(|c| json!({"color": c})).collect(),
        "callout" => colors
            .iter()
            .map(|c| json!({"fill": c, "outline": c, "textColor": on(c)}))
            .collect(),
        "shape" => colors
            .iter()
            .map(|c| json!({"outline": c, "fill": null, "outlineWidth": 4}))
            .collect(),
        "step" => colors
            .iter()
            .map(|c| json!({"fill": c, "outline": c, "textColor": on(c)}))
            .collect(),
        // A highlighter or a pen in white shows nothing on most captures.
        "highlight" => colors
            .iter()
            .filter(|c| dark(c) || *c != "#FFFFFF")
            .map(|c| json!({"color": c}))
            .collect(),
        "pen" => colors.iter().map(|c| json!({"color": c, "width": 4})).collect(),
        "magnify" => colors.iter().map(|c| json!({"outline": c})).collect(),
        "spotlight" => vec![json!({"color": darkest}), json!({"color": darkest, "blur": 40})],
        "blur" => vec![
            json!({"blurType": "blur"}),
            json!({"blurType": "pixelate"}),
            json!({"blurType": "solid", "color": darkest}),
        ],
        _ => Vec::new(),
    };
    for v in &mut out {
        if let Some(s) = &shadow {
            v["shadow"] = s.clone();
        }
        if let (Some(f), true) = (&font, tool == "text" || tool == "callout") {
            v["font"] = json!(f);
        }
    }
    out.into_iter().filter_map(|v| v.as_object().cloned()).collect()
}

/// `changes` over the tool's default style, checked: a whole style.
pub fn whole(tool: &str, changes: &Map<String, Value>) -> Result<Value, String> {
    Styles::default().set(tool, changes)
}

impl Library {
    pub fn load(stored: Option<Value>) -> Library {
        stored
            .and_then(|v| serde_json::from_value::<Library>(v).ok())
            .filter(|l| l.themes.iter().any(|t| t.id == l.active))
            .unwrap_or_default()
    }

    fn new_id(&mut self, prefix: &str) -> String {
        self.next_id += 1;
        format!("{prefix}{}", self.next_id)
    }

    pub fn active(&self) -> &Theme {
        self.themes
            .iter()
            .find(|t| t.id == self.active)
            .unwrap_or(&self.themes[0])
    }

    fn active_mut(&mut self) -> &mut Theme {
        let id = self.active.clone();
        let i = self.themes.iter().position(|t| t.id == id).unwrap_or(0);
        &mut self.themes[i]
    }

    pub fn index_of(&self, id: &str) -> Option<usize> {
        self.themes.iter().position(|t| t.id == id)
    }

    /// Generates the saved styles of theme `i`: of every tool, or of `only`.
    pub fn generate(&mut self, i: usize, only: Option<&str>) {
        let tools: Vec<&str> = match only {
            Some(t) => vec![t],
            None => STYLED_TOOLS.to_vec(),
        };
        for tool in tools {
            let theme = self.themes[i].clone();
            let styles: Vec<Saved> = generated(&theme, tool)
                .into_iter()
                .filter_map(|c| whole(tool, &c).ok())
                .map(|style| Saved {
                    id: String::new(),
                    name: None,
                    style,
                })
                .collect();
            let styles = styles
                .into_iter()
                .map(|mut s| {
                    s.id = self.new_id("s");
                    s
                })
                .collect();
            self.themes[i].styles.insert(tool.to_string(), styles);
        }
    }

    /// The saved styles of `tool` in the active theme.
    pub fn styles_of(&self, tool: &str) -> &[Saved] {
        self.active().styles.get(tool).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn find(&self, tool: &str, id: &str) -> Option<&Saved> {
        self.styles_of(tool).iter().find(|s| s.id == id)
    }

    /// Saves `style` as a new style of `tool` at the end (EDT-015).
    pub fn add(&mut self, tool: &str, style: Value, name: Option<String>) -> String {
        let id = self.new_id("s");
        self.active_mut()
            .styles
            .entry(tool.to_string())
            .or_default()
            .push(Saved {
                id: id.clone(),
                name,
                style,
            });
        id
    }

    fn saved_mut(&mut self, tool: &str, id: &str) -> Result<&mut Saved, String> {
        self.active_mut()
            .styles
            .get_mut(tool)
            .and_then(|v| v.iter_mut().find(|s| s.id == id))
            .ok_or_else(|| format!("there is no style {id} of the {tool} tool"))
    }

    pub fn update(&mut self, tool: &str, id: &str, style: Value) -> Result<(), String> {
        self.saved_mut(tool, id)?.style = style;
        Ok(())
    }

    pub fn rename(&mut self, tool: &str, id: &str, name: &str) -> Result<(), String> {
        let name = name.trim();
        self.saved_mut(tool, id)?.name = (!name.is_empty()).then(|| name.to_string());
        Ok(())
    }

    pub fn delete(&mut self, tool: &str, id: &str) -> Result<(), String> {
        let list = self.active_mut().styles.entry(tool.to_string()).or_default();
        let before = list.len();
        list.retain(|s| s.id != id);
        if list.len() == before {
            return Err(format!("there is no style {id} of the {tool} tool"));
        }
        Ok(())
    }

    /// Moves a style to position `to` (0 is first).
    pub fn move_to(&mut self, tool: &str, id: &str, to: usize) -> Result<(), String> {
        let list = self.active_mut().styles.entry(tool.to_string()).or_default();
        let from = list
            .iter()
            .position(|s| s.id == id)
            .ok_or_else(|| format!("there is no style {id} of the {tool} tool"))?;
        let s = list.remove(from);
        list.insert(to.min(list.len()), s);
        Ok(())
    }

    /// Back to the theme's styles for `tool` (EDT-015 Reset styles).
    pub fn reset(&mut self, tool: &str) {
        let i = self.index_of(&self.active.clone()).unwrap_or(0);
        self.generate(i, Some(tool));
    }

    pub fn favorite(&mut self, tool: &str, style: Value) -> String {
        let id = self.new_id("f");
        self.favorites.push(Favorite {
            id: id.clone(),
            tool: tool.to_string(),
            style,
        });
        id
    }

    pub fn unfavorite(&mut self, id: &str) -> Result<(), String> {
        let before = self.favorites.len();
        self.favorites.retain(|f| f.id != id);
        if self.favorites.len() == before {
            return Err(format!("there is no favorite {id}"));
        }
        Ok(())
    }

    pub fn move_favorite(&mut self, id: &str, to: usize) -> Result<(), String> {
        let from = self
            .favorites
            .iter()
            .position(|f| f.id == id)
            .ok_or_else(|| format!("there is no favorite {id}"))?;
        let f = self.favorites.remove(from);
        self.favorites.insert(to.min(self.favorites.len()), f);
        Ok(())
    }

    /// A name no other theme has: `name`, or with " (2)", " (3)"…
    pub fn free_name(&self, name: &str, except: Option<&str>) -> String {
        let taken = |n: &str| {
            self.themes
                .iter()
                .any(|t| t.name == n && Some(t.id.as_str()) != except)
        };
        if !taken(name) {
            return name.to_string();
        }
        (2..)
            .map(|k| format!("{name} ({k})"))
            .find(|n| !taken(n))
            .expect("a free name")
    }

    /// Creates a theme (no `id`) or changes one, and generates its styles again
    /// (EDT-017). Returns its id.
    pub fn save_theme(&mut self, id: Option<&str>, mut theme: Theme) -> Result<String, String> {
        check(&theme)?;
        if theme.name.trim().is_empty() {
            let n = (1..)
                .map(|k| format!("Tema sin nombre {k}"))
                .find(|n| !self.themes.iter().any(|t| &t.name == n))
                .expect("a free name");
            theme.name = n;
        }
        let i = match id {
            Some(id) => {
                let i = self
                    .index_of(id)
                    .ok_or_else(|| format!("there is no theme {id}"))?;
                if self.themes[i].built_in {
                    return Err("a theme that comes with the app cannot be changed: duplicate it".into());
                }
                theme.id = id.to_string();
                theme.built_in = false;
                theme.name = self.free_name(&theme.name, Some(id));
                theme.styles = BTreeMap::new();
                self.themes[i] = theme;
                i
            }
            None => {
                theme.id = self.new_id("t");
                theme.built_in = false;
                theme.name = self.free_name(&theme.name, None);
                theme.styles = BTreeMap::new();
                self.themes.push(theme);
                self.themes.len() - 1
            }
        };
        self.generate(i, None);
        Ok(self.themes[i].id.clone())
    }

    pub fn duplicate(&mut self, id: &str) -> Result<String, String> {
        let i = self
            .index_of(id)
            .ok_or_else(|| format!("there is no theme {id}"))?;
        let mut copy = self.themes[i].clone();
        copy.id = self.new_id("t");
        copy.built_in = false;
        copy.name = self.free_name(&copy.name, None);
        let new_id = copy.id.clone();
        self.themes.push(copy);
        Ok(new_id)
    }

    pub fn delete_theme(&mut self, id: &str) -> Result<(), String> {
        let i = self
            .index_of(id)
            .ok_or_else(|| format!("there is no theme {id}"))?;
        if self.themes[i].built_in {
            return Err("a theme that comes with the app cannot be deleted".into());
        }
        self.themes.remove(i);
        if self.active == id {
            self.active = self.themes[0].id.clone();
        }
        Ok(())
    }

    pub fn rename_theme(&mut self, id: &str, name: &str) -> Result<(), String> {
        let i = self
            .index_of(id)
            .ok_or_else(|| format!("there is no theme {id}"))?;
        if self.themes[i].built_in {
            return Err("a theme that comes with the app cannot be renamed: duplicate it".into());
        }
        let name = name.trim();
        if name.is_empty() {
            return Err("the name is empty".into());
        }
        self.themes[i].name = self.free_name(name, Some(id));
        Ok(())
    }
}

/// A theme's rules (EDT-017): 2 to 8 colors, none twice.
pub fn check(theme: &Theme) -> Result<(), String> {
    if !(2..=8).contains(&theme.colors.len()) {
        return Err("a theme has 2 to 8 colors".into());
    }
    let mut seen: Vec<String> = Vec::new();
    for c in &theme.colors {
        let rgba = xs_image::color::Rgba::parse(c).ok_or_else(|| format!("{c} is not a color"))?;
        let norm = format!("#{:02X}{:02X}{:02X}", rgba.r, rgba.g, rgba.b);
        if seen.contains(&norm) {
            return Err(format!("{c} is in the theme twice"));
        }
        seen.push(norm);
    }
    for t in &theme.shadow_off {
        if !SHADOWED.contains(&t.as_str()) && t != "stamp" && t != "image" {
            return Err(format!("the {t} tool has no shadow"));
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// .xtheme files (EDT-018)

/// The format's name and the newest version this build reads.
pub const FORMAT: &str = "xtheme";
pub const VERSION: u64 = 1;

/// The `.xtheme` of a theme: its settings and its saved styles.
pub fn to_file(theme: &Theme) -> Value {
    json!({
        "format": FORMAT,
        "version": VERSION,
        "name": theme.name,
        "colors": theme.colors,
        "font": theme.font,
        "shadow": theme.shadow,
        "shadowOff": theme.shadow_off,
        "styles": theme.styles.iter().map(|(tool, list)| {
            (tool.clone(), json!(list.iter().map(|s| json!({"name": s.name, "style": s.style})).collect::<Vec<_>>()))
        }).collect::<Map<String, Value>>(),
    })
}

/// Why a `.xtheme` cannot be read.
#[derive(Debug, PartialEq)]
pub enum FileError {
    Unreadable(String),
    Newer,
}

/// A theme from a `.xtheme` (without id): checked, styles that do not fit dropped.
pub fn from_file(v: &Value) -> Result<Theme, FileError> {
    if v.get("format").and_then(Value::as_str) != Some(FORMAT) {
        return Err(FileError::Unreadable("it is not a theme file".into()));
    }
    match v.get("version").and_then(Value::as_u64) {
        Some(n) if n > VERSION => return Err(FileError::Newer),
        Some(_) => {}
        None => return Err(FileError::Unreadable("it has no version".into())),
    }
    let colors: Vec<String> = v
        .get("colors")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(|c| c.as_str().map(str::to_string)).collect())
        .unwrap_or_default();
    let mut theme = Theme {
        id: String::new(),
        name: v.get("name").and_then(Value::as_str).unwrap_or("").to_string(),
        colors,
        font: v.get("font").and_then(Value::as_str).map(str::to_string),
        shadow: v.get("shadow").and_then(Value::as_bool).unwrap_or(false),
        shadow_off: v
            .get("shadowOff")
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(|c| c.as_str().map(str::to_string)).collect())
            .unwrap_or_default(),
        built_in: false,
        styles: BTreeMap::new(),
    };
    check(&theme).map_err(FileError::Unreadable)?;
    if let Some(styles) = v.get("styles").and_then(Value::as_object) {
        for (tool, list) in styles {
            if !STYLED_TOOLS.contains(&tool.as_str()) {
                continue;
            }
            let saved: Vec<Saved> = list
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|s| {
                    let style = s.get("style")?.as_object()?;
                    Some(Saved {
                        id: String::new(),
                        name: s.get("name").and_then(Value::as_str).map(str::to_string),
                        style: whole(tool, style).ok()?,
                    })
                })
                .collect();
            theme.styles.insert(tool.clone(), saved);
        }
    }
    Ok(theme)
}

impl Library {
    /// Adds an imported theme (EDT-018): with `replace`, over the one of the same name;
    /// otherwise beside it, as "Name (2)". It becomes the active theme. Its id.
    pub fn import(&mut self, mut theme: Theme, replace: bool) -> String {
        let same = self
            .themes
            .iter()
            .position(|t| t.name == theme.name && !t.built_in);
        theme.built_in = false;
        let bare = theme.styles.is_empty();
        let id = match (replace, same) {
            (true, Some(i)) => {
                theme.id = self.themes[i].id.clone();
                self.themes[i] = theme;
                self.themes[i].id.clone()
            }
            _ => {
                theme.id = self.new_id("t");
                theme.name = self.free_name(&theme.name, None);
                self.themes.push(theme);
                self.themes.last().expect("pushed").id.clone()
            }
        };
        let i = self.index_of(&id).expect("just added");
        // Ids for its styles; a file without styles gets the theme's own.
        if bare {
            self.generate(i, None);
        } else {
            let mut styles = std::mem::take(&mut self.themes[i].styles);
            for list in styles.values_mut() {
                for s in list.iter_mut() {
                    s.id = self.new_id("s");
                }
            }
            for tool in STYLED_TOOLS {
                if !styles.contains_key(*tool) {
                    let theme = self.themes[i].clone();
                    let list = generated(&theme, tool)
                        .into_iter()
                        .filter_map(|c| whole(tool, &c).ok())
                        .map(|style| Saved {
                            id: String::new(),
                            name: None,
                            style,
                        })
                        .collect::<Vec<_>>();
                    let list = list
                        .into_iter()
                        .map(|mut s| {
                            s.id = self.new_id("s");
                            s
                        })
                        .collect();
                    styles.insert(tool.to_string(), list);
                }
            }
            self.themes[i].styles = styles;
        }
        self.active = id.clone();
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_themes_generate_their_styles() {
        let lib = Library::default();
        assert_eq!(lib.active().name, "Base");
        let arrows = lib.styles_of("arrow");
        assert_eq!(arrows.len(), 5);
        assert_eq!(arrows[1].style["color"], "#F2545B");
        assert!(
            arrows[0].style["shadow"].is_object(),
            "Base: arrows have a shadow"
        );
        assert!(
            lib.styles_of("text")[0].style.get("shadow").is_none(),
            "Base: text has none"
        );
        // A callout's text reads on its fill.
        let callouts = lib.styles_of("callout");
        assert_eq!(callouts[0].style["textColor"], "#FFFFFF");
        assert_eq!(callouts[4].style["textColor"], "#1F2430");
    }

    #[test]
    fn a_new_theme_uses_its_colors_and_font() {
        let mut lib = Library::default();
        let theme = Theme {
            id: String::new(),
            name: "Mío".into(),
            colors: vec!["#112233".into(), "#445566".into(), "#778899".into()],
            font: Some("Georgia".into()),
            shadow: false,
            shadow_off: Vec::new(),
            built_in: false,
            styles: BTreeMap::new(),
        };
        let id = lib.save_theme(None, theme.clone()).unwrap();
        lib.active = id.clone();
        let callouts = lib.styles_of("callout");
        assert_eq!(callouts.len(), 3);
        assert_eq!(callouts[0].style["font"], "Georgia");
        assert!(callouts[0].style.get("shadow").is_none());
        // A color twice, or a single color: refused.
        let mut twice = theme.clone();
        twice.colors = vec!["#112233".into(), "#112233".into()];
        assert!(lib.save_theme(None, twice).is_err());
        let mut one = theme;
        one.colors = vec!["#112233".into()];
        assert!(lib.save_theme(None, one).is_err());
        // Built-in themes cannot be deleted.
        assert!(lib.delete_theme("base").is_err());
        assert!(lib.delete_theme(&id).is_ok());
    }

    #[test]
    fn styles_are_added_moved_and_reset_per_tool() {
        let mut lib = Library::default();
        let style = whole("arrow", json!({"width": 20}).as_object().unwrap()).unwrap();
        let id = lib.add("arrow", style, None);
        assert_eq!(lib.styles_of("arrow").last().unwrap().id, id);
        lib.move_to("arrow", &id, 0).unwrap();
        assert_eq!(lib.styles_of("arrow")[0].id, id);
        let shapes = lib.styles_of("shape").len();
        lib.add("shape", whole("shape", &Map::new()).unwrap(), None);
        lib.reset("shape");
        assert_eq!(lib.styles_of("shape").len(), shapes, "user styles gone");
        assert_eq!(lib.styles_of("arrow")[0].id, id, "arrows untouched");
        // Favorites keep a copy.
        let fav = lib.favorite("arrow", lib.styles_of("arrow")[0].style.clone());
        lib.reset("arrow");
        assert_eq!(lib.favorites[0].id, fav);
        assert_eq!(lib.favorites[0].style["width"], json!(20.0));
    }

    #[test]
    fn xtheme_files_go_and_come_back() {
        let mut lib = Library::default();
        let base = lib.active().clone();
        let file = to_file(&base);
        let back = from_file(&file).unwrap();
        assert_eq!(back.colors, base.colors);
        assert_eq!(back.styles["arrow"].len(), base.styles["arrow"].len());
        // Same name: keep both makes "Base (2)"; a built-in is never replaced.
        let id = lib.import(back.clone(), true);
        assert_eq!(lib.active().name, "Base (2)");
        lib.import(back, false);
        assert!(lib.themes.iter().any(|t| t.name == "Base (3)"));
        assert_ne!(id, "base");
        // Newer or damaged files are refused.
        let mut newer = file.clone();
        newer["version"] = json!(99);
        assert_eq!(from_file(&newer), Err(FileError::Newer));
        assert!(from_file(&json!({"format": "xtheme", "version": 1, "colors": ["#000000"]})).is_err());
    }
}
