//! macOS permissions (APP-024). Windows needs none in M0 (APP-026 lands in M1).

#[cfg(target_os = "macos")]
use xs_core::host::PermissionState;
use xs_core::host::{Permission, PermissionStatus};
use xs_core::CommandError;

#[cfg(target_os = "macos")]
mod mac {
    use super::*;
    use std::collections::BTreeSet;
    use std::path::PathBuf;
    use std::sync::Mutex;

    use block2::RcBlock;
    use objc2::runtime::Bool;
    use objc2_av_foundation::{AVAuthorizationStatus, AVCaptureDevice, AVMediaTypeAudio, AVMediaTypeVideo};

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGPreflightScreenCaptureAccess() -> bool;
        fn CGRequestScreenCaptureAccess() -> bool;
    }

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrusted() -> bool;
    }

    /// Screen Recording and Accessibility cannot tell "denied" from "never asked",
    /// so we remember which ones we already asked for.
    static ASKED: Mutex<Option<(PathBuf, BTreeSet<String>)>> = Mutex::new(None);

    pub fn init(config_dir: PathBuf) {
        let path = config_dir.join("permissions-asked.json");
        let set = std::fs::read(&path)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        *ASKED.lock().unwrap() = Some((path, set));
    }

    fn asked(p: Permission) -> bool {
        ASKED
            .lock()
            .unwrap()
            .as_ref()
            .is_some_and(|(_, s)| s.contains(p.as_str()))
    }

    fn mark_asked(p: Permission) {
        if let Some((path, set)) = ASKED.lock().unwrap().as_mut() {
            if set.insert(p.as_str().to_string()) {
                if let Ok(bytes) = serde_json::to_vec(set) {
                    let _ = xs_core::paths::write_atomic(path, &bytes);
                }
            }
        }
    }

    fn av_status(p: Permission) -> AVAuthorizationStatus {
        // SAFETY: the media type statics are provided by AVFoundation; the call is
        // documented as safe for video and audio media types.
        unsafe {
            let media = match p {
                Permission::Camera => AVMediaTypeVideo,
                _ => AVMediaTypeAudio,
            };
            match media {
                Some(m) => AVCaptureDevice::authorizationStatusForMediaType(m),
                None => AVAuthorizationStatus::NotDetermined,
            }
        }
    }

    pub fn status(p: Permission) -> PermissionStatus {
        let (state, can_request) = match p {
            Permission::ScreenRecording | Permission::Accessibility => {
                // SAFETY: plain C calls without arguments.
                let granted = unsafe {
                    if p == Permission::ScreenRecording {
                        CGPreflightScreenCaptureAccess()
                    } else {
                        AXIsProcessTrusted()
                    }
                };
                if granted {
                    (PermissionState::Granted, false)
                } else if asked(p) {
                    (PermissionState::Denied, false)
                } else {
                    (PermissionState::NotDetermined, true)
                }
            }
            Permission::Camera | Permission::Microphone => match av_status(p) {
                AVAuthorizationStatus::Authorized => (PermissionState::Granted, false),
                AVAuthorizationStatus::NotDetermined => (PermissionState::NotDetermined, true),
                _ => (PermissionState::Denied, false),
            },
        };
        PermissionStatus {
            permission: p,
            state,
            can_request,
        }
    }

    /// Shows the system request. Returns `false` when the system will not ask again
    /// and the caller should open System Settings instead.
    pub fn request(p: Permission) -> bool {
        mark_asked(p);
        match p {
            Permission::ScreenRecording => {
                // SAFETY: plain C call; shows the system prompt the first time only.
                unsafe { CGRequestScreenCaptureAccess() };
                true
            }
            Permission::Accessibility => false,
            Permission::Camera | Permission::Microphone => {
                if av_status(p) != AVAuthorizationStatus::NotDetermined {
                    return false;
                }
                let handler = RcBlock::new(|_granted: Bool| {});
                // SAFETY: valid media type and a 'static completion block.
                unsafe {
                    let media = if p == Permission::Camera {
                        AVMediaTypeVideo
                    } else {
                        AVMediaTypeAudio
                    };
                    if let Some(m) = media {
                        AVCaptureDevice::requestAccessForMediaType_completionHandler(m, &handler);
                    }
                }
                true
            }
        }
    }
}

pub fn init(_config_dir: std::path::PathBuf) {
    #[cfg(target_os = "macos")]
    mac::init(_config_dir);
}

pub fn all() -> Vec<PermissionStatus> {
    #[cfg(target_os = "macos")]
    {
        Permission::ALL.into_iter().map(mac::status).collect()
    }
    #[cfg(not(target_os = "macos"))]
    {
        Vec::new()
    }
}

/// Requests a permission; when the system no longer asks, opens its settings page.
pub fn request(
    p: Permission,
    open_uri: impl Fn(&str) -> Result<(), CommandError>,
) -> Result<(), CommandError> {
    #[cfg(target_os = "macos")]
    {
        if !mac::request(p) {
            open_uri(p.macos_settings_uri())?;
        }
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (p, open_uri);
        Err(CommandError::new(
            xs_core::ErrorCode::NotSupportedOnPlatform,
            "permissions are only requested on macOS",
        ))
    }
}
