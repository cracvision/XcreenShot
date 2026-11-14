//! Windows side of multi-capture (CAP-067).
//!
//! - A thread of its own runs a low-level mouse and keyboard hook. A press over an
//!   app is held back: the window under the pointer is captured first, and the click
//!   is given back with `SendInput` (marked, so the hook lets it through). A drag is
//!   captured as a region and never reaches the app. `Esc` ends the session.
//! - Four thin windows frame what a click would capture. Like the panel, they never
//!   appear in a capture (`WDA_EXCLUDEFROMCAPTURE`).
//! - A worker thread takes the captures and gives the clicks back, in order, and
//!   keeps the list of windows under the pointer up to date; another one saves.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use windows_sys::Win32::Foundation::{CloseHandle, BOOL, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED, DWMWA_EXTENDED_FRAME_BOUNDS};
use windows_sys::Win32::Graphics::Gdi::CreateSolidBrush;
use windows_sys::Win32::Security::{
    GetSidSubAuthority, GetSidSubAuthorityCount, GetTokenInformation, TokenIntegrityLevel,
    TOKEN_MANDATORY_LABEL, TOKEN_QUERY,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Threading::{
    GetCurrentProcessId, GetCurrentThreadId, OpenProcess, OpenProcessToken, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, GetDoubleClickTime, SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_ABSOLUTE,
    MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MOVE, MOUSEEVENTF_VIRTUALDESK, MOUSEINPUT,
    VK_CONTROL, VK_ESCAPE, VK_LWIN, VK_RWIN,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, EnumWindows,
    GetClassNameW, GetMessageW, GetSystemMetrics, GetWindowLongPtrW, GetWindowRect, GetWindowTextW,
    GetWindowThreadProcessId, IsIconic, IsWindowVisible, PostThreadMessageW, RegisterClassExW,
    SetLayeredWindowAttributes, SetWindowDisplayAffinity, SetWindowPos, SetWindowsHookExW, ShowWindow,
    TranslateMessage, UnhookWindowsHookEx, GWL_EXSTYLE, HC_ACTION, HHOOK, HWND_TOPMOST, KBDLLHOOKSTRUCT,
    LLKHF_ALTDOWN, LWA_ALPHA, MSG, MSLLHOOKSTRUCT, SM_CXDOUBLECLK, SM_CXVIRTUALSCREEN, SM_CYDOUBLECLK,
    SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN, SWP_NOACTIVATE, SWP_SHOWWINDOW, SW_HIDE,
    WDA_EXCLUDEFROMCAPTURE, WH_KEYBOARD_LL, WH_MOUSE_LL, WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN, WM_LBUTTONUP,
    WM_MOUSEMOVE, WM_QUIT, WM_SYSKEYDOWN, WM_SYSKEYUP, WNDCLASSEXW, WS_EX_LAYERED, WS_EX_NOACTIVATE,
    WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
};
use xs_core::Core;

use crate::multi::{self, Effect, Gesture, Input, Over};
use crate::{Rect, Shot};

/// Marks the clicks XcreenShot gives back, so the hook lets them through.
const MAGIC: usize = 0x5853_4D43;
/// How often the list of windows under the pointer is refreshed.
const REFRESH: Duration = Duration::from_millis(400);
/// After a click the app may open a window: look again sooner.
const AFTER_CLICK: Duration = Duration::from_millis(150);
/// Width of the frame around what a click captures, in physical pixels.
const FRAME: i32 = 3;
/// Frame color: the logo's bright blue (#0C9AFC), visible on light and dark windows.
const FRAME_COLOR: u32 = 0x0C | (0x9A << 8) | (0xFC << 16);

/// A window or monitor, top of the stack first.
#[derive(Debug, Clone)]
struct Entry {
    rect: Rect,
    hwnd: isize,
    pid: u32,
    /// XcreenShot's own windows and the taskbar: clicks there are left alone.
    pass: bool,
    elevated: bool,
}

enum Job {
    Shoot {
        seq: u64,
        entry: Entry,
    },
    Drop {
        seq: u64,
    },
    Click {
        seq: Option<u64>,
        down: (i32, i32),
        up: (i32, i32),
    },
    /// A dragged region, with the window under its center for its name (CAP-056).
    Region(Rect, Option<Entry>),
    Snap(Entry),
    /// `Esc`: end the session.
    Stop,
    Quit,
}

/// A running session: its hook thread and its worker.
pub struct Session {
    hook_thread: u32,
    jobs: Sender<Job>,
}

impl Session {
    pub fn start(core: Arc<Core>) -> Result<Session, String> {
        let (jobs, inbox) = mpsc::channel::<Job>();
        let stack = Arc::new(Mutex::new(Vec::new()));
        let mut levels = HashMap::new();
        let ours = integrity(unsafe { GetCurrentProcessId() }).unwrap_or(0x2000);
        *stack.lock().unwrap() = list_stack(&mut levels, ours);

        let (ready_tx, ready_rx) = mpsc::channel();
        let hook_jobs = jobs.clone();
        let hook_stack = stack.clone();
        std::thread::Builder::new()
            .name("multicapture-hook".into())
            .spawn(move || hook_thread(hook_jobs, hook_stack, ready_tx))
            .map_err(|e| e.to_string())?;
        let hook_thread = ready_rx
            .recv_timeout(Duration::from_secs(3))
            .map_err(|_| "the input hook did not start".to_string())??;

        std::thread::Builder::new()
            .name("multicapture-worker".into())
            .spawn(move || worker(core, inbox, stack, levels, ours))
            .map_err(|e| e.to_string())?;
        Ok(Session { hook_thread, jobs })
    }

    /// Ends the hook and the worker; captures already taken are still saved.
    pub fn stop(self) {
        // SAFETY: posting to a thread id; harmless if that thread already ended.
        unsafe { PostThreadMessageW(self.hook_thread, WM_QUIT, 0, 0) };
        let _ = self.jobs.send(Job::Quit);
    }
}

// ---------------------------------------------------------------------------
// The hook thread

struct HookState {
    gesture: Gesture,
    jobs: Sender<Job>,
    stack: Arc<Mutex<Vec<Entry>>>,
    frame: Frame,
    /// What the frame shows now.
    shown: Option<Rect>,
    /// `Esc` went down and was kept from the app: keep its release too.
    esc_down: bool,
}

thread_local! {
    static STATE: RefCell<Option<HookState>> = const { RefCell::new(None) };
}

fn hook_thread(jobs: Sender<Job>, stack: Arc<Mutex<Vec<Entry>>>, ready: Sender<Result<u32, String>>) {
    // SAFETY: plain Win32 calls on this thread; the hooks are removed before it ends.
    unsafe {
        let double = (GetSystemMetrics(SM_CXDOUBLECLK), GetSystemMetrics(SM_CYDOUBLECLK));
        let frame = Frame::create();
        STATE.with(|s| {
            *s.borrow_mut() = Some(HookState {
                gesture: Gesture::new(GetDoubleClickTime(), double),
                jobs,
                stack,
                frame,
                shown: None,
                esc_down: false,
            })
        });
        let module = GetModuleHandleW(std::ptr::null());
        let mouse = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), module, 0);
        let keys = SetWindowsHookExW(WH_KEYBOARD_LL, Some(key_proc), module, 0);
        if mouse.is_null() || keys.is_null() {
            unhook(mouse);
            unhook(keys);
            end_state();
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
        end_state();
    }
}

fn unhook(h: HHOOK) {
    if !h.is_null() {
        // SAFETY: a hook this thread installed, removed once.
        unsafe { UnhookWindowsHookEx(h) };
    }
}

fn end_state() {
    if let Some(state) = STATE.with(|s| s.borrow_mut().take()) {
        state.frame.destroy();
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
        if info.dwExtraInfo != MAGIC && with_state(|s| s.mouse(wparam as u32, info)).unwrap_or(false) {
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
        if info.dwExtraInfo != MAGIC && with_state(|s| s.key(wparam as u32, info)).unwrap_or(false) {
            return 1;
        }
    }
    // SAFETY: passing the event on, as the hook contract asks.
    unsafe { CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam) }
}

impl HookState {
    /// The topmost window or monitor under the point.
    fn entry_at(&self, x: i32, y: i32) -> Option<Entry> {
        let stack = self.stack.lock().ok()?;
        stack.iter().find(|e| e.rect.contains(x, y)).cloned()
    }

    fn highlight(&mut self, rect: Option<Rect>) {
        if rect == self.shown {
            return;
        }
        match rect {
            Some(r) => self.frame.show(r),
            None => self.frame.hide(),
        }
        self.shown = rect;
    }

    /// Returns whether the event is kept from the app.
    fn mouse(&mut self, message: u32, info: &MSLLHOOKSTRUCT) -> bool {
        let (x, y) = (info.pt.x, info.pt.y);
        let input = match message {
            WM_MOUSEMOVE => Input::Move(x, y),
            WM_LBUTTONDOWN => Input::Down(x, y, info.time),
            WM_LBUTTONUP => Input::Up(x, y),
            _ => return false,
        };
        let entry = self.entry_at(x, y);
        let over = match &entry {
            Some(e) if e.pass => Over::Own,
            Some(e) if e.elevated => Over::Elevated,
            _ => Over::App,
        };
        if matches!(input, Input::Move(..)) && !self.gesture.busy() {
            let rect = entry.as_ref().filter(|e| !e.pass).map(|e| e.rect);
            self.highlight(rect);
        }
        let reaction = self.gesture.on(input, over);
        for effect in reaction.effects {
            let job = match effect {
                Effect::Shoot { seq, .. } => entry.clone().map(|entry| Job::Shoot { seq, entry }),
                Effect::Drop { seq } => Some(Job::Drop { seq }),
                Effect::Rubber(r) => {
                    self.highlight(Some(r));
                    None
                }
                Effect::Click { seq, down, up } => Some(Job::Click { seq, down, up }),
                Effect::Region(r) => {
                    self.highlight(None);
                    let under = self
                        .entry_at(r.x + r.width as i32 / 2, r.y + r.height as i32 / 2)
                        .filter(|e| e.hwnd != 0 && !e.pass);
                    Some(Job::Region(r, under))
                }
                Effect::Snap { .. } => entry.clone().map(Job::Snap),
            };
            if let Some(job) = job {
                let _ = self.jobs.send(job);
            }
        }
        reaction.swallow
    }

    /// `Esc` alone ends the session and never reaches the app.
    fn key(&mut self, message: u32, info: &KBDLLHOOKSTRUCT) -> bool {
        if info.vkCode != VK_ESCAPE as u32 {
            return false;
        }
        match message {
            WM_KEYDOWN | WM_SYSKEYDOWN => {
                // SAFETY: reading the keyboard state.
                let held = |vk: u16| unsafe { GetAsyncKeyState(vk as i32) } as u16 & 0x8000 != 0;
                let modified =
                    info.flags & LLKHF_ALTDOWN != 0 || held(VK_CONTROL) || held(VK_LWIN) || held(VK_RWIN);
                if modified {
                    return false;
                }
                if !self.esc_down {
                    self.esc_down = true;
                    let _ = self.jobs.send(Job::Stop);
                }
                true
            }
            WM_KEYUP | WM_SYSKEYUP if self.esc_down => {
                self.esc_down = false;
                true
            }
            _ => false,
        }
    }
}

// ---------------------------------------------------------------------------
// The frame

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

/// Four thin click-through windows around a rectangle, never in a capture.
struct Frame {
    sides: [HWND; 4],
}

unsafe extern "system" fn frame_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // SAFETY: default handling of a message of our own window.
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

impl Frame {
    fn create() -> Frame {
        let class = wide("XcreenShotMultiCaptureFrame");
        // SAFETY: a window class and windows of this thread; the strings outlive the calls.
        unsafe {
            let module = GetModuleHandleW(std::ptr::null());
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: 0,
                lpfnWndProc: Some(frame_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: module,
                hIcon: std::ptr::null_mut(),
                hCursor: std::ptr::null_mut(),
                hbrBackground: CreateSolidBrush(FRAME_COLOR),
                lpszMenuName: std::ptr::null(),
                lpszClassName: class.as_ptr(),
                hIconSm: std::ptr::null_mut(),
            };
            // A second session registers it again: that fails harmlessly.
            RegisterClassExW(&wc);
            let sides = [(); 4].map(|_| {
                let hwnd = CreateWindowExW(
                    WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOACTIVATE,
                    class.as_ptr(),
                    std::ptr::null(),
                    WS_POPUP,
                    0,
                    0,
                    1,
                    1,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    module,
                    std::ptr::null(),
                );
                if !hwnd.is_null() {
                    SetLayeredWindowAttributes(hwnd, 0, 255, LWA_ALPHA);
                    SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE);
                }
                hwnd
            });
            Frame { sides }
        }
    }

    /// Inside the rectangle's edges, so it shows even on a maximized window.
    fn show(&self, r: Rect) {
        let (x, y, w, h) = (r.x, r.y, r.width as i32, r.height as i32);
        let t = FRAME.min(w / 2).min(h / 2).max(1);
        let places = [
            (x, y, w, t),
            (x, y + h - t, w, t),
            (x, y, t, h),
            (x + w - t, y, t, h),
        ];
        for (hwnd, (x, y, w, h)) in self.sides.iter().zip(places) {
            if !hwnd.is_null() {
                // SAFETY: windows of this thread.
                unsafe { SetWindowPos(*hwnd, HWND_TOPMOST, x, y, w, h, SWP_NOACTIVATE | SWP_SHOWWINDOW) };
            }
        }
    }

    fn hide(&self) {
        for hwnd in self.sides.iter().filter(|h| !h.is_null()) {
            // SAFETY: windows of this thread.
            unsafe { ShowWindow(*hwnd, SW_HIDE) };
        }
    }

    fn destroy(&self) {
        for hwnd in self.sides.iter().filter(|h| !h.is_null()) {
            // SAFETY: windows of this thread, destroyed once.
            unsafe { DestroyWindow(*hwnd) };
        }
    }
}

// ---------------------------------------------------------------------------
// Windows under the pointer

unsafe extern "system" fn collect(hwnd: HWND, lparam: LPARAM) -> BOOL {
    // SAFETY: lparam is the vector list_stack passes to EnumWindows.
    let list = unsafe { &mut *(lparam as *mut Vec<HWND>) };
    list.push(hwnd);
    1
}

fn class_name(hwnd: HWND) -> String {
    let mut buf = [0u16; 64];
    // SAFETY: the buffer outlives the call.
    let n = unsafe { GetClassNameW(hwnd, buf.as_mut_ptr(), buf.len() as i32) };
    String::from_utf16_lossy(&buf[..n.max(0) as usize])
}

fn bounds(hwnd: HWND) -> Option<Rect> {
    let mut r = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    // SAFETY: out-parameter calls with the right sizes.
    let ok = unsafe {
        DwmGetWindowAttribute(
            hwnd,
            DWMWA_EXTENDED_FRAME_BOUNDS as u32,
            (&mut r as *mut RECT).cast(),
            std::mem::size_of::<RECT>() as u32,
        ) == 0
            || GetWindowRect(hwnd, &mut r) != 0
    };
    let (w, h) = (r.right - r.left, r.bottom - r.top);
    (ok && w > 1 && h > 1).then_some(Rect {
        x: r.left,
        y: r.top,
        width: w as u32,
        height: h as u32,
    })
}

/// Integrity level of a process (its mandatory label), if XcreenShot may read it.
fn integrity(pid: u32) -> Option<u32> {
    // SAFETY: handles are checked and closed; the buffer is sized by the API.
    unsafe {
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if process.is_null() {
            return None;
        }
        let mut token = std::ptr::null_mut();
        let opened = OpenProcessToken(process, TOKEN_QUERY, &mut token) != 0;
        CloseHandle(process);
        if !opened {
            return None;
        }
        let mut len = 0u32;
        GetTokenInformation(token, TokenIntegrityLevel, std::ptr::null_mut(), 0, &mut len);
        let mut buf = vec![0u64; (len as usize).div_ceil(8).max(1)];
        let ok = GetTokenInformation(token, TokenIntegrityLevel, buf.as_mut_ptr().cast(), len, &mut len) != 0;
        CloseHandle(token);
        if !ok {
            return None;
        }
        let label = &*(buf.as_ptr() as *const TOKEN_MANDATORY_LABEL);
        let sid = label.Label.Sid;
        let count = *GetSidSubAuthorityCount(sid);
        (count > 0).then(|| *GetSidSubAuthority(sid, count as u32 - 1))
    }
}

/// Visible windows top first, then the monitors (what a click on the desktop takes).
fn list_stack(levels: &mut HashMap<u32, bool>, ours: u32) -> Vec<Entry> {
    let own = unsafe { GetCurrentProcessId() };
    let mut hwnds: Vec<HWND> = Vec::new();
    // SAFETY: the callback only pushes into the vector passed as lparam.
    unsafe { EnumWindows(Some(collect), &mut hwnds as *mut Vec<HWND> as LPARAM) };
    let mut out = Vec::new();
    for hwnd in hwnds {
        // SAFETY: queries on window handles; stale handles just fail.
        let (visible, ex, pid, cloaked) = unsafe {
            if IsWindowVisible(hwnd) == 0 || IsIconic(hwnd) != 0 {
                continue;
            }
            let mut cloaked = 0u32;
            DwmGetWindowAttribute(
                hwnd,
                DWMWA_CLOAKED as u32,
                (&mut cloaked as *mut u32).cast(),
                std::mem::size_of::<u32>() as u32,
            );
            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, &mut pid);
            (true, GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32, pid, cloaked)
        };
        if !visible || cloaked != 0 || ex & WS_EX_TRANSPARENT != 0 {
            continue;
        }
        let class = class_name(hwnd);
        if matches!(class.as_str(), "Progman" | "WorkerW") {
            continue;
        }
        let taskbar = matches!(class.as_str(), "Shell_TrayWnd" | "Shell_SecondaryTrayWnd");
        let mine = pid == own;
        if !mine && !taskbar && ex & WS_EX_TOOLWINDOW != 0 {
            continue;
        }
        let Some(rect) = bounds(hwnd) else {
            continue;
        };
        let pass = mine || taskbar;
        let elevated = !pass
            && *levels
                .entry(pid)
                .or_insert_with(|| integrity(pid).is_none_or(|l| l > ours));
        out.push(Entry {
            rect,
            hwnd: hwnd as isize,
            pid,
            pass,
            elevated,
        });
    }
    if let Ok(list) = crate::monitors() {
        out.extend(list.into_iter().map(|(_, m)| Entry {
            rect: m.rect,
            hwnd: 0,
            pid: 0,
            pass: false,
            elevated: false,
        }));
    }
    out
}

// ---------------------------------------------------------------------------
// The worker

fn title(hwnd: isize) -> Option<String> {
    let mut buf = [0u16; 512];
    // SAFETY: the buffer outlives the call; a stale handle returns 0.
    let n = unsafe { GetWindowTextW(hwnd as HWND, buf.as_mut_ptr(), buf.len() as i32) };
    (n > 0).then(|| String::from_utf16_lossy(&buf[..n as usize]))
}

fn shoot_entry(core: &Core, entry: &Entry) -> Option<Shot> {
    let window = entry.hwnd != 0;
    let shot = multi::shoot(
        entry.rect,
        if window { "window" } else { "fullscreen" },
        window.then_some(entry.pid),
        if window { title(entry.hwnd) } else { None },
        core.lang(),
    );
    match shot {
        Ok(s) => Some(s),
        Err(e) => {
            tracing::warn!(error = %e.message, "multi-capture: capture failed");
            None
        }
    }
}

/// Gives a held click back to the app, at the points where it happened.
fn replay(down: (i32, i32), up: (i32, i32)) {
    // SAFETY: plain calls; SendInput reads the array for the duration of the call.
    unsafe {
        let (vx, vy) = (
            GetSystemMetrics(SM_XVIRTUALSCREEN),
            GetSystemMetrics(SM_YVIRTUALSCREEN),
        );
        let (vw, vh) = (
            GetSystemMetrics(SM_CXVIRTUALSCREEN).max(1),
            GetSystemMetrics(SM_CYVIRTUALSCREEN).max(1),
        );
        // Normalized 0..65535 over the virtual desktop, aiming at the pixel's center.
        let norm =
            |v: i32, origin: i32, size: i32| (((v - origin) as f64 + 0.5) * 65536.0 / size as f64) as i32;
        let input = |(x, y): (i32, i32), button: u32| INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: norm(x, vx, vw),
                    dy: norm(y, vy, vh),
                    mouseData: 0,
                    dwFlags: button | MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK,
                    time: 0,
                    dwExtraInfo: MAGIC,
                },
            },
        };
        let inputs = [input(down, MOUSEEVENTF_LEFTDOWN), input(up, MOUSEEVENTF_LEFTUP)];
        let sent = SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            std::mem::size_of::<INPUT>() as i32,
        );
        if sent as usize != inputs.len() {
            tracing::warn!(sent, "multi-capture: the click could not be given back");
        }
    }
}

fn worker(
    core: Arc<Core>,
    inbox: Receiver<Job>,
    stack: Arc<Mutex<Vec<Entry>>>,
    mut levels: HashMap<u32, bool>,
    ours: u32,
) {
    // Saving (PNG, thumbnail, library) never delays the next click.
    let (save, saved) = mpsc::channel::<Shot>();
    let saver = core.clone();
    let _ = std::thread::Builder::new()
        .name("multicapture-save".into())
        .spawn(move || {
            for shot in saved {
                multi::keep(&saver, shot);
            }
        });
    let mut pending: HashMap<u64, Shot> = HashMap::new();
    let mut next_refresh = Instant::now() + REFRESH;
    loop {
        let wait = next_refresh.saturating_duration_since(Instant::now());
        match inbox.recv_timeout(wait) {
            Ok(Job::Shoot { seq, entry }) => {
                if let Some(shot) = shoot_entry(&core, &entry) {
                    pending.insert(seq, shot);
                }
            }
            Ok(Job::Drop { seq }) => {
                pending.remove(&seq);
            }
            Ok(Job::Click { seq, down, up }) => {
                replay(down, up);
                if let Some(shot) = seq.and_then(|s| pending.remove(&s)) {
                    let _ = save.send(shot);
                }
                next_refresh = next_refresh.min(Instant::now() + AFTER_CLICK);
            }
            Ok(Job::Region(rect, under)) => match multi::shoot(
                rect,
                "region",
                under.as_ref().map(|e| e.pid),
                under.as_ref().and_then(|e| title(e.hwnd)),
                core.lang(),
            ) {
                Ok(shot) => {
                    let _ = save.send(shot);
                }
                Err(e) => tracing::warn!(error = %e.message, "multi-capture: region failed"),
            },
            Ok(Job::Snap(entry)) => {
                if let Some(shot) = shoot_entry(&core, &entry) {
                    let _ = save.send(shot);
                }
            }
            Ok(Job::Stop) => {
                multi::stop(&core, false);
            }
            Ok(Job::Quit) | Err(RecvTimeoutError::Disconnected) => break,
            Err(RecvTimeoutError::Timeout) => {
                let fresh = list_stack(&mut levels, ours);
                if let Ok(mut s) = stack.lock() {
                    *s = fresh;
                }
                next_refresh = Instant::now() + REFRESH;
            }
        }
    }
}
