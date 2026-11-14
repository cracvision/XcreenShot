//! The camera (VID-031): the video capture devices Windows offers (built-in, USB,
//! virtual cameras of other apps), and one of them read on a thread of its own as BGRA
//! frames, the latest kept for the recording and the preview. It is opened only while
//! it shows and let go when dropped, so the system's camera light goes off.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use windows::core::PWSTR;
use windows::Win32::Media::MediaFoundation::*;
use windows::Win32::System::Com::{CoInitializeEx, CoTaskMemFree, COINIT_MULTITHREADED};

/// A camera: its id (the system's link to it) and the name the system shows.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct CameraInfo {
    pub id: String,
    pub name: String,
}

pub use crate::camlook::Frame;

/// Why a camera does not give pictures (VID-038).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Trouble {
    /// The system's privacy does not let XcreenShot use it.
    Denied,
    /// Another app has it and the system does not share it.
    InUse,
    /// It went away (unplugged).
    Gone,
    /// Anything else.
    Failed,
}

fn trouble_of(e: &windows::core::Error) -> Trouble {
    const ACCESS_DENIED: i32 = 0x8007_0005_u32 as i32;
    match e.code().0 {
        ACCESS_DENIED => Trouble::Denied,
        c if c == MF_E_HW_MFT_FAILED_START_STREAMING.0 || c == MF_E_VIDEO_RECORDING_DEVICE_PREEMPTED.0 => {
            Trouble::InUse
        }
        c if c == MF_E_VIDEO_RECORDING_DEVICE_INVALIDATED.0 => Trouble::Gone,
        _ => Trouble::Failed,
    }
}

fn start_mf() {
    // SAFETY: COM and Media Foundation for this thread; both are reference counted.
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let _ = MFStartup(MF_VERSION, MFSTARTUP_FULL);
    }
}

/// The camera devices, as the system lists them.
fn devices() -> windows::core::Result<Vec<IMFActivate>> {
    start_mf();
    // SAFETY: the list Media Foundation allocates is read within its count and freed.
    unsafe {
        let mut attrs = None;
        MFCreateAttributes(&mut attrs, 1)?;
        let attrs = attrs.ok_or_else(|| windows::core::Error::from_hresult(windows::core::HRESULT(-1)))?;
        attrs.SetGUID(
            &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE,
            &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_GUID,
        )?;
        let mut list: *mut Option<IMFActivate> = std::ptr::null_mut();
        let mut count = 0u32;
        MFEnumDeviceSources(&attrs, &mut list, &mut count)?;
        let mut out = Vec::with_capacity(count as usize);
        for i in 0..count as usize {
            if let Some(a) = std::ptr::read(list.add(i)) {
                out.push(a);
            }
        }
        CoTaskMemFree(Some(list as *const _));
        Ok(out)
    }
}

fn text(a: &IMFActivate, key: &windows::core::GUID) -> String {
    // SAFETY: the string Media Foundation allocates is copied and freed.
    unsafe {
        let mut p = PWSTR::null();
        let mut len = 0u32;
        if a.GetAllocatedString(key, &mut p, &mut len).is_err() {
            return String::new();
        }
        let s = p.to_string().unwrap_or_default();
        CoTaskMemFree(Some(p.0 as *const _));
        s
    }
}

/// The cameras that are plugged in.
pub fn cameras() -> Result<Vec<CameraInfo>, String> {
    let list = devices().map_err(|e| format!("cameras: {}", e.message()))?;
    Ok(list
        .iter()
        .map(|a| CameraInfo {
            id: text(a, &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_SYMBOLIC_LINK),
            name: text(a, &MF_DEVSOURCE_ATTRIBUTE_FRIENDLY_NAME),
        })
        .collect())
}

struct Shared {
    stop: AtomicBool,
    latest: Mutex<Option<Arc<Frame>>>,
    trouble: Mutex<Option<Trouble>>,
}

/// A camera being read. Dropping it lets the camera go.
pub struct Camera {
    shared: Arc<Shared>,
    thread: Option<std::thread::JoinHandle<()>>,
    /// The size of its frames.
    pub width: u32,
    pub height: u32,
}

/// The picture size asked of the camera: enough for a corner and for the whole
/// video, without the cost of more.
const WANT: (u32, u32) = (1280, 720);

/// Opens the camera `id` (or the first one) and starts reading it; returns once its
/// size is known, or why it cannot be read.
pub fn open(id: Option<&str>) -> Result<Camera, Trouble> {
    let shared = Arc::new(Shared {
        stop: AtomicBool::new(false),
        latest: Mutex::new(None),
        trouble: Mutex::new(None),
    });
    let (tx, rx) = std::sync::mpsc::channel::<Result<(u32, u32), Trouble>>();
    let s = shared.clone();
    let id = id.map(str::to_string);
    let thread = std::thread::Builder::new()
        .name("xs-camera".into())
        .spawn(move || read(&s, id.as_deref(), tx))
        .map_err(|_| Trouble::Failed)?;
    match rx.recv_timeout(Duration::from_secs(10)) {
        Ok(Ok((width, height))) => Ok(Camera {
            shared,
            thread: Some(thread),
            width,
            height,
        }),
        Ok(Err(t)) => {
            let _ = thread.join();
            Err(t)
        }
        Err(_) => {
            shared.stop.store(true, Ordering::Relaxed);
            Err(Trouble::Failed)
        }
    }
}

impl Camera {
    /// Where a recording gets this camera's frames.
    pub fn feed(self: &Arc<Self>) -> crate::camlook::Feed {
        let me = self.clone();
        Arc::new(move || me.latest())
    }

    /// The latest frame.
    pub fn latest(&self) -> Option<Arc<Frame>> {
        self.shared.latest.lock().unwrap().clone()
    }

    /// Why it stopped giving pictures, if it did (VID-038).
    pub fn trouble(&self) -> Option<Trouble> {
        *self.shared.trouble.lock().unwrap()
    }
}

impl Drop for Camera {
    fn drop(&mut self) {
        self.shared.stop.store(true, Ordering::Relaxed);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

/// The camera's own picture closest to [`WANT`] (at least 15 fps), as it gives it.
fn best_native(reader: &IMFSourceReader, stream: u32) -> Option<IMFMediaType> {
    // SAFETY: Media Foundation reads of the reader's types.
    unsafe { best_native_in(reader, stream) }
}

/// # Safety
/// Media Foundation calls on a reader of this thread.
unsafe fn best_native_in(reader: &IMFSourceReader, stream: u32) -> Option<IMFMediaType> {
    // SAFETY: as the caller promises.
    unsafe {
        let mut best: Option<(i64, IMFMediaType)> = None;
        for i in 0.. {
            let Ok(t) = reader.GetNativeMediaType(stream, i) else {
                break;
            };
            let Ok(size) = t.GetUINT64(&MF_MT_FRAME_SIZE) else {
                continue;
            };
            let (w, h) = ((size >> 32) as i64, (size & 0xFFFF_FFFF) as i64);
            let fps = t
                .GetUINT64(&MF_MT_FRAME_RATE)
                .map(|r| (r >> 32) as f64 / ((r & 0xFFFF_FFFF).max(1)) as f64)
                .unwrap_or(30.0);
            if fps < 14.5 {
                continue;
            }
            // Nearest in pixels to the size wanted; then the faster.
            let score = (w * h - WANT.0 as i64 * WANT.1 as i64).abs() * 100 - fps.min(60.0) as i64;
            if best.as_ref().is_none_or(|(s, _)| score < *s) {
                best = Some((score, t));
            }
        }
        best.map(|(_, t)| t)
    }
}

fn read(s: &Shared, id: Option<&str>, ready: std::sync::mpsc::Sender<Result<(u32, u32), Trouble>>) {
    start_mf();
    // SAFETY: Media Foundation calls on this thread's reader; each buffer is unlocked
    // after its rows are copied, within the pitch and size it reports.
    let opened = unsafe {
        (|| -> windows::core::Result<(IMFSourceReader, u32, u32)> {
            let list = devices()?;
            let activate = match id {
                Some(id) => list
                    .iter()
                    .find(|a| text(a, &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_SYMBOLIC_LINK) == id)
                    .or(list.first()),
                None => list.first(),
            }
            .ok_or_else(|| windows::core::Error::from_hresult(MF_E_VIDEO_RECORDING_DEVICE_INVALIDATED))?;
            let source: IMFMediaSource = activate.ActivateObject()?;
            let mut attrs = None;
            MFCreateAttributes(&mut attrs, 1)?;
            let attrs =
                attrs.ok_or_else(|| windows::core::Error::from_hresult(windows::core::HRESULT(-1)))?;
            // The reader converts whatever the camera gives (MJPG, YUY2, NV12) to BGRA.
            attrs.SetUINT32(&MF_SOURCE_READER_ENABLE_VIDEO_PROCESSING, 1)?;
            let reader = MFCreateSourceReaderFromMediaSource(&source, &attrs)?;
            let stream = MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32;
            if let Some(native) = best_native(&reader, stream) {
                let _ = reader.SetCurrentMediaType(stream, None, &native);
            }
            let want = MFCreateMediaType()?;
            want.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
            want.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_RGB32)?;
            reader.SetCurrentMediaType(stream, None, &want)?;
            let t = reader.GetCurrentMediaType(stream)?;
            let size = t.GetUINT64(&MF_MT_FRAME_SIZE)?;
            Ok((reader, (size >> 32) as u32, size as u32))
        })()
    };
    let (reader, w, h) = match opened {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(error = %e.message(), "camera: could not open");
            let _ = ready.send(Err(trouble_of(&e)));
            return;
        }
    };
    let _ = ready.send(Ok((w, h)));
    let stream = MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32;
    while !s.stop.load(Ordering::Relaxed) {
        let mut flags = 0u32;
        let mut sample = None;
        // SAFETY: as above.
        let got = unsafe { reader.ReadSample(stream, 0, None, Some(&mut flags), None, Some(&mut sample)) };
        if let Err(e) = got {
            *s.trouble.lock().unwrap() = Some(trouble_of(&e));
            break;
        }
        if flags & MF_SOURCE_READERF_ERROR.0 as u32 != 0
            || flags & MF_SOURCE_READERF_ENDOFSTREAM.0 as u32 != 0
        {
            *s.trouble.lock().unwrap() = Some(Trouble::Gone);
            break;
        }
        let Some(sample) = sample else { continue };
        // SAFETY: as above.
        if let Some(bgra) = unsafe { pixels(&sample, w, h) } {
            *s.latest.lock().unwrap() = Some(Arc::new(Frame {
                width: w,
                height: h,
                bgra,
                at: Instant::now(),
            }));
        }
    }
}

/// A sample's pixels, top row first, opaque.
///
/// # Safety
/// The sample holds a `w` × `h` RGB32 picture.
unsafe fn pixels(sample: &IMFSample, w: u32, h: u32) -> Option<Vec<u8>> {
    // SAFETY: as the caller promises.
    unsafe { pixels_in(sample, w, h) }
}

/// # Safety
/// As [`pixels`].
unsafe fn pixels_in(sample: &IMFSample, w: u32, h: u32) -> Option<Vec<u8>> {
    // SAFETY: as the caller promises.
    unsafe {
        let buffer = sample.ConvertToContiguousBuffer().ok()?;
        let row = w as usize * 4;
        let mut out = vec![0u8; row * h as usize];
        if let Ok(b2) = windows::core::Interface::cast::<IMF2DBuffer>(&buffer) {
            let mut line0 = std::ptr::null_mut();
            let mut pitch = 0i32;
            b2.Lock2D(&mut line0, &mut pitch).ok()?;
            for y in 0..h as usize {
                let src = line0.offset(y as isize * pitch as isize);
                std::ptr::copy_nonoverlapping(src, out.as_mut_ptr().add(y * row), row);
            }
            let _ = b2.Unlock2D();
        } else {
            let mut data = std::ptr::null_mut();
            let mut len = 0u32;
            buffer.Lock(&mut data, None, Some(&mut len)).ok()?;
            let n = (len as usize).min(out.len());
            std::ptr::copy_nonoverlapping(data, out.as_mut_ptr(), n);
            let _ = buffer.Unlock();
        }
        // RGB32 leaves the fourth byte undefined: the camera is opaque.
        for px in out.as_chunks_mut::<4>().0 {
            px[3] = 255;
        }
        Some(out)
    }
}

#[cfg(test)]
mod tests {
    /// The cameras of this computer (by hand: nothing is opened).
    #[test]
    #[ignore]
    fn lists_the_cameras() {
        println!("{:?}", super::cameras());
    }

    /// Opens the first camera for 2 s and says what it gives (by hand: its light goes
    /// on; nothing is kept).
    #[test]
    #[ignore]
    fn reads_the_camera() {
        let started = std::time::Instant::now();
        match super::open(None) {
            Ok(c) => {
                println!("opened {}×{} in {:?}", c.width, c.height, started.elapsed());
                let mut frames = 0;
                let mut last = None;
                let until = std::time::Instant::now() + std::time::Duration::from_secs(2);
                while std::time::Instant::now() < until {
                    if let Some(f) = c.latest() {
                        if last.is_none_or(|l| l != f.at) {
                            frames += 1;
                            last = Some(f.at);
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
                let f = c.latest().unwrap();
                let mean = f.bgra.chunks(4).map(|p| p[1] as u64).sum::<u64>() / (f.bgra.len() as u64 / 4);
                println!(
                    "{frames} frames in 2 s, mean green {mean}, trouble {:?}",
                    c.trouble()
                );
            }
            Err(t) => println!("could not open: {t:?}"),
        }
    }
}
