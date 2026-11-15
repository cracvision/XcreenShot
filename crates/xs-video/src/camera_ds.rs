//! DirectShow cameras (VID-031): the virtual cameras many apps install only there
//! (GoPro Webcam, NVIDIA Broadcast, OBS and others) and that Media Foundation does not
//! list. They are offered next to the Media Foundation ones and read into the same BGRA
//! frames: a capture graph with the system's sample grabber (`qedit.dll`), asked for
//! RGB32, whose latest buffer is copied about 30 times a second.

use std::ffi::c_void;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};

use windows::core::{Interface, GUID, HRESULT, PCWSTR};
use windows::Win32::Media::DirectShow::*;
use windows::Win32::Media::MediaFoundation::{
    CLSID_CaptureGraphBuilder2, CLSID_FilterGraph, CLSID_SystemDeviceEnum, CLSID_VideoInputDeviceCategory,
    FORMAT_VideoInfo, MEDIATYPE_Video, AM_MEDIA_TYPE, MEDIASUBTYPE_RGB32, PIN_CATEGORY_CAPTURE,
    VIDEOINFOHEADER,
};
use windows::Win32::System::Com::StructuredStorage::IPropertyBag;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoTaskMemFree, IMoniker, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED,
};
use windows::Win32::System::Variant::{VariantClear, VARIANT, VT_BSTR};

use crate::camera::{CameraInfo, Frame, Shared, Trouble, WANT};

/// Ids of DirectShow cameras start with this, so `open` knows which way to read them.
pub const PREFIX: &str = "ds:";

/// The system's sample grabber and null renderer (qedit.dll, still in Windows 10 and 11;
/// the SDK no longer declares them).
const CLSID_SAMPLE_GRABBER: GUID = GUID::from_u128(0xC1F400A0_3F08_11D3_9F0B_006008039E37);
const CLSID_NULL_RENDERER: GUID = GUID::from_u128(0xC1F400A4_3F08_11D3_9F0B_006008039E37);

/// The sample grabber's interface, declared here (the SDK no longer does); its methods
/// keep their COM names.
#[allow(non_snake_case)]
mod qedit {
    use std::ffi::c_void;

    use windows::core::{interface, IUnknown, IUnknown_Vtbl, BOOL, HRESULT};
    use windows::Win32::Media::MediaFoundation::AM_MEDIA_TYPE;

    #[interface("6B652FFF-11FE-4fce-92AD-0266B5D7C78F")]
    pub unsafe trait ISampleGrabber: IUnknown {
        pub fn SetOneShot(&self, one_shot: BOOL) -> HRESULT;
        pub fn SetMediaType(&self, mt: *const AM_MEDIA_TYPE) -> HRESULT;
        pub fn GetConnectedMediaType(&self, mt: *mut AM_MEDIA_TYPE) -> HRESULT;
        pub fn SetBufferSamples(&self, buffer: BOOL) -> HRESULT;
        pub fn GetCurrentBuffer(&self, size: *mut i32, buffer: *mut i32) -> HRESULT;
        pub fn GetCurrentSample(&self, sample: *mut *mut c_void) -> HRESULT;
        pub fn SetCallback(&self, callback: *mut c_void, which: i32) -> HRESULT;
    }
}
use qedit::ISampleGrabber;

fn start_com() {
    // SAFETY: COM for this thread; reference counted.
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    }
}

/// A camera as DirectShow lists it.
struct Device {
    moniker: IMoniker,
    name: String,
    /// The physical device's path, when it has one (the same camera may be in Media
    /// Foundation's list too).
    path: String,
    /// What identifies it here: its path, else its class id, else its name.
    key: String,
}

fn property(bag: &IPropertyBag, name: &str) -> String {
    let wide: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
    // SAFETY: a VARIANT read by the property bag and cleared after its string is copied.
    unsafe {
        let mut v = VARIANT::default();
        if bag.Read(PCWSTR(wide.as_ptr()), &mut v, None).is_err() {
            return String::new();
        }
        let s = if v.Anonymous.Anonymous.vt == VT_BSTR {
            v.Anonymous.Anonymous.Anonymous.bstrVal.to_string()
        } else {
            String::new()
        };
        let _ = VariantClear(&mut v);
        s
    }
}

fn devices() -> windows::core::Result<Vec<Device>> {
    start_com();
    // SAFETY: COM calls; every interface is reference counted.
    unsafe {
        let devenum: ICreateDevEnum = CoCreateInstance(&CLSID_SystemDeviceEnum, None, CLSCTX_INPROC_SERVER)?;
        let mut list = None;
        devenum.CreateClassEnumerator(&CLSID_VideoInputDeviceCategory, &mut list, 0)?;
        // No video devices at all: S_FALSE and no list.
        let Some(list) = list else {
            return Ok(Vec::new());
        };
        let mut out = Vec::new();
        loop {
            let mut one = [None];
            let mut fetched = 0u32;
            if list.Next(&mut one, Some(&mut fetched)).is_err() || fetched == 0 {
                break;
            }
            let Some(moniker) = one[0].take() else { break };
            let Ok(bag) = moniker.BindToStorage::<_, _, IPropertyBag>(None, None) else {
                continue;
            };
            let name = property(&bag, "FriendlyName");
            let path = property(&bag, "DevicePath");
            let clsid = property(&bag, "CLSID");
            let key = [&path, &clsid, &name]
                .into_iter()
                .find(|s| !s.is_empty())
                .cloned()
                .unwrap_or_default();
            out.push(Device {
                moniker,
                name,
                path,
                key,
            });
        }
        Ok(out)
    }
}

/// The DirectShow cameras that Media Foundation did not list already (`known`: its
/// cameras), with ids that start with [`PREFIX`].
pub fn cameras(known: &[CameraInfo]) -> Vec<CameraInfo> {
    let Ok(list) = devices() else {
        return Vec::new();
    };
    let same = |d: &Device, c: &CameraInfo| {
        (!d.path.is_empty() && d.path.eq_ignore_ascii_case(&c.id)) || d.name == c.name
    };
    list.iter()
        .filter(|d| !d.key.is_empty() && !known.iter().any(|c| same(d, c)))
        .map(|d| CameraInfo {
            id: format!("{PREFIX}{}", d.key),
            name: d.name.clone(),
        })
        .collect()
}

/// Frees a media type DirectShow allocated (`DeleteMediaType`).
///
/// # Safety
/// `mt` came from DirectShow (GetStreamCaps) and is not used after.
unsafe fn delete_media_type(mt: *mut AM_MEDIA_TYPE) {
    if mt.is_null() {
        return;
    }
    // SAFETY: as the caller promises.
    unsafe {
        free_format(&mut *mt);
        CoTaskMemFree(Some(mt as *const c_void));
    }
}

/// Frees a media type's format block and object (`FreeMediaType`).
///
/// # Safety
/// The block and the object were allocated by DirectShow.
unsafe fn free_format(mt: &mut AM_MEDIA_TYPE) {
    // SAFETY: as the caller promises.
    unsafe {
        if !mt.pbFormat.is_null() {
            CoTaskMemFree(Some(mt.pbFormat as *const c_void));
            mt.pbFormat = std::ptr::null_mut();
            mt.cbFormat = 0;
        }
        std::mem::ManuallyDrop::drop(&mut mt.pUnk);
    }
}

/// Asks the camera for its format nearest to [`WANT`] at 15 fps or more, when it offers
/// a choice (many virtual cameras have only one).
///
/// # Safety
/// COM calls on this thread's graph.
unsafe fn choose_format(builder: &ICaptureGraphBuilder2, source: &IBaseFilter) {
    // SAFETY: as the caller promises; each media type from GetStreamCaps is freed.
    unsafe {
        let mut config: *mut c_void = std::ptr::null_mut();
        let found = (Interface::vtable(builder).FindInterface)(
            builder.as_raw(),
            &PIN_CATEGORY_CAPTURE,
            &MEDIATYPE_Video,
            source.as_raw(),
            &IAMStreamConfig::IID,
            &mut config,
        );
        if found.is_err() || config.is_null() {
            return;
        }
        let config = IAMStreamConfig::from_raw(config);
        let (mut count, mut size) = (0i32, 0i32);
        if config.GetNumberOfCapabilities(&mut count, &mut size).is_err() {
            return;
        }
        let mut caps = vec![0u8; size.max(0) as usize];
        let mut best: Option<(i64, i32)> = None;
        for i in 0..count {
            let mut mt: *mut AM_MEDIA_TYPE = std::ptr::null_mut();
            if config.GetStreamCaps(i, &mut mt, caps.as_mut_ptr()).is_err() || mt.is_null() {
                continue;
            }
            let t = &*mt;
            if t.formattype == FORMAT_VideoInfo
                && t.cbFormat as usize >= std::mem::size_of::<VIDEOINFOHEADER>()
            {
                let vih = &*(t.pbFormat as *const VIDEOINFOHEADER);
                let (w, h) = (vih.bmiHeader.biWidth as i64, vih.bmiHeader.biHeight.abs() as i64);
                let fps = if vih.AvgTimePerFrame > 0 {
                    10_000_000.0 / vih.AvgTimePerFrame as f64
                } else {
                    30.0
                };
                if fps >= 14.5 && w > 0 && h > 0 {
                    let score = (w * h - WANT.0 as i64 * WANT.1 as i64).abs() * 100 - fps.min(60.0) as i64;
                    if best.is_none_or(|(s, _)| score < s) {
                        best = Some((score, i));
                    }
                }
            }
            delete_media_type(mt);
        }
        if let Some((_, i)) = best {
            let mut mt: *mut AM_MEDIA_TYPE = std::ptr::null_mut();
            if config.GetStreamCaps(i, &mut mt, caps.as_mut_ptr()).is_ok() && !mt.is_null() {
                let _ = config.SetFormat(mt);
                delete_media_type(mt);
            }
        }
    }
}

/// A running capture graph.
struct Graph {
    control: IMediaControl,
    events: IMediaEvent,
    grabber: ISampleGrabber,
    width: u32,
    height: u32,
    /// RGB32 rows come bottom row first unless the height is negative.
    bottom_up: bool,
}

fn build(key: Option<&str>) -> windows::core::Result<Graph> {
    let list = devices()?;
    let device = match key {
        Some(k) => list.iter().find(|d| d.key == k),
        None => list.first(),
    }
    .ok_or_else(|| windows::core::Error::from_hresult(VFW_E_NOT_FOUND))?;
    // SAFETY: COM calls on this thread's graph; the media types are freed as DirectShow
    // asks; the vtable calls pass the arguments the interface declares.
    unsafe {
        let source: IBaseFilter = device.moniker.BindToObject(None, None)?;
        let graph: IGraphBuilder = CoCreateInstance(&CLSID_FilterGraph, None, CLSCTX_INPROC_SERVER)?;
        graph.AddFilter(&source, windows::core::w!("Source"))?;

        let grab_filter: IBaseFilter = CoCreateInstance(&CLSID_SAMPLE_GRABBER, None, CLSCTX_INPROC_SERVER)?;
        let grabber: ISampleGrabber = grab_filter.cast()?;
        let want = AM_MEDIA_TYPE {
            majortype: MEDIATYPE_Video,
            subtype: MEDIASUBTYPE_RGB32,
            formattype: FORMAT_VideoInfo,
            ..Default::default()
        };
        grabber.SetMediaType(&want).ok()?;
        grabber.SetBufferSamples(true.into()).ok()?;
        grabber.SetOneShot(false.into()).ok()?;
        graph.AddFilter(&grab_filter, windows::core::w!("Grabber"))?;

        let null: IBaseFilter = CoCreateInstance(&CLSID_NULL_RENDERER, None, CLSCTX_INPROC_SERVER)?;
        graph.AddFilter(&null, windows::core::w!("Null"))?;

        let builder: ICaptureGraphBuilder2 =
            CoCreateInstance(&CLSID_CaptureGraphBuilder2, None, CLSCTX_INPROC_SERVER)?;
        builder.SetFiltergraph(&graph)?;
        choose_format(&builder, &source);
        // The capture pin, else any video pin (some virtual cameras only have one).
        let mut rendered = HRESULT(-1);
        for category in [&PIN_CATEGORY_CAPTURE as *const GUID, std::ptr::null()] {
            rendered = (Interface::vtable(&builder).RenderStream)(
                builder.as_raw(),
                category,
                &MEDIATYPE_Video,
                source.as_raw(),
                grab_filter.as_raw(),
                null.as_raw(),
            );
            if rendered.is_ok() {
                break;
            }
        }
        rendered.ok()?;

        let mut mt = AM_MEDIA_TYPE::default();
        grabber.GetConnectedMediaType(&mut mt).ok()?;
        let size = if mt.formattype == FORMAT_VideoInfo
            && mt.cbFormat as usize >= std::mem::size_of::<VIDEOINFOHEADER>()
        {
            let vih = &*(mt.pbFormat as *const VIDEOINFOHEADER);
            Some((vih.bmiHeader.biWidth, vih.bmiHeader.biHeight))
        } else {
            None
        };
        free_format(&mut mt);
        let (w, h) = size.ok_or_else(|| windows::core::Error::from_hresult(VFW_E_INVALIDMEDIATYPE))?;

        let control: IMediaControl = graph.cast()?;
        let events: IMediaEvent = graph.cast()?;
        control.Run()?;
        Ok(Graph {
            control,
            events,
            grabber,
            width: w.unsigned_abs(),
            height: h.unsigned_abs(),
            bottom_up: h > 0,
        })
    }
}

/// Reads a DirectShow camera (`id` with [`PREFIX`], or the first one) until stopped,
/// as `camera::read` does for Media Foundation.
pub(crate) fn read(
    s: &Shared,
    id: Option<&str>,
    ready: std::sync::mpsc::Sender<Result<(u32, u32), Trouble>>,
) {
    start_com();
    let key = id.and_then(|i| i.strip_prefix(PREFIX));
    let g = match build(key) {
        Ok(g) => g,
        Err(e) => {
            tracing::warn!(error = %e.message(), "camera (DirectShow): could not open");
            let trouble = if e.code() == VFW_E_NOT_FOUND {
                Trouble::Gone
            } else if e.code().0 == 0x8007_0005_u32 as i32 {
                Trouble::Denied
            } else if e.code() == VFW_E_CANNOT_CONNECT {
                Trouble::InUse
            } else {
                Trouble::Failed
            };
            let _ = ready.send(Err(trouble));
            return;
        }
    };
    let _ = ready.send(Ok((g.width, g.height)));
    let (w, h) = (g.width, g.height);
    let row = w as usize * 4;
    let mut buf = vec![0u8; row * h as usize];
    let mut last: Option<u64> = None;
    while !s.stop.load(Ordering::Relaxed) {
        // A lost device or an aborted graph ends the reading (VID-038).
        let (mut code, mut p1, mut p2) = (0i32, 0isize, 0isize);
        // SAFETY: event parameters are freed as the interface asks.
        if unsafe { g.events.GetEvent(&mut code, &mut p1, &mut p2, 0) }.is_ok() {
            // SAFETY: as above.
            unsafe {
                let _ = g.events.FreeEventParams(code, p1, p2);
            }
            if code as u32 == EC_DEVICE_LOST || code as u32 == EC_ERRORABORT {
                *s.trouble.lock().unwrap() = Some(Trouble::Gone);
                break;
            }
        }
        let mut size = buf.len() as i32;
        // SAFETY: the grabber copies at most `size` bytes into the buffer.
        let got = unsafe {
            g.grabber
                .GetCurrentBuffer(&mut size, buf.as_mut_ptr() as *mut i32)
        };
        if got.is_ok() && size as usize >= buf.len() {
            // A cheap fingerprint, so an unchanged picture is not sent as a new frame.
            let print = buf
                .iter()
                .step_by(4099)
                .fold(size as u64, |a, b| a.wrapping_mul(31).wrapping_add(*b as u64));
            if last != Some(print) {
                last = Some(print);
                let mut bgra = vec![0u8; buf.len()];
                for y in 0..h as usize {
                    let from = if g.bottom_up { h as usize - 1 - y } else { y };
                    bgra[y * row..(y + 1) * row].copy_from_slice(&buf[from * row..(from + 1) * row]);
                }
                // RGB32 leaves the fourth byte undefined: the camera is opaque.
                for px in bgra.as_chunks_mut::<4>().0 {
                    px[3] = 255;
                }
                *s.latest.lock().unwrap() = Some(Arc::new(Frame {
                    width: w,
                    height: h,
                    bgra,
                    at: Instant::now(),
                }));
            }
        }
        std::thread::sleep(Duration::from_millis(30));
    }
    // SAFETY: stopping this thread's graph.
    unsafe {
        let _ = g.control.Stop();
    }
}
