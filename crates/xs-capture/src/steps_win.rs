//! Windows side of the step recorder (SMT-022 to SMT-027, ADR 0009).
//!
//! - A thread of its own runs low-level mouse and keyboard hooks that only listen: a
//!   click always reaches the app at once. The only key kept from the apps is the
//!   cancel key (`Esc` or `Shift+Esc`, SMT-023). Presses on XcreenShot's own windows,
//!   or outside the chosen monitor or window, are left out whole.
//! - [`Probe`] reads what was clicked with UI Automation: its kind and name, whether a
//!   check box was checked, and its containers, for the framing (SMT-025, SMT-026).
//! - With "Note keyboard use" (SMT-024) the keyboard hook tells shortcuts by their name
//!   and that something was typed, never what.

use std::cell::RefCell;
use std::sync::mpsc::{self, Sender};
use std::time::Duration;

use windows::Win32::Foundation::POINT as WPOINT;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED,
};
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, IUIAutomationElement, IUIAutomationTogglePattern, IUIAutomationTreeWalker,
    ToggleState_On, UIA_ButtonControlTypeId, UIA_CheckBoxControlTypeId, UIA_ComboBoxControlTypeId,
    UIA_DataItemControlTypeId, UIA_DocumentControlTypeId, UIA_EditControlTypeId, UIA_GroupControlTypeId,
    UIA_HyperlinkControlTypeId, UIA_ListControlTypeId, UIA_ListItemControlTypeId, UIA_MenuBarControlTypeId,
    UIA_MenuControlTypeId, UIA_MenuItemControlTypeId, UIA_PaneControlTypeId, UIA_RadioButtonControlTypeId,
    UIA_SplitButtonControlTypeId, UIA_TabItemControlTypeId, UIA_TogglePatternId, UIA_ToolBarControlTypeId,
    UIA_TreeItemControlTypeId, UIA_WindowControlTypeId,
};
use windows_sys::Win32::Foundation::{LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Threading::{GetCurrentProcessId, GetCurrentThreadId};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, GetDoubleClickTime, GetKeyNameTextW, VK_CONTROL, VK_ESCAPE, VK_LCONTROL, VK_LMENU,
    VK_LSHIFT, VK_LWIN, VK_MENU, VK_RCONTROL, VK_RMENU, VK_RSHIFT, VK_RWIN, VK_SHIFT,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetAncestor, GetForegroundWindow, GetMessageW, GetSystemMetrics,
    GetWindowThreadProcessId, PostThreadMessageW, SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx,
    WindowFromPoint, GA_ROOT, HC_ACTION, HHOOK, KBDLLHOOKSTRUCT, LLKHF_ALTDOWN, MSG, MSLLHOOKSTRUCT,
    SM_CXDOUBLECLK, WH_KEYBOARD_LL, WH_MOUSE_LL, WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN, WM_LBUTTONUP,
    WM_MOUSEMOVE, WM_QUIT, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

use crate::steps::{Button, Event, Role, Target};

/// What the hooks heard.
#[derive(Debug, Clone, PartialEq)]
pub enum Heard {
    Mouse(Event),
    /// The cancel key (SMT-023).
    Cancel,
    /// A shortcut, by its name ("Ctrl+S"), with "Note keyboard use".
    Shortcut(String),
    /// Something was typed (never what), with "Note keyboard use".
    Typed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CancelKey {
    Esc,
    ShiftEsc,
    None,
}

impl CancelKey {
    pub fn parse(s: &str) -> CancelKey {
        match s {
            "shiftEsc" => CancelKey::ShiftEsc,
            "none" => CancelKey::None,
            _ => CancelKey::Esc,
        }
    }
}

/// Which clicks count (SMT-022): all, those on one monitor, or those on one window.
#[derive(Debug, Clone, Copy, Default)]
pub struct Filter {
    pub monitor: Option<[i32; 4]>,
    pub window: Option<isize>,
    /// "One window": the window of the first click is the one (provisional, SMT-022).
    pub pick_window: bool,
}

/// The system's double click: its time, ms, and its rectangle's width, px.
pub fn double_click() -> (u64, f64) {
    // SAFETY: plain system queries.
    unsafe {
        (
            GetDoubleClickTime() as u64,
            GetSystemMetrics(SM_CXDOUBLECLK) as f64,
        )
    }
}

pub struct Hook {
    thread: u32,
}

impl Hook {
    pub fn start(
        filter: Filter,
        keyboard: bool,
        cancel: CancelKey,
        emit: Box<dyn Fn(Heard) + Send>,
    ) -> Result<Hook, String> {
        let (ready_tx, ready_rx) = mpsc::channel();
        std::thread::Builder::new()
            .name("steps-hook".into())
            .spawn(move || hook_thread(filter, keyboard, cancel, emit, ready_tx))
            .map_err(|e| e.to_string())?;
        let thread = ready_rx
            .recv_timeout(Duration::from_secs(3))
            .map_err(|_| "the input hook did not start".to_string())??;
        Ok(Hook { thread })
    }

    pub fn stop(&self) {
        // SAFETY: posting to a thread id; harmless if that thread already ended.
        unsafe { PostThreadMessageW(self.thread, WM_QUIT, 0, 0) };
    }
}

impl Drop for Hook {
    fn drop(&mut self) {
        self.stop();
    }
}

struct State {
    emit: Box<dyn Fn(Heard) + Send>,
    filter: Filter,
    keyboard: bool,
    cancel: CancelKey,
    ours: u32,
    /// The button whose press is left out, until it is released.
    skipped: Option<Button>,
    pressed: bool,
    /// The cancel key went down and was kept: its release is kept too.
    kept_key: Option<u32>,
}

thread_local! {
    static STATE: RefCell<Option<State>> = const { RefCell::new(None) };
}

fn hook_thread(
    filter: Filter,
    keyboard: bool,
    cancel: CancelKey,
    emit: Box<dyn Fn(Heard) + Send>,
    ready: Sender<Result<u32, String>>,
) {
    // SAFETY: plain Win32 calls on this thread; the hooks are removed before it ends.
    unsafe {
        STATE.with(|s| {
            *s.borrow_mut() = Some(State {
                emit,
                filter,
                keyboard,
                cancel,
                ours: GetCurrentProcessId(),
                skipped: None,
                pressed: false,
                kept_key: None,
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

fn with_state<R>(f: impl FnOnce(&mut State) -> R) -> Option<R> {
    STATE.with(|s| s.try_borrow_mut().ok().and_then(|mut s| s.as_mut().map(f)))
}

unsafe extern "system" fn mouse_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 {
        // SAFETY: for HC_ACTION, lparam points to the event's MSLLHOOKSTRUCT.
        let info = unsafe { &*(lparam as *const MSLLHOOKSTRUCT) };
        with_state(|s| s.mouse(wparam as u32, info));
    }
    // Never kept: the click reaches the app as it would without the recorder.
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

/// The root window under a point, and its process.
fn root_at(x: i32, y: i32) -> (isize, u32) {
    // SAFETY: window queries; the handles are only read.
    unsafe {
        let hwnd = WindowFromPoint(POINT { x, y });
        if hwnd.is_null() {
            return (0, 0);
        }
        let root = GetAncestor(hwnd, GA_ROOT);
        let mut pid = 0u32;
        GetWindowThreadProcessId(root, &mut pid);
        (root as isize, pid)
    }
}

impl State {
    /// Whether a press here counts (SMT-022, SMT-023).
    fn counts(&mut self, x: i32, y: i32) -> bool {
        if let Some([mx, my, mw, mh]) = self.filter.monitor {
            if x < mx || y < my || x >= mx + mw || y >= my + mh {
                return false;
            }
        }
        let (root, pid) = root_at(x, y);
        if pid == self.ours {
            return false;
        }
        if self.filter.pick_window && self.filter.window.is_none() && root != 0 {
            self.filter.window = Some(root);
        }
        self.filter.window.is_none_or(|w| w == root)
    }

    fn mouse(&mut self, message: u32, info: &MSLLHOOKSTRUCT) {
        let (x, y, t) = (info.pt.x, info.pt.y, info.time as u64);
        let (fx, fy) = (x as f64, y as f64);
        match message {
            WM_LBUTTONDOWN | WM_RBUTTONDOWN => {
                let button = if message == WM_LBUTTONDOWN {
                    Button::Left
                } else {
                    Button::Right
                };
                if self.pressed || self.skipped.is_some() {
                    return;
                }
                if !self.counts(x, y) {
                    self.skipped = Some(button);
                    return;
                }
                self.pressed = true;
                (self.emit)(Heard::Mouse(Event::Down {
                    button,
                    x: fx,
                    y: fy,
                    t,
                }));
            }
            WM_MOUSEMOVE if self.pressed => (self.emit)(Heard::Mouse(Event::Move { x: fx, y: fy })),
            WM_LBUTTONUP | WM_RBUTTONUP => {
                let button = if message == WM_LBUTTONUP {
                    Button::Left
                } else {
                    Button::Right
                };
                if self.skipped == Some(button) {
                    self.skipped = None;
                    return;
                }
                if self.pressed {
                    self.pressed = false;
                    (self.emit)(Heard::Mouse(Event::Up {
                        button,
                        x: fx,
                        y: fy,
                        t,
                    }));
                }
            }
            _ => {}
        }
    }

    /// Returns whether the key is kept from the apps (only the cancel key).
    fn key(&mut self, message: u32, info: &KBDLLHOOKSTRUCT) -> bool {
        let vk = info.vkCode;
        match message {
            WM_KEYDOWN | WM_SYSKEYDOWN => {
                let shift = held(VK_SHIFT);
                let ctrl = held(VK_CONTROL);
                let alt = info.flags & LLKHF_ALTDOWN != 0;
                let win = held(VK_LWIN) || held(VK_RWIN);
                if vk == VK_ESCAPE as u32 && !ctrl && !alt && !win {
                    let wanted = match self.cancel {
                        CancelKey::Esc => !shift,
                        CancelKey::ShiftEsc => shift,
                        CancelKey::None => false,
                    };
                    if wanted {
                        (self.emit)(Heard::Cancel);
                        self.kept_key = Some(vk);
                        return true;
                    }
                }
                if !self.keyboard || is_modifier(vk) || foreground_is_ours(self.ours) {
                    return false;
                }
                if ctrl || alt || win {
                    let mut name = Vec::new();
                    if ctrl {
                        name.push("Ctrl".to_string());
                    }
                    if alt {
                        name.push("Alt".to_string());
                    }
                    if shift {
                        name.push("Shift".to_string());
                    }
                    if win {
                        name.push("Win".to_string());
                    }
                    name.push(key_name(vk, info.scanCode, info.flags));
                    (self.emit)(Heard::Shortcut(name.join("+")));
                } else {
                    // That something was typed; never which key.
                    (self.emit)(Heard::Typed);
                }
                false
            }
            WM_KEYUP | WM_SYSKEYUP if self.kept_key == Some(vk) => {
                self.kept_key = None;
                true
            }
            _ => false,
        }
    }
}

fn is_modifier(vk: u32) -> bool {
    [
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
    .contains(&(vk as u16))
}

fn foreground_is_ours(ours: u32) -> bool {
    // SAFETY: window queries.
    unsafe {
        let w = GetForegroundWindow();
        let mut pid = 0u32;
        GetWindowThreadProcessId(w, &mut pid);
        pid == ours
    }
}

/// A key's name for a shortcut: letters and digits as they are, the rest as the
/// keyboard calls them ("F5", "Supr").
fn key_name(vk: u32, scan: u32, flags: u32) -> String {
    if (b'A' as u32..=b'Z' as u32).contains(&vk) || (b'0' as u32..=b'9' as u32).contains(&vk) {
        return char::from(vk as u8).to_string();
    }
    if (0x70..=0x87).contains(&vk) {
        return format!("F{}", vk - 0x6F);
    }
    let mut lparam = (scan as i32) << 16;
    if flags & 1 != 0 {
        lparam |= 1 << 24;
    }
    let mut buf = [0u16; 64];
    // SAFETY: a buffer of its stated length.
    let n = unsafe { GetKeyNameTextW(lparam, buf.as_mut_ptr(), buf.len() as i32) };
    if n > 0 {
        String::from_utf16_lossy(&buf[..n as usize])
    } else {
        format!("0x{vk:02X}")
    }
}

/// The visible bounds of a top-level window, desktop pixels.
pub fn window_rect(hwnd: isize) -> Option<[f64; 4]> {
    let mut r = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    // SAFETY: a struct of the attribute's size.
    let ok = unsafe {
        DwmGetWindowAttribute(
            hwnd as _,
            DWMWA_EXTENDED_FRAME_BOUNDS as u32,
            &mut r as *mut RECT as *mut _,
            std::mem::size_of::<RECT>() as u32,
        )
    } == 0;
    (ok && r.right > r.left && r.bottom > r.top).then(|| {
        [
            r.left as f64,
            r.top as f64,
            (r.right - r.left) as f64,
            (r.bottom - r.top) as f64,
        ]
    })
}

/// The window in front and its process.
pub fn foreground() -> Option<(isize, u32)> {
    // SAFETY: window queries.
    unsafe {
        let w = GetForegroundWindow();
        if w.is_null() {
            return None;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(w, &mut pid);
        Some((w as isize, pid))
    }
}

/// What was clicked (SMT-025, SMT-026).
#[derive(Debug, Clone, Default)]
pub struct Probed {
    /// Where it was asked, desktop pixels.
    pub point: (f64, f64),
    pub target: Option<Target>,
    /// Containers of the element, innermost first, desktop pixels.
    pub containers: Vec<[f64; 4]>,
    pub window: Option<[f64; 4]>,
    pub pid: u32,
}

/// UI Automation on the thread that made it (COM, multithreaded).
pub struct Probe {
    uia: Option<IUIAutomation>,
    walker: Option<IUIAutomationTreeWalker>,
}

impl Probe {
    /// Call on the thread that will use it.
    pub fn new() -> Probe {
        // SAFETY: COM on this thread, then plain UI Automation creation.
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let uia: Option<IUIAutomation> =
                CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER).ok();
            let walker = uia.as_ref().and_then(|u| u.ControlViewWalker().ok());
            Probe { uia, walker }
        }
    }

    fn parent(&self, e: &IUIAutomationElement) -> Option<IUIAutomationElement> {
        // SAFETY: plain UI Automation queries.
        unsafe { self.walker.as_ref()?.GetParentElement(e).ok() }
    }

    /// What is at a desktop point.
    pub fn at(&self, x: i32, y: i32) -> Probed {
        let (root, pid) = root_at(x, y);
        let mut out = Probed {
            point: (x as f64, y as f64),
            window: (root != 0).then(|| window_rect(root)).flatten(),
            pid,
            ..Probed::default()
        };
        let Some(uia) = &self.uia else { return out };
        // SAFETY: plain UI Automation queries; failures leave the fields empty.
        unsafe {
            let Ok(e) = uia.ElementFromPoint(WPOINT { x, y }) else {
                return out;
            };
            let kind = e.CurrentControlType().map(|t| t.0).unwrap_or(0);
            let name = e.CurrentName().map(|b| b.to_string()).unwrap_or_default();
            let parent = self.parent(&e);
            let parent_kind = parent
                .as_ref()
                .and_then(|p| p.CurrentControlType().ok())
                .map(|t| t.0)
                .unwrap_or(0);
            let role = role_of(kind, parent_kind);
            let checked = (role == Role::CheckBox)
                .then(|| {
                    e.GetCurrentPatternAs::<IUIAutomationTogglePattern>(UIA_TogglePatternId)
                        .and_then(|p| p.CurrentToggleState())
                        .ok()
                        .map(|s| s == ToggleState_On)
                })
                .flatten();
            // A window, a pane or a group says little: recognized text says more. A
            // password field's name is fine; its value is never read.
            let vague = [
                UIA_WindowControlTypeId.0,
                UIA_PaneControlTypeId.0,
                UIA_GroupControlTypeId.0,
                UIA_ToolBarControlTypeId.0,
                UIA_MenuBarControlTypeId.0,
            ]
            .contains(&kind);
            let generic = role == Role::Other && (name.trim().is_empty() || vague);
            if !generic {
                out.target = Some(Target::Element { role, name, checked });
            }
            // The containers, innermost first, up to the window (SMT-026).
            let container_kinds = [
                UIA_WindowControlTypeId.0,
                UIA_MenuControlTypeId.0,
                UIA_ToolBarControlTypeId.0,
                UIA_PaneControlTypeId.0,
                UIA_GroupControlTypeId.0,
                UIA_ListControlTypeId.0,
                UIA_MenuBarControlTypeId.0,
            ];
            let mut current = parent;
            let mut steps = 0;
            while let Some(c) = current {
                steps += 1;
                if steps > 24 {
                    break;
                }
                let k = c.CurrentControlType().map(|t| t.0).unwrap_or(0);
                if container_kinds.contains(&k) {
                    if let Ok(r) = c.CurrentBoundingRectangle() {
                        if r.right > r.left && r.bottom > r.top {
                            out.containers.push([
                                r.left as f64,
                                r.top as f64,
                                (r.right - r.left) as f64,
                                (r.bottom - r.top) as f64,
                            ]);
                        }
                    }
                }
                if k == UIA_WindowControlTypeId.0
                    && c.CurrentNativeWindowHandle().is_ok_and(|h| h.0 as isize == root)
                {
                    break;
                }
                current = self.parent(&c);
            }
        }
        out
    }

    /// The name of the focused field, for "and type in the field…" (SMT-024); never
    /// its value.
    pub fn focused_field(&self) -> Option<String> {
        let uia = self.uia.as_ref()?;
        // SAFETY: plain UI Automation queries.
        unsafe {
            let e = uia.GetFocusedElement().ok()?;
            let name = e.CurrentName().ok()?.to_string();
            (!name.trim().is_empty()).then_some(name)
        }
    }
}

/// The role of a UI Automation control type (SMT-025); a menu item on the menu bar is a
/// menu.
fn role_of(kind: i32, parent_kind: i32) -> Role {
    match kind {
        k if k == UIA_ButtonControlTypeId.0 || k == UIA_SplitButtonControlTypeId.0 => Role::Button,
        k if k == UIA_TabItemControlTypeId.0 => Role::Tab,
        k if k == UIA_MenuItemControlTypeId.0 && parent_kind == UIA_MenuBarControlTypeId.0 => Role::Menu,
        k if k == UIA_MenuItemControlTypeId.0 => Role::MenuItem,
        k if k == UIA_CheckBoxControlTypeId.0 => Role::CheckBox,
        k if k == UIA_RadioButtonControlTypeId.0 => Role::Radio,
        k if k == UIA_EditControlTypeId.0 || k == UIA_DocumentControlTypeId.0 => Role::Field,
        k if k == UIA_HyperlinkControlTypeId.0 => Role::Link,
        k if k == UIA_ListControlTypeId.0 || k == UIA_ComboBoxControlTypeId.0 => Role::List,
        k if k == UIA_ListItemControlTypeId.0
            || k == UIA_TreeItemControlTypeId.0
            || k == UIA_DataItemControlTypeId.0 =>
        {
            Role::ListItem
        }
        _ => Role::Other,
    }
}
