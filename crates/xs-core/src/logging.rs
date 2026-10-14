//! Local logs (APP-046): rotate daily or at 10 MB, keep the 7 newest files and
//! at most 50 MB in total. Messages are always in English. Nothing is ever uploaded.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use chrono::{Local, NaiveDate};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::reload;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

pub const MAX_FILE_BYTES: u64 = 10 * 1024 * 1024;
pub const MAX_FILES: usize = 7;
pub const MAX_TOTAL_BYTES: u64 = 50 * 1024 * 1024;
const PREFIX: &str = "xcreenshot-";

struct State {
    dir: PathBuf,
    date: NaiveDate,
    index: u32,
    file: Option<File>,
    size: u64,
    max_file_bytes: u64,
}

impl State {
    fn path(&self) -> PathBuf {
        let date = self.date.format("%Y-%m-%d");
        if self.index == 0 {
            self.dir.join(format!("{PREFIX}{date}.log"))
        } else {
            self.dir.join(format!("{PREFIX}{date}.{}.log", self.index))
        }
    }

    fn open(&mut self) -> io::Result<()> {
        fs::create_dir_all(&self.dir)?;
        loop {
            let path = self.path();
            let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            if size < self.max_file_bytes {
                self.file = Some(OpenOptions::new().create(true).append(true).open(&path)?);
                self.size = size;
                break;
            }
            self.index += 1;
        }
        prune(&self.dir, MAX_FILES, MAX_TOTAL_BYTES, Some(&self.path()));
        Ok(())
    }

    fn rotate_if_needed(&mut self, today: NaiveDate) -> io::Result<()> {
        if today != self.date {
            self.date = today;
            self.index = 0;
            self.file = None;
        } else if self.size >= self.max_file_bytes {
            self.index += 1;
            self.file = None;
        }
        if self.file.is_none() {
            self.open()?;
        }
        Ok(())
    }
}

/// Size- and date-rotating log file writer.
#[derive(Clone)]
pub struct RollingWriter {
    state: Arc<Mutex<State>>,
}

impl RollingWriter {
    pub fn new(dir: &Path) -> RollingWriter {
        Self::with_limit(dir, MAX_FILE_BYTES)
    }

    fn with_limit(dir: &Path, max_file_bytes: u64) -> RollingWriter {
        RollingWriter {
            state: Arc::new(Mutex::new(State {
                dir: dir.to_path_buf(),
                date: Local::now().date_naive(),
                index: 0,
                file: None,
                size: 0,
                max_file_bytes,
            })),
        }
    }

    fn write_at(&self, buf: &[u8], today: NaiveDate) -> io::Result<usize> {
        let mut st = self.state.lock().unwrap();
        st.rotate_if_needed(today)?;
        let n = st.file.as_mut().expect("opened").write(buf)?;
        st.size += n as u64;
        Ok(n)
    }
}

impl Write for RollingWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write_at(buf, Local::now().date_naive())
    }

    fn flush(&mut self) -> io::Result<()> {
        match self.state.lock().unwrap().file.as_mut() {
            Some(f) => f.flush(),
            None => Ok(()),
        }
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for RollingWriter {
    type Writer = RollingWriter;
    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

fn log_files(dir: &Path) -> Vec<(PathBuf, u64, std::time::SystemTime)> {
    let mut files: Vec<_> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            name.starts_with(PREFIX) && name.ends_with(".log")
        })
        .filter_map(|e| {
            let m = e.metadata().ok()?;
            Some((e.path(), m.len(), m.modified().ok()?))
        })
        .collect();
    // Newest first.
    files.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| b.0.cmp(&a.0)));
    files
}

/// Deletes the oldest log files beyond `max_files` or `max_total` bytes, never `keep`.
pub fn prune(dir: &Path, max_files: usize, max_total: u64, keep: Option<&Path>) {
    let mut total = 0u64;
    for (i, (path, size, _)) in log_files(dir).into_iter().enumerate() {
        total += size;
        let protected = keep.is_some_and(|k| k == path);
        if !protected && (i >= max_files || total > max_total) {
            let _ = fs::remove_file(&path);
        }
    }
}

/// Last `n` lines across the newest log files (for diagnostics).
pub fn tail(dir: &Path, n: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for (path, _, _) in log_files(dir) {
        let content = fs::read_to_string(&path).unwrap_or_default();
        let mut chunk: Vec<String> = content.lines().map(str::to_string).collect();
        chunk.append(&mut lines);
        lines = chunk;
        if lines.len() >= n {
            break;
        }
    }
    let start = lines.len().saturating_sub(n);
    lines.split_off(start)
}

/// Handle to switch between normal and verbose logging at runtime.
pub struct LogControl {
    handle: reload::Handle<EnvFilter, tracing_subscriber::Registry>,
    pub dir: PathBuf,
}

fn filter(verbose: bool) -> EnvFilter {
    let base = if verbose { "debug" } else { "info" };
    // Keep third-party crates quiet in normal mode.
    EnvFilter::new(format!(
        "warn,xs_core={base},xs_cli={base},xcreenshot_app={base},xcreenshot_app_lib={base},xs_capture={base}"
    ))
}

impl LogControl {
    pub fn set_verbose(&self, verbose: bool) {
        let _ = self.handle.modify(|f| *f = filter(verbose));
    }
}

/// Installs the global subscriber writing to `dir`. Debug builds also log to stderr.
pub fn init(dir: &Path, verbose: bool) -> LogControl {
    let (filter_layer, handle) = reload::Layer::new(filter(verbose));
    let file_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_target(true)
        .with_writer(RollingWriter::new(dir));
    let stderr_layer =
        cfg!(debug_assertions).then(|| tracing_subscriber::fmt::layer().with_writer(io::stderr));
    let _ = tracing_subscriber::registry()
        .with(filter_layer)
        .with(file_layer)
        .with(stderr_layer)
        .try_init();
    LogControl {
        handle,
        dir: dir.to_path_buf(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotates_by_size_and_date() {
        let dir = tempfile::tempdir().unwrap();
        let w = RollingWriter::with_limit(dir.path(), 100);
        let day1 = NaiveDate::from_ymd_opt(2026, 10, 12).unwrap();
        let day2 = NaiveDate::from_ymd_opt(2026, 10, 13).unwrap();
        w.state.lock().unwrap().date = day1;
        for _ in 0..5 {
            w.write_at(&[b'x'; 60], day1).unwrap();
        }
        w.write_at(b"next day", day2).unwrap();
        let names: Vec<String> = log_files(dir.path())
            .into_iter()
            .map(|(p, _, _)| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert!(
            names.contains(&"xcreenshot-2026-10-12.log".to_string()),
            "{names:?}"
        );
        assert!(
            names.contains(&"xcreenshot-2026-10-12.1.log".to_string()),
            "{names:?}"
        );
        assert!(
            names.contains(&"xcreenshot-2026-10-13.log".to_string()),
            "{names:?}"
        );
    }

    #[test]
    fn prune_caps_count_and_total_size() {
        let dir = tempfile::tempdir().unwrap();
        for i in 0..10 {
            fs::write(
                dir.path().join(format!("{PREFIX}2026-10-{:02}.log", i + 1)),
                vec![b'x'; 1000],
            )
            .unwrap();
            std::thread::sleep(std::time::Duration::from_millis(15));
        }
        fs::write(dir.path().join("unrelated.txt"), b"keep").unwrap();
        prune(dir.path(), 7, 5_500, None);
        let remaining = log_files(dir.path());
        assert_eq!(remaining.len(), 5);
        assert!(dir.path().join("unrelated.txt").exists());
        assert!(dir.path().join(format!("{PREFIX}2026-10-10.log")).exists());
    }

    #[test]
    fn tail_reads_newest_lines() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(format!("{PREFIX}2026-10-01.log")), "a\nb\n").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(15));
        fs::write(dir.path().join(format!("{PREFIX}2026-10-02.log")), "c\nd\n").unwrap();
        assert_eq!(tail(dir.path(), 3), vec!["b", "c", "d"]);
    }
}
