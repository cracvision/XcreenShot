//! Window picker (CAP-030): every window of the current desktop, covered and minimized
//! ones too, laid out apart with its app and title; the one picked is captured at its
//! real size (CAP-027). A minimized window is restored for a moment, captured and
//! minimized again. Windows of other virtual desktops are not listed.
//!
//! The picker opens at once; the thumbnails arrive as they are taken.

use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::Serialize;
use serde_json::{json, Value};
use xs_core::i18n::tr;
use xs_core::{CommandError, CommandResult, Context, Core, ErrorCode};

use crate::{capture_window, finish, platform, pointer, Options, WindowTarget};

/// Longest side of a thumbnail, in pixels.
const THUMB: u32 = 360;
/// Most windows listed.
const MOST: usize = 60;

/// A window the picker shows.
#[derive(Debug, Clone, Serialize)]
pub struct Entry {
    pub id: u32,
    pub app: String,
    pub title: String,
    pub minimized: bool,
    /// Its thumbnail is ready (`/picker/<id>`).
    pub thumb: bool,
}

/// What the user does in the picker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    Window(u32),
    Cancel,
}

/// What reaches the open picker: the user's choice, or a thumbnail taken.
enum Msg {
    Choice(Choice),
    Thumb(u32, Vec<u8>),
}

#[derive(Default)]
struct Shared {
    choices: Option<Sender<Msg>>,
    entries: Vec<Entry>,
    thumbs: std::collections::HashMap<u32, Arc<Vec<u8>>>,
}

#[derive(Default)]
pub struct PickerState {
    shared: Mutex<Shared>,
}

fn service(core: &Core) -> Arc<PickerState> {
    core.service(PickerState::default)
}

/// The picker's list (`capture.picker_state`).
pub fn state(core: &Core) -> Value {
    let s = service(core);
    let shared = s.shared.lock().unwrap();
    json!({ "open": shared.choices.is_some(), "windows": shared.entries })
}

/// A window picked, or Cancel; false when no picker is open.
pub fn choose(core: &Core, choice: Choice) -> bool {
    let s = service(core);
    let shared = s.shared.lock().unwrap();
    shared
        .choices
        .as_ref()
        .is_some_and(|tx| tx.send(Msg::Choice(choice)).is_ok())
}

/// A thumbnail, as PNG.
pub fn thumbnail_png(core: &Core, id: u32) -> Option<Arc<Vec<u8>>> {
    service(core).shared.lock().unwrap().thumbs.get(&id).cloned()
}

/// The windows to show, topmost first.
fn windows() -> Vec<(xcap::Window, Entry)> {
    let own = std::process::id();
    let mut list: Vec<(xcap::Window, Entry)> = xcap::Window::all()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|w| {
            let id = w.id().ok()?;
            let minimized = w.is_minimized().unwrap_or(false);
            let ok = w.pid().ok() != Some(own)
                && platform::is_regular_window(id)
                && !platform::is_cloaked(id)
                && platform::is_app_window(id)
                && (minimized || (w.width().unwrap_or(0) > 1 && w.height().unwrap_or(0) > 1));
            let title = w.title().unwrap_or_default();
            (ok && !title.trim().is_empty()).then(|| {
                let app = platform::app_info(w.pid().unwrap_or(0), &w.app_name().unwrap_or_default()).name;
                (
                    w,
                    Entry {
                        id,
                        app,
                        title,
                        minimized,
                        thumb: false,
                    },
                )
            })
        })
        .collect();
    list.sort_by_key(|(w, _)| w.z().unwrap_or(i32::MAX));
    list.truncate(MOST);
    list
}

/// Opens the picker and blocks until a window is picked (and captured) or it is
/// cancelled (CAP-030).
pub fn run(core: &Core, ctx: &Context, opts: &Options) -> CommandResult {
    let lang = ctx.lang;
    // The pointer counts only if it is over the window picked (CAP-030, CAP-047).
    let pointer_now = opts.cursor.then(pointer::grab).flatten();
    let at = platform::cursor_position();
    let found = windows();
    let (tx, rx) = mpsc::channel();
    {
        let s = service(core);
        let mut shared = s.shared.lock().unwrap();
        shared.choices = Some(tx);
        shared.entries = found.iter().map(|(_, e)| e.clone()).collect();
        shared.thumbs.clear();
    }
    core.host().picker_window(true);
    core.host().picker_event(state(core));
    // The thumbnails, one by one, while the picker is open.
    let thumbs_tx = {
        let s = service(core);
        let shared = s.shared.lock().unwrap();
        shared.choices.clone()
    };
    let to_thumb: Vec<(u32, xcap::Window)> = found
        .into_iter()
        .filter(|(_, e)| !e.minimized)
        .map(|(w, e)| (e.id, w))
        .collect();
    if let Some(tx) = thumbs_tx {
        std::thread::spawn(move || {
            for (id, w) in to_thumb {
                let Ok(image) = w.capture_image() else {
                    continue;
                };
                let small = xs_image::encode::thumbnail(&image, THUMB);
                let Ok(png) = xs_image::encode::png(&small, 0.0, true) else {
                    continue;
                };
                // The picker is gone: stop.
                if tx.send(Msg::Thumb(id, png)).is_err() {
                    break;
                }
            }
        });
    }
    let choice = loop {
        match rx.recv() {
            Ok(Msg::Choice(c)) => break c,
            Ok(Msg::Thumb(id, png)) => {
                {
                    let s = service(core);
                    let mut shared = s.shared.lock().unwrap();
                    shared.thumbs.insert(id, Arc::new(png));
                    if let Some(e) = shared.entries.iter_mut().find(|e| e.id == id) {
                        e.thumb = true;
                    }
                }
                core.host().picker_event(state(core));
            }
            Err(_) => break Choice::Cancel,
        }
    };
    let minimized = {
        let s = service(core);
        let mut shared = s.shared.lock().unwrap();
        shared.choices = None;
        match choice {
            Choice::Window(id) => shared.entries.iter().find(|e| e.id == id).map(|e| e.minimized),
            Choice::Cancel => None,
        }
    };
    core.host().picker_window(false);
    let Choice::Window(id) = choice else {
        return Err(crate::cancelled(lang));
    };
    let minimized = minimized.ok_or_else(|| {
        CommandError::new(
            ErrorCode::NotFound,
            tr(lang, "errors.window_not_found", &[("title", &id.to_string())]),
        )
    })?;
    // Give the picker time to go before the window is taken.
    std::thread::sleep(Duration::from_millis(120));
    if minimized {
        platform::show_window(id, true);
        std::thread::sleep(Duration::from_millis(450));
    }
    let shot = capture_window(&WindowTarget::Id(id), lang);
    if minimized {
        platform::show_window(id, false);
    }
    let mut shot = shot?;
    let over = at.is_some_and(|(x, y)| shot.rect.contains(x, y));
    shot.pointer = pointer_now.filter(|_| over);
    finish(core, ctx, shot, opts)
}
