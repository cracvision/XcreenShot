//! Windows side of live drawing (VID-048).
//!
//! - A thread of its own runs a low-level mouse and keyboard hook while the mode is on.
//!   Left and right clicks draw instead of reaching the apps; the pointer still moves
//!   and the wheel still scrolls the app underneath. Clicks on XcreenShot's own windows
//!   (the recording bar, the drawing toolbar) go through. Keys without Ctrl, Alt or Win
//!   are live drawing's (the rest are kept from the apps); with them they pass, so the
//!   global shortcuts still work (Ctrl+Z is undo).
//! - A worker sends the model's view to the overlay when it changes, at most at 60 Hz,
//!   and forgets the shapes that have faded.

use std::cell::RefCell;
use std::collections::HashSet;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use windows_sys::Win32::Foundation::{LPARAM, LRESULT, POINT, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Threading::{GetCurrentProcessId, GetCurrentThreadId};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VK_BACK, VK_CONTROL, VK_DELETE, VK_ESCAPE, VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_LWIN,
    VK_MENU, VK_RCONTROL, VK_RMENU, VK_RSHIFT, VK_RWIN, VK_SHIFT,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetAncestor, GetCursorPos, GetMessageW, GetWindowLongPtrW,
    GetWindowThreadProcessId, PostThreadMessageW, SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx,
    WindowFromPoint, GA_ROOT, GWL_EXSTYLE, HC_ACTION, HHOOK, KBDLLHOOKSTRUCT, LLKHF_ALTDOWN, MSG,
    MSLLHOOKSTRUCT, WH_KEYBOARD_LL, WH_MOUSE_LL, WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN, WM_LBUTTONUP,
    WM_MOUSEMOVE, WM_QUIT, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SYSKEYDOWN, WM_SYSKEYUP, WS_EX_TRANSPARENT,
};

use crate::lens_win::Lens;
use crate::livedraw::{Key, KeyResult, Model, Tool};

/// What the hook tells the worker.
pub enum Note {
    Changed,
    /// `Esc`: the mode ends.
    Exit,
    Quit,
}

pub struct Session {
    hook_thread: u32,
    notes: Sender<Note>,
}

impl Session {
    /// Starts the hook over the monitor whose top left is `origin`, of `size` physical
    /// pixels. `push` gets the model's view; `exit` runs when `Esc` ends the mode.
    pub fn start(
        model: Arc<Mutex<Model>>,
        origin: (i32, i32),
        size: (u32, u32),
        clock: Instant,
        push: Box<dyn Fn(serde_json::Value) + Send>,
        exit: Box<dyn Fn() + Send>,
    ) -> Result<Session, String> {
        let (notes, inbox) = mpsc::channel::<Note>();
        let (ready_tx, ready_rx) = mpsc::channel();
        let hook_notes = notes.clone();
        let hook_model = model.clone();
        std::thread::Builder::new()
            .name("livedraw-hook".into())
            .spawn(move || hook_thread(hook_model, origin, size, clock, hook_notes, ready_tx))
            .map_err(|e| e.to_string())?;
        let hook_thread = ready_rx
            .recv_timeout(Duration::from_secs(3))
            .map_err(|_| "the input hook did not start".to_string())??;
        std::thread::Builder::new()
            .name("livedraw-worker".into())
            .spawn(move || worker(model, clock, inbox, push, exit))
            .map_err(|e| e.to_string())?;
        Ok(Session { hook_thread, notes })
    }

    /// Something changed outside the hook (the toolbar, automation).
    pub fn changed(&self) {
        let _ = self.notes.send(Note::Changed);
    }

    /// Ends the hook at once (the mouse and keyboard go back to the apps); the worker
    /// keeps the overlay up to date while the shapes fade.
    pub fn stop(self) {
        // SAFETY: posting to a thread id; harmless if that thread already ended.
        unsafe { PostThreadMessageW(self.hook_thread, WM_QUIT, 0, 0) };
        let _ = self.notes.send(Note::Changed);
        let notes = self.notes;
        // The worker ends once the fading is over.
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(800));
            let _ = notes.send(Note::Quit);
        });
    }
}

/// Sends the view when it changes (at most at 60 Hz) and forgets what has faded.
fn worker(
    model: Arc<Mutex<Model>>,
    clock: Instant,
    inbox: Receiver<Note>,
    push: Box<dyn Fn(serde_json::Value) + Send>,
    exit: Box<dyn Fn() + Send>,
) {
    let mut last_push = Instant::now() - Duration::from_secs(1);
    let mut dirty = true;
    loop {
        match inbox.recv_timeout(Duration::from_millis(250)) {
            Ok(Note::Changed) => dirty = true,
            Ok(Note::Exit) => exit(),
            Ok(Note::Quit) | Err(RecvTimeoutError::Disconnected) => break,
            Err(RecvTimeoutError::Timeout) => {}
        }
        let now = clock.elapsed().as_secs_f64();
        {
            let mut m = model.lock().unwrap();
            let before = m.shapes().len();
            m.prune(now);
            if m.shapes().len() != before {
                dirty = true;
            }
        }
        if dirty && last_push.elapsed() >= Duration::from_millis(16) {
            let view = model.lock().unwrap().view(now);
            push(view);
            last_push = Instant::now();
            dirty = false;
        }
    }
}

struct HookState {
    model: Arc<Mutex<Model>>,
    origin: (i32, i32),
    size: (u32, u32),
    clock: Instant,
    notes: Sender<Note>,
    /// The left button went down on the drawing.
    drawing: bool,
    /// The right button went down and was kept: keep its release too.
    right: bool,
    /// Keys kept from the apps on their way down: their release is kept too.
    held: HashSet<u32>,
    ours: u32,
    /// The magnifier, while it is the tool (VID-049).
    lens: Option<Lens>,
}

thread_local! {
    static STATE: RefCell<Option<HookState>> = const { RefCell::new(None) };
}

fn hook_thread(
    model: Arc<Mutex<Model>>,
    origin: (i32, i32),
    size: (u32, u32),
    clock: Instant,
    notes: Sender<Note>,
    ready: Sender<Result<u32, String>>,
) {
    // SAFETY: plain Win32 calls on this thread; the hooks are removed before it ends.
    unsafe {
        STATE.with(|s| {
            *s.borrow_mut() = Some(HookState {
                model,
                origin,
                size,
                clock,
                notes,
                drawing: false,
                right: false,
                held: HashSet::new(),
                ours: GetCurrentProcessId(),
                lens: None,
            })
        });
        let module = GetModuleHandleW(std::ptr::null());
        let mouse = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), module, 0);
        let keys = SetWindowsHookExW(WH_KEYBOARD_LL, Some(key_proc), module, 0);
        if mouse.is_null() || keys.is_null() {
            unhook(mouse);
            unhook(keys);
            STATE.with(|s| s.borrow_mut().take());
            let _ = ready.send(Err("could not install the input hook".to_string()));
            return;
        }
        let _ = ready.send(Ok(GetCurrentThreadId()));
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        unhook(mouse);
        unhook(keys);
        STATE.with(|s| s.borrow_mut().take());
    }
}

fn unhook(h: HHOOK) {
    if !h.is_null() {
        // SAFETY: a hook this thread installed, removed once.
        unsafe { UnhookWindowsHookEx(h) };
    }
}

/// Runs `f` on the hook state, unless it is busy (never blocks the input).
fn with_state<R>(f: impl FnOnce(&mut HookState) -> R) -> Option<R> {
    STATE.with(|s| s.try_borrow_mut().ok().and_then(|mut s| s.as_mut().map(f)))
}

unsafe extern "system" fn mouse_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 {
        // SAFETY: for HC_ACTION, lparam points to the event's MSLLHOOKSTRUCT.
        let info = unsafe { &*(lparam as *const MSLLHOOKSTRUCT) };
        if with_state(|s| s.mouse(wparam as u32, info)).unwrap_or(false) {
            return 1;
        }
    }
    // SAFETY: passing the event on, as the hook contract asks.
    unsafe { CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam) }
}

unsafe extern "system" fn key_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 {
        // SAFETY: for HC_ACTION, lparam points to the event's KBDLLHOOKSTRUCT.
        let info = unsafe { &*(lparam as *const KBDLLHOOKSTRUCT) };
        if with_state(|s| s.key(wparam as u32, info)).unwrap_or(false) {
            return 1;
        }
    }
    // SAFETY: passing the event on, as the hook contract asks.
    unsafe { CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam) }
}

fn held(vk: u16) -> bool {
    // SAFETY: reading the keyboard state.
    unsafe { GetAsyncKeyState(vk as i32) as u16 & 0x8000 != 0 }
}

impl HookState {
    fn now(&self) -> f64 {
        self.clock.elapsed().as_secs_f64()
    }

    fn local(&self, x: i32, y: i32) -> (f64, f64) {
        ((x - self.origin.0) as f64, (y - self.origin.1) as f64)
    }

    fn on_monitor(&self, x: i32, y: i32) -> bool {
        let (lx, ly) = (x - self.origin.0, y - self.origin.1);
        lx >= 0 && ly >= 0 && (lx as u32) < self.size.0 && (ly as u32) < self.size.1
    }

    /// One of XcreenShot's own windows that takes clicks (not the overlay or the
    /// recording border, which let them through).
    fn own_window_at(&self, x: i32, y: i32) -> bool {
        // SAFETY: window queries; the handles are only read.
        unsafe {
            let hwnd = WindowFromPoint(POINT { x, y });
            if hwnd.is_null() {
                return false;
            }
            let root = GetAncestor(hwnd, GA_ROOT);
            let mut pid = 0u32;
            GetWindowThreadProcessId(root, &mut pid);
            pid == self.ours && GetWindowLongPtrW(root, GWL_EXSTYLE) as u32 & WS_EX_TRANSPARENT == 0
        }
    }

    fn changed(&self) {
        let _ = self.notes.send(Note::Changed);
    }

    /// The magnifier follows the pointer while it is the tool (VID-049), on this
    /// monitor; otherwise it goes away.
    fn follow_lens(&mut self, x: i32, y: i32) {
        let (wanted, scale) = {
            let m = self.model.lock().unwrap();
            (m.tool == Tool::Magnifier, m.scale)
        };
        if !wanted || !self.on_monitor(x, y) {
            self.lens = None;
            return;
        }
        if self.lens.is_none() {
            self.lens = Lens::new((220.0 * scale.max(1.0)) as i32);
        }
        if let Some(l) = &self.lens {
            l.move_to(x, y);
        }
    }

    /// Returns whether the event is kept from the apps.
    fn mouse(&mut self, message: u32, info: &MSLLHOOKSTRUCT) -> bool {
        let (x, y) = (info.pt.x, info.pt.y);
        let even = held(VK_SHIFT);
        self.follow_lens(x, y);
        // The magnifier only looks: clicks reach the apps (VID-049).
        if self.lens.is_some() && !self.drawing {
            return false;
        }
        match message {
            WM_LBUTTONDOWN => {
                if !self.on_monitor(x, y) || self.own_window_at(x, y) {
                    return false;
                }
                let (lx, ly) = self.local(x, y);
                let now = self.now();
                self.model.lock().unwrap().press(lx, ly, now);
                self.drawing = true;
                self.changed();
                true
            }
            WM_MOUSEMOVE => {
                if self.drawing {
                    let (lx, ly) = self.local(x, y);
                    self.model.lock().unwrap().drag(lx, ly, even);
                    self.changed();
                }
                // The pointer always moves.
                false
            }
            WM_LBUTTONUP if self.drawing => {
                self.drawing = false;
                let (lx, ly) = self.local(x, y);
                let now = self.now();
                self.model.lock().unwrap().release(lx, ly, even, now);
                self.changed();
                true
            }
            WM_RBUTTONDOWN => {
                self.right = self.on_monitor(x, y) && !self.own_window_at(x, y);
                self.right
            }
            WM_RBUTTONUP if self.right => {
                self.right = false;
                true
            }
            // The wheel and the rest reach the apps.
            _ => false,
        }
    }

    /// Returns whether the key is kept from the apps.
    fn key(&mut self, message: u32, info: &KBDLLHOOKSTRUCT) -> bool {
        let vk = info.vkCode;
        match message {
            WM_KEYDOWN | WM_SYSKEYDOWN => {
                // Modifiers alone always pass: Shift makes squares, Ctrl+Z undoes.
                let modifier = [
                    VK_SHIFT,
                    VK_LSHIFT,
                    VK_RSHIFT,
                    VK_CONTROL,
                    VK_LCONTROL,
                    VK_RCONTROL,
                    VK_MENU,
                    VK_LMENU,
                    VK_RMENU,
                    VK_LWIN,
                    VK_RWIN,
                ]
                .contains(&(vk as u16));
                if modifier {
                    return false;
                }
                let ctrl = held(VK_CONTROL);
                let alt = info.flags & LLKHF_ALTDOWN != 0;
                let win = held(VK_LWIN) || held(VK_RWIN);
                let key = if ctrl && !alt && !win && vk == b'Z' as u32 {
                    Some(Key::Undo)
                } else if ctrl || alt || win {
                    // The global shortcuts.
                    return false;
                } else if vk == VK_ESCAPE as u32 {
                    Some(Key::Escape)
                } else if vk == VK_DELETE as u32 || vk == VK_BACK as u32 {
                    Some(Key::Delete)
                } else if (b'A' as u32..=b'Z' as u32).contains(&vk)
                    || (b'0' as u32..=b'9' as u32).contains(&vk)
                {
                    Some(Key::Char(char::from(vk as u8)))
                } else {
                    None
                };
                let now = self.now();
                let result = match key {
                    Some(k) => self.model.lock().unwrap().key(k, now),
                    None => KeyResult::Ignored,
                };
                match result {
                    KeyResult::Exit => {
                        let _ = self.notes.send(Note::Exit);
                    }
                    KeyResult::Used => {
                        self.changed();
                        // `M` shows the magnifier at once; another tool takes it away.
                        let mut at = POINT { x: 0, y: 0 };
                        // SAFETY: reading the pointer's position.
                        if unsafe { GetCursorPos(&mut at) } != 0 {
                            self.follow_lens(at.x, at.y);
                        }
                    }
                    KeyResult::Ignored => {}
                }
                // Typing does not reach the apps while drawing (VID-048).
                self.held.insert(vk);
                true
            }
            WM_KEYUP | WM_SYSKEYUP => self.held.remove(&vk),
            _ => false,
        }
    }
}
