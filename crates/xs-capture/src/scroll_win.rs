//! Windows side of the scrolling capture (CAP-034, CAP-035).
//!
//! - A low-level keyboard hook, on a thread of its own, turns `Enter` and `Esc` (alone)
//!   into Done and Cancel and keeps them from the app being scrolled, which keeps the
//!   focus all along.
//! - During an automatic capture a low-level mouse hook watches for the user's own click
//!   or wheel inside the area (XcreenShot's own wheel turns are marked as injected).
//! - The wheel turns are sent with `SendInput` over the area's center; the pointer goes
//!   back where it was at the end.

use std::cell::RefCell;
use std::sync::mpsc::{self, Sender};
use std::time::Duration;

use windows_sys::Win32::Foundation::{LPARAM, LRESULT, POINT, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Threading::GetCurrentThreadId;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_HWHEEL, MOUSEEVENTF_WHEEL,
    MOUSEINPUT, VK_CONTROL, VK_ESCAPE, VK_LWIN, VK_RETURN, VK_RWIN, VK_SHIFT,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetCursorPos, GetMessageW, PostThreadMessageW, SetCursorPos,
    SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx, HC_ACTION, HHOOK, KBDLLHOOKSTRUCT,
    LLKHF_ALTDOWN, LLMHF_INJECTED, MSG, MSLLHOOKSTRUCT, WH_KEYBOARD_LL, WH_MOUSE_LL, WM_KEYDOWN, WM_KEYUP,
    WM_LBUTTONDOWN, WM_MOUSEHWHEEL, WM_MOUSEWHEEL, WM_QUIT, WM_RBUTTONDOWN, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

use crate::scroll::Action;
use crate::Rect;

/// The hooks of one scrolling capture; dropping it removes them.
pub struct Hooks {
    thread: u32,
}

struct HookState {
    actions: Sender<Action>,
    /// The area watched for the user's own clicks and wheel (automatic captures only).
    watch: Option<Rect>,
    /// A key kept from the app on its way down: keep its release too.
    held: Option<u32>,
}

thread_local! {
    static STATE: RefCell<Option<HookState>> = const { RefCell::new(None) };
}

impl Hooks {
    /// Starts the hooks. `watch`: the area of an automatic capture, where the user's own
    /// click or wheel stops it.
    pub fn start(actions: Sender<Action>, watch: Option<Rect>) -> Result<Hooks, String> {
        let (ready_tx, ready_rx) = mpsc::channel();
        std::thread::Builder::new()
            .name("scroll-capture-hook".into())
            .spawn(move || hook_thread(actions, watch, ready_tx))
            .map_err(|e| e.to_string())?;
        let thread = ready_rx
            .recv_timeout(Duration::from_secs(3))
            .map_err(|_| "the input hook did not start".to_string())??;
        Ok(Hooks { thread })
    }
}

impl Drop for Hooks {
    fn drop(&mut self) {
        // SAFETY: posting to a thread id; harmless if that thread already ended.
        unsafe { PostThreadMessageW(self.thread, WM_QUIT, 0, 0) };
    }
}

fn hook_thread(actions: Sender<Action>, watch: Option<Rect>, ready: Sender<Result<u32, String>>) {
    STATE.with(|s| {
        *s.borrow_mut() = Some(HookState {
            actions,
            watch,
            held: None,
        })
    });
    // SAFETY: plain Win32 calls on this thread; the hooks are removed before it ends.
    unsafe {
        let module = GetModuleHandleW(std::ptr::null());
        let keys = SetWindowsHookExW(WH_KEYBOARD_LL, Some(key_proc), module, 0);
        let mouse = if watch.is_some() {
            SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), module, 0)
        } else {
            std::ptr::null_mut()
        };
        if keys.is_null() || (watch.is_some() && mouse.is_null()) {
            unhook(keys);
            unhook(mouse);
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
        unhook(keys);
        unhook(mouse);
    }
    STATE.with(|s| s.borrow_mut().take());
}

fn unhook(h: HHOOK) {
    if !h.is_null() {
        // SAFETY: a hook this thread installed, removed once.
        unsafe { UnhookWindowsHookEx(h) };
    }
}

unsafe extern "system" fn key_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 {
        // SAFETY: for WH_KEYBOARD_LL, lparam points to a KBDLLHOOKSTRUCT.
        let info = unsafe { &*(lparam as *const KBDLLHOOKSTRUCT) };
        let swallow = STATE.with(|s| {
            s.borrow_mut()
                .as_mut()
                .is_some_and(|state| state.key(wparam as u32, info))
        });
        if swallow {
            return 1;
        }
    }
    // SAFETY: passing the event on, as every hook must.
    unsafe { CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam) }
}

unsafe extern "system" fn mouse_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 {
        // SAFETY: for WH_MOUSE_LL, lparam points to a MSLLHOOKSTRUCT.
        let info = unsafe { &*(lparam as *const MSLLHOOKSTRUCT) };
        STATE.with(|s| {
            if let Some(state) = s.borrow().as_ref() {
                state.mouse(wparam as u32, info);
            }
        });
    }
    // SAFETY: passing the event on, as every hook must.
    unsafe { CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam) }
}

impl HookState {
    /// `Enter` is Done and `Esc` is Cancel, both kept from the app (CAP-034, CAP-035).
    fn key(&mut self, message: u32, info: &KBDLLHOOKSTRUCT) -> bool {
        let vk = info.vkCode;
        let ours = vk == VK_RETURN as u32 || vk == VK_ESCAPE as u32;
        match message {
            WM_KEYDOWN | WM_SYSKEYDOWN if ours => {
                // SAFETY: reading the keyboard state.
                let held = |k: u16| unsafe { GetAsyncKeyState(k as i32) } as u16 & 0x8000 != 0;
                let modified = info.flags & LLKHF_ALTDOWN != 0
                    || held(VK_CONTROL)
                    || held(VK_SHIFT)
                    || held(VK_LWIN)
                    || held(VK_RWIN);
                if modified {
                    return false;
                }
                if self.held.is_none() {
                    let action = if vk == VK_RETURN as u32 {
                        Action::Done
                    } else {
                        Action::Cancel
                    };
                    let _ = self.actions.send(action);
                }
                self.held = Some(vk);
                true
            }
            WM_KEYUP | WM_SYSKEYUP if self.held == Some(vk) => {
                self.held = None;
                true
            }
            _ => false,
        }
    }

    /// The user's own click or wheel inside the area stops an automatic capture and
    /// keeps what it has (CAP-035).
    fn mouse(&self, message: u32, info: &MSLLHOOKSTRUCT) {
        let Some(area) = self.watch else {
            return;
        };
        if info.flags & LLMHF_INJECTED != 0 {
            return;
        }
        let user = matches!(
            message,
            WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MOUSEWHEEL | WM_MOUSEHWHEEL
        );
        if user && area.contains(info.pt.x, info.pt.y) {
            let _ = self.actions.send(Action::Done);
        }
    }
}

/// Where the pointer is now.
pub fn pointer() -> Option<(i32, i32)> {
    let mut p = POINT { x: 0, y: 0 };
    // SAFETY: writing into a local POINT.
    (unsafe { GetCursorPos(&mut p) } != 0).then_some((p.x, p.y))
}

/// Moves the pointer.
pub fn move_pointer(at: (i32, i32)) {
    // SAFETY: plain Win32 call.
    unsafe { SetCursorPos(at.0, at.1) };
}

/// Turns the wheel `notches` times at `at` (negative: up or left). Each notch is the
/// standard 120 units; `horizontal` uses the sideways wheel.
pub fn wheel(at: (i32, i32), notches: i32, horizontal: bool) {
    if notches == 0 {
        return;
    }
    move_pointer(at);
    // Up is positive for the vertical wheel, right for the sideways one.
    let delta = if horizontal { notches * 120 } else { -notches * 120 };
    let input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: delta as u32,
                dwFlags: if horizontal {
                    MOUSEEVENTF_HWHEEL
                } else {
                    MOUSEEVENTF_WHEEL
                },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    // SAFETY: one well-formed INPUT.
    unsafe { SendInput(1, &input, std::mem::size_of::<INPUT>() as i32) };
}
