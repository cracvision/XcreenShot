//! The apps in front during a recording (VID-054): the one in front when it starts, and
//! every app that stayed in front at least 5 s, for the video's capture details. Asked
//! once a second; only the app's name and version are kept, never a window's title.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::platform::{self, AppInfo};

/// Seconds in front for an app to count (VID-054).
const AT_LEAST: u32 = 5;

/// Watches the app in front from its start until it is dropped or finished.
pub(crate) struct AppWatch {
    stop: Arc<AtomicBool>,
    first: Option<AppInfo>,
    /// Each app and its seconds in front, in the order they came.
    seen: Arc<Mutex<Vec<(String, u32)>>>,
}

/// The app in front, unless it is XcreenShot itself (its bar or its windows).
fn in_front() -> Option<(u32, AppInfo)> {
    let pid = platform::foreground_pid().filter(|p| *p != 0 && *p != std::process::id())?;
    let info = platform::app_info(pid, "");
    (!info.name.trim().is_empty()).then_some((pid, info))
}

impl AppWatch {
    pub(crate) fn start() -> AppWatch {
        let first = in_front().map(|(_, info)| info);
        let stop = Arc::new(AtomicBool::new(false));
        let seen: Arc<Mutex<Vec<(String, u32)>>> = Arc::default();
        let (stop2, seen2) = (stop.clone(), seen.clone());
        let started = std::thread::Builder::new()
            .name("rec-apps".into())
            .spawn(move || {
                // The name of the last process asked about: its version info is read once.
                let mut last: Option<(u32, String)> = None;
                while !stop2.load(Ordering::Relaxed) {
                    std::thread::sleep(Duration::from_secs(1));
                    let pid = platform::foreground_pid().filter(|p| *p != 0 && *p != std::process::id());
                    let name = match (pid, &last) {
                        (Some(p), Some((lp, n))) if p == *lp => Some(n.clone()),
                        (Some(p), _) => {
                            let n = platform::app_info(p, "").name;
                            last = Some((p, n.clone()));
                            Some(n)
                        }
                        (None, _) => None,
                    };
                    if let Some(name) = name.filter(|n| !n.trim().is_empty()) {
                        let mut s = seen2.lock().unwrap();
                        match s.iter_mut().find(|(n, _)| *n == name) {
                            Some((_, secs)) => *secs += 1,
                            None => s.push((name, 1)),
                        }
                    }
                }
            });
        if let Err(e) = started {
            tracing::warn!(error = %e, "the apps in front are not watched");
        }
        AppWatch { stop, first, seen }
    }

    /// Stops watching: the app in front at the start, and the apps in front at least
    /// 5 s (that one too, if it stayed).
    pub(crate) fn finish(&self) -> (Option<AppInfo>, Vec<String>) {
        self.stop.store(true, Ordering::Relaxed);
        let apps = self
            .seen
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, secs)| *secs >= AT_LEAST)
            .map(|(n, _)| n.clone())
            .collect();
        (self.first.clone(), apps)
    }
}

impl Drop for AppWatch {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}
