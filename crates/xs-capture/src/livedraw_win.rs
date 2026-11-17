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
//! - The hook can never keep the mouse and keyboard: `Esc` removes it at once on its own
//!   thread, without waiting for the rest of the app; it never waits for a lock (a busy
//!   model lets the event through); and when the worker stops answering for 2 s (the app
//!   hangs), the hook removes itself. `XS_INPUT_RELEASE_SECS` also removes it after
//!   that many seconds (supervised tests).

use std::cell::RefCell;
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex, MutexGuard};
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
    GetWindowThreadProcessId, KillTimer, PostQuitMessage, PostThreadMessageW, SetTimer, SetWindowsHookExW,
    TranslateMessage, UnhookWindowsHookEx, WindowFromPoint, GA_ROOT, GWL_EXSTYLE, HC_ACTION, HHOOK,
    KBDLLHOOKSTRUCT, LLKHF_ALTDOWN, MSG, MSLLHOOKSTRUCT, WH_KEYBOARD_LL, WH_MOUSE_LL, WM_KEYDOWN, WM_KEYUP,
    WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_QUIT, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SYSKEYDOWN,
    WM_SYSKEYUP, WM_TIMER, WS_EX_TRANSPARENT,
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

/// Without news from the worker for this long, the hook removes itself.
const STALE_MS: u64 = 2000;

/// Milliseconds since `clock`.
fn ms(clock: Instant) -> u64 {
    clock.elapsed().as_millis() as u64
}

pub struct Session {
    /// The hook's thread; `None` without a hook (an isolated test copy: only automation
    /// draws, the mouse and keyboard are never touched).
    hook_thread: Option<u32>,
    notes: Sender<Note>,
}

impl Session {
    /// Starts the hook over the monitor whose top left is `origin`, of `size` physical
    /// pixels. `push` gets the model's view; `exit` runs when `Esc` ends the mode (or
    /// the hook removed itself). Without `hook`, nothing listens to the mouse and
    /// keyboard.
    pub fn start(
        model: Arc<Mutex<Model>>,
        origin: (i32, i32),
        size: (u32, u32),
        clock: Instant,
        hook: bool,
        push: Box<dyn Fn(serde_json::Value) + Send>,
        exit: Box<dyn Fn() + Send>,
    ) -> Result<Session, String> {
        let (notes, inbox) = mpsc::channel::<Note>();
        let alive = Arc::new(AtomicU64::new(ms(clock)));
        let hook_thread = if hook {
            let (ready_tx, ready_rx) = mpsc::channel();
            let (hook_model, hook_notes, hook_alive) = (model.clone(), notes.clone(), alive.clone());
            let limit = crate::release_limit();
            std::thread::Builder::new()
                .name("livedraw-hook".into())
                .spawn(move || {
                    // Made on its thread: the magnifier's window belongs to it.
                    let state = HookState {
                        model: hook_model,
                        origin,
                        size,
                        clock,
                        notes: hook_notes,
                        alive: hook_alive,
                        limit,
                        released: false,
                        drawing: false,
                        right: false,
                        held: HashSet::new(),
                        ours: 0,
                        lens: None,
                    };
                    hook_thread(state, ready_tx)
                })
                .map_err(|e| e.to_string())?;
            Some(
                ready_rx
                    .recv_timeout(Duration::from_secs(3))
                    .map_err(|_| "the input hook did not start".to_string())??,
            )
        } else {
            None
        };
        std::thread::Builder::new()
            .name("livedraw-worker".into())
            .spawn(move || worker(model, clock, alive, inbox, push, exit))
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
        if let Some(thread) = self.hook_thread {
            // SAFETY: posting to a thread id; harmless if that thread already ended.
            unsafe { PostThreadMessageW(thread, WM_QUIT, 0, 0) };
        }
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
    alive: Arc<AtomicU64>,
    inbox: Receiver<Note>,
    push: Box<dyn Fn(serde_json::Value) + Send>,
    exit: Box<dyn Fn() + Send>,
) {
    let mut last_push = Instant::now() - Duration::from_secs(1);
    let mut dirty = true;
    loop {
        // The hook's sign that the app still answers.
        alive.store(ms(clock), Ordering::Relaxed);
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
    /// When the worker last showed it is alive (ms of `clock`).
    alive: Arc<AtomicU64>,
    /// Seconds after which the hook removes itself (`XS_INPUT_RELEASE_SECS`).
    limit: Option<f64>,
    /// The hook gave the mouse and keyboard back: every event passes until it is gone.
    released: bool,
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

fn hook_thread(mut state: HookState, ready: Sender<Result<u32, String>>) {
    // SAFETY: plain Win32 calls on this thread; the hooks are removed before it ends.
    unsafe {
        state.ours = GetCurrentProcessId();
        STATE.with(|s| *s.borrow_mut() = Some(state));
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
        // A beat to notice, even without input, that the app stopped answering.
        let timer = SetTimer(std::ptr::null_mut(), 0, 250, None);
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            if msg.message == WM_TIMER {
                if with_state(|s| s.should_release()).unwrap_or(false) {
                    break;
                }
                continue;
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        // The input goes back to the apps before anything else.
        unhook(mouse);
        unhook(keys);
        if timer != 0 {
            KillTimer(std::ptr::null_mut(), timer);
        }
        // Gone on its own (Esc, stale or time limit): the mode ends too.
        STATE.with(|s| {
            if let Some(s) = s.borrow().as_ref() {
                let _ = s.notes.send(Note::Exit);
            }
        });
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
        let Some((wanted, scale)) = self.model().map(|m| (m.tool == Tool::Magnifier, m.scale)) else {
            return;
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

    /// The model, unless someone else has it now: the hook never waits.
    fn model(&self) -> Option<MutexGuard<'_, Model>> {
        self.model.try_lock().ok()
    }

    /// Whether the hook must give the input back: after `Esc`, when the worker stopped
    /// answering (the app hangs) or after the time limit.
    fn should_release(&self) -> bool {
        self.released
            || ms(self.clock).saturating_sub(self.alive.load(Ordering::Relaxed)) > STALE_MS
            || self.limit.is_some_and(|l| self.now() >= l)
    }

    /// Gives the mouse and keyboard back at once: the hook's loop ends and removes the
    /// hooks; until then every event passes.
    fn release(&mut self) {
        if !self.released {
            self.released = true;
            self.lens = None;
            // SAFETY: ends this thread's message loop.
            unsafe { PostQuitMessage(0) };
        }
    }

    /// Returns whether the event is kept from the apps.
    fn mouse(&mut self, message: u32, info: &MSLLHOOKSTRUCT) -> bool {
        if self.should_release() {
            self.release();
            return false;
        }
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
                let Some(mut m) = self.model() else {
                    return false;
                };
                m.press(lx, ly, now);
                drop(m);
                self.drawing = true;
                self.changed();
                true
            }
            WM_MOUSEMOVE => {
                if self.drawing {
                    let (lx, ly) = self.local(x, y);
                    if let Some(mut m) = self.model() {
                        m.drag(lx, ly, even);
                        drop(m);
                        self.changed();
                    }
                }
                // The pointer always moves.
                false
            }
            WM_LBUTTONUP if self.drawing => {
                self.drawing = false;
                let (lx, ly) = self.local(x, y);
                let now = self.now();
                if let Some(mut m) = self.model() {
                    m.release(lx, ly, even, now);
                }
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
        if self.should_release() {
            self.release();
            return false;
        }
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
                    // Out at once, here: nothing else has to answer first (the mode
                    // ends when the hook's loop does).
                    self.release();
                    return true;
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
                    Some(k) => self.model().map_or(KeyResult::Ignored, |mut m| m.key(k, now)),
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A hook's state without the hook: nothing here listens to the real input.
    fn state(clock: Instant) -> (HookState, Receiver<Note>) {
        let (notes, inbox) = mpsc::channel();
        let s = HookState {
            model: Arc::new(Mutex::new(Model::default())),
            origin: (-100_000, -100_000),
            size: (1000, 1000),
            clock,
            notes,
            alive: Arc::new(AtomicU64::new(ms(clock))),
            limit: None,
            released: false,
            drawing: false,
            right: false,
            held: HashSet::new(),
            ours: 0,
            lens: None,
        };
        (s, inbox)
    }

    fn key(vk: u16) -> KBDLLHOOKSTRUCT {
        KBDLLHOOKSTRUCT {
            vkCode: vk as u32,
            scanCode: 0,
            flags: 0,
            time: 0,
            dwExtraInfo: 0,
        }
    }

    fn click(x: i32, y: i32) -> MSLLHOOKSTRUCT {
        MSLLHOOKSTRUCT {
            pt: POINT { x, y },
            mouseData: 0,
            flags: 0,
            time: 0,
            dwExtraInfo: 0,
        }
    }

    /// `Esc` gives the input back at once, without the model; after it everything passes.
    #[test]
    fn escape_releases_at_once() {
        let (mut s, _inbox) = state(Instant::now());
        // Even with the model busy elsewhere.
        let model = s.model.clone();
        let _busy = model.lock().unwrap();
        assert!(s.key(WM_KEYDOWN, &key(VK_ESCAPE)));
        assert!(s.released);
        assert!(!s.key(WM_KEYDOWN, &key(b'A' as u16)));
        assert!(!s.mouse(WM_LBUTTONDOWN, &click(-99_500, -99_500)));
    }

    /// A busy model never makes the hook wait: the click reaches the apps.
    #[test]
    fn a_busy_model_lets_the_input_through() {
        let (mut s, _inbox) = state(Instant::now());
        let model = s.model.clone();
        let busy = model.lock().unwrap();
        assert!(!s.mouse(WM_LBUTTONDOWN, &click(-99_500, -99_500)));
        assert!(!s.drawing);
        drop(busy);
        assert!(s.mouse(WM_LBUTTONDOWN, &click(-99_500, -99_500)));
        assert!(s.drawing);
    }

    /// When the app stops answering for 2 s, or after the time limit, the hook lets go.
    #[test]
    fn a_hung_app_or_the_limit_releases() {
        let started = Instant::now() - Duration::from_secs(5);
        let (mut s, _inbox) = state(started);
        s.alive.store(0, Ordering::Relaxed);
        assert!(s.should_release());
        assert!(!s.key(WM_KEYDOWN, &key(b'A' as u16)));
        assert!(s.released);

        let (mut s, _inbox) = state(started);
        assert!(!s.should_release());
        s.limit = Some(4.0);
        assert!(s.should_release());
    }
}
