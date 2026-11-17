//! The pictures of a document (`images/`: floating images, template zones, a guide's
//! steps): kept compressed and decoded only when drawn, through a cache of limited size.
//! A guide of hundreds of steps of a large monitor would take gigabytes decoded at once
//! (SMT-027, ADR 0009); this way only the pictures being drawn are.

use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use tiny_skia::Pixmap;

/// Decoded pictures kept for reuse, at most (premultiplied RGBA bytes).
const BUDGET: usize = 512 << 20;

/// One picture: its file's bytes and its size.
pub struct Picture {
    id: u64,
    bytes: Arc<Vec<u8>>,
    width: u32,
    height: u32,
}

impl std::fmt::Debug for Picture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Picture({}×{})", self.width, self.height)
    }
}

impl Picture {
    /// A picture from its file's bytes; only its size is read now.
    pub fn new(bytes: &[u8]) -> Result<Picture, String> {
        let (width, height) = dimensions(bytes)?;
        if width == 0 || height == 0 {
            return Err("empty image".into());
        }
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Ok(Picture {
            id: NEXT.fetch_add(1, Ordering::Relaxed),
            bytes: Arc::new(bytes.to_vec()),
            width,
            height,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// The picture decoded and premultiplied, ready to draw: from the cache, or decoded
    /// now (and kept while the cache has room).
    pub fn pixmap(&self) -> Option<Arc<Pixmap>> {
        if let Some(p) = cache().lock().unwrap().take(self.id) {
            return Some(p);
        }
        let image = crate::encode::decode(&self.bytes).ok()?;
        let pm = Arc::new(crate::render::premultiplied(&image)?);
        cache().lock().unwrap().keep(self.id, pm.clone());
        Some(pm)
    }
}

/// Width and height without decoding the pixels.
fn dimensions(bytes: &[u8]) -> Result<(u32, u32), String> {
    if let Some(size) = crate::encode::png_size(bytes) {
        return Ok(size);
    }
    let reader = image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| e.to_string())?;
    match reader.into_dimensions() {
        Ok(size) => Ok(size),
        // A format only the full decoder reads: decoded once to know.
        Err(_) => crate::encode::decode(bytes).map(|i| i.dimensions()),
    }
}

/// The pictures of a document, by file name.
#[derive(Debug, Clone, Default)]
pub struct Pictures(BTreeMap<String, Arc<Picture>>);

impl Pictures {
    /// A picture decoded, ready to draw.
    pub fn get(&self, name: &str) -> Option<Arc<Pixmap>> {
        self.0.get(name)?.pixmap()
    }

    /// A picture, not decoded.
    pub fn picture(&self, name: &str) -> Option<&Arc<Picture>> {
        self.0.get(name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.0.contains_key(name)
    }

    pub fn insert(&mut self, name: String, picture: Arc<Picture>) {
        self.0.insert(name, picture);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Arc<Picture>)> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Decoded pictures, least recently used out first when over [`BUDGET`].
#[derive(Default)]
struct Cache {
    map: HashMap<u64, (Arc<Pixmap>, u64)>,
    bytes: usize,
    clock: u64,
}

impl Cache {
    fn take(&mut self, id: u64) -> Option<Arc<Pixmap>> {
        self.clock += 1;
        let clock = self.clock;
        let entry = self.map.get_mut(&id)?;
        entry.1 = clock;
        Some(entry.0.clone())
    }

    fn keep(&mut self, id: u64, pm: Arc<Pixmap>) {
        self.clock += 1;
        let size = pm.data().len();
        if let Some(old) = self.map.insert(id, (pm, self.clock)) {
            self.bytes -= old.0.data().len();
        }
        self.bytes += size;
        while self.bytes > BUDGET && self.map.len() > 1 {
            let Some((&oldest, _)) = self
                .map
                .iter()
                .filter(|(k, _)| **k != id)
                .min_by_key(|(_, (_, used))| *used)
            else {
                break;
            };
            if let Some((p, _)) = self.map.remove(&oldest) {
                self.bytes -= p.data().len();
            }
        }
    }
}

fn cache() -> &'static Mutex<Cache> {
    static CACHE: OnceLock<Mutex<Cache>> = OnceLock::new();
    CACHE.get_or_init(Mutex::default)
}

/// Decoded bytes the cache holds now (for tests and diagnostics).
pub fn cached_bytes() -> usize {
    cache().lock().unwrap().bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ADR 0009: pictures are not decoded until drawn, and the cache never holds more
    /// than its budget, however many pictures are drawn.
    #[test]
    fn decoded_only_when_drawn_and_within_budget() {
        let img = image::RgbaImage::from_pixel(2000, 2000, image::Rgba([10, 20, 30, 255]));
        let png = crate::encode::png(&img, 0.0, true).unwrap();
        let mut all = Pictures::default();
        for i in 0..60 {
            all.insert(format!("p{i}.png"), Arc::new(Picture::new(&png).unwrap()));
        }
        assert_eq!(all.picture("p0.png").unwrap().width(), 2000);
        for i in 0..60 {
            let pm = all.get(&format!("p{i}.png")).unwrap();
            assert_eq!(pm.width(), 2000);
            assert!(cached_bytes() <= BUDGET + 16 * 1024 * 1024, "{}", cached_bytes());
        }
        assert!(Picture::new(b"not an image").is_err());
    }
}
