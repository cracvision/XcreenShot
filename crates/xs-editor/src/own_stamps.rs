//! The user's own stamps (EDT-032), stamp packs (`.xstamps`, EDT-034) and the Emoji
//! stamps (EDT-033).
//!
//! Own stamps live in `<local data>/stamps/`: a picture per stamp (PNG; an SVG is drawn
//! once, large) and `stamps.json` with the categories and each stamp's name and
//! keywords. A stamp placed in a document carries its picture in the document's
//! `images/` (EDT-031), so the `.xshot` shows it on a computer without that stamp.
//!
//! Ids: `own:<hex>` for own stamps, `emoji:<code points>` for emoji, the collection's
//! own ids for the built-in stamps.

use std::io::{Read, Write};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use super::*;

pub const OWN: &str = "own:";
pub const EMOJI: &str = "emoji:";
/// The category every own stamp goes to unless another is chosen.
pub const MINE: &str = "mine";
/// Longest side of an own stamp's picture.
const MAX_SIDE: u32 = 1024;
/// Largest pack accepted (EDT-034).
const MAX_PACK: u64 = 100 << 20;
/// The version of `pack.json` this app writes.
const PACK_FORMAT: u32 = 1;

/// A pack a category came from (EDT-034).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Pack {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub license: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Category {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pack: Option<Pack>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OwnStamp {
    /// Without the `own:` prefix; also its picture's file name.
    pub id: String,
    pub category: String,
    pub name: String,
    /// Its Spanish name, from a pack.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name_es: String,
    /// Words to find it by, separated by commas or spaces.
    #[serde(default)]
    pub keywords: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub keywords_es: String,
    pub w: u32,
    pub h: u32,
}

impl OwnStamp {
    pub fn shown_name(&self, lang: Lang) -> &str {
        if lang == Lang::Es && !self.name_es.is_empty() {
            &self.name_es
        } else {
            &self.name
        }
    }

    fn matches(&self, query: &str) -> bool {
        let hay = xs_image::stamps::fold(
            &format!(
                "{} {} {} {}",
                self.name, self.name_es, self.keywords, self.keywords_es
            )
            .to_lowercase(),
        );
        xs_image::stamps::fold(&query.to_lowercase())
            .split_whitespace()
            .all(|w| hay.contains(w))
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Shelf {
    #[serde(default)]
    pub categories: Vec<Category>,
    #[serde(default)]
    pub stamps: Vec<OwnStamp>,
}

/// The own stamps of this computer.
pub struct OwnStamps {
    dir: PathBuf,
    shelf: Mutex<Shelf>,
}

pub fn own_stamps(core: &Core) -> Arc<OwnStamps> {
    core.service(|| OwnStamps::open(core.paths.local_dir.join("stamps")))
}

/// An error a person can read, in the app's language.
fn bad(lang: Lang, param: &str, reason: &str) -> CommandError {
    invalid(tr(
        lang,
        "errors.invalid_param",
        &[("param", param), ("reason", reason)],
    ))
}

fn not_found(lang: Lang, id: &str) -> CommandError {
    CommandError::new(
        ErrorCode::NotFound,
        tr(lang, "errors.stamp_not_found", &[("id", id)]),
    )
}

fn new_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static N: AtomicU64 = AtomicU64::new(0);
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or_default();
    format!("{:x}{:04x}", t, N.fetch_add(1, Ordering::Relaxed) & 0xffff)
}

/// A picture file as a stamp's picture: PNG, JPG, GIF (its first frame), WebP or SVG;
/// no larger than [`MAX_SIDE`].
pub fn picture_of(bytes: &[u8]) -> Result<image::RgbaImage, String> {
    let img = if xs_image::svg::is_svg(bytes) {
        xs_image::svg::rasterize(bytes, MAX_SIDE)?
    } else if bytes.starts_with(b"GIF8") {
        xs_image::gif_anim::first_frame(bytes).ok_or("the GIF could not be read")?
    } else {
        xs_image::encode::decode(bytes)?
    };
    if img.width() == 0 || img.height() == 0 {
        return Err("the picture is empty".into());
    }
    Ok(if img.width().max(img.height()) > MAX_SIDE {
        xs_image::encode::thumbnail(&img, MAX_SIDE)
    } else {
        img
    })
}

/// Only image files go in a pack (EDT-034).
fn is_picture_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    [".png", ".svg", ".jpg", ".jpeg", ".webp", ".gif"]
        .iter()
        .any(|e| lower.ends_with(e))
}

impl OwnStamps {
    pub fn open(dir: PathBuf) -> OwnStamps {
        let shelf = std::fs::read(dir.join("stamps.json"))
            .ok()
            .and_then(|b| serde_json::from_slice::<Shelf>(&b).ok())
            .unwrap_or_default();
        OwnStamps {
            dir,
            shelf: Mutex::new(shelf),
        }
    }

    pub fn shelf(&self) -> Shelf {
        self.shelf.lock().unwrap().clone()
    }

    fn save(&self, shelf: &Shelf) -> Result<(), String> {
        std::fs::create_dir_all(&self.dir).map_err(|e| e.to_string())?;
        let tmp = self.dir.join("stamps.json.tmp");
        let bytes = serde_json::to_vec_pretty(shelf).map_err(|e| e.to_string())?;
        std::fs::write(&tmp, bytes).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, self.dir.join("stamps.json")).map_err(|e| e.to_string())
    }

    /// Changes the shelf and writes it; nothing changes if writing fails.
    fn change<T>(&self, f: impl FnOnce(&mut Shelf) -> Result<T, String>) -> Result<T, String> {
        let mut guard = self.shelf.lock().unwrap();
        let mut next = guard.clone();
        let out = f(&mut next)?;
        self.save(&next)?;
        *guard = next;
        Ok(out)
    }

    fn picture_path(&self, id: &str) -> PathBuf {
        self.dir.join(format!("{id}.png"))
    }

    /// An own stamp's picture (PNG), by its id without the prefix.
    pub fn picture(&self, id: &str) -> Option<(Vec<u8>, OwnStamp)> {
        let stamp = self
            .shelf
            .lock()
            .unwrap()
            .stamps
            .iter()
            .find(|s| s.id == id)
            .cloned()?;
        let png = std::fs::read(self.picture_path(id)).ok()?;
        Some((png, stamp))
    }

    /// Adds a stamp from a picture, to `category` (made if it is a new name).
    pub fn add(
        &self,
        img: &image::RgbaImage,
        name: &str,
        keywords: &str,
        category: &str,
    ) -> Result<OwnStamp, String> {
        let png = xs_image::encode::png(img, 0.0, false)?;
        let id = new_id();
        std::fs::create_dir_all(&self.dir).map_err(|e| e.to_string())?;
        std::fs::write(self.picture_path(&id), &png).map_err(|e| e.to_string())?;
        let stamp = OwnStamp {
            id: id.clone(),
            category: category.to_string(),
            name: name.trim().chars().take(80).collect(),
            name_es: String::new(),
            keywords: keywords.trim().to_string(),
            keywords_es: String::new(),
            w: img.width(),
            h: img.height(),
        };
        let added = stamp.clone();
        let r = self.change(|s| {
            ensure_category(s, category);
            s.stamps.push(stamp);
            Ok(())
        });
        if let Err(e) = r {
            let _ = std::fs::remove_file(self.picture_path(&id));
            return Err(e);
        }
        Ok(added)
    }
}

/// The category `id` exists: Mine always; any other made with that name.
fn ensure_category(s: &mut Shelf, id: &str) {
    if !s.categories.iter().any(|c| c.id == id) {
        s.categories.push(Category {
            id: id.to_string(),
            name: if id == MINE { String::new() } else { id.to_string() },
            pack: None,
        });
    }
}

/// A category id for a new name: the name itself, made unique.
fn category_id(s: &Shelf, name: &str) -> String {
    let base: String = name.trim().chars().take(60).collect();
    let base = if base.is_empty() {
        "category".to_string()
    } else {
        base
    };
    let mut id = base.clone();
    let mut n = 2;
    while id == MINE || s.categories.iter().any(|c| c.id == id) {
        id = format!("{base} ({n})");
        n += 1;
    }
    id
}

// ---------------------------------------------------------------------------
// Packs (EDT-034)

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct Words {
    #[serde(default)]
    en: String,
    #[serde(default)]
    es: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PackStamp {
    file: String,
    #[serde(default)]
    name: Words,
    #[serde(default)]
    keywords: Words,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PackFile {
    #[serde(default = "one")]
    format: u32,
    name: String,
    #[serde(default)]
    version: String,
    #[serde(default)]
    author: String,
    #[serde(default)]
    license: String,
    #[serde(default)]
    category: String,
    stamps: Vec<PackStamp>,
}

fn one() -> u32 {
    1
}

/// A pack read and checked: its description and its pictures.
pub struct ReadPack {
    pack: PackFile,
    pictures: Vec<(PackStamp, image::RgbaImage)>,
}

/// Reads a `.xstamps` and checks it (EDT-034): at most 100 MB, only pictures and
/// `pack.json`, every listed picture readable.
pub fn read_pack(path: &Path) -> Result<ReadPack, String> {
    let size = std::fs::metadata(path).map_err(|e| e.to_string())?.len();
    if size > MAX_PACK {
        return Err("the pack is larger than 100 MB".into());
    }
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipArchive::new(file).map_err(|_| "it is not a stamp pack (ZIP)".to_string())?;
    let mut total = 0u64;
    for i in 0..zip.len() {
        let f = zip.by_index(i).map_err(|e| e.to_string())?;
        if f.is_dir() {
            continue;
        }
        let name = f.name().map(|n| n.to_string()).map_err(|e| e.to_string())?;
        total += f.size();
        if name != "pack.json" && !is_picture_name(&name) {
            return Err(format!("it has a file that is not a picture: {name}"));
        }
    }
    if total > MAX_PACK {
        return Err("the pack is larger than 100 MB".into());
    }
    let mut read = |name: &str| -> Result<Vec<u8>, String> {
        let mut f = zip
            .by_name(name)
            .map_err(|_| format!("{name} is missing from the pack"))?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).map_err(|e| e.to_string())?;
        Ok(buf)
    };
    let pack: PackFile =
        serde_json::from_slice(&read("pack.json")?).map_err(|e| format!("pack.json: {e}"))?;
    if pack.name.trim().is_empty() {
        return Err("pack.json has no name".into());
    }
    if pack.format > PACK_FORMAT {
        return Err("the pack was made by a newer version of XcreenShot".into());
    }
    let mut pictures = Vec::new();
    for s in &pack.stamps {
        if !is_picture_name(&s.file) {
            return Err(format!("it has a file that is not a picture: {}", s.file));
        }
        let img = picture_of(&read(&s.file)?).map_err(|e| format!("{}: {e}", s.file))?;
        pictures.push((s.clone(), img));
    }
    Ok(ReadPack { pack, pictures })
}

/// Compares dotted versions: "1.10" is newer than "1.9".
fn newer(a: &str, b: &str) -> bool {
    let parts = |v: &str| -> Vec<u64> { v.split('.').map(|p| p.trim().parse().unwrap_or(0)).collect() };
    parts(a) > parts(b)
}

impl OwnStamps {
    /// The category a pack of this name came in, with its version.
    pub fn pack_category(&self, name: &str) -> Option<Category> {
        self.shelf
            .lock()
            .unwrap()
            .categories
            .iter()
            .find(|c| c.pack.as_ref().is_some_and(|p| p.name.eq_ignore_ascii_case(name)))
            .cloned()
    }

    /// Adds a pack's stamps as a category; `replace` is the category it updates.
    fn install(&self, read: ReadPack, replace: Option<&str>) -> Result<(String, usize), String> {
        let ReadPack { pack, pictures } = read;
        let mut written = Vec::new();
        let mut stamps = Vec::new();
        std::fs::create_dir_all(&self.dir).map_err(|e| e.to_string())?;
        for (s, img) in &pictures {
            let id = new_id();
            let png = xs_image::encode::png(img, 0.0, false)?;
            std::fs::write(self.picture_path(&id), png).map_err(|e| e.to_string())?;
            written.push(id.clone());
            let file_name = Path::new(&s.file)
                .file_stem()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            stamps.push(OwnStamp {
                id,
                category: String::new(),
                name: if s.name.en.trim().is_empty() {
                    file_name
                } else {
                    s.name.en.trim().to_string()
                },
                name_es: s.name.es.trim().to_string(),
                keywords: s.keywords.en.trim().to_string(),
                keywords_es: s.keywords.es.trim().to_string(),
                w: img.width(),
                h: img.height(),
            });
        }
        let count = stamps.len();
        let mut gone = Vec::new();
        let r = self.change(|shelf| {
            let id = match replace {
                Some(id) => {
                    gone = shelf
                        .stamps
                        .iter()
                        .filter(|s| s.category == id)
                        .map(|s| s.id.clone())
                        .collect();
                    shelf.stamps.retain(|s| s.category != id);
                    id.to_string()
                }
                None => {
                    let title = if pack.category.trim().is_empty() {
                        &pack.name
                    } else {
                        &pack.category
                    };
                    let id = category_id(shelf, title);
                    shelf.categories.push(Category {
                        id: id.clone(),
                        name: title.trim().to_string(),
                        pack: None,
                    });
                    id
                }
            };
            if let Some(c) = shelf.categories.iter_mut().find(|c| c.id == id) {
                c.pack = Some(Pack {
                    name: pack.name.trim().to_string(),
                    version: pack.version.clone(),
                    author: pack.author.clone(),
                    license: pack.license.clone(),
                });
            }
            for mut s in stamps {
                s.category = id.clone();
                shelf.stamps.push(s);
            }
            Ok(id)
        });
        match r {
            Ok(id) => {
                for g in gone {
                    let _ = std::fs::remove_file(self.picture_path(&g));
                }
                Ok((id, count))
            }
            Err(e) => {
                for w in written {
                    let _ = std::fs::remove_file(self.picture_path(&w));
                }
                Err(e)
            }
        }
    }

    /// Writes category `id` as a `.xstamps` (EDT-034).
    pub fn export(
        &self,
        id: &str,
        out: &Path,
        name: &str,
        author: &str,
        license: &str,
        version: &str,
    ) -> Result<usize, String> {
        let shelf = self.shelf();
        let category = shelf
            .categories
            .iter()
            .find(|c| c.id == id)
            .ok_or("there is no such category")?;
        let title = if category.id == MINE {
            name.to_string()
        } else {
            category.name.clone()
        };
        let stamps: Vec<&OwnStamp> = shelf.stamps.iter().filter(|s| s.category == id).collect();
        let tmp = out.with_extension("xstamps.tmp");
        let file = std::fs::File::create(&tmp).map_err(|e| e.to_string())?;
        let mut zip = zip::ZipWriter::new(file);
        let opts =
            zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        let mut listed = Vec::new();
        for (i, s) in stamps.iter().enumerate() {
            let png = std::fs::read(self.picture_path(&s.id)).map_err(|e| e.to_string())?;
            let file = format!("{:03}.png", i + 1);
            zip.start_file(file.as_str(), opts).map_err(|e| e.to_string())?;
            zip.write_all(&png).map_err(|e| e.to_string())?;
            listed.push(PackStamp {
                file,
                name: Words {
                    en: s.name.clone(),
                    es: s.name_es.clone(),
                },
                keywords: Words {
                    en: s.keywords.clone(),
                    es: s.keywords_es.clone(),
                },
            });
        }
        let pack = PackFile {
            format: PACK_FORMAT,
            name: name.to_string(),
            version: if version.trim().is_empty() {
                "1.0".into()
            } else {
                version.trim().into()
            },
            author: author.to_string(),
            license: license.to_string(),
            category: title,
            stamps: listed,
        };
        zip.start_file(
            "pack.json",
            opts.compression_method(zip::CompressionMethod::Deflated),
        )
        .map_err(|e| e.to_string())?;
        zip.write_all(&serde_json::to_vec_pretty(&pack).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        zip.finish().map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, out).map_err(|e| e.to_string())?;
        Ok(stamps.len())
    }
}

// ---------------------------------------------------------------------------
// Placing a stamp

/// What a stamp id places: its drawing, its own color and its proportion; pictures
/// (own stamps, emoji) are added to the document's `images/`.
pub fn stamp_art(
    core: &Core,
    document: Option<&str>,
    id: &str,
    lang: Lang,
) -> Result<(Value, Option<String>, f64), String> {
    let picture = |png: Vec<u8>, w: u32, h: u32| -> Result<(Value, Option<String>, f64), String> {
        let name = format!("stamp-{:016x}.png", content_hash(&png));
        if let Some(doc) = document {
            editor(core)
                .store
                .add_images(doc, &BTreeMap::from([(name.clone(), png)]))
                .map_err(|e| e.to_string())?;
        }
        Ok((
            xs_image::stamps::picture_art(&name, w, h),
            None,
            w as f64 / h.max(1) as f64,
        ))
    };
    if let Some(own) = id.strip_prefix(OWN) {
        let (png, s) = own_stamps(core)
            .picture(own)
            .ok_or_else(|| format!("there is no stamp \"{id}\""))?;
        return picture(png, s.w, s.h);
    }
    if let Some(e) = id.strip_prefix(EMOJI) {
        let e = xs_image::emoji::find(e).ok_or_else(|| format!("there is no stamp \"{id}\""))?;
        let img = e.image().ok_or("the emoji could not be read")?;
        let png = xs_image::encode::png(&img, 0.0, false)?;
        return picture(png, img.width(), img.height());
    }
    let st = xs_image::stamps::find(id).ok_or_else(|| format!("there is no stamp \"{id}\""))?;
    let l = if lang == Lang::Es {
        xs_image::stamps::Lang::Es
    } else {
        xs_image::stamps::Lang::En
    };
    Ok((
        st.art(l),
        Some(st.ink.unwrap_or("#3A3F4B").to_string()),
        st.aspect(),
    ))
}

/// Whether a stamp id exists (styles keep only stamps that do).
pub fn exists(core: &Core, id: &str) -> bool {
    if let Some(own) = id.strip_prefix(OWN) {
        return own_stamps(core).shelf().stamps.iter().any(|s| s.id == own);
    }
    if let Some(e) = id.strip_prefix(EMOJI) {
        return xs_image::emoji::find(e).is_some();
    }
    xs_image::stamps::find(id).is_some()
}

/// A stamp's picture in a `size` px square, for the collection's grid.
pub fn thumbnail(core: &Core, id: &str, lang: Lang, size: u32) -> Option<image::RgbaImage> {
    let img = if let Some(own) = id.strip_prefix(OWN) {
        let (png, _) = own_stamps(core).picture(own)?;
        xs_image::encode::decode(&png).ok()?
    } else if let Some(e) = id.strip_prefix(EMOJI) {
        xs_image::emoji::find(e)?.image()?
    } else {
        let l = if lang == Lang::Es {
            xs_image::stamps::Lang::Es
        } else {
            xs_image::stamps::Lang::En
        };
        return xs_image::stamps::thumbnail(id, l, size);
    };
    let fit = xs_image::encode::thumbnail(&img, size - 4);
    let mut out = image::RgbaImage::new(size, size);
    let (x, y) = ((size - fit.width()) / 2, (size - fit.height()) / 2);
    image::imageops::overlay(&mut out, &fit, x as i64, y as i64);
    Some(out)
}

// ---------------------------------------------------------------------------
// Commands

/// `editor.stamps` (EDT-031 to EDT-034): the categories (built-in, Emoji, own) and the
/// stamps that `query` finds.
pub fn stamps_command(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let query = p.get("query").and_then(Value::as_str).unwrap_or_default();
    let category = p.get("category").and_then(Value::as_str);
    let lang = ctx.lang;
    let es = lang == Lang::Es;
    let wants = |c: &str| category.is_none_or(|w| w == c);
    let mut stamps: Vec<Value> = xs_image::stamps::collection()
        .iter()
        .filter(|s| wants(s.category) && s.matches(query))
        .map(|s| json!({"id": s.id, "category": s.category, "name": if es { s.es } else { s.en }}))
        .collect();
    let shelf = own_stamps(core).shelf();
    let mine = mine_category();
    let mut own_categories: Vec<&Category> = shelf.categories.iter().collect();
    if !own_categories.iter().any(|c| c.id == MINE) {
        own_categories.insert(0, &mine);
    }
    for c in &own_categories {
        let in_it = format!("{OWN}{}", c.id);
        if !wants(&in_it) {
            continue;
        }
        stamps.extend(
            shelf
                .stamps
                .iter()
                .filter(|s| s.category == c.id && s.matches(query))
                .map(|s| {
                    json!({
                        "id": format!("{OWN}{}", s.id),
                        "category": in_it,
                        "name": s.shown_name(lang),
                        "keywords": s.keywords,
                        "own": true,
                    })
                }),
        );
    }
    // Emoji last: there are many, and a search usually means another stamp.
    if wants("emoji") && (category.is_some() || !query.trim().is_empty()) {
        stamps.extend(
            xs_image::emoji::all()
                .iter()
                .filter(|e| e.matches(query))
                .map(|e| json!({"id": format!("{EMOJI}{}", e.id), "category": "emoji", "name": if es { &e.es } else { &e.en }})),
        );
    }
    let mut categories: Vec<Value> = xs_image::stamps::CATEGORIES
        .iter()
        .map(|c| json!({"id": c, "name": tr(lang, &format!("stamps.{c}"), &[])}))
        .collect();
    categories.push(json!({"id": "emoji", "name": tr(lang, "stamps.emoji", &[])}));
    for c in own_categories {
        categories.push(json!({
            "id": format!("{OWN}{}", c.id),
            "name": category_name(c, lang),
            "own": true,
            "pack": c.pack,
            "count": shelf.stamps.iter().filter(|s| s.category == c.id).count(),
        }));
    }
    Ok(json!({ "categories": categories, "stamps": stamps }))
}

fn mine_category() -> Category {
    Category {
        id: MINE.to_string(),
        name: String::new(),
        pack: None,
    }
}

fn category_name(c: &Category, lang: Lang) -> String {
    if c.id == MINE {
        tr(lang, "stamps.mine", &[])
    } else {
        c.name.clone()
    }
}

/// An own category's id from `own:<id>` or `<id>`.
fn own_category(v: &str) -> &str {
    v.strip_prefix(OWN).unwrap_or(v)
}

/// The category asked for: an existing own one, or a new one with `newCategory`.
fn target_category(core: &Core, lang: Lang, p: &Value) -> Result<String, CommandError> {
    if let Some(name) = p
        .get("newCategory")
        .and_then(Value::as_str)
        .filter(|n| !n.trim().is_empty())
    {
        let stamps = own_stamps(core);
        return stamps
            .change(|s| {
                let id = category_id(s, name);
                s.categories.push(Category {
                    id: id.clone(),
                    name: name.trim().to_string(),
                    pack: None,
                });
                Ok(id)
            })
            .map_err(|e| internal(lang, name, e));
    }
    let id = p
        .get("category")
        .and_then(Value::as_str)
        .map(own_category)
        .unwrap_or(MINE)
        .to_string();
    if id != MINE && !own_stamps(core).shelf().categories.iter().any(|c| c.id == id) {
        return Err(bad(lang, "category", "there is no such own category"));
    }
    Ok(id)
}

/// `stamps.add` (EDT-032): pictures from files to an own category.
pub fn add_command(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let files: Vec<String> = match p.get("files") {
        Some(Value::String(s)) => vec![s.clone()],
        Some(Value::Array(a)) => a.iter().filter_map(Value::as_str).map(str::to_string).collect(),
        _ => Vec::new(),
    };
    if files.is_empty() {
        return Err(bad(lang, "files", "give at least one picture file"));
    }
    let category = target_category(core, lang, p)?;
    let stamps = own_stamps(core);
    let mut added = Vec::new();
    let mut failed = Vec::new();
    for f in &files {
        let path = Path::new(f);
        let name = path
            .file_stem()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let r = std::fs::read(path)
            .map_err(|e| e.to_string())
            .and_then(|b| picture_of(&b))
            .and_then(|img| stamps.add(&img, &name, "", &category));
        match r {
            Ok(s) => added.push(json!({"id": format!("{OWN}{}", s.id), "name": s.name})),
            Err(reason) => failed.push(json!({"file": f, "reason": reason})),
        }
    }
    if !added.is_empty() {
        core.host().editor_event("stamps_changed", json!({}));
    }
    Ok(json!({ "added": added, "failed": failed, "category": format!("{OWN}{category}") }))
}

/// `stamps.update` (EDT-032): rename, keywords, another own category, another place.
pub fn update_command(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let id = p["id"].as_str().unwrap_or_default();
    let own = id
        .strip_prefix(OWN)
        .ok_or_else(|| bad(lang, "id", "only own stamps change"))?;
    let category = match p.get("category").and_then(Value::as_str) {
        Some(c) => {
            let c = own_category(c).to_string();
            if c != MINE && !own_stamps(core).shelf().categories.iter().any(|x| x.id == c) {
                return Err(bad(lang, "category", "there is no such own category"));
            }
            Some(c)
        }
        None => None,
    };
    let before = p
        .get("before")
        .and_then(Value::as_str)
        .map(|b| own_category(b).to_string());
    let r = own_stamps(core).change(|s| {
        let at = s.stamps.iter().position(|x| x.id == own).ok_or("missing")?;
        let mut stamp = s.stamps.remove(at);
        if let Some(n) = p.get("name").and_then(Value::as_str) {
            let n = n.trim();
            if n.is_empty() {
                return Err("empty name".into());
            }
            stamp.name = n.chars().take(80).collect();
            stamp.name_es.clear();
        }
        if let Some(k) = p.get("keywords").and_then(Value::as_str) {
            stamp.keywords = k.trim().to_string();
            stamp.keywords_es.clear();
        }
        if let Some(c) = &category {
            ensure_category(s, c);
            stamp.category = c.clone();
        }
        let to = before
            .as_deref()
            .map(|b| b.strip_prefix(OWN).unwrap_or(b))
            .and_then(|b| s.stamps.iter().position(|x| x.id == b))
            .unwrap_or(if p.get("before").is_some() {
                s.stamps.len()
            } else {
                at.min(s.stamps.len())
            });
        s.stamps.insert(to, stamp);
        Ok(())
    });
    match r {
        Ok(()) => {
            core.host().editor_event("stamps_changed", json!({}));
            Ok(json!({ "id": id }))
        }
        Err(e) if e == "missing" => Err(not_found(lang, id)),
        Err(e) => Err(bad(lang, "name", &e)),
    }
}

/// `stamps.delete` (EDT-032): own stamps; placed ones stay in their documents.
pub fn delete_command(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let ids: Vec<String> = p["ids"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(|s| own_category(s).to_string())
                .collect()
        })
        .unwrap_or_default();
    let stamps = own_stamps(core);
    let gone = stamps
        .change(|s| {
            let before = s.stamps.len();
            s.stamps.retain(|x| !ids.contains(&x.id));
            Ok(before - s.stamps.len())
        })
        .map_err(|e| internal(lang, "", e))?;
    for id in &ids {
        let _ = std::fs::remove_file(stamps.picture_path(id));
    }
    core.host().editor_event("stamps_changed", json!({}));
    Ok(json!({ "deleted": gone }))
}

/// `stamps.category` (EDT-032, EDT-034): create, rename or delete an own category
/// (deleting a pack's category removes the pack).
pub fn category_command(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let action = p["action"].as_str().unwrap_or_default();
    let name = p
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_string();
    let id = p
        .get("id")
        .and_then(Value::as_str)
        .map(own_category)
        .unwrap_or_default()
        .to_string();
    let stamps = own_stamps(core);
    let out = match action {
        "create" => {
            if name.is_empty() {
                return Err(bad(lang, "name", "give the category a name"));
            }
            let id = stamps
                .change(|s| {
                    let id = category_id(s, &name);
                    s.categories.push(Category {
                        id: id.clone(),
                        name: name.clone(),
                        pack: None,
                    });
                    Ok(id)
                })
                .map_err(|e| internal(lang, &name, e))?;
            json!({ "id": format!("{OWN}{id}") })
        }
        "rename" => {
            if name.is_empty() {
                return Err(bad(lang, "name", "give the category a name"));
            }
            if id == MINE {
                return Err(bad(lang, "id", "My stamps keeps its name"));
            }
            stamps
                .change(|s| {
                    let c = s.categories.iter_mut().find(|c| c.id == id).ok_or("missing")?;
                    c.name = name.clone();
                    Ok(())
                })
                .map_err(|_| not_found(lang, &id))?;
            json!({ "id": format!("{OWN}{id}") })
        }
        "delete" => {
            if id == MINE {
                return Err(bad(lang, "id", "My stamps cannot be deleted"));
            }
            let gone = stamps
                .change(|s| {
                    let before = s.categories.len();
                    s.categories.retain(|c| c.id != id);
                    if s.categories.len() == before {
                        return Err("missing".into());
                    }
                    let gone: Vec<String> = s
                        .stamps
                        .iter()
                        .filter(|x| x.category == id)
                        .map(|x| x.id.clone())
                        .collect();
                    s.stamps.retain(|x| x.category != id);
                    Ok(gone)
                })
                .map_err(|_| not_found(lang, &id))?;
            for g in &gone {
                let _ = std::fs::remove_file(stamps.picture_path(g));
            }
            json!({ "deleted": gone.len() })
        }
        _ => return Err(bad(lang, "action", "create, rename or delete")),
    };
    core.host().editor_event("stamps_changed", json!({}));
    Ok(out)
}

/// `stamps.import` (EDT-034): a `.xstamps` as a category. A pack already here with the
/// same name is not touched unless `existing` says update (only to a newer version) or
/// both.
pub fn import_command(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let path = PathBuf::from(p["path"].as_str().unwrap_or_default());
    let read = read_pack(&path).map_err(|reason| {
        invalid(tr(
            lang,
            "errors.stamp_pack",
            &[
                (
                    "name",
                    &path
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default(),
                ),
                ("reason", &reason),
            ],
        ))
    })?;
    let stamps = own_stamps(core);
    let existing = stamps.pack_category(&read.pack.name);
    let choice = p.get("existing").and_then(Value::as_str);
    let replace = match (&existing, choice) {
        (None, _) | (Some(_), Some("both")) => None,
        (Some(c), Some("update")) => {
            let have = c.pack.as_ref().map(|p| p.version.clone()).unwrap_or_default();
            if !newer(&read.pack.version, &have) {
                return Err(bad(
                    lang,
                    "existing",
                    "the pack here is the same version or newer",
                ));
            }
            Some(c.id.clone())
        }
        (Some(c), _) => {
            let have = c.pack.as_ref().map(|p| p.version.clone()).unwrap_or_default();
            return Ok(json!({
                "status": "exists",
                "name": read.pack.name,
                "version": read.pack.version,
                "installed": have,
                "category": format!("{OWN}{}", c.id),
                "canUpdate": newer(&read.pack.version, &have),
            }));
        }
    };
    let (id, count) = stamps
        .install(read, replace.as_deref())
        .map_err(|e| internal(lang, "", e))?;
    core.host().editor_event("stamps_changed", json!({}));
    Ok(json!({
        "status": if replace.is_some() { "updated" } else { "imported" },
        "category": format!("{OWN}{id}"),
        "count": count,
    }))
}

/// `stamps.export` (EDT-034): an own category as a `.xstamps`.
pub fn export_command(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let id = own_category(p["category"].as_str().unwrap_or_default()).to_string();
    let mut path = PathBuf::from(p["path"].as_str().unwrap_or_default());
    if path
        .extension()
        .is_none_or(|e| !e.eq_ignore_ascii_case("xstamps"))
    {
        path.set_extension("xstamps");
    }
    let shelf = own_stamps(core).shelf();
    let category = shelf
        .categories
        .iter()
        .find(|c| c.id == id)
        .cloned()
        .or_else(|| (id == MINE).then(mine_category))
        .ok_or_else(|| not_found(lang, &id))?;
    let shown = category_name(&category, lang);
    let name = p
        .get("name")
        .and_then(Value::as_str)
        .filter(|n| !n.trim().is_empty())
        .unwrap_or(&shown);
    if id == MINE {
        // Mine has no entry until it has stamps.
        let _ = own_stamps(core).change(|s| {
            ensure_category(s, MINE);
            Ok(())
        });
    }
    let count = own_stamps(core)
        .export(
            &id,
            &path,
            name,
            p.get("author").and_then(Value::as_str).unwrap_or_default(),
            p.get("license").and_then(Value::as_str).unwrap_or_default(),
            p.get("version").and_then(Value::as_str).unwrap_or("1.0"),
        )
        .map_err(|e| internal(lang, name, e))?;
    Ok(json!({ "path": path, "count": count }))
}

/// `document.save_as_stamp` (EDT-032): the pixel selection (its shape kept, outside
/// transparent), or the whole flattened image, as an own stamp.
pub fn save_as_stamp(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let id = resolve(core, lang, p)?;
    let ed = editor(core);
    let selection = ed.store.with(&id, |d| d.pixel_selection.clone()).flatten();
    let img = match selection {
        Some(sel) => {
            let doc = ed
                .store
                .with(&id, |d| d.document.clone())
                .ok_or_else(|| CommandError::internal("document closed"))?;
            let base = ed.store.base(&id).map_err(|e| internal(lang, "", e))?;
            let picture = match &sel.target {
                Some(oid) => Some(picture_pixels(core, &id, &doc, oid, lang)?),
                None => None,
            };
            let pixels = pixels::target_pixels(&doc, &base, sel.target.as_deref(), picture)
                .ok_or_else(|| CommandError::internal("no pixels"))?;
            xs_image::selection::selected_pixels(&pixels, sel.mask.as_ref())
                .map(|(cut, _)| cut)
                .ok_or_else(|| bad(lang, "documentId", "nothing is selected"))?
        }
        None => final_image(core, &id).map_err(|e| internal(lang, "", e))?,
    };
    let img = if img.width().max(img.height()) > MAX_SIDE {
        xs_image::encode::thumbnail(&img, MAX_SIDE)
    } else {
        img
    };
    let name = p
        .get("name")
        .and_then(Value::as_str)
        .filter(|n| !n.trim().is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| {
            ed.store
                .with(&id, |d| d.name())
                .filter(|n| !n.is_empty())
                .unwrap_or_else(|| tr(lang, "stamps.mine", &[]))
        });
    let category = target_category(core, lang, p)?;
    let s = own_stamps(core)
        .add(
            &img,
            &name,
            p.get("keywords").and_then(Value::as_str).unwrap_or_default(),
            &category,
        )
        .map_err(|e| internal(lang, &name, e))?;
    core.host().editor_event("stamps_changed", json!({}));
    Ok(json!({ "id": format!("{OWN}{}", s.id), "name": s.name, "width": s.w, "height": s.h }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dot(color: [u8; 4]) -> image::RgbaImage {
        let mut img = image::RgbaImage::new(20, 10);
        for (x, y, p) in img.enumerate_pixels_mut() {
            if (x as i32 - 5).pow(2) + (y as i32 - 5).pow(2) <= 16 {
                *p = image::Rgba(color);
            }
        }
        img
    }

    #[test]
    fn own_stamps_are_kept_and_found() {
        let dir = tempfile::tempdir().unwrap();
        let st = OwnStamps::open(dir.path().to_path_buf());
        let a = st
            .add(&dot([255, 0, 0, 255]), "Logo", "empresa marca", MINE)
            .unwrap();
        let again = OwnStamps::open(dir.path().to_path_buf());
        let shelf = again.shelf();
        assert_eq!(shelf.stamps.len(), 1);
        assert!(shelf.stamps[0].matches("marca"));
        assert!(!shelf.stamps[0].matches("otra"));
        let (png, s) = again.picture(&a.id).unwrap();
        assert_eq!((s.w, s.h), (20, 10));
        let img = xs_image::encode::decode(&png).unwrap();
        assert_eq!(img.get_pixel(19, 0).0[3], 0, "transparency kept");
        assert_eq!(img.get_pixel(5, 5).0, [255, 0, 0, 255]);
    }

    #[test]
    fn a_pack_goes_out_and_comes_back_with_its_words() {
        let dir = tempfile::tempdir().unwrap();
        let st = OwnStamps::open(dir.path().join("a"));
        st.add(&dot([0, 0, 255, 255]), "Blue", "azul punto", MINE)
            .unwrap();
        st.add(&dot([0, 255, 0, 255]), "Green", "verde", MINE).unwrap();
        let file = dir.path().join("dots.xstamps");
        assert_eq!(st.export(MINE, &file, "Dots", "Carlos", "CC0", "1.2").unwrap(), 2);
        let other = OwnStamps::open(dir.path().join("b"));
        let read = read_pack(&file).unwrap();
        assert_eq!(read.pack.version, "1.2");
        let (id, n) = other.install(read, None).unwrap();
        assert_eq!(n, 2);
        let shelf = other.shelf();
        let c = shelf.categories.iter().find(|c| c.id == id).unwrap();
        assert_eq!(c.pack.as_ref().unwrap().name, "Dots");
        assert_eq!(
            shelf.stamps.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(),
            ["Blue", "Green"]
        );
        assert!(shelf.stamps[0].matches("azul"));
        assert!(newer("1.10", "1.9") && !newer("1.0", "1.0"));
    }

    #[test]
    fn a_pack_with_a_program_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("bad.xstamps");
        let mut zip = zip::ZipWriter::new(std::fs::File::create(&file).unwrap());
        let o = zip::write::SimpleFileOptions::default();
        zip.start_file("pack.json", o).unwrap();
        zip.write_all(br#"{"name":"Bad","stamps":[]}"#).unwrap();
        zip.start_file("setup.exe", o).unwrap();
        zip.write_all(b"MZ").unwrap();
        zip.finish().unwrap();
        let err = read_pack(&file).err().unwrap();
        assert!(err.contains("setup.exe"), "{err}");
    }
}
