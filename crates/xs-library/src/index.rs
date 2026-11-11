//! The library's own index, `library.db` (LIB-022): one row per item, so listing and
//! filtering never depend on the system indexer. Everything durable also lives next to
//! the files (`meta.json`, `.xmeta`), so the index can be deleted and rebuilt.

use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension, Row};
use serde_json::{json, Value};

/// Bumped when the schema changes; an index of another version is rebuilt.
const VERSION: i64 = 1;

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
    tags TEXT NOT NULL DEFAULT '[]'
);
CREATE INDEX IF NOT EXISTS items_captured ON items (captured DESC);
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
            "kind": "image",
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
        if version != 0 && version != VERSION {
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
            "INSERT INTO items (id, path, name, format, size, modified, captured, width, height, app, url, favorite, tags)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
             ON CONFLICT(id) DO UPDATE SET path = ?2, name = ?3, format = ?4, size = ?5, modified = ?6,
               captured = ?7, width = ?8, height = ?9, app = ?10, url = ?11, favorite = ?12, tags = ?13",
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
            ],
        )?;
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
