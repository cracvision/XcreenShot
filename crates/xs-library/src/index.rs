//! The library's own index, `library.db` (LIB-022): one row per item, so listing and
//! filtering never depend on the system indexer. Everything durable also lives next to
//! the files (`meta.json`, `.xmeta`), so the index can be deleted and rebuilt.

use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension, Row};
use serde_json::{json, Value};

/// Bumped when the schema changes; an index of another version is rebuilt.
/// 2: the text of annotations (LIB-007), and the app's name read from CAP-056's
/// `{name, version, id}`. 3: whether the file is only in the cloud (LIB-029).
/// 4: the steps of a guide (SMT-035).
const VERSION: i64 = 4;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS items (
    id TEXT PRIMARY KEY,
    path TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    format TEXT NOT NULL,
    size INTEGER NOT NULL,
    modified INTEGER NOT NULL,
    captured INTEGER NOT NULL,
    width INTEGER,
    height INTEGER,
    app TEXT,
    url TEXT,
    favorite INTEGER NOT NULL DEFAULT 0,
    tags TEXT NOT NULL DEFAULT '[]',
    text TEXT NOT NULL DEFAULT '',
    cloud INTEGER NOT NULL DEFAULT 0,
    steps INTEGER
);
CREATE INDEX IF NOT EXISTS items_captured ON items (captured DESC);
CREATE TABLE IF NOT EXISTS seen (
    id TEXT PRIMARY KEY,
    modified INTEGER NOT NULL,
    text TEXT NOT NULL
);
";

/// An indexed item. `path` is relative to the library folder, with `/` separators.
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub id: String,
    pub path: String,
    pub name: String,
    /// `xshot`, `png` or `jpg`.
    pub format: String,
    pub size: u64,
    /// Milliseconds since 1970.
    pub modified: i64,
    /// Capture time (CAP-056), or the file's time for a loose file.
    pub captured: i64,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub app: Option<String>,
    pub url: Option<String>,
    pub favorite: bool,
    pub tags: Vec<String>,
    /// What its annotations say, one line each, for the search (LIB-007).
    pub text: String,
    /// Only in the cloud: not downloaded on this computer (LIB-029).
    pub cloud: bool,
    /// A guide of steps: how many (SMT-035).
    pub steps: Option<u32>,
}

impl Item {
    fn from_row(r: &Row) -> rusqlite::Result<Item> {
        let tags: String = r.get("tags")?;
        Ok(Item {
            id: r.get("id")?,
            path: r.get("path")?,
            name: r.get("name")?,
            format: r.get("format")?,
            size: r.get::<_, i64>("size")?.max(0) as u64,
            modified: r.get("modified")?,
            captured: r.get("captured")?,
            width: r.get("width")?,
            height: r.get("height")?,
            app: r.get("app")?,
            url: r.get("url")?,
            favorite: r.get::<_, i64>("favorite")? != 0,
            tags: serde_json::from_str(&tags).unwrap_or_default(),
            text: r.get("text")?,
            cloud: r.get::<_, i64>("cloud")? != 0,
            steps: r.get::<_, Option<i64>>("steps")?.map(|n| n.max(0) as u32),
        })
    }

    /// What commands and the Library view get. `root` makes the path absolute.
    pub fn to_json(&self, root: &Path) -> Value {
        let abs = root.join(self.path.replace('/', std::path::MAIN_SEPARATOR_STR));
        let time = |ms: i64| {
            chrono::DateTime::from_timestamp_millis(ms)
                .map(|t| t.to_rfc3339())
                .unwrap_or_default()
        };
        json!({
            "id": self.id,
            "name": self.name,
            "path": abs,
            "kind": crate::query::kind(self),
            "format": self.format,
            "size": self.size,
            "modified": time(self.modified),
            "captured": time(self.captured),
            "width": self.width,
            "height": self.height,
            "app": self.app,
            "url": self.url,
            "favorite": self.favorite,
            "tags": self.tags,
            "cloud": self.cloud,
            "steps": self.steps,
        })
    }
}

pub struct Index {
    db: Connection,
}

impl Index {
    /// Opens the index at `path`, building it again if it is damaged or of another
    /// version (LIB-022, LIB-023): nothing is lost, the files hold the data.
    pub fn open(path: &Path) -> rusqlite::Result<Index> {
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        match Index::try_open(path) {
            Ok(index) => Ok(index),
            Err(e) => {
                tracing::warn!(error = %e, "library index unusable, rebuilding it");
                for suffix in ["", "-journal", "-wal", "-shm"] {
                    let mut p = path.as_os_str().to_os_string();
                    p.push(suffix);
                    let _ = std::fs::remove_file(p);
                }
                Index::try_open(path)
            }
        }
    }

    fn try_open(path: &Path) -> rusqlite::Result<Index> {
        let db = Connection::open(path)?;
        let version: i64 = db.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        // Versions 2 and 3 only lack columns: added, nothing to read again. A guide
        // already indexed is found again at its next change.
        if version == 2 {
            db.execute_batch("ALTER TABLE items ADD COLUMN cloud INTEGER NOT NULL DEFAULT 0")?;
        }
        if version == 2 || version == 3 {
            db.execute_batch("ALTER TABLE items ADD COLUMN steps INTEGER")?;
        } else if version != 0 && version != VERSION {
            return Err(rusqlite::Error::InvalidQuery);
        }
        db.execute_batch(SCHEMA)?;
        db.execute_batch(&format!("PRAGMA user_version = {VERSION}"))?;
        // A damaged file opens but fails on the first real read.
        db.query_row("SELECT COUNT(*) FROM items", [], |r| r.get::<_, i64>(0))?;
        Ok(Index { db })
    }

    pub fn in_memory() -> Index {
        let db = Connection::open_in_memory().expect("in-memory SQLite");
        db.execute_batch(SCHEMA).expect("schema");
        Index { db }
    }

    pub fn upsert(&self, item: &Item) -> rusqlite::Result<()> {
        self.db.execute(
            "INSERT INTO items (id, path, name, format, size, modified, captured, width, height, app, url, favorite, tags, text, cloud, steps)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
             ON CONFLICT(id) DO UPDATE SET path = ?2, name = ?3, format = ?4, size = ?5, modified = ?6,
               captured = ?7, width = ?8, height = ?9, app = ?10, url = ?11, favorite = ?12, tags = ?13, text = ?14,
               cloud = ?15, steps = ?16",
            params![
                item.id,
                item.path,
                item.name,
                item.format,
                item.size as i64,
                item.modified,
                item.captured,
                item.width,
                item.height,
                item.app,
                item.url,
                item.favorite as i64,
                serde_json::to_string(&item.tags).unwrap_or_else(|_| "[]".into()),
                item.text,
                item.cloud as i64,
                item.steps.map(|n| n as i64),
            ],
        )?;
        Ok(())
    }

    /// Tags and favorite of an item (LIB-009, LIB-011): its file and dates stay.
    pub fn set_marks(&self, id: &str, tags: &[String], favorite: bool) -> rusqlite::Result<()> {
        self.db.execute(
            "UPDATE items SET tags = ?2, favorite = ?3 WHERE id = ?1",
            params![
                id,
                serde_json::to_string(tags).unwrap_or_else(|_| "[]".into()),
                favorite as i64
            ],
        )?;
        Ok(())
    }

    /// The text read in an item's image, for the version of the file it was read from
    /// (LIB-008).
    pub fn set_seen(&self, id: &str, modified: i64, text: &str) -> rusqlite::Result<()> {
        self.db.execute(
            "INSERT INTO seen (id, modified, text) VALUES (?1, ?2, ?3)
             ON CONFLICT(id) DO UPDATE SET modified = ?2, text = ?3",
            params![id, modified, text],
        )?;
        Ok(())
    }

    /// The text read of each item whose file has not changed since (LIB-008).
    pub fn seen_map(&self) -> rusqlite::Result<std::collections::HashMap<String, String>> {
        let mut q = self.db.prepare(
            "SELECT seen.id, seen.text FROM seen JOIN items ON items.id = seen.id AND items.modified = seen.modified",
        )?;
        let rows = q.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
        rows.collect()
    }

    /// Images still to read, newest first: pictures and documents, not videos, not
    /// only in the cloud (LIB-008).
    pub fn unseen(&self, limit: usize) -> rusqlite::Result<Vec<Item>> {
        let mut q = self.db.prepare(
            "SELECT * FROM items WHERE cloud = 0 AND format IN ('xshot', 'png', 'jpg', 'jpeg')
             AND NOT EXISTS (SELECT 1 FROM seen WHERE seen.id = items.id AND seen.modified = items.modified)
             ORDER BY captured DESC LIMIT ?1",
        )?;
        let rows = q.query_map([limit as i64], Item::from_row)?;
        rows.collect()
    }

    /// How many images are read and how many wait (LIB-008).
    pub fn seen_counts(&self) -> rusqlite::Result<(u64, u64)> {
        let done: i64 = self.db.query_row(
            "SELECT COUNT(*) FROM seen JOIN items ON items.id = seen.id AND items.modified = seen.modified",
            [],
            |r| r.get(0),
        )?;
        let waiting: i64 = self.db.query_row(
            "SELECT COUNT(*) FROM items WHERE cloud = 0 AND format IN ('xshot', 'png', 'jpg', 'jpeg')
             AND NOT EXISTS (SELECT 1 FROM seen WHERE seen.id = items.id AND seen.modified = items.modified)",
            [],
            |r| r.get(0),
        )?;
        Ok((done.max(0) as u64, waiting.max(0) as u64))
    }

    /// Forgets every text read (LIB-008 switched off).
    pub fn clear_seen(&self) -> rusqlite::Result<()> {
        self.db.execute("DELETE FROM seen", [])?;
        Ok(())
    }

    /// Empties the index; a scan fills it again from the files (LIB-023).
    pub fn clear(&self) -> rusqlite::Result<()> {
        self.db.execute("DELETE FROM items", [])?;
        Ok(())
    }

    pub fn remove(&self, id: &str) -> rusqlite::Result<()> {
        self.db.execute("DELETE FROM items WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn get(&self, id: &str) -> rusqlite::Result<Option<Item>> {
        self.db
            .query_row("SELECT * FROM items WHERE id = ?1", [id], Item::from_row)
            .optional()
    }

    pub fn by_path(&self, path: &str) -> rusqlite::Result<Option<Item>> {
        self.db
            .query_row("SELECT * FROM items WHERE path = ?1", [path], Item::from_row)
            .optional()
    }

    /// Every item, newest capture first (LIB-002).
    pub fn all(&self) -> rusqlite::Result<Vec<Item>> {
        let mut q = self
            .db
            .prepare("SELECT * FROM items ORDER BY captured DESC, path")?;
        let rows = q.query_map([], Item::from_row)?;
        rows.collect()
    }

    pub fn count(&self) -> rusqlite::Result<u64> {
        self.db
            .query_row("SELECT COUNT(*) FROM items", [], |r| r.get::<_, i64>(0))
            .map(|n| n.max(0) as u64)
    }
}
