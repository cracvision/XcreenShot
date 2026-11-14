//! The computer awake while a recording is Ready, counting down, recording or paused
//! (VID-021), and its going to sleep and waking told to whoever listens, so a
//! recording pauses when the user puts the computer to sleep.

use std::sync::{Mutex, Once};

use windows::core::PWSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Power::{
    PowerClearRequest, PowerCreateRequest, PowerRegisterSuspendResumeNotification,
    PowerRequestDisplayRequired, PowerRequestSystemRequired, PowerSetRequest,
    DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS,
};
use windows::Win32::System::Threading::{
    POWER_REQUEST_CONTEXT_SIMPLE_STRING, REASON_CONTEXT, REASON_CONTEXT_0,
};
use windows::Win32::UI::WindowsAndMessaging::{
    DEVICE_NOTIFY_CALLBACK, PBT_APMRESUMEAUTOMATIC, PBT_APMSUSPEND,
};

/// Keeps the computer from sleeping and the screen on until dropped (VID-021): a
/// power request, so any thread can hold it and drop it. What the user asks for
/// (closing the lid, Sleep in the Start menu) still happens.
pub struct StayAwake(HANDLE);

// SAFETY: a power request handle can be set, cleared and closed from any thread.
unsafe impl Send for StayAwake {}
unsafe impl Sync for StayAwake {}

/// A power request that keeps the computer and the screen awake, or None if Windows
/// refuses it.
pub fn stay_awake() -> Option<StayAwake> {
    // The reason shown by `powercfg /requests`.
    let mut reason: Vec<u16> = "XcreenShot: recording a video"
        .encode_utf16()
        .chain([0])
        .collect();
    let context = REASON_CONTEXT {
        Version: 0,
        Flags: POWER_REQUEST_CONTEXT_SIMPLE_STRING,
        Reason: REASON_CONTEXT_0 {
            SimpleReasonString: PWSTR(reason.as_mut_ptr()),
        },
    };
    // SAFETY: the context and its string live through the call; the handle is closed
    // when the guard drops.
    unsafe {
        let handle = PowerCreateRequest(&context).ok()?;
        let _ = PowerSetRequest(handle, PowerRequestSystemRequired);
        let _ = PowerSetRequest(handle, PowerRequestDisplayRequired);
        Some(StayAwake(handle))
    }
}

impl Drop for StayAwake {
    fn drop(&mut self) {
        // SAFETY: the request made by `stay_awake`, cleared and closed once.
        unsafe {
            let _ = PowerClearRequest(self.0, PowerRequestSystemRequired);
            let _ = PowerClearRequest(self.0, PowerRequestDisplayRequired);
            let _ = CloseHandle(self.0);
        }
    }
}

type Watcher = Box<dyn Fn(bool) + Send + Sync>;

static WATCHERS: Mutex<Vec<Watcher>> = Mutex::new(Vec::new());
static REGISTERED: Once = Once::new();

/// Calls `f(false)` when the computer goes to sleep and `f(true)` when it wakes up.
pub fn watch_suspend(f: impl Fn(bool) + Send + Sync + 'static) {
    WATCHERS.lock().unwrap().push(Box::new(f));
    REGISTERED.call_once(|| {
        // Kept for the life of the app, as the registration is.
        let params = Box::leak(Box::new(DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS {
            Callback: Some(changed),
            Context: std::ptr::null_mut(),
        }));
        let mut registration = std::ptr::null_mut();
        // SAFETY: the parameters live forever; Windows calls `changed` on its own thread.
        let result = unsafe {
            PowerRegisterSuspendResumeNotification(
                DEVICE_NOTIFY_CALLBACK,
                HANDLE(params as *mut DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS as *mut _),
                &mut registration,
            )
        };
        if result.0 != 0 {
            tracing::warn!(code = result.0, "video: sleep and wake are not watched");
        }
    });
}

unsafe extern "system" fn changed(
    _context: *const core::ffi::c_void,
    kind: u32,
    _setting: *const core::ffi::c_void,
) -> u32 {
    let resumed = match kind {
        PBT_APMSUSPEND => false,
        PBT_APMRESUMEAUTOMATIC => true,
        _ => return 0,
    };
    if let Ok(watchers) = WATCHERS.lock() {
        for w in watchers.iter() {
            w(resumed);
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Windows lists the request while it is held (`powercfg /requests` needs an
    /// administrator, so only that it is made and dropped without error).
    #[test]
    fn a_request_is_made_and_dropped() {
        let held = stay_awake();
        assert!(held.is_some());
        drop(held);
        // The watcher list takes a callback and calls it.
        let seen = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let s = seen.clone();
        watch_suspend(move |resumed| s.store(resumed, std::sync::atomic::Ordering::SeqCst));
        // SAFETY: the callback as Windows calls it.
        unsafe { changed(std::ptr::null(), PBT_APMRESUMEAUTOMATIC, std::ptr::null()) };
        assert!(seen.load(std::sync::atomic::Ordering::SeqCst));
    }
}
