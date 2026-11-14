//! The Windows engine: Desktop Duplication → D3D11 video processor (crop, scale,
//! BGRA → NV12 on the GPU) → Media Foundation Sink Writer (H.264 in MP4).
//!
//! A new frame is written only when the screen changed, at most `fps` times a second
//! (variable frame rate, VID-025). The hardware encoder gets the GPU frames directly;
//! without one (or with "always software", VID-029) the frames go through memory to
//! the system's software encoder.

#![allow(clippy::too_many_arguments)]

use std::mem::ManuallyDrop;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};

use windows::core::{Interface, BOOL, HSTRING};
use windows::Win32::Foundation::{HMODULE, RECT};
use windows::Win32::Graphics::Direct3D::{
    D3D_DRIVER_TYPE_UNKNOWN, D3D_FEATURE_LEVEL_10_1, D3D_FEATURE_LEVEL_11_0,
};
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Graphics::Dxgi::*;
use windows::Win32::Media::MediaFoundation::*;
use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};
use windows::Win32::UI::HiDpi::{SetThreadDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2};

use crate::effects::{Effects, Live};
use crate::{Area, Options, Shared, State, Summary};

/// 100-nanosecond units in a second (Media Foundation's clock).
pub(crate) const HNS: f64 = 10_000_000.0;

pub(crate) fn err(context: &str, e: windows::core::Error) -> String {
    format!("{context}: {} (0x{:08X})", e.message(), e.code().0 as u32)
}

/// The adapter and output that show the middle of `area`, with the output's place.
fn find_output(area: Area) -> Result<(IDXGIAdapter1, IDXGIOutput1, RECT), String> {
    let cx = area.x + area.width as i32 / 2;
    let cy = area.y + area.height as i32 / 2;
    // SAFETY: plain DXGI enumeration; every interface is reference counted.
    unsafe {
        let factory: IDXGIFactory1 = CreateDXGIFactory1().map_err(|e| err("DXGI", e))?;
        let mut a = 0;
        while let Ok(adapter) = factory.EnumAdapters1(a) {
            let mut o = 0;
            while let Ok(output) = adapter.EnumOutputs(o) {
                o += 1;
                let Ok(desc) = output.GetDesc() else { continue };
                let r = desc.DesktopCoordinates;
                if desc.AttachedToDesktop.as_bool()
                    && cx >= r.left
                    && cx < r.right
                    && cy >= r.top
                    && cy < r.bottom
                {
                    let output1: IDXGIOutput1 = output.cast().map_err(|e| err("DXGI output", e))?;
                    return Ok((adapter, output1, r));
                }
            }
            a += 1;
        }
    }
    Err("no monitor shows the area".into())
}

struct Gpu {
    device: ID3D11Device,
    context: ID3D11DeviceContext,
    video: ID3D11VideoDevice,
    vctx: ID3D11VideoContext1,
}

fn gpu(adapter: &IDXGIAdapter1) -> Result<Gpu, String> {
    // SAFETY: device creation with out-parameters that live through the call.
    unsafe {
        let mut device = None;
        let mut context = None;
        D3D11CreateDevice(
            adapter,
            D3D_DRIVER_TYPE_UNKNOWN,
            HMODULE::default(),
            D3D11_CREATE_DEVICE_BGRA_SUPPORT | D3D11_CREATE_DEVICE_VIDEO_SUPPORT,
            Some(&[D3D_FEATURE_LEVEL_11_0, D3D_FEATURE_LEVEL_10_1]),
            D3D11_SDK_VERSION,
            Some(&mut device),
            None,
            Some(&mut context),
        )
        .map_err(|e| err("Direct3D", e))?;
        let device: ID3D11Device = device.ok_or("no Direct3D device")?;
        let context: ID3D11DeviceContext = context.ok_or("no Direct3D context")?;
        // The encoder uses the device from its own threads.
        if let Ok(mt) = context.cast::<ID3D11Multithread>() {
            let _ = mt.SetMultithreadProtected(true);
        }
        let video: ID3D11VideoDevice = device.cast().map_err(|e| err("video device", e))?;
        let vctx: ID3D11VideoContext1 = context.cast().map_err(|e| err("video context", e))?;
        Ok(Gpu {
            device,
            context,
            video,
            vctx,
        })
    }
}

/// The Direct3D device kept between recordings, and its adapter (VID-030): making
/// one takes about 100 ms of the start.
struct Kept(Gpu, (u32, i32));

// SAFETY: Direct3D 11 devices are free-threaded and the immediate context is
// multithread-protected (`gpu`); one recording uses it at a time.
unsafe impl Send for Kept {}

static DEVICE: std::sync::Mutex<Option<Kept>> = std::sync::Mutex::new(None);

/// A recording's device, put back for the next one when it ends.
struct KeepOnDrop(Option<Gpu>, (u32, i32));

impl Drop for KeepOnDrop {
    fn drop(&mut self) {
        if let (Some(g), Ok(mut slot)) = (self.0.take(), DEVICE.lock()) {
            *slot = Some(Kept(g, self.1));
        }
    }
}

impl std::ops::Deref for KeepOnDrop {
    type Target = Gpu;
    fn deref(&self) -> &Gpu {
        self.0.as_ref().expect("a device until dropped")
    }
}
static ENCODER_WARM: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn adapter_id(adapter: &IDXGIAdapter1) -> (u32, i32) {
    // SAFETY: a plain query.
    unsafe { adapter.GetDesc1() }
        .map(|d| (d.AdapterLuid.LowPart, d.AdapterLuid.HighPart))
        .unwrap_or_default()
}

/// The kept device for `adapter` if it still works, or a new one.
fn take_gpu(slot: &mut Option<Kept>, adapter: &IDXGIAdapter1) -> Result<Gpu, String> {
    if let Some(Kept(g, id)) = slot.take() {
        // SAFETY: a plain query.
        if id == adapter_id(adapter) && unsafe { g.device.GetDeviceRemovedReason() }.is_ok() {
            return Ok(g);
        }
    }
    gpu(adapter)
}

/// Gets the recording engine ready for `area` (VID-030), in the background: the
/// Direct3D device, and the first time the hardware encoder's libraries. A
/// recording that starts meanwhile waits for it and uses what it made.
pub fn warm_up(area: Area, software_only: bool) {
    let _ = std::thread::Builder::new()
        .name("xs-record-warm".into())
        .spawn(move || {
            // SAFETY: COM and Media Foundation for this thread.
            unsafe {
                let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
                let _ = MFStartup(MF_VERSION, MFSTARTUP_FULL);
            }
            let Ok((adapter, _, _)) = find_output(area) else {
                return;
            };
            let mut slot = DEVICE.lock().unwrap();
            let Ok(gpu) = take_gpu(&mut slot, &adapter) else {
                return;
            };
            if !software_only && !ENCODER_WARM.swap(true, Ordering::Relaxed) {
                let path = std::env::temp_dir().join(format!("xs-warm-{}.mp4", std::process::id()));
                drop(writer(&gpu, &path, (1280, 720), 30, 2_000_000, true, None));
                let _ = std::fs::remove_file(&path);
            }
            *slot = Some(Kept(gpu, adapter_id(&adapter)));
        });
}

fn texture(
    gpu: &Gpu,
    w: u32,
    h: u32,
    format: DXGI_FORMAT,
    bind: u32,
    staging: bool,
) -> Result<ID3D11Texture2D, String> {
    let desc = D3D11_TEXTURE2D_DESC {
        Width: w,
        Height: h,
        MipLevels: 1,
        ArraySize: 1,
        Format: format,
        SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
        Usage: if staging {
            D3D11_USAGE_STAGING
        } else {
            D3D11_USAGE_DEFAULT
        },
        BindFlags: bind,
        CPUAccessFlags: if staging {
            D3D11_CPU_ACCESS_READ.0 as u32
        } else {
            0
        },
        MiscFlags: 0,
    };
    let mut t = None;
    // SAFETY: the description and out-parameter live through the call.
    unsafe { gpu.device.CreateTexture2D(&desc, None, Some(&mut t)) }.map_err(|e| err("texture", e))?;
    t.ok_or_else(|| "no texture".to_string())
}

/// HD video is BT.709; smaller video BT.601, which is what players assume when the
/// file does not say.
/// HD pictures (720p and up) are BT.709; smaller ones BT.601.
pub(crate) fn is_hd(size: (u32, u32)) -> bool {
    hd(size)
}

fn hd(size: (u32, u32)) -> bool {
    size.0 >= 1280 || size.1 >= 720
}

/// Crops, scales and converts the monitor's picture into NV12 frames.
struct Converter {
    venum: ID3D11VideoProcessorEnumerator,
    vp: ID3D11VideoProcessor,
    input: ID3D11VideoProcessorInputView,
    /// The last picture of the monitor (BGRA).
    frame: ID3D11Texture2D,
    /// What is converted: the picture with the pointer drawn on it (GDI-compatible).
    composite: ID3D11Texture2D,
    /// The monitor's top left on the virtual screen, and its size.
    origin: (i32, i32),
    monitor: (u32, u32),
    /// The part of the monitor recorded, for the key labels (VID-047).
    area: std::cell::Cell<RECT>,
    /// The monitor's scale, for the effects' sizes.
    scale: f32,
    /// Draws the cursor and keyboard effects (VID-045 to VID-047).
    painter: Option<crate::win_fx::Painter>,
    /// The camera's frame and look for the next picture (VID-032 to VID-036), and its
    /// frame as a bitmap of the painter.
    camera: std::cell::RefCell<Option<(Arc<crate::camlook::Frame>, crate::camlook::Look)>>,
    camera_bitmap: std::cell::RefCell<Option<crate::win_fx::CameraBitmap>>,
    width: u32,
    height: u32,
}

fn converter(
    gpu: &Gpu,
    monitor: (u32, u32),
    origin: (i32, i32),
    source: RECT,
    size: (u32, u32),
    fps: u32,
    scale: f32,
) -> Result<Converter, String> {
    let frame = texture(
        gpu,
        monitor.0,
        monitor.1,
        DXGI_FORMAT_B8G8R8A8_UNORM,
        (D3D11_BIND_RENDER_TARGET.0 | D3D11_BIND_SHADER_RESOURCE.0) as u32,
        false,
    )?;
    let composite = {
        let desc = D3D11_TEXTURE2D_DESC {
            Width: monitor.0,
            Height: monitor.1,
            MipLevels: 1,
            ArraySize: 1,
            Format: DXGI_FORMAT_B8G8R8A8_UNORM,
            SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: (D3D11_BIND_RENDER_TARGET.0 | D3D11_BIND_SHADER_RESOURCE.0) as u32,
            CPUAccessFlags: 0,
            MiscFlags: D3D11_RESOURCE_MISC_GDI_COMPATIBLE.0 as u32,
        };
        let mut t = None;
        // SAFETY: the description and out-parameter live through the call.
        unsafe { gpu.device.CreateTexture2D(&desc, None, Some(&mut t)) }.map_err(|e| err("texture", e))?;
        t.ok_or("no texture")?
    };
    let rate = DXGI_RATIONAL {
        Numerator: fps,
        Denominator: 1,
    };
    let content = D3D11_VIDEO_PROCESSOR_CONTENT_DESC {
        InputFrameFormat: D3D11_VIDEO_FRAME_FORMAT_PROGRESSIVE,
        InputFrameRate: rate,
        InputWidth: monitor.0,
        InputHeight: monitor.1,
        OutputFrameRate: rate,
        OutputWidth: size.0,
        OutputHeight: size.1,
        Usage: D3D11_VIDEO_USAGE_OPTIMAL_QUALITY,
    };
    // SAFETY: COM calls with descriptions that live through each call.
    unsafe {
        let venum = gpu
            .video
            .CreateVideoProcessorEnumerator(&content)
            .map_err(|e| err("video processor", e))?;
        let vp = gpu
            .video
            .CreateVideoProcessor(&venum, 0)
            .map_err(|e| err("video processor", e))?;
        let desc = D3D11_VIDEO_PROCESSOR_INPUT_VIEW_DESC {
            FourCC: 0,
            ViewDimension: D3D11_VPIV_DIMENSION_TEXTURE2D,
            Anonymous: D3D11_VIDEO_PROCESSOR_INPUT_VIEW_DESC_0 {
                Texture2D: D3D11_TEX2D_VPIV {
                    MipSlice: 0,
                    ArraySlice: 0,
                },
            },
        };
        let mut input = None;
        gpu.video
            .CreateVideoProcessorInputView(&composite, &venum, &desc, Some(&mut input))
            .map_err(|e| err("video input", e))?;
        let input = input.ok_or("no video input")?;
        let ctx = &gpu.vctx;
        ctx.VideoProcessorSetStreamFrameFormat(&vp, 0, D3D11_VIDEO_FRAME_FORMAT_PROGRESSIVE);
        ctx.VideoProcessorSetStreamAutoProcessingMode(&vp, 0, false);
        ctx.VideoProcessorSetStreamSourceRect(&vp, 0, true, Some(&source));
        let whole = RECT {
            left: 0,
            top: 0,
            right: size.0 as i32,
            bottom: size.1 as i32,
        };
        ctx.VideoProcessorSetStreamDestRect(&vp, 0, true, Some(&whole));
        ctx.VideoProcessorSetOutputTargetRect(&vp, true, Some(&whole));
        // Screen RGB (full range) to video YCbCr (studio range).
        // Black around a window whose shape changed (VID-005).
        let black = D3D11_VIDEO_COLOR {
            Anonymous: D3D11_VIDEO_COLOR_0 {
                RGBA: D3D11_VIDEO_COLOR_RGBA {
                    R: 0.0,
                    G: 0.0,
                    B: 0.0,
                    A: 1.0,
                },
            },
        };
        ctx.VideoProcessorSetOutputBackgroundColor(&vp, false, &black);
        ctx.VideoProcessorSetStreamColorSpace1(&vp, 0, DXGI_COLOR_SPACE_RGB_FULL_G22_NONE_P709);
        ctx.VideoProcessorSetOutputColorSpace1(
            &vp,
            if hd(size) {
                DXGI_COLOR_SPACE_YCBCR_STUDIO_G22_LEFT_P709
            } else {
                DXGI_COLOR_SPACE_YCBCR_STUDIO_G22_LEFT_P601
            },
        );
        let painter = match crate::win_fx::Painter::new(&composite) {
            Ok(p) => Some(p),
            Err(e) => {
                tracing::warn!(error = %e, "video: no cursor effects");
                None
            }
        };
        Ok(Converter {
            venum,
            vp,
            input,
            frame,
            composite,
            origin,
            monitor,
            area: std::cell::Cell::new(source),
            scale,
            painter,
            camera: std::cell::RefCell::new(None),
            camera_bitmap: std::cell::RefCell::new(None),
            width: size.0,
            height: size.1,
        })
    }
}

impl Converter {
    /// Records `area` (virtual-screen pixels) from now on, fitted into the video with
    /// black bands if its shape changed (VID-005). False when it is not on this
    /// monitor any more.
    fn retarget(&self, gpu: &Gpu, area: RECT) -> bool {
        let (mw, mh) = (self.monitor.0 as i32, self.monitor.1 as i32);
        let src = RECT {
            left: (area.left - self.origin.0).clamp(0, mw),
            top: (area.top - self.origin.1).clamp(0, mh),
            right: (area.right - self.origin.0).clamp(0, mw),
            bottom: (area.bottom - self.origin.1).clamp(0, mh),
        };
        let (sw, sh) = ((src.right - src.left) as f64, (src.bottom - src.top) as f64);
        if sw < 2.0 || sh < 2.0 {
            return false;
        }
        let (w, h) = (self.width as f64, self.height as f64);
        let f = (w / sw).min(h / sh);
        let (dw, dh) = (sw * f, sh * f);
        let dest = RECT {
            left: ((w - dw) / 2.0).round() as i32,
            top: ((h - dh) / 2.0).round() as i32,
            right: ((w + dw) / 2.0).round() as i32,
            bottom: ((h + dh) / 2.0).round() as i32,
        };
        self.area.set(src);
        // SAFETY: state of the video processor, set on the immediate context.
        unsafe {
            gpu.vctx
                .VideoProcessorSetStreamSourceRect(&self.vp, 0, true, Some(&src));
            gpu.vctx
                .VideoProcessorSetStreamDestRect(&self.vp, 0, true, Some(&dest));
        }
        true
    }

    /// The last picture, with the effects (VID-045 to VID-047) and the pointer on it
    /// when `cursor` (VID-044). Without the pointer only the keys are shown.
    fn compose(&self, gpu: &Gpu, cursor: bool, fx: &Effects, live: &Live, now: Instant) {
        use windows::Win32::UI::WindowsAndMessaging::*;
        // SAFETY: GPU copy, then Direct2D and GDI drawing on the texture, the DC
        // released at once; the icon's bitmaps from GetIconInfo are deleted.
        unsafe {
            gpu.context.CopyResource(&self.composite, &self.frame);
            // The camera over the screen (VID-032); filling it, nothing goes over it
            // (VID-034: no pointer, effects or keys).
            let a = self.area.get();
            let area = (
                a.left as f32,
                a.top as f32,
                (a.right - a.left) as f32,
                (a.bottom - a.top) as f32,
            );
            if let (Some(p), Some((frame, look))) = (&self.painter, self.camera.borrow().as_ref()) {
                p.draw_camera(
                    &mut self.camera_bitmap.borrow_mut(),
                    frame,
                    look,
                    area,
                    self.scale,
                );
                if look.placement == crate::camlook::Placement::Fullscreen {
                    return;
                }
            }
            let mut info = CURSORINFO {
                cbSize: std::mem::size_of::<CURSORINFO>() as u32,
                ..Default::default()
            };
            let visible = cursor
                && GetCursorInfo(&mut info).is_ok()
                && info.flags == CURSOR_SHOWING
                && !info.hCursor.is_invalid();
            let mut shown = fx.clone();
            if !cursor {
                shown.highlight = None;
                shown.clicks = None;
            }
            if let (Some(p), true) = (&self.painter, shown.any()) {
                let a = self.area.get();
                let pointer = visible.then(|| {
                    (
                        (info.ptScreenPos.x - self.origin.0) as f32,
                        (info.ptScreenPos.y - self.origin.1) as f32,
                    )
                });
                p.draw(
                    &shown,
                    live,
                    pointer,
                    (
                        a.left as f32,
                        a.top as f32,
                        (a.right - a.left) as f32,
                        (a.bottom - a.top) as f32,
                    ),
                    (self.origin.0 as f32, self.origin.1 as f32),
                    self.scale,
                    now,
                );
            }
            if !visible {
                return;
            }
            let mut icon = ICONINFO::default();
            let hotspot = if GetIconInfo(HICON(info.hCursor.0), &mut icon).is_ok() {
                let hs = (icon.xHotspot as i32, icon.yHotspot as i32);
                use windows::Win32::Graphics::Gdi::DeleteObject;
                if !icon.hbmMask.is_invalid() {
                    let _ = DeleteObject(icon.hbmMask.into());
                }
                if !icon.hbmColor.is_invalid() {
                    let _ = DeleteObject(icon.hbmColor.into());
                }
                hs
            } else {
                (0, 0)
            };
            let Ok(surface) = self.composite.cast::<IDXGISurface1>() else {
                return;
            };
            if let Ok(dc) = surface.GetDC(false) {
                let _ = DrawIconEx(
                    dc,
                    info.ptScreenPos.x - self.origin.0 - hotspot.0,
                    info.ptScreenPos.y - self.origin.1 - hotspot.1,
                    HICON(info.hCursor.0),
                    0,
                    0,
                    0,
                    None,
                    DI_NORMAL,
                );
                let _ = surface.ReleaseDC(None);
            }
        }
    }

    /// A new NV12 frame from the last picture.
    fn nv12(&self, gpu: &Gpu, cursor: bool, fx: &Effects, live: &Live) -> Result<ID3D11Texture2D, String> {
        self.compose(gpu, cursor, fx, live, Instant::now());
        let out = texture(
            gpu,
            self.width,
            self.height,
            DXGI_FORMAT_NV12,
            D3D11_BIND_RENDER_TARGET.0 as u32,
            false,
        )?;
        let desc = D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC {
            ViewDimension: D3D11_VPOV_DIMENSION_TEXTURE2D,
            Anonymous: D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC_0 {
                Texture2D: D3D11_TEX2D_VPOV { MipSlice: 0 },
            },
        };
        // SAFETY: COM calls; the stream's input view is released after the blit.
        unsafe {
            let mut view = None;
            gpu.video
                .CreateVideoProcessorOutputView(&out, &self.venum, &desc, Some(&mut view))
                .map_err(|e| err("video output", e))?;
            let view = view.ok_or("no video output")?;
            let stream = D3D11_VIDEO_PROCESSOR_STREAM {
                Enable: BOOL::from(true),
                pInputSurface: ManuallyDrop::new(Some(self.input.clone())),
                ..Default::default()
            };
            let streams = [stream];
            let blit = gpu.vctx.VideoProcessorBlt(&self.vp, &view, 0, &streams);
            let [stream] = streams;
            drop(ManuallyDrop::into_inner(stream.pInputSurface));
            blit.map_err(|e| err("video conversion", e))?;
        }
        Ok(out)
    }
}

/// The Media Foundation writer and how it gets its frames.
struct Writer {
    sink: IMFSinkWriter,
    stream: u32,
    /// The sound's stream and its rate (VID-040).
    audio: Option<(u32, u32)>,
    hardware: bool,
    /// The encoder, to ask for a key frame (VID-022: fragmented files).
    codec: Option<ICodecAPI>,
    /// For the software encoder: frames go through memory.
    staging: Option<ID3D11Texture2D>,
    width: u32,
    height: u32,
}

/// What the H.264 encoder is told: no B-frames. With them, Windows' encoders delay
/// every time by one frame, and with a variable frame rate one frame can be seconds
/// (the MP4 gets no edit list to undo it).
pub(crate) fn encoder_settings() -> Option<IMFAttributes> {
    // SAFETY: a new attribute store with one value.
    unsafe {
        let mut attrs = None;
        MFCreateAttributes(&mut attrs, 1).ok()?;
        let attrs: IMFAttributes = attrs?;
        attrs.SetUINT32(&CODECAPI_AVEncMPVDefaultBPictureCount, 0).ok()?;
        Some(attrs)
    }
}

pub(crate) fn media_type(
    subtype: &windows::core::GUID,
    size: (u32, u32),
    fps: u32,
    bitrate: Option<u32>,
) -> Result<IMFMediaType, String> {
    // SAFETY: attribute setters on a new media type.
    unsafe {
        let t = MFCreateMediaType().map_err(|e| err("media type", e))?;
        let set = |r: windows::core::Result<()>| r.map_err(|e| err("media type", e));
        set(t.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video))?;
        set(t.SetGUID(&MF_MT_SUBTYPE, subtype))?;
        set(t.SetUINT32(&MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive.0 as u32))?;
        set(t.SetUINT64(&MF_MT_FRAME_SIZE, ((size.0 as u64) << 32) | size.1 as u64))?;
        set(t.SetUINT64(&MF_MT_FRAME_RATE, ((fps as u64) << 32) | 1))?;
        set(t.SetUINT64(&MF_MT_PIXEL_ASPECT_RATIO, (1u64 << 32) | 1))?;
        let matrix = if hd(size) {
            MFVideoTransferMatrix_BT709
        } else {
            MFVideoTransferMatrix_BT601
        };
        set(t.SetUINT32(&MF_MT_YUV_MATRIX, matrix.0 as u32))?;
        set(t.SetUINT32(&MF_MT_VIDEO_NOMINAL_RANGE, MFNominalRange_16_235.0 as u32))?;
        if let Some(b) = bitrate {
            set(t.SetUINT32(&MF_MT_AVG_BITRATE, b))?;
        } else {
            set(t.SetUINT32(&MF_MT_DEFAULT_STRIDE, size.0))?;
        }
        Ok(t)
    }
}

fn writer(
    gpu: &Gpu,
    path: &std::path::Path,
    size: (u32, u32),
    fps: u32,
    bitrate: u32,
    hardware: bool,
    audio_rate: Option<u32>,
) -> Result<Writer, String> {
    writer_in(gpu, path, size, fps, bitrate, hardware, audio_rate, false)
}

/// A writer; `fragmented`, a fragmented MP4: what was written up to a second or two
/// before stays readable if the app dies (VID-022).
#[allow(clippy::too_many_arguments)]
fn writer_in(
    gpu: &Gpu,
    path: &std::path::Path,
    size: (u32, u32),
    fps: u32,
    bitrate: u32,
    hardware: bool,
    audio_rate: Option<u32>,
    fragmented: bool,
) -> Result<Writer, String> {
    // SAFETY: Media Foundation calls with arguments that live through each call.
    unsafe {
        let mut attrs = None;
        MFCreateAttributes(&mut attrs, 4).map_err(|e| err("attributes", e))?;
        let attrs: IMFAttributes = attrs.ok_or("no attributes")?;
        let set = |r: windows::core::Result<()>| r.map_err(|e| err("attributes", e));
        set(attrs.SetUINT32(&MF_SINK_WRITER_DISABLE_THROTTLING, 1))?;
        let container = if fragmented {
            MFTranscodeContainerType_FMPEG4
        } else {
            MFTranscodeContainerType_MPEG4
        };
        set(attrs.SetGUID(&MF_TRANSCODE_CONTAINERTYPE, &container))?;
        if hardware {
            let mut token = 0u32;
            let mut manager = None;
            MFCreateDXGIDeviceManager(&mut token, &mut manager).map_err(|e| err("device manager", e))?;
            let manager: IMFDXGIDeviceManager = manager.ok_or("no device manager")?;
            manager
                .ResetDevice(&gpu.device, token)
                .map_err(|e| err("device manager", e))?;
            set(attrs.SetUINT32(&MF_READWRITE_ENABLE_HARDWARE_TRANSFORMS, 1))?;
            set(attrs.SetUnknown(&MF_SINK_WRITER_D3D_MANAGER, &manager))?;
        } else {
            set(attrs.SetUINT32(&MF_READWRITE_ENABLE_HARDWARE_TRANSFORMS, 0))?;
        }
        let sink = MFCreateSinkWriterFromURL(&HSTRING::from(path.as_os_str()), None, &attrs)
            .map_err(|e| err("video file", e))?;
        let out = media_type(&MFVideoFormat_H264, size, fps, Some(bitrate))?;
        let _ = out.SetUINT32(&MF_MT_MPEG2_PROFILE, eAVEncH264VProfile_High.0 as u32);
        let stream = match sink.AddStream(&out) {
            Ok(s) => s,
            Err(_) => {
                // Some encoders refuse the High profile: their default then.
                let _ = out.DeleteItem(&MF_MT_MPEG2_PROFILE);
                sink.AddStream(&out).map_err(|e| err("video stream", e))?
            }
        };
        let input = media_type(&MFVideoFormat_NV12, size, fps, None)?;
        sink.SetInputMediaType(stream, &input, encoder_settings().as_ref())
            .map_err(|e| err("video input type", e))?;
        // The sound: 16-bit stereo PCM in, AAC at 128 kbps out (VID-026).
        let audio = match audio_rate {
            Some(rate) => {
                let t = |subtype: &windows::core::GUID| -> windows::core::Result<IMFMediaType> {
                    let t = MFCreateMediaType()?;
                    t.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Audio)?;
                    t.SetGUID(&MF_MT_SUBTYPE, subtype)?;
                    t.SetUINT32(&MF_MT_AUDIO_BITS_PER_SAMPLE, 16)?;
                    t.SetUINT32(&MF_MT_AUDIO_SAMPLES_PER_SECOND, rate)?;
                    t.SetUINT32(&MF_MT_AUDIO_NUM_CHANNELS, 2)?;
                    Ok(t)
                };
                let out = t(&MFAudioFormat_AAC).map_err(|e| err("audio type", e))?;
                let _ = out.SetUINT32(&MF_MT_AUDIO_AVG_BYTES_PER_SECOND, 16_000);
                let index = sink.AddStream(&out).map_err(|e| err("audio stream", e))?;
                let input = t(&MFAudioFormat_PCM).map_err(|e| err("audio type", e))?;
                let _ = input.SetUINT32(&MF_MT_AUDIO_BLOCK_ALIGNMENT, 4);
                let _ = input.SetUINT32(&MF_MT_AUDIO_AVG_BYTES_PER_SECOND, rate * 4);
                sink.SetInputMediaType(index, &input, None)
                    .map_err(|e| err("audio input type", e))?;
                Some((index, rate))
            }
            None => None,
        };
        sink.BeginWriting().map_err(|e| err("video writing", e))?;
        let codec = if fragmented {
            encoder_api(&sink, stream)
        } else {
            None
        };
        if fragmented && codec.is_none() {
            tracing::info!("video: no key frames on demand; a crash may lose more");
        }
        let staging = if hardware {
            None
        } else {
            Some(texture(gpu, size.0, size.1, DXGI_FORMAT_NV12, 0, true)?)
        };
        Ok(Writer {
            sink,
            stream,
            audio,
            hardware,
            codec,
            staging,
            width: size.0,
            height: size.1,
        })
    }
}

/// The encoder of the writer's `stream`, to change its settings while it works.
fn encoder_api(sink: &IMFSinkWriter, stream: u32) -> Option<ICodecAPI> {
    let ex: IMFSinkWriterEx = sink.cast().ok()?;
    for i in 0..4 {
        let mut category = windows::core::GUID::zeroed();
        let mut transform = None;
        // SAFETY: out-parameters that live through the call; an error ends the list.
        if unsafe { ex.GetTransformForStream(stream, i, Some(&mut category), &mut transform) }.is_err() {
            break;
        }
        if category == MFT_CATEGORY_VIDEO_ENCODER {
            if let Some(api) = transform.and_then(|t| t.cast::<ICodecAPI>().ok()) {
                return Some(api);
            }
        }
    }
    None
}

impl Writer {
    /// The next frame is a key frame: it closes the fragment before it (VID-022).
    fn force_key(&self) {
        let Some(api) = &self.codec else { return };
        let mut v = windows::Win32::System::Variant::VARIANT::default();
        // SAFETY: a VT_UI4 VARIANT, filled as such and read by the encoder.
        unsafe {
            let inner = &mut *v.Anonymous.Anonymous;
            inner.vt = windows::Win32::System::Variant::VT_UI4;
            inner.Anonymous.ulVal = 1;
            let _ = api.SetValue(&CODECAPI_AVEncVideoForceKeyFrame, &v);
        }
    }

    /// Writes `frame` (NV12 on the GPU) at `time` (seconds) lasting `length`.
    fn write(&self, gpu: &Gpu, frame: &ID3D11Texture2D, time: f64, length: f64) -> Result<(), String> {
        // SAFETY: Media Foundation and D3D11 calls; a mapped texture is unmapped before
        // returning and the copy stays within both buffers.
        unsafe {
            let buffer = if self.hardware {
                let b = MFCreateDXGISurfaceBuffer(&ID3D11Texture2D::IID, frame, 0, false)
                    .map_err(|e| err("frame buffer", e))?;
                if let Ok(b2) = b.cast::<IMF2DBuffer>() {
                    if let Ok(len) = b2.GetContiguousLength() {
                        let _ = b.SetCurrentLength(len);
                    }
                }
                b
            } else {
                let staging = self.staging.as_ref().ok_or("no staging")?;
                gpu.context.CopyResource(staging, frame);
                let (w, h) = (self.width as usize, self.height as usize);
                let size = w * h * 3 / 2;
                let b = MFCreateMemoryBuffer(size as u32).map_err(|e| err("frame buffer", e))?;
                let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
                gpu.context
                    .Map(staging, 0, D3D11_MAP_READ, 0, Some(&mut mapped))
                    .map_err(|e| err("frame copy", e))?;
                let mut dst = std::ptr::null_mut();
                let locked = b.Lock(&mut dst, None, None);
                if locked.is_ok() {
                    let pitch = mapped.RowPitch as usize;
                    let src = mapped.pData as *const u8;
                    // Luma rows, then the interleaved chroma rows below them.
                    for row in 0..h + h / 2 {
                        std::ptr::copy_nonoverlapping(src.add(row * pitch), dst.add(row * w), w);
                    }
                    let _ = b.Unlock();
                    let _ = b.SetCurrentLength(size as u32);
                }
                gpu.context.Unmap(staging, 0);
                locked.map_err(|e| err("frame copy", e))?;
                b
            };
            let sample = MFCreateSample().map_err(|e| err("sample", e))?;
            sample.AddBuffer(&buffer).map_err(|e| err("sample", e))?;
            sample
                .SetSampleTime((time * HNS) as i64)
                .map_err(|e| err("sample", e))?;
            sample
                .SetSampleDuration((length * HNS).max(1.0) as i64)
                .map_err(|e| err("sample", e))?;
            self.sink
                .WriteSample(self.stream, &sample)
                .map_err(|e| err("encoding", e))
        }
    }

    /// Writes stereo 16-bit `samples` starting at `time` (seconds).
    fn write_audio(&self, samples: &[i16], time: f64) -> Result<(), String> {
        let Some((stream, rate)) = self.audio else {
            return Ok(());
        };
        if samples.is_empty() {
            return Ok(());
        }
        let bytes = samples.len() * 2;
        // SAFETY: the buffer is locked, filled within its size and unlocked.
        unsafe {
            let b = MFCreateMemoryBuffer(bytes as u32).map_err(|e| err("audio buffer", e))?;
            let mut dst = std::ptr::null_mut();
            b.Lock(&mut dst, None, None).map_err(|e| err("audio buffer", e))?;
            std::ptr::copy_nonoverlapping(samples.as_ptr() as *const u8, dst, bytes);
            let _ = b.Unlock();
            let _ = b.SetCurrentLength(bytes as u32);
            let sample = MFCreateSample().map_err(|e| err("sample", e))?;
            sample.AddBuffer(&b).map_err(|e| err("sample", e))?;
            let frames = (samples.len() / 2) as f64;
            let _ = sample.SetSampleTime((time * HNS) as i64);
            let _ = sample.SetSampleDuration((frames / rate as f64 * HNS) as i64);
            self.sink
                .WriteSample(stream, &sample)
                .map_err(|e| err("sound", e))
        }
    }

    fn finish(&self) -> Result<(), String> {
        // SAFETY: closes the file.
        unsafe { self.sink.Finalize() }.map_err(|e| err("closing the video", e))
    }
}

/// The monitor's picture as it is now, read with GDI into `conv.frame`: the first
/// frame, so a screen that does not change (or a display that sleeps, when Desktop
/// Duplication gets nothing) is never recorded black.
/// The monitor's picture with GDI (BGRA, rows of w × 4 bytes): a still or sleeping
/// screen is never recorded black.
fn screen_bits(monitor: RECT) -> Result<Vec<u8>, String> {
    use windows::Win32::Graphics::Gdi::*;
    let (w, h) = (monitor.right - monitor.left, monitor.bottom - monitor.top);
    // SAFETY: GDI objects are released on every path; the bits pointer is valid while
    // the DIB section lives and holds w × h × 4 bytes.
    unsafe {
        let screen = GetDC(None);
        let mem = CreateCompatibleDC(Some(screen));
        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w,
                biHeight: -h,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bits = std::ptr::null_mut();
        let result = match CreateDIBSection(Some(mem), &info, DIB_RGB_COLORS, &mut bits, None, 0) {
            Ok(dib) => {
                let old = SelectObject(mem, dib.into());
                let copied = BitBlt(
                    mem,
                    0,
                    0,
                    w,
                    h,
                    Some(screen),
                    monitor.left,
                    monitor.top,
                    SRCCOPY | CAPTUREBLT,
                );
                // GDI leaves alpha at 0: the converter ignores it.
                let pixels = copied
                    .map(|_| std::slice::from_raw_parts(bits as *const u8, (w * h * 4) as usize).to_vec());
                SelectObject(mem, old);
                let _ = DeleteObject(dib.into());
                pixels.map_err(|e| err("screen picture", e))
            }
            Err(e) => Err(err("screen picture", e)),
        };
        let _ = DeleteDC(mem);
        ReleaseDC(None, screen);
        result
    }
}

/// Where a window is (its visible frame, without the invisible resize borders), or
/// `Err(true)` when it closed, `Err(false)` while it is minimized.
fn window_bounds(hwnd: isize) -> Result<RECT, bool> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS};
    use windows::Win32::UI::WindowsAndMessaging::{IsIconic, IsWindow};
    let h = HWND(hwnd as *mut core::ffi::c_void);
    // SAFETY: plain queries on a window handle; a closed one answers false.
    unsafe {
        if !IsWindow(Some(h)).as_bool() {
            return Err(true);
        }
        if IsIconic(h).as_bool() {
            return Err(false);
        }
        let mut r = RECT::default();
        DwmGetWindowAttribute(
            h,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            &mut r as *mut RECT as *mut core::ffi::c_void,
            std::mem::size_of::<RECT>() as u32,
        )
        .map_err(|_| false)?;
        Ok(r)
    }
}

/// Free bytes on the disk that holds `path` (VID-023).
pub fn free_space(path: &std::path::Path) -> Option<u64> {
    let dir = path.parent()?;
    let mut free = 0u64;
    // SAFETY: a plain query with an out-parameter.
    unsafe {
        windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW(
            &HSTRING::from(dir.as_os_str()),
            Some(&mut free),
            None,
            None,
        )
    }
    .ok()?;
    Some(free)
}

/// While it lives, Windows neither sleeps nor turns the screen off for being idle
/// (VID-021); what the user asks for (the lid, the power menu) still works.
struct Awake;

impl Awake {
    fn hold() -> Awake {
        use windows::Win32::System::Power::*;
        // SAFETY: a plain call for this thread.
        unsafe {
            SetThreadExecutionState(ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED);
        }
        Awake
    }
}

impl Drop for Awake {
    fn drop(&mut self) {
        use windows::Win32::System::Power::*;
        // SAFETY: a plain call for this thread.
        unsafe {
            SetThreadExecutionState(ES_CONTINUOUS);
        }
    }
}

/// Grabs the next picture into `conv.frame`: true when the screen changed.
fn grab(
    gpu: &Gpu,
    dup: &IDXGIOutputDuplication,
    conv: &Converter,
    wait_ms: u32,
    cursor: bool,
) -> Result<bool, windows::core::Error> {
    // SAFETY: the frame is released before returning; the copy is a GPU command.
    unsafe {
        let mut info = DXGI_OUTDUPL_FRAME_INFO::default();
        let mut resource = None;
        match dup.AcquireNextFrame(wait_ms, &mut info, &mut resource) {
            Ok(()) => {}
            Err(e) if e.code() == DXGI_ERROR_WAIT_TIMEOUT => return Ok(false),
            Err(e) => return Err(e),
        }
        let changed = info.LastPresentTime != 0 || info.AccumulatedFrames > 0;
        if changed {
            if let Some(tex) = resource.and_then(|r| r.cast::<ID3D11Texture2D>().ok()) {
                gpu.context.CopyResource(&conv.frame, &tex);
            }
        }
        let _ = dup.ReleaseFrame();
        Ok(changed || (cursor && info.LastMouseUpdateTime != 0))
    }
}

pub(crate) fn record(
    options: Options,
    size: (u32, u32),
    shared: Arc<Shared>,
    ready: std::sync::mpsc::Sender<Result<(), String>>,
) -> Result<Summary, String> {
    // SAFETY: COM and DPI setup for this thread only.
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        // Desktop Duplication needs physical pixels.
        let _ = SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let _ = MFStartup(MF_VERSION, MFSTARTUP_FULL);
    }
    let fps = options.quality.fps();
    // VID-022: written into the fragmented file while it records, when there is one.
    let target = options.recovery.as_deref().unwrap_or(&options.path);
    let fragmented = options.recovery.is_some();
    let mut keeper = crate::Keeper::default();
    let setup = (|| {
        let t0 = Instant::now();
        let (adapter, output, mr) = find_output(options.area)?;
        // The first picture, read while the engine starts (VID-030).
        let picture = std::thread::spawn(move || screen_bits(mr));
        let gpu = take_gpu(&mut DEVICE.lock().unwrap(), &adapter)?;
        let t_device = t0.elapsed();
        // SAFETY: duplication of the output on this device.
        let dup = unsafe { output.DuplicateOutput(&gpu.device) }.map_err(|e| err("screen duplication", e))?;
        let monitor = ((mr.right - mr.left) as u32, (mr.bottom - mr.top) as u32);
        // SAFETY: a plain query.
        let mode = unsafe { dup.GetDesc() }.ModeDesc;
        tracing::info!(
            width = mode.Width,
            height = mode.Height,
            format = mode.Format.0,
            "video: duplicating the screen"
        );
        // The area inside the monitor, clipped to it.
        let a = options.area;
        let source = RECT {
            left: (a.x - mr.left).clamp(0, monitor.0 as i32),
            top: (a.y - mr.top).clamp(0, monitor.1 as i32),
            right: (a.x - mr.left + a.width as i32).clamp(0, monitor.0 as i32),
            bottom: (a.y - mr.top + a.height as i32).clamp(0, monitor.1 as i32),
        };
        if source.right - source.left < 2 || source.bottom - source.top < 2 {
            return Err("the area is outside the monitor".to_string());
        }
        let conv = converter(
            &gpu,
            monitor,
            (mr.left, mr.top),
            source,
            size,
            fps,
            options.scale as f32,
        )?;
        let t_dup = t0.elapsed();
        let bitrate = options.quality.bitrate(size.0, size.1);
        // The system's sound, when asked; without it the video goes on silent.
        let loopback = if options.system_audio {
            match crate::audio::Loopback::open() {
                Ok(l) => Some(l),
                Err(e) => {
                    tracing::warn!(error = %e, "video: no system sound");
                    None
                }
            }
        } else {
            None
        };
        // The microphone (VID-039), at the system sound's rate so they add up.
        let mic = options.microphone.as_ref().and_then(|m| {
            let rate = loopback.as_ref().map_or(48_000, |l| l.rate);
            match crate::audio::microphone(m.device.as_deref(), rate) {
                Ok(c) => Some(c),
                Err(e) => {
                    tracing::warn!(error = %e, "video: no microphone");
                    None
                }
            }
        });
        // A sound track whenever some sound was asked for, even if its device is not
        // there yet: it can come (VID-043) or be switched on later (VID-011).
        let audio_rate = (options.system_audio || options.microphone.is_some())
            .then(|| loopback.as_ref().map_or(48_000, |l| l.rate));
        let _ = std::fs::remove_file(target);
        let w = if options.software_only {
            writer_in(&gpu, target, size, fps, bitrate, false, audio_rate, fragmented)?
        } else {
            match writer_in(&gpu, target, size, fps, bitrate, true, audio_rate, fragmented) {
                Ok(w) => w,
                Err(e) => {
                    tracing::info!(error = %e, "video: no hardware encoder, software instead");
                    let _ = std::fs::remove_file(target);
                    writer_in(&gpu, target, size, fps, bitrate, false, audio_rate, fragmented)?
                }
            }
        };
        let t_enc = t0.elapsed();
        // The first picture, whatever Desktop Duplication does.
        let bits = picture.join().map_err(|_| "screen picture".to_string())??;
        // SAFETY: the pixels are the monitor's size in BGRA, rows of width × 4 bytes.
        unsafe {
            gpu.context
                .UpdateSubresource(&conv.frame, 0, None, bits.as_ptr() as _, monitor.0 * 4, 0);
        }
        tracing::info!(
            device_ms = t_device.as_millis() as u64,
            duplication_ms = (t_dup - t_device).as_millis() as u64,
            encoder_ms = (t_enc - t_dup).as_millis() as u64,
            total_ms = t0.elapsed().as_millis() as u64,
            "video: engine started"
        );
        let gpu = KeepOnDrop(Some(gpu), adapter_id(&adapter));
        Ok((gpu, output, dup, conv, w, loopback, mic, audio_rate))
    })();
    let (gpu, output, mut dup, conv, mut w, mut loopback, mut mic, audio_rate) = match setup {
        Ok(s) => s,
        Err(e) => {
            let _ = ready.send(Err(e.clone()));
            return Err(e);
        }
    };
    shared.clock.lock().unwrap().started = Some(Instant::now());
    shared.set_state(State::Recording);
    let _awake = Awake::hold();
    let _ = ready.send(Ok(()));

    let interval = 1.0 / fps as f64;
    let mut frames = 0u64;
    let mut last_write: Option<Instant> = None;
    let mut last_time = -1.0f64;
    let mut last_frame: Option<ID3D11Texture2D> = None;
    // The snapshot is the first frame.
    let mut dirty = true;
    let mut was_paused = false;
    let mut failure: Option<String> = None;
    // When a lost output was last looked for.
    let mut last_reopen = Instant::now();
    // The sound, put together (VID-039, VID-040): the system's and the microphone's,
    // each at its moment; the microphone leveled.
    let mut mixer = audio_rate.map(crate::mix::Mixer::new);
    let (mut system_track, mut mic_track) = (crate::mix::Track::default(), crate::mix::Track::default());
    let mut agc = crate::mix::Agc::default();
    let mut had_mic = mic.is_some();
    let mut had_system = loopback.is_some();
    let mut last_mic_reopen = Instant::now();
    // The microphone wanted now, which device is open, and the meters (VID-011).
    let mut want_mic = options.microphone.clone();
    let mut mic_open_for = mic
        .as_ref()
        .map(|_| want_mic.as_ref().and_then(|m| m.device.clone()));
    let mut mic_seen = 0u32;
    let (mut mic_gauge, mut system_gauge) = (crate::mix::Gauge::default(), crate::mix::Gauge::default());
    let (mut mic_muted, mut last_mute): (bool, Option<Instant>) = (false, None);
    // Cursor and keyboard effects (VID-045 to VID-047): what shows, the hooks that
    // feed it while clicks or keys are wanted, and the password check.
    let mut live = Live::default();
    let mut hooks: Option<crate::win_fx::Hooks> = None;
    let focus = crate::win_fx::Focus::new();
    let mut fx = shared.effects.lock().unwrap().clone();
    // The disk is looked at every two seconds (VID-023).
    let mut last_disk = Instant::now();
    let mut low_disk = false;
    // A followed window (VID-005): where it was, and whether it closed or hides.
    let mut window_at: Option<RECT> = None;
    let mut window_closed = false;
    let mut hold = false;
    // VID-030: when the first frame was written, since when a change waits, and the
    // frame turns changes missed.
    let mut first_frame_at: Option<Instant> = None;
    let mut dirty_since: Option<Instant> = None;
    let mut late_frames = 0u64;
    // When the screen last changed: the frame's time, not when it is written, which
    // can be up to a frame later (VID-030).
    let mut changed_at: Option<Instant> = None;
    let recorded_at = |at: Instant| shared.clock.lock().unwrap().recorded(at).as_secs_f64();
    let now_recorded = || recorded_at(Instant::now());

    // Writes the current picture at the recorded time. A hardware encoder that fails
    // on the first frame is replaced by software; one that fails later stops the
    // recording with what was written (continuing in software: still to do, VID-029).
    let write_frame = |w: &Writer,
                       last_frame: &mut Option<ID3D11Texture2D>,
                       last_time: &mut f64,
                       frames: &mut u64,
                       fx: &Effects,
                       live: &Live,
                       at: Option<Instant>|
     -> Result<(), String> {
        let frame = conv.nv12(&gpu, shared.cursor.load(Ordering::Relaxed), fx, live)?;
        let t = at.map_or_else(now_recorded, recorded_at).max(*last_time + 0.001);
        w.write(&gpu, &frame, t, interval)?;
        *last_time = t;
        *last_frame = Some(frame);
        *frames += 1;
        Ok(())
    };
    // VID-022, into a fragmented file: the frame of a turn of the grid at `t`, the
    // picture now when `fresh`, else the last one again; a key frame when `key`.
    let grid_frame = |w: &Writer,
                      last_frame: &mut Option<ID3D11Texture2D>,
                      fresh: bool,
                      key: bool,
                      t: f64,
                      fx: &Effects,
                      live: &Live|
     -> Result<(), String> {
        let frame = match (fresh, last_frame.as_ref()) {
            (false, Some(f)) => f.clone(),
            _ => conv.nv12(&gpu, shared.cursor.load(Ordering::Relaxed), fx, live)?,
        };
        if key {
            w.force_key();
        }
        w.write(&gpu, &frame, t, interval)?;
        *last_frame = Some(frame);
        Ok(())
    };
    // The next turn of the grid (VID-022).
    let mut slot = 0u64;

    loop {
        if shared.stop.load(Ordering::Relaxed) {
            break;
        }
        if options.limit.is_some_and(|l| now_recorded() >= l.as_secs_f64()) {
            break;
        }
        if let Some(hwnd) = options.window {
            match window_bounds(hwnd) {
                Ok(r) => {
                    hold = false;
                    if window_at != Some(r) {
                        // It moved or resized: follow it; off this monitor, hold.
                        hold = !conv.retarget(&gpu, r);
                        window_at = Some(r);
                        dirty = true;
                        changed_at = None;
                    }
                }
                Err(true) => {
                    tracing::info!("video: the window closed, stopping");
                    window_closed = true;
                    break;
                }
                // Minimized: the last picture stays (VID-005).
                Err(false) => hold = true,
            }
        }
        if last_disk.elapsed() > Duration::from_secs(2) {
            last_disk = Instant::now();
            let low = |p: &std::path::Path| free_space(p).is_some_and(crate::low_disk);
            if low(&options.path) || (fragmented && low(target)) {
                tracing::warn!("video: the disk is almost full, stopping");
                low_disk = true;
                break;
            }
        }
        let due_in = match last_write {
            // Into a fragmented file: until the next turn.
            _ if fragmented => (slot as f64 * interval - now_recorded()).clamp(0.0, 0.1),
            Some(t) if dirty => (interval - t.elapsed().as_secs_f64()).max(0.0),
            _ => 0.1,
        };
        match grab(
            &gpu,
            &dup,
            &conv,
            (due_in * 1000.0) as u32,
            shared.cursor.load(Ordering::Relaxed),
        ) {
            Ok(true) => {
                dirty = true;
                changed_at = Some(Instant::now());
            }
            Ok(false) => {}
            Err(e) if e.code() == DXGI_ERROR_ACCESS_LOST => {
                // A mode change, or the secure desktop: duplicate again.
                std::thread::sleep(Duration::from_millis(100));
                // SAFETY: duplication of the output on this device.
                match unsafe { output.DuplicateOutput(&gpu.device) } {
                    Ok(d) => {
                        dup = d;
                        dirty = true;
                    }
                    Err(_) => continue,
                }
            }
            Err(e) => {
                failure = Some(err("screen", e));
                break;
            }
        }
        // The sound (VID-039, VID-040): each source placed at the moment it was heard;
        // what all have had time to give is written, silence where nothing played.
        if let Some(mx) = mixer.as_mut() {
            let paused = shared.paused.load(Ordering::Relaxed);
            let rate = mx.rate as f64;
            let at_now = Instant::now();
            // The system's sound, kept while it is on (VID-011); measured always.
            let system_on = shared.system_audio.load(Ordering::Relaxed);
            if let Some(l) = &loopback {
                match l.read() {
                    Ok(samples) => {
                        let p = crate::mix::peak(&samples);
                        system_gauge.feed(p, p, p >= 0.99, at_now);
                        if system_on && !paused && !samples.is_empty() {
                            let target = now_recorded() - samples.len() as f64 / 2.0 / rate;
                            mx.place(&mut system_track, target, &samples);
                        }
                    }
                    Err(e) if crate::audio::device_gone(&e) => {
                        // Another output was chosen: listen to the new one, at the same rate.
                        loopback = crate::audio::Loopback::open()
                            .ok()
                            .filter(|n| Some(n.rate) == audio_rate);
                    }
                    Err(_) => {}
                }
            } else if system_on && last_reopen.elapsed() > Duration::from_secs(1) {
                last_reopen = Instant::now();
                loopback = crate::audio::Loopback::open()
                    .ok()
                    .filter(|n| Some(n.rate) == audio_rate);
                had_system |= loopback.is_some();
            }
            // The microphone switched on, off or to another device (VID-011).
            let changed = shared.microphone_changed.load(Ordering::Relaxed);
            if changed != mic_seen {
                mic_seen = changed;
                want_mic = shared.microphone.lock().unwrap().clone();
                let device = want_mic.as_ref().map(|m| m.device.clone());
                if device.is_none() || mic_open_for != device {
                    mic = None;
                    mic_open_for = None;
                    last_mic_reopen = Instant::now() - Duration::from_secs(2);
                }
            }
            if let Some(m) = &mic {
                match m.read() {
                    Ok(mut samples) => {
                        let raw = crate::mix::peak(&samples);
                        let auto_level = want_mic.as_ref().is_some_and(|m| m.auto_level);
                        agc.process(&mut samples, mx.rate, auto_level);
                        mic_gauge.feed(raw, agc.peak, agc.clipped, at_now);
                        if !paused && !samples.is_empty() {
                            let target = now_recorded() - samples.len() as f64 / 2.0 / rate;
                            mx.place(&mut mic_track, target, &samples);
                        }
                    }
                    Err(e) if crate::audio::device_gone(&e) => {
                        // Unplugged: the recording goes on without it (VID-043).
                        tracing::warn!("video: the microphone went away");
                        mic = None;
                        mic_open_for = None;
                    }
                    Err(_) => {}
                }
            } else if let Some(want) = want_mic.as_ref() {
                mic_gauge.feed(0.0, 0.0, false, at_now);
                // Plugged in again, or switched on: it comes (VID-043).
                if last_mic_reopen.elapsed() > Duration::from_secs(1) {
                    last_mic_reopen = Instant::now();
                    mic = crate::audio::microphone(want.device.as_deref(), mx.rate).ok();
                    if mic.is_some() {
                        mic_open_for = Some(want.device.clone());
                        had_mic = true;
                    }
                }
            }
            // Muted in the system (VID-019), looked at every 2 s.
            if let Some(want) = want_mic.as_ref() {
                if last_mute.is_none_or(|t| t.elapsed() > Duration::from_secs(2)) {
                    last_mute = Some(at_now);
                    mic_muted = crate::audio::input_muted(want.device.as_deref()).unwrap_or(false);
                }
            }
            *shared.levels.lock().unwrap() = crate::mix::Levels {
                microphone: want_mic.is_some().then(|| mic_gauge.reading(at_now)),
                system: system_on.then(|| system_gauge.reading(at_now)),
                microphone_muted: want_mic.is_some() && mic_muted,
            };
            let upto = ((now_recorded() - 0.25).max(0.0) * rate) as u64;
            let at = mx.written() as f64 / rate;
            let out = mx.take(upto);
            if !out.is_empty() {
                let _ = w.write_audio(&out, at);
            }
        }
        fx = shared.effects.lock().unwrap().clone();
        let cursor_on = shared.cursor.load(Ordering::Relaxed);
        let wants_clicks = cursor_on && fx.clicks.is_some();
        let wants_keys = fx.keys.is_some();
        if wants_clicks || wants_keys {
            if hooks.as_ref().is_none_or(|h| h.keys != wants_keys) {
                // The old ones go first: they clear where the new ones send.
                drop(hooks.take());
                hooks = crate::win_fx::Hooks::start(wants_keys);
            }
        } else {
            hooks = None;
        }
        if let Some(h) = &hooks {
            while let Ok(input) = h.events.try_recv() {
                match input {
                    crate::win_fx::Input::Button {
                        down,
                        secondary,
                        x,
                        y,
                        at,
                    } if wants_clicks => live.button(down, secondary, x, y, at),
                    crate::win_fx::Input::Key { vk, down, at } => {
                        if let Some(k) = &fx.keys {
                            live.key(vk, down, k, at, &|| focus.shows_keys());
                        }
                    }
                    _ => {}
                }
            }
        }
        // The camera's latest frame, while it shows (VID-031).
        let camera = if shared.camera_on.load(Ordering::Relaxed) {
            shared.camera_feed.lock().unwrap().as_ref().and_then(|f| f())
        } else {
            None
        };
        let filming = camera.is_some();
        conv.camera
            .replace(camera.map(|f| (f, shared.camera_look.lock().unwrap().clone())));
        // Waves, a drag, labels or the camera showing: frames keep coming.
        let animating = live.tick(Instant::now()) || filming;
        if animating {
            dirty = true;
        }
        if shared.paused.load(Ordering::Relaxed) {
            was_paused = true;
            dirty_since = None;
            changed_at = None;
            continue;
        }
        if was_paused {
            // The picture at the moment it resumed.
            was_paused = false;
            dirty = true;
        }
        if dirty && !hold {
            dirty_since.get_or_insert_with(Instant::now);
        } else {
            dirty_since = None;
        }
        if fragmented {
            keeper.note(target, now_recorded());
            // VID-022: into the fragmented file, a frame on every turn (the picture
            // again when nothing changed): its fragments time each frame by its
            // length, so a turn without one would shorten the video. A key frame now
            // and then while something new waits in the open fragment.
            let upto = (now_recorded() / interval).floor() as u64;
            while slot <= upto {
                let fresh = last_frame.is_none() || (slot == upto && dirty && !hold);
                if slot < upto && dirty && !hold {
                    // A change waited past this turn.
                    late_frames += 1;
                }
                let t = slot as f64 * interval;
                let key = keeper.key_due(t);
                match grid_frame(&w, &mut last_frame, fresh, key, t, &fx, &live) {
                    Ok(()) => {
                        frames += 1;
                        slot += 1;
                        keeper.wrote(t, key || frames == 1, fresh);
                        first_frame_at.get_or_insert_with(Instant::now);
                        last_write = Some(Instant::now());
                        if fresh {
                            dirty = false;
                            dirty_since = None;
                        }
                    }
                    Err(e) if w.hardware && frames == 0 => {
                        tracing::warn!(error = %e, "video: hardware encoder failed, software instead");
                        let _ = std::fs::remove_file(target);
                        let bitrate = options.quality.bitrate(size.0, size.1);
                        match writer_in(&gpu, target, size, fps, bitrate, false, audio_rate, true) {
                            Ok(sw) => w = sw,
                            Err(e) => {
                                failure = Some(e);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        failure = Some(e);
                        break;
                    }
                }
            }
            if failure.is_some() {
                break;
            }
            continue;
        }
        let due = last_write.is_none_or(|t| t.elapsed().as_secs_f64() >= interval);
        if dirty && due && !hold {
            // Effects drawn now: the frame is of now.
            let at = changed_at.filter(|_| !animating);
            match write_frame(&w, &mut last_frame, &mut last_time, &mut frames, &fx, &live, at) {
                Ok(()) => {
                    changed_at = None;
                    let now = Instant::now();
                    // The change was due one interval after the last frame, or when
                    // it came if later: every interval past that is a frame lost.
                    if let (Some(since), Some(prev)) = (dirty_since, last_write) {
                        let due_at = since.max(prev + Duration::from_secs_f64(interval));
                        let late = now.saturating_duration_since(due_at).as_secs_f64();
                        late_frames += (late / interval) as u64;
                    }
                    dirty_since = None;
                    first_frame_at.get_or_insert(now);
                    last_write = Some(now);
                    dirty = false;
                }
                Err(e) if w.hardware && frames == 0 => {
                    // The hardware encoder failed on the first frame: software.
                    tracing::warn!(error = %e, "video: hardware encoder failed, software instead");
                    let _ = std::fs::remove_file(target);
                    match writer_in(
                        &gpu,
                        target,
                        size,
                        fps,
                        options.quality.bitrate(size.0, size.1),
                        false,
                        audio_rate,
                        fragmented,
                    ) {
                        Ok(sw) => w = sw,
                        Err(e) => {
                            failure = Some(e);
                            break;
                        }
                    }
                }
                Err(e) => {
                    failure = Some(e);
                    break;
                }
            }
        }
    }

    shared.set_state(State::Finalizing);
    let cancelled = shared.cancel.load(Ordering::Relaxed);
    let duration = now_recorded();
    let forget = |p: &std::path::Path| {
        let _ = std::fs::remove_file(p);
        let _ = std::fs::remove_file(crate::time_file(p));
    };
    if cancelled {
        drop(w);
        forget(target);
        shared.set_state(State::Done);
        return Ok(Summary {
            path: options.path,
            width: size.0,
            height: size.1,
            fps,
            duration,
            frames,
            bytes: 0,
            encoder: "none",
            all_black: false,
            audio: false,
            low_disk: false,
            window_closed: false,
            microphone: false,
            system_audio: false,
            cancelled: true,
            first_frame_at,
            late_frames,
        });
    }
    if failure.is_none() && fragmented {
        // The turns up to the end (VID-022).
        let upto = (duration / interval).floor() as u64;
        while slot <= upto {
            let t = slot as f64 * interval;
            if grid_frame(&w, &mut last_frame, slot == upto && dirty, false, t, &fx, &live).is_err() {
                break;
            }
            frames += 1;
            slot += 1;
        }
    } else if failure.is_none() {
        // The last change, and the last picture held until the end.
        if dirty {
            let _ = write_frame(&w, &mut last_frame, &mut last_time, &mut frames, &fx, &live, None);
        }
        if let Some(f) = &last_frame {
            if duration - last_time > interval {
                let t = (duration - interval).max(last_time + 0.001);
                let _ = w.write(&gpu, f, t, interval);
            }
        }
    }
    // The rest of the sound, up to the end.
    if let (Some(mx), None) = (mixer.as_mut(), &failure) {
        let rate = mx.rate as f64;
        let at = mx.written() as f64 / rate;
        let out = mx.take((duration * rate) as u64);
        if !out.is_empty() {
            let _ = w.write_audio(&out, at);
        }
    }
    let encoder = if w.hardware { "hardware" } else { "software" };
    let finished = if frames > 0 {
        w.finish()
    } else {
        Err("nothing was recorded".into())
    };
    drop(w);
    if let Some(e) = &failure {
        if frames == 0 {
            forget(target);
            shared.set_state(State::Done);
            return Err(e.clone());
        }
        tracing::warn!(error = %e, "video: stopped early");
    }
    // VID-022: the fragmented file becomes the MP4 kept, the same picture and sound
    // without coding them again. If that fails it stays for Recover work.
    let finished = finished.and_then(|()| match options.recovery.as_deref() {
        Some(fragmented_file) => {
            crate::remux(fragmented_file, &options.path).map(|_| forget(fragmented_file))
        }
        None => Ok(()),
    });
    shared.set_state(State::Done);
    finished?;
    let bytes = std::fs::metadata(&options.path).map(|m| m.len()).unwrap_or(0);
    Ok(Summary {
        path: options.path,
        width: size.0,
        height: size.1,
        fps,
        duration,
        frames,
        bytes,
        encoder,
        all_black: false,
        audio: audio_rate.is_some(),
        low_disk,
        window_closed,
        microphone: had_mic,
        system_audio: had_system,
        cancelled: false,
        first_frame_at,
        late_frames,
    })
}

/// What a video file holds, read back with Media Foundation: size, frames and the
/// last frame's time (for tests and `video.info`).
pub fn probe(path: &std::path::Path) -> Result<(u32, u32, u64, f64), String> {
    // SAFETY: Media Foundation reading calls; samples are dropped as read.
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let _ = MFStartup(MF_VERSION, MFSTARTUP_FULL);
        let reader = MFCreateSourceReaderFromURL(&HSTRING::from(path.as_os_str()), None)
            .map_err(|e| err("reading the video", e))?;
        let stream = MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32;
        let t = reader
            .GetCurrentMediaType(stream)
            .map_err(|e| err("video type", e))?;
        let size = t.GetUINT64(&MF_MT_FRAME_SIZE).map_err(|e| err("video size", e))?;
        let (w, h) = ((size >> 32) as u32, size as u32);
        let mut frames = 0u64;
        let mut last = 0i64;
        loop {
            let mut flags = 0u32;
            let mut time = 0i64;
            let mut sample = None;
            reader
                .ReadSample(
                    stream,
                    0,
                    None,
                    Some(&mut flags),
                    Some(&mut time),
                    Some(&mut sample),
                )
                .map_err(|e| err("reading a frame", e))?;
            if sample.is_some() {
                frames += 1;
                last = last.max(time);
            }
            if flags & MF_SOURCE_READERF_ENDOFSTREAM.0 as u32 != 0 {
                break;
            }
        }
        Ok((w, h, frames, last as f64 / HNS))
    }
}

/// A video's size in pixels, reading only its header (the library's index, VID-054).
pub fn size(path: &std::path::Path) -> Result<(u32, u32), String> {
    // SAFETY: Media Foundation reading calls.
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let _ = MFStartup(MF_VERSION, MFSTARTUP_FULL);
        let reader = MFCreateSourceReaderFromURL(&HSTRING::from(path.as_os_str()), None)
            .map_err(|e| err("reading the video", e))?;
        let t = reader
            .GetCurrentMediaType(MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32)
            .map_err(|e| err("video type", e))?;
        let size = t.GetUINT64(&MF_MT_FRAME_SIZE).map_err(|e| err("video size", e))?;
        Ok(((size >> 32) as u32, size as u32))
    }
}

/// The first frame of a video as BGRA pixels (width, height, pixels): for
/// thumbnails, "Save frame" (VID-061) and spotting all-black recordings (VID-029).
pub fn first_frame(path: &std::path::Path) -> Result<(u32, u32, Vec<u8>), String> {
    // SAFETY: Media Foundation reading calls; the buffer is unlocked before returning
    // and the copy stays within it.
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let _ = MFStartup(MF_VERSION, MFSTARTUP_FULL);
        let mut attrs = None;
        MFCreateAttributes(&mut attrs, 1).map_err(|e| err("attributes", e))?;
        let attrs: IMFAttributes = attrs.ok_or("no attributes")?;
        let _ = attrs.SetUINT32(&MF_SOURCE_READER_ENABLE_VIDEO_PROCESSING, 1);
        let reader = MFCreateSourceReaderFromURL(&HSTRING::from(path.as_os_str()), &attrs)
            .map_err(|e| err("reading the video", e))?;
        let stream = MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32;
        let want = MFCreateMediaType().map_err(|e| err("media type", e))?;
        let _ = want.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video);
        let _ = want.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_RGB32);
        reader
            .SetCurrentMediaType(stream, None, &want)
            .map_err(|e| err("video type", e))?;
        let t = reader
            .GetCurrentMediaType(stream)
            .map_err(|e| err("video type", e))?;
        let size = t.GetUINT64(&MF_MT_FRAME_SIZE).map_err(|e| err("video size", e))?;
        let (w, h) = ((size >> 32) as u32, size as u32);
        loop {
            let mut flags = 0u32;
            let mut sample = None;
            reader
                .ReadSample(stream, 0, None, Some(&mut flags), None, Some(&mut sample))
                .map_err(|e| err("reading a frame", e))?;
            if let Some(sample) = sample {
                let buffer = sample.ConvertToContiguousBuffer().map_err(|e| err("frame", e))?;
                let mut data = std::ptr::null_mut();
                let mut len = 0u32;
                buffer
                    .Lock(&mut data, None, Some(&mut len))
                    .map_err(|e| err("frame", e))?;
                let need = (w * h * 4) as usize;
                let pixels = std::slice::from_raw_parts(data, (len as usize).min(need)).to_vec();
                let _ = buffer.Unlock();
                return Ok((w, h, pixels));
            }
            if flags & MF_SOURCE_READERF_ENDOFSTREAM.0 as u32 != 0 {
                return Err("the video has no frames".into());
            }
        }
    }
}

/// What a video holds (VID-064): size, length, frames and their rate, codec, bit
/// rate and sound.
pub fn info(path: &std::path::Path) -> Result<crate::Info, String> {
    let bytes = std::fs::metadata(path).map_err(|e| e.to_string())?.len();
    // SAFETY: Media Foundation reading calls; samples are dropped as read.
    let (width, height, codec, times, end) = unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let _ = MFStartup(MF_VERSION, MFSTARTUP_FULL);
        let reader = MFCreateSourceReaderFromURL(&HSTRING::from(path.as_os_str()), None)
            .map_err(|e| err("reading the video", e))?;
        let stream = MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32;
        let native = reader
            .GetNativeMediaType(stream, 0)
            .map_err(|e| err("video type", e))?;
        let size = native
            .GetUINT64(&MF_MT_FRAME_SIZE)
            .map_err(|e| err("video size", e))?;
        let codec = match native.GetGUID(&MF_MT_SUBTYPE) {
            Ok(g) if g == MFVideoFormat_H264 => "H.264".to_string(),
            Ok(g) if g == MFVideoFormat_HEVC => "H.265".to_string(),
            Ok(g) if g == MFVideoFormat_AV1 => "AV1".to_string(),
            Ok(g) if g == MFVideoFormat_VP90 => "VP9".to_string(),
            Ok(g) if g == MFVideoFormat_MP4V => "MPEG-4".to_string(),
            Ok(g) if g == MFVideoFormat_MJPG => "Motion JPEG".to_string(),
            Ok(g) if g == MFVideoFormat_WMV3 => "WMV".to_string(),
            Ok(g) => format!("{g:?}"),
            Err(_) => String::new(),
        };
        let mut times = Vec::new();
        let mut end = 0i64;
        loop {
            let mut flags = 0u32;
            let mut time = 0i64;
            let mut sample = None;
            reader
                .ReadSample(
                    stream,
                    0,
                    None,
                    Some(&mut flags),
                    Some(&mut time),
                    Some(&mut sample),
                )
                .map_err(|e| err("reading a frame", e))?;
            if let Some(s) = sample {
                times.push(time as f64 / HNS);
                end = end.max(time + s.GetSampleDuration().unwrap_or(0));
            }
            if flags & MF_SOURCE_READERF_ENDOFSTREAM.0 as u32 != 0 {
                break;
            }
        }
        ((size >> 32) as u32, size as u32, codec, times, end as f64 / HNS)
    };
    let audio = audio_info(path)?;
    // Until the last frame ends (a still screen can be seconds of one frame).
    let last = end.max(times.last().copied().unwrap_or(0.0));
    let duration = audio.map(|a| a.2).unwrap_or(0.0).max(last);
    Ok(crate::Info {
        width,
        height,
        duration: (duration * 100.0).round() / 100.0,
        frames: times.len() as u64,
        fps_average: if duration > 0.0 {
            (times.len() as f64 / duration * 10.0).round() / 10.0
        } else {
            0.0
        },
        fps_max: crate::busiest_second(&times),
        codec,
        bitrate: if duration > 0.0 {
            (bytes as f64 * 8.0 / duration) as u64
        } else {
            0
        },
        audio: audio.map(|(c, r, _)| (c, r)),
    })
}

/// The sound of a video: channels, rate and the end of its last sample (seconds),
/// or `None` without sound.
pub fn audio_info(path: &std::path::Path) -> Result<Option<(u32, u32, f64)>, String> {
    // SAFETY: Media Foundation reading calls; samples are dropped as read.
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let _ = MFStartup(MF_VERSION, MFSTARTUP_FULL);
        let reader = MFCreateSourceReaderFromURL(&HSTRING::from(path.as_os_str()), None)
            .map_err(|e| err("reading the video", e))?;
        let stream = MF_SOURCE_READER_FIRST_AUDIO_STREAM.0 as u32;
        let Ok(t) = reader.GetCurrentMediaType(stream) else {
            return Ok(None);
        };
        let channels = t.GetUINT32(&MF_MT_AUDIO_NUM_CHANNELS).unwrap_or(0);
        let rate = t.GetUINT32(&MF_MT_AUDIO_SAMPLES_PER_SECOND).unwrap_or(0);
        let mut end = 0i64;
        loop {
            let mut flags = 0u32;
            let mut time = 0i64;
            let mut sample = None;
            reader
                .ReadSample(
                    stream,
                    0,
                    None,
                    Some(&mut flags),
                    Some(&mut time),
                    Some(&mut sample),
                )
                .map_err(|e| err("reading sound", e))?;
            if let Some(s) = sample {
                end = end.max(time + s.GetSampleDuration().unwrap_or(0));
            }
            if flags & MF_SOURCE_READERF_ENDOFSTREAM.0 as u32 != 0 {
                break;
            }
        }
        Ok(Some((channels, rate, end as f64 / HNS)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MaxSize, Quality, Recorder};

    /// Prints what a video holds (`XS_VIDEO_PROBE=<path>`), for checks by hand.
    #[test]
    #[ignore]
    fn probe_a_file() {
        let Some(path) = std::env::var_os("XS_VIDEO_PROBE") else {
            return;
        };
        let path = std::path::PathBuf::from(path);
        println!("{:?}", probe(&path));
        let (w, h, px) = first_frame(&path).unwrap();
        let at = |x: u32, y: u32| {
            let i = ((y * w + x) * 4) as usize;
            [px[i + 2], px[i + 1], px[i]]
        };
        println!(
            "first frame {w}x{h}: center {:?}, corner {:?}",
            at(w / 2, h / 2),
            at(5, 5)
        );
    }

    #[test]
    #[ignore]
    fn duplication_diagnostics() {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let prev = SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
            println!("dpi context set: {:?}", prev.0);
        }
        let (adapter, output, r) = find_output(Area {
            x: 0,
            y: 0,
            width: 1280,
            height: 720,
        })
        .unwrap();
        println!("monitor {:?}", (r.left, r.top, r.right, r.bottom));
        let gpu = gpu(&adapter).unwrap();
        let dup = unsafe { output.DuplicateOutput(&gpu.device) }.unwrap();
        let d = unsafe { dup.GetDesc() };
        println!(
            "mode {}x{} format {} rotation {} in system memory {}",
            d.ModeDesc.Width,
            d.ModeDesc.Height,
            d.ModeDesc.Format.0,
            d.Rotation.0,
            d.DesktopImageInSystemMemory.as_bool()
        );
        for i in 0..5 {
            unsafe {
                let mut info = DXGI_OUTDUPL_FRAME_INFO::default();
                let mut res = None;
                let r = dup.AcquireNextFrame(200, &mut info, &mut res);
                println!(
                    "{i}: {:?} present {} accumulated {} resource {}",
                    r.as_ref().map_err(|e| format!("0x{:08X}", e.code().0 as u32)),
                    info.LastPresentTime,
                    info.AccumulatedFrames,
                    res.is_some()
                );
                if r.is_ok() {
                    let _ = dup.ReleaseFrame();
                }
            }
        }
    }

    /// Records a test window of known colors (red left, blue right) shown at
    /// (100, 100) logical (150 % here) by hand, and checks them in the decoded video.
    #[test]
    #[ignore]
    fn colors_survive() {
        let dir = std::env::temp_dir().join("xs-video-test");
        let _ = std::fs::create_dir_all(&dir);
        for (software, width, height) in [
            (false, 1280, 720),
            (true, 1280, 720),
            (false, 900, 500),
            (true, 900, 500),
        ] {
            let path = dir.join("color.mp4");
            let r = Recorder::start(Options {
                area: Area {
                    x: 150,
                    y: 150,
                    width,
                    height,
                },
                scale: 1.0,
                quality: Quality::High,
                max_size: MaxSize::Original,
                limit: None,
                software_only: software,
                cursor: false,
                system_audio: false,
                window: None,
                effects: Default::default(),
                path: path.clone(),
                recovery: None,
                microphone: None,
            })
            .unwrap();
            std::thread::sleep(Duration::from_millis(700));
            let s = r.stop().unwrap();
            let (w, _h, px) = first_frame(&path).unwrap();
            let at = |x: u32, y: u32| {
                let i = ((y * w + x) * 4) as usize;
                [px[i + 2], px[i + 1], px[i]]
            };
            println!(
                "{} {:?}: left {:?} right {:?}",
                s.encoder,
                (s.width, s.height),
                at(200, 300),
                at(700, 300)
            );
            let near =
                |a: [u8; 3], b: [u8; 3]| a.iter().zip(b).all(|(x, y)| (*x as i32 - y as i32).abs() <= 12);
            assert!(near(at(200, 300), [200, 40, 40]));
            if width > 700 {
                assert!(near(at(700, 300), [30, 160, 220]));
            }
            let _ = std::fs::remove_file(&path);
        }
    }

    /// The sound's way into the file (VID-026), without a sound device: a 440 Hz
    /// tone written next to a few frames comes back as a 2 s stereo AAC track.
    #[test]
    fn a_tone_becomes_an_aac_track() {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let _ = MFStartup(MF_VERSION, MFSTARTUP_FULL);
        }
        let Ok((adapter, _, _)) = find_output(Area {
            x: 0,
            y: 0,
            width: 64,
            height: 64,
        }) else {
            return; // no monitor (a headless runner)
        };
        // A runner without video hardware or encoders skips it.
        let Ok(gpu) = gpu(&adapter) else { return };
        let dir = tempfile_dir();
        let path = dir.join("tone.mp4");
        let Ok(w) = writer(&gpu, &path, (320, 240), 30, 1_000_000, false, Some(48_000)) else {
            return;
        };
        let frame = texture(
            &gpu,
            320,
            240,
            DXGI_FORMAT_NV12,
            D3D11_BIND_RENDER_TARGET.0 as u32,
            false,
        )
        .unwrap();
        for i in 0..4 {
            w.write(&gpu, &frame, i as f64 * 0.5, 0.5).unwrap();
        }
        let tone: Vec<i16> = (0..96_000)
            .flat_map(|n| {
                let v = ((n as f64 / 48_000.0 * 440.0 * std::f64::consts::TAU).sin() * 8000.0) as i16;
                [v, v]
            })
            .collect();
        w.write_audio(&tone, 0.0).unwrap();
        w.finish().unwrap();
        drop(w);
        let (channels, rate, end) = audio_info(&path).unwrap().expect("a sound track");
        assert_eq!((channels, rate), (2, 48_000));
        assert!((end - 2.0).abs() < 0.1, "{end}");
        let _ = std::fs::remove_file(&path);
    }

    /// The flashes (mean brightness jumps from dark to light) and the beeps (sound
    /// after a second of quiet) in a recording of `scripts/dev/sync-signal.html`,
    /// in seconds.
    fn signals(path: &std::path::Path) -> (Vec<f64>, Vec<f64>) {
        // SAFETY: Media Foundation reading calls; each buffer is unlocked after use
        // and read within its length.
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let _ = MFStartup(MF_VERSION, MFSTARTUP_FULL);
            let reader = MFCreateSourceReaderFromURL(&HSTRING::from(path.as_os_str()), None).unwrap();
            let video = MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32;
            let audio = MF_SOURCE_READER_FIRST_AUDIO_STREAM.0 as u32;
            let want = |major: &windows::core::GUID, sub: &windows::core::GUID| {
                let t = MFCreateMediaType().unwrap();
                t.SetGUID(&MF_MT_MAJOR_TYPE, major).unwrap();
                t.SetGUID(&MF_MT_SUBTYPE, sub).unwrap();
                t
            };
            reader
                .SetCurrentMediaType(video, None, &want(&MFMediaType_Video, &MFVideoFormat_NV12))
                .unwrap();
            let pcm = want(&MFMediaType_Audio, &MFAudioFormat_PCM);
            pcm.SetUINT32(&MF_MT_AUDIO_BITS_PER_SAMPLE, 16).unwrap();
            reader.SetCurrentMediaType(audio, None, &pcm).unwrap();
            let size = reader
                .GetCurrentMediaType(video)
                .unwrap()
                .GetUINT64(&MF_MT_FRAME_SIZE)
                .unwrap();
            let (w, h) = ((size >> 32) as usize, (size & 0xffff_ffff) as usize);
            let at = reader.GetCurrentMediaType(audio).unwrap();
            let rate = at.GetUINT32(&MF_MT_AUDIO_SAMPLES_PER_SECOND).unwrap() as f64;
            let channels = at.GetUINT32(&MF_MT_AUDIO_NUM_CHANNELS).unwrap() as usize;
            let read = |stream: u32, each: &mut dyn FnMut(f64, &[u8])| loop {
                let (mut flags, mut time, mut sample) = (0u32, 0i64, None);
                reader
                    .ReadSample(
                        stream,
                        0,
                        None,
                        Some(&mut flags),
                        Some(&mut time),
                        Some(&mut sample),
                    )
                    .unwrap();
                if let Some(s) = sample {
                    let b = s.ConvertToContiguousBuffer().unwrap();
                    let (mut data, mut len) = (std::ptr::null_mut(), 0u32);
                    b.Lock(&mut data, None, Some(&mut len)).unwrap();
                    each(time as f64 / HNS, std::slice::from_raw_parts(data, len as usize));
                    let _ = b.Unlock();
                }
                if flags & MF_SOURCE_READERF_ENDOFSTREAM.0 as u32 != 0 {
                    break;
                }
            };
            // The picture: the mean of every 8th luma sample.
            let mut flashes = Vec::new();
            let mut dark = true;
            read(video, &mut |t, px| {
                let luma = &px[..(w * h).min(px.len())];
                let n = luma.len() / 8;
                let mean = luma.iter().step_by(8).map(|v| *v as u64).sum::<u64>() / n.max(1) as u64;
                if dark && mean > 128 {
                    flashes.push(t);
                }
                if mean < 64 {
                    dark = true;
                } else if mean > 128 {
                    dark = false;
                }
            });
            // The sound: the first loud sample after a second of quiet.
            let mut beeps = Vec::new();
            let mut quiet_since = 0.0f64;
            let mut last_loud = f64::NEG_INFINITY;
            read(audio, &mut |t, bytes| {
                for (i, frame) in bytes.chunks_exact(2 * channels).enumerate() {
                    let at = t + i as f64 / rate;
                    let loud = frame
                        .as_chunks::<2>()
                        .0
                        .iter()
                        .any(|s| i16::from_le_bytes(*s).unsigned_abs() > 6000);
                    if loud {
                        if at - last_loud > 1.0 && at - quiet_since > 1.0 {
                            beeps.push(at);
                        }
                        last_loud = at;
                    } else if at - last_loud > 0.2 && last_loud > quiet_since {
                        quiet_since = last_loud;
                    }
                }
            });
            (flashes, beeps)
        }
    }

    /// VID-030: the file keeps the picture and the sound where they were written
    /// (no shift from the sound's encoder), and `signals` finds them: flashes at 3,
    /// 13 and 23 s with beeps 20 ms after, on time and 30 ms before.
    #[test]
    fn flashes_and_beeps_stay_in_place() {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let _ = MFStartup(MF_VERSION, MFSTARTUP_FULL);
        }
        let Ok((adapter, _, _)) = find_output(Area {
            x: 0,
            y: 0,
            width: 64,
            height: 64,
        }) else {
            return; // no monitor (a headless runner)
        };
        let Ok(gpu) = gpu(&adapter) else { return };
        let (w, h, fps) = (320u32, 240u32, 30u32);
        let whole = RECT {
            left: 0,
            top: 0,
            right: w as i32,
            bottom: h as i32,
        };
        let conv = converter(&gpu, (w, h), (0, 0), whole, (w, h), fps, 1.0).unwrap();
        let path = tempfile_dir().join("signals.mp4");
        let Ok(writer) = writer(&gpu, &path, (w, h), fps, 1_000_000, false, Some(48_000)) else {
            return;
        };
        let frame_of = |v: u8| {
            let px = vec![v; (w * h * 4) as usize];
            // SAFETY: the pixels are w × h BGRA, rows of w × 4 bytes.
            unsafe {
                gpu.context
                    .UpdateSubresource(&conv.frame, 0, None, px.as_ptr() as _, w * 4, 0);
            }
            conv.nv12(&gpu, false, &Effects::default(), &Live::default())
                .unwrap()
        };
        let (black, white) = (frame_of(0), frame_of(255));
        let interval = 1.0 / fps as f64;
        for i in 0..25 * fps {
            let t = i as f64 * interval;
            let lit = [3.0, 13.0, 23.0].iter().any(|f| t >= *f - 1e-6 && t < f + 0.1);
            writer
                .write(&gpu, if lit { &white } else { &black }, t, interval)
                .unwrap();
        }
        let beeps = [3.02, 13.0, 22.97];
        for second in 0..25 {
            let chunk: Vec<i16> = (0..48_000)
                .flat_map(|n| {
                    let t = second as f64 + n as f64 / 48_000.0;
                    let on = beeps.iter().any(|b| t >= *b && t < b + 0.1);
                    let v = if on {
                        ((t * 1000.0 * std::f64::consts::TAU).sin() * 16_000.0) as i16
                    } else {
                        0
                    };
                    [v, v]
                })
                .collect();
            writer.write_audio(&chunk, second as f64).unwrap();
        }
        writer.finish().unwrap();
        drop(writer);
        let (flashes, found) = signals(&path);
        let _ = std::fs::remove_file(&path);
        assert_eq!(flashes.len(), 3, "{flashes:?}");
        assert_eq!(found.len(), 3, "{found:?}");
        for (f, want) in flashes.iter().zip([3.0, 13.0, 23.0]) {
            assert!((f - want).abs() < 0.002, "flash {f} for {want}");
        }
        for (b, want) in found.iter().zip(beeps) {
            assert!((b - want).abs() < 0.003, "beep {b} for {want}");
        }
    }

    /// VID-030's sync: the gap between each flash and its beep in a recording of
    /// `scripts/dev/sync-signal.html` (`XS_SYNC_FILE=<video>`), at most 40 ms.
    /// `cargo test -p xs-video --release -- --ignored --nocapture sync_check`
    #[test]
    #[ignore]
    fn sync_check() {
        let Some(path) = std::env::var_os("XS_SYNC_FILE") else {
            return;
        };
        let (flashes, beeps) = signals(std::path::Path::new(&path));
        println!("{} flashes, {} beeps", flashes.len(), beeps.len());
        let mut worst: f64 = 0.0;
        let mut paired = 0;
        for f in &flashes {
            let Some(b) = beeps
                .iter()
                .copied()
                .filter(|b| (b - f).abs() < 0.5)
                .min_by(|a, b| (a - f).abs().total_cmp(&(b - f).abs()))
            else {
                println!("{f:8.3} s: flash without a beep");
                continue;
            };
            let gap = (b - f) * 1000.0;
            println!("{f:8.3} s: the sound {gap:+.0} ms after the picture");
            worst = worst.max(gap.abs());
            paired += 1;
        }
        println!("{paired} pairs, worst {worst:.0} ms");
        assert!(paired > 0, "no flash with its beep");
        assert!(worst <= 40.0, "worst gap {worst:.0} ms");
    }

    /// VID-030's start: in 20 recordings of a small area of the screen, each after
    /// half a second of Ready (the engine warming up, as the bar does), the first
    /// frame is written at most 300 ms after asking to record (the engine alone;
    /// `record.stop`'s `startupMs` measures it from the Record button). The videos
    /// are deleted at once.
    /// `cargo test -p xs-video --release -- --ignored --nocapture twenty_starts`
    #[test]
    #[ignore]
    fn twenty_starts() {
        let dir = tempfile_dir();
        let mut times = Vec::new();
        for i in 0..20 {
            let path = dir.join(format!("start-{i}.mp4"));
            let area = Area {
                x: 0,
                y: 0,
                width: 320,
                height: 200,
            };
            warm_up(area, false);
            std::thread::sleep(Duration::from_millis(500));
            let asked = Instant::now();
            let r = Recorder::start(Options {
                area: Area {
                    x: 0,
                    y: 0,
                    width: 320,
                    height: 200,
                },
                scale: 1.0,
                quality: Quality::High,
                max_size: MaxSize::Original,
                limit: None,
                software_only: false,
                cursor: false,
                system_audio: false,
                window: None,
                effects: Default::default(),
                path: path.clone(),
                recovery: None,
                microphone: None,
            })
            .unwrap();
            std::thread::sleep(Duration::from_millis(400));
            let s = r.stop().unwrap();
            let _ = std::fs::remove_file(&path);
            let ms = s.first_frame_at.unwrap().duration_since(asked).as_millis();
            println!(
                "{i:2}: first frame after {ms} ms ({}), {} late",
                s.encoder, s.late_frames
            );
            times.push(ms);
        }
        let worst = *times.iter().max().unwrap();
        println!("worst {worst} ms");
        assert!(worst <= 300, "{times:?}");
    }

    /// The recording side of `a_recording_cut_short_is_recovered`: 3 s of the demo,
    /// then a still screen, written into a fragmented file the way the recorder
    /// does (a key frame when due, the time noted every second) until 12 s, and the
    /// process dies without finishing the file.
    #[test]
    #[ignore]
    fn cut_short_child() {
        let Some(path) = std::env::var_os("XS_CUT_SHORT") else {
            return;
        };
        let path = std::path::PathBuf::from(path);
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let _ = MFStartup(MF_VERSION, MFSTARTUP_FULL);
        }
        let Ok((adapter, _, _)) = find_output(Area {
            x: 0,
            y: 0,
            width: 64,
            height: 64,
        }) else {
            return;
        };
        let Ok(gpu) = gpu(&adapter) else { return };
        let (w, h, fps) = (640u32, 360u32, 30u32);
        let whole = RECT {
            left: 0,
            top: 0,
            right: w as i32,
            bottom: h as i32,
        };
        let conv = converter(&gpu, (w, h), (0, 0), whole, (w, h), fps, 1.0).unwrap();
        let hardware = std::env::var("XS_CUT_SHORT_HW").is_ok_and(|v| v == "1");
        let Ok(writer) = writer_in(&gpu, &path, (w, h), fps, 1_000_000, hardware, None, true) else {
            return;
        };
        let mut demo = crate::demo::Demo::new(w, h);
        let mut keeper = crate::Keeper::default();
        let interval = 1.0 / fps as f64;
        let mut last = None;
        for i in 0..12 * fps {
            let t = i as f64 * interval;
            if i % fps == 0 {
                std::fs::write(crate::time_file(&path), format!("{t:.3}")).unwrap();
            }
            let key = keeper.key_due(t);
            // A frame every turn: the same picture when nothing changed.
            let changed = i < 3 * fps && demo.step(i, fps);
            if changed || last.is_none() {
                unsafe {
                    gpu.context
                        .UpdateSubresource(&conv.frame, 0, None, demo.px.as_ptr() as _, w * 4, 0);
                }
                last = Some(
                    conv.nv12(&gpu, false, &Effects::default(), &Live::default())
                        .unwrap(),
                );
            }
            if key {
                writer.force_key();
            }
            writer.write(&gpu, last.as_ref().unwrap(), t, interval).unwrap();
            keeper.wrote(t, key || i == 0, changed);
        }
        std::thread::sleep(Duration::from_millis(500));
        std::process::abort();
    }

    /// VID-022: a recording whose app died at 12 s, with the screen still since 3 s,
    /// becomes an MP4 of about 12 s (at most 5 s lost), with each encoder.
    #[test]
    fn a_recording_cut_short_is_recovered() {
        let exe = std::env::current_exe().unwrap();
        for hw in ["1", "0"] {
            let path = tempfile_dir().join(format!("cut-short-{hw}.mp4"));
            let _ = std::fs::remove_file(&path);
            let _ = std::process::Command::new(&exe)
                .args([
                    "--ignored",
                    "--exact",
                    "win::tests::cut_short_child",
                    "--test-threads=1",
                ])
                .env("XS_CUT_SHORT", &path)
                .env("XS_CUT_SHORT_HW", hw)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
            if !path.exists() {
                continue; // no monitor or no encoder (a headless runner)
            }
            let length = crate::recovered_length(&path).unwrap();
            assert!((11.0..=12.0).contains(&length), "{hw}: {length}");
            let fixed = path.with_extension("recovered.mp4");
            let end = crate::recover(&path, &fixed).unwrap();
            let (_, _, frames, last) = probe(&fixed).unwrap();
            println!("{hw}: {length:.2} s noted, {end:.2} s recovered, {frames} frames, ends at {last:.2}");
            assert!(end >= 12.0 - 5.0, "{hw}: {end}");
            // The picture of the still screen, held to the end.
            assert!(crate::frame_at(&fixed, end - 0.5).is_ok());
            assert_eq!(crate::facts(&fixed).map(|f| f.duration >= 7.0).ok(), Some(true));
            for p in [&path, &crate::time_file(&path), &fixed] {
                let _ = std::fs::remove_file(p);
            }
        }
    }

    /// VID-022: a recording through its fragmented file (a small area of the screen,
    /// 2 s with a pause) ends as a regular MP4 as long as it was recorded, a frame on
    /// every turn, and leaves nothing in the recovery folder. Deleted at once.
    /// `cargo test -p xs-video --release -- --ignored --nocapture through_a_fragmented_file`
    #[test]
    #[ignore]
    fn through_a_fragmented_file() {
        let dir = tempfile_dir();
        let path = dir.join("kept.mp4");
        let recovery = dir.join("recovery.mp4");
        let r = Recorder::start(Options {
            area: Area {
                x: 0,
                y: 0,
                width: 320,
                height: 200,
            },
            scale: 1.0,
            quality: Quality::High,
            max_size: MaxSize::Original,
            limit: None,
            software_only: false,
            cursor: false,
            system_audio: false,
            window: None,
            effects: Default::default(),
            path: path.clone(),
            recovery: Some(recovery.clone()),
            microphone: None,
        })
        .unwrap();
        std::thread::sleep(Duration::from_millis(1000));
        assert!(recovery.exists() && crate::time_file(&recovery).exists());
        r.pause();
        std::thread::sleep(Duration::from_millis(700));
        r.resume();
        std::thread::sleep(Duration::from_millis(1000));
        let s = r.stop().unwrap();
        let (_, _, frames, last) = probe(&path).unwrap();
        println!(
            "{} s, {} frames written, {frames} read, last at {last:.3}, {} late",
            s.duration, s.frames, s.late_frames
        );
        let _ = std::fs::remove_file(&path);
        assert!(!recovery.exists() && !crate::time_file(&recovery).exists());
        assert!((s.duration - 2.0).abs() < 0.2, "{}", s.duration);
        assert!((last - s.duration).abs() < 0.1, "{last}");
        assert_eq!(frames, s.frames);
    }

    /// VID-030's size: the reference minute (an interface demo drawn in memory, no
    /// screen) at 1080p30 in High takes at most 20 MB, with each encoder there is.
    /// `cargo test -p xs-video --release -- --ignored --nocapture reference_minute`
    #[test]
    #[ignore]
    fn reference_minute() {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let _ = MFStartup(MF_VERSION, MFSTARTUP_FULL);
        }
        let Ok((adapter, _, _)) = find_output(Area {
            x: 0,
            y: 0,
            width: 64,
            height: 64,
        }) else {
            return;
        };
        let Ok(gpu) = gpu(&adapter) else { return };
        let (w, h) = (1920u32, 1080u32);
        let quality = Quality::High;
        let fps = quality.fps();
        let interval = 1.0 / fps as f64;
        let whole = RECT {
            left: 0,
            top: 0,
            right: w as i32,
            bottom: h as i32,
        };
        let conv = converter(&gpu, (w, h), (0, 0), whole, (w, h), fps, 1.0).unwrap();
        let dir = tempfile_dir();
        let mut measured = 0;
        // Each encoder, into a regular file and into a fragmented one with a key
        // frame every 3 s while something changes (VID-022), then made regular.
        for (hardware, fragmented) in [(true, false), (true, true), (false, false), (false, true)] {
            let name = match (hardware, fragmented) {
                (true, false) => "hardware",
                (true, true) => "hardware, fragmented",
                (false, false) => "software",
                (false, true) => "software, fragmented",
            };
            let path = dir.join(format!("reference-{}.mp4", name.replace(", ", "-")));
            let _ = std::fs::remove_file(&path);
            let Ok(writer) = writer_in(
                &gpu,
                &path,
                (w, h),
                fps,
                quality.bitrate(w, h),
                hardware,
                None,
                fragmented,
            ) else {
                println!("{name}: no encoder");
                continue;
            };
            let mut demo = crate::demo::Demo::new(w, h);
            let mut keeper = crate::Keeper::default();
            let started = Instant::now();
            let mut frames = 0;
            let mut last = None;
            for i in 0..60 * fps {
                let t = i as f64 * interval;
                let key = fragmented && keeper.key_due(t);
                let new = demo.step(i, fps);
                // Fragmented: a frame every turn, the same picture when nothing changed.
                if !new && !fragmented {
                    continue;
                }
                if key {
                    writer.force_key();
                }
                let frame = if new || last.is_none() {
                    // SAFETY: the pixels are w × h BGRA, rows of w × 4 bytes.
                    unsafe {
                        gpu.context
                            .UpdateSubresource(&conv.frame, 0, None, demo.px.as_ptr() as _, w * 4, 0);
                    }
                    conv.nv12(&gpu, false, &Effects::default(), &Live::default())
                        .unwrap()
                } else {
                    last.clone().unwrap()
                };
                writer.write(&gpu, &frame, t, interval).unwrap();
                keeper.wrote(t, key || frames == 0, new);
                last = Some(frame);
                frames += 1;
            }
            // The last picture held to the minute's end, like the recorder.
            if let (Some(f), false) = (&last, fragmented) {
                writer.write(&gpu, f, 60.0 - interval, interval).unwrap();
                frames += 1;
            }
            writer.finish().unwrap();
            drop(writer);
            let path = if fragmented {
                let regular = path.with_extension("regular.mp4");
                crate::remux(&path, &regular).unwrap();
                let _ = std::fs::remove_file(&path);
                regular
            } else {
                path
            };
            let bytes = std::fs::metadata(&path).unwrap().len();
            let (_, _, read, end) = probe(&path).unwrap();
            assert_eq!(read, frames, "{name}: every frame is in the file");
            assert!((end - (60.0 - interval)).abs() < 0.01, "{name}: ends at {end}");
            println!(
                "{name}: {frames} frames, {:.1} s, {:.2} MB, encoded in {:.1} s",
                end,
                bytes as f64 / 1e6,
                started.elapsed().as_secs_f64()
            );
            let _ = std::fs::remove_file(&path);
            assert!(bytes <= 20_000_000, "{name}: {bytes} bytes");
            measured += 1;
        }
        assert!(measured > 0, "no encoder at all");
    }

    /// The effects drawn on a grey picture in memory (no screen, no input): the
    /// highlight's colour under the pointer, a wave, and a dark key label.
    #[test]
    fn effects_are_drawn() {
        use crate::effects::*;
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        }
        let Ok((adapter, _, _)) = find_output(Area {
            x: 0,
            y: 0,
            width: 64,
            height: 64,
        }) else {
            return;
        };
        let Ok(gpu) = gpu(&adapter) else { return };
        let (w, h) = (400u32, 300u32);
        let desc = D3D11_TEXTURE2D_DESC {
            Width: w,
            Height: h,
            MipLevels: 1,
            ArraySize: 1,
            Format: DXGI_FORMAT_B8G8R8A8_UNORM,
            SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: (D3D11_BIND_RENDER_TARGET.0 | D3D11_BIND_SHADER_RESOURCE.0) as u32,
            CPUAccessFlags: 0,
            MiscFlags: D3D11_RESOURCE_MISC_GDI_COMPATIBLE.0 as u32,
        };
        let grey = vec![128u8; (w * h * 4) as usize];
        let data = D3D11_SUBRESOURCE_DATA {
            pSysMem: grey.as_ptr() as *const _,
            SysMemPitch: w * 4,
            SysMemSlicePitch: 0,
        };
        let mut tex = None;
        unsafe { gpu.device.CreateTexture2D(&desc, Some(&data), Some(&mut tex)) }.unwrap();
        let tex = tex.unwrap();
        let Ok(painter) = crate::win_fx::Painter::new(&tex) else {
            return;
        };
        let fx = Effects {
            highlight: Some(Highlight {
                color: [255, 0, 0],
                diameter: 60.0,
                opacity: 0.5,
                ring: false,
            }),
            clicks: Some(Clicks {
                primary: [0, 0, 255],
                secondary: [255, 128, 0],
                diameter: 64.0,
            }),
            keys: Some(Keys {
                all: true,
                place: KeyPlace::Center,
                size: 18.0,
                hidden: Vec::new(),
            }),
        };
        let now = std::time::Instant::now();
        let mut live = Live::default();
        live.button(true, false, 300, 80, now - Duration::from_millis(100));
        live.labels.push_back(("Ctrl + C".into(), now + LABEL));
        painter.draw(
            &fx,
            &live,
            Some((100.0, 100.0)),
            (0.0, 0.0, w as f32, h as f32),
            (0.0, 0.0),
            1.0,
            now,
        );
        // Read it back.
        let staging = texture(&gpu, w, h, DXGI_FORMAT_B8G8R8A8_UNORM, 0, true).unwrap();
        let px = unsafe {
            gpu.context.CopyResource(&staging, &tex);
            let mut m = D3D11_MAPPED_SUBRESOURCE::default();
            gpu.context
                .Map(&staging, 0, D3D11_MAP_READ, 0, Some(&mut m))
                .unwrap();
            let pitch = m.RowPitch as usize;
            let src = std::slice::from_raw_parts(m.pData as *const u8, pitch * h as usize).to_vec();
            gpu.context.Unmap(&staging, 0);
            move |x: usize, y: usize| {
                let i = y * pitch + x * 4;
                [src[i + 2], src[i + 1], src[i]]
            }
        };
        // To look at it: XS_FX_RAW=<file> keeps the picture (RGB, 400 × 300).
        if let Some(out) = std::env::var_os("XS_FX_RAW") {
            let mut rgb = Vec::with_capacity((w * h * 3) as usize);
            for y in 0..h as usize {
                for x in 0..w as usize {
                    rgb.extend_from_slice(&px(x, y));
                }
            }
            let _ = std::fs::write(out, rgb);
        }
        let centre = px(100, 100);
        println!(
            "highlight {centre:?}, outside {:?}, label {:?}",
            px(100, 140),
            px(200, 260)
        );
        assert!(centre[0] > 180 && centre[1] < 90, "red over grey: {centre:?}");
        assert_eq!(px(100, 140), [128, 128, 128], "outside the circle");
        let label = px(200, 262);
        assert!(label[0] < 60 && label[1] < 60, "the label's dark box: {label:?}");
        // The wave: some blue around (300, 80).
        let blue = (60..100).any(|y| {
            (270..330).any(|x| {
                let p = px(x, y);
                p[2] > p[0] + 40
            })
        });
        assert!(blue, "the click's wave");
    }

    /// The camera drawn on a grey picture in memory (no screen, no camera): in its
    /// corner with the margin, mirrored, in its shape, with its border (VID-032 to
    /// VID-036); and filling everything at full screen (VID-034).
    #[test]
    fn the_camera_is_drawn() {
        use crate::camlook::*;
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        }
        let Ok((adapter, _, _)) = find_output(Area {
            x: 0,
            y: 0,
            width: 64,
            height: 64,
        }) else {
            return;
        };
        let Ok(gpu) = gpu(&adapter) else { return };
        let (w, h) = (400u32, 300u32);
        let desc = D3D11_TEXTURE2D_DESC {
            Width: w,
            Height: h,
            MipLevels: 1,
            ArraySize: 1,
            Format: DXGI_FORMAT_B8G8R8A8_UNORM,
            SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: (D3D11_BIND_RENDER_TARGET.0 | D3D11_BIND_SHADER_RESOURCE.0) as u32,
            CPUAccessFlags: 0,
            MiscFlags: D3D11_RESOURCE_MISC_GDI_COMPATIBLE.0 as u32,
        };
        // A camera picture: its left half red, its right half blue (BGRA).
        let (cw, ch) = (160u32, 90u32);
        let mut cam = Vec::with_capacity((cw * ch * 4) as usize);
        for _ in 0..ch {
            for x in 0..cw {
                cam.extend_from_slice(if x < cw / 2 {
                    &[0, 0, 255, 255]
                } else {
                    &[255, 0, 0, 255]
                });
            }
        }
        let frame = Frame {
            width: cw,
            height: ch,
            bgra: cam,
            at: std::time::Instant::now(),
        };
        let draw = |look: &Look| {
            let grey = vec![128u8; (w * h * 4) as usize];
            let data = D3D11_SUBRESOURCE_DATA {
                pSysMem: grey.as_ptr() as *const _,
                SysMemPitch: w * 4,
                SysMemSlicePitch: 0,
            };
            let mut tex = None;
            unsafe { gpu.device.CreateTexture2D(&desc, Some(&data), Some(&mut tex)) }.unwrap();
            let tex = tex.unwrap();
            let painter = crate::win_fx::Painter::new(&tex).unwrap();
            painter.draw_camera(&mut None, &frame, look, (0.0, 0.0, w as f32, h as f32), 1.0);
            let staging = texture(&gpu, w, h, DXGI_FORMAT_B8G8R8A8_UNORM, 0, true).unwrap();
            unsafe {
                gpu.context.CopyResource(&staging, &tex);
                let mut m = D3D11_MAPPED_SUBRESOURCE::default();
                gpu.context
                    .Map(&staging, 0, D3D11_MAP_READ, 0, Some(&mut m))
                    .unwrap();
                let pitch = m.RowPitch as usize;
                let src = std::slice::from_raw_parts(m.pData as *const u8, pitch * h as usize).to_vec();
                gpu.context.Unmap(&staging, 0);
                move |x: usize, y: usize| {
                    let i = y * pitch + x * 4;
                    [src[i + 2], src[i + 1], src[i]]
                }
            }
        };
        let red = |p: [u8; 3]| p[0] > 200 && p[2] < 60;
        let blue = |p: [u8; 3]| p[2] > 200 && p[0] < 60;
        // 16:9, a quarter of 300 high, bottom right with 24 px of margin: from
        // (242.7, 201) to (376, 276). Mirrored: blue on the left, red on the right.
        let px = draw(&Look::default());
        assert!(blue(px(260, 240)), "{:?}", px(260, 240));
        assert!(red(px(360, 240)), "{:?}", px(360, 240));
        assert_eq!(px(230, 240), [128, 128, 128], "the margin");
        assert_eq!(px(300, 190), [128, 128, 128], "above it");
        // Without the mirror, as the camera gives it.
        let px = draw(&Look {
            mirror: false,
            ..Look::default()
        });
        assert!(red(px(260, 240)) && blue(px(360, 240)));
        // A circle with an 8 px white border: the box's corner stays grey; a ring
        // round it.
        let px = draw(&Look {
            shape: Shape::Circle,
            border: Border::Solid,
            border_colors: ([255, 255, 255], [0, 0, 0]),
            border_width: 8.0,
            ..Look::default()
        });
        // A circle of 75 px, its centre at (338.5, 238.5).
        assert_eq!(px(303, 203), [128, 128, 128], "outside the circle");
        let ring = px(338, 238 - 37 - 4);
        assert!(ring.iter().all(|c| *c > 230), "the ring: {ring:?}");
        // Full screen: the camera everywhere, cut to fill 4:3 from 16:9.
        let px = draw(&Look {
            placement: Placement::Fullscreen,
            mirror: false,
            ..Look::default()
        });
        assert!(red(px(5, 5)) && blue(px(395, 295)));
    }

    fn tempfile_dir() -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("xs-video-unit-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&d);
        d
    }

    /// Follows the test window of `colors_survive` (shown by hand) while it moves and
    /// resizes, and stops when it closes (VID-005).
    #[test]
    #[ignore]
    fn follows_a_window() {
        use windows::core::w;
        use windows::Win32::UI::WindowsAndMessaging::*;
        unsafe {
            let _ = SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        }
        let hwnd = unsafe { FindWindowW(None, w!("XS color test")) }.expect("the test window");
        let r = window_bounds(hwnd.0 as isize).unwrap();
        let dir = tempfile_dir();
        let path = dir.join("window.mp4");
        let rec = Recorder::start(Options {
            area: Area {
                x: r.left,
                y: r.top,
                width: (r.right - r.left) as u32,
                height: (r.bottom - r.top) as u32,
            },
            scale: 1.0,
            quality: Quality::High,
            max_size: MaxSize::Original,
            limit: None,
            software_only: false,
            cursor: false,
            system_audio: false,
            window: Some(hwnd.0 as isize),
            effects: Default::default(),
            path: path.clone(),
            recovery: None,
            microphone: None,
        })
        .unwrap();
        std::thread::sleep(Duration::from_millis(500));
        unsafe {
            let _ = SetWindowPos(
                hwnd,
                None,
                r.left + 300,
                r.top + 120,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER,
            );
        }
        std::thread::sleep(Duration::from_millis(500));
        unsafe {
            let _ = SetWindowPos(hwnd, None, 0, 0, 500, 500, SWP_NOMOVE | SWP_NOZORDER);
        }
        std::thread::sleep(Duration::from_millis(500));
        unsafe {
            let _ = PostMessageW(
                Some(hwnd),
                WM_CLOSE,
                windows::Win32::Foundation::WPARAM(0),
                windows::Win32::Foundation::LPARAM(0),
            );
        }
        std::thread::sleep(Duration::from_millis(800));
        assert!(rec.finished(), "it stopped when the window closed");
        let s = rec.stop().unwrap();
        println!("{s:?}");
        assert!(s.window_closed);
        assert!(s.frames >= 2);
        let (w, h, px) = first_frame(&path).unwrap();
        let i = (((h / 2) * w + w / 4) * 4) as usize;
        println!(
            "first frame {w}x{h}: left middle {:?}",
            [px[i + 2], px[i + 1], px[i]]
        );
        let _ = std::fs::remove_file(&path);
    }

    /// With the system's sound on, the video has a stereo AAC track as long as the
    /// picture (silence when nothing plays).
    #[test]
    #[ignore]
    fn system_audio_track() {
        let dir = std::env::temp_dir().join("xs-video-test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("audio.mp4");
        let r = Recorder::start(Options {
            area: Area {
                x: 0,
                y: 0,
                width: 640,
                height: 360,
            },
            scale: 1.0,
            quality: Quality::High,
            max_size: MaxSize::Original,
            limit: None,
            software_only: false,
            cursor: false,
            system_audio: true,
            window: None,
            effects: Default::default(),
            path: path.clone(),
            recovery: None,
            microphone: None,
        })
        .unwrap();
        std::thread::sleep(Duration::from_millis(2000));
        let s = r.stop().unwrap();
        let audio = audio_info(&path).unwrap();
        println!("{} s, audio {:?}", s.duration, audio);
        if crate::audio::Loopback::open().is_err() {
            // No sound output on this computer now: the video is silent, as it should.
            assert!(!s.audio && audio.is_none());
            return;
        }
        assert!(s.audio);
        let (channels, rate, end) = audio.expect("a sound track");
        assert_eq!(channels, 2);
        assert!(rate == 48_000 || rate == 44_100);
        assert!(
            (end - s.duration).abs() < 0.3,
            "sound {end} s, picture {} s",
            s.duration
        );
        let _ = std::fs::remove_file(&path);
    }

    /// The pointer is drawn on the frames: put over the test window of
    /// `colors_survive` (red), it leaves dark or white pixels where it is.
    #[test]
    #[ignore]
    fn cursor_is_drawn() {
        use windows::Win32::Foundation::POINT;
        use windows::Win32::UI::WindowsAndMessaging::{GetCursorPos, SetCursorPos};
        unsafe {
            let _ = SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        }
        let mut before = POINT::default();
        unsafe {
            let _ = GetCursorPos(&mut before);
            let _ = SetCursorPos(400, 400);
        }
        let dir = std::env::temp_dir().join("xs-video-test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("cursor.mp4");
        let r = Recorder::start(Options {
            area: Area {
                x: 150,
                y: 150,
                width: 1280,
                height: 720,
            },
            scale: 1.0,
            quality: Quality::High,
            max_size: MaxSize::Original,
            limit: None,
            software_only: false,
            cursor: true,
            system_audio: false,
            window: None,
            effects: Default::default(),
            path: path.clone(),
            recovery: None,
            microphone: None,
        })
        .unwrap();
        std::thread::sleep(Duration::from_millis(600));
        let _ = r.stop().unwrap();
        unsafe {
            let _ = SetCursorPos(before.x, before.y);
        }
        let (w, _h, px) = first_frame(&path).unwrap();
        let at = |x: u32, y: u32| {
            let i = ((y * w + x) * 4) as usize;
            [px[i + 2], px[i + 1], px[i]]
        };
        // The arrow's tip is at (250, 250) of the area; look just below and right.
        let marked = (0..20)
            .flat_map(|dy| (0..12).map(move |dx| (250 + dx, 250 + dy)))
            .filter(|&(x, y)| {
                let p = at(x, y);
                let dark = p.iter().all(|c| *c < 60);
                let white = p.iter().all(|c| *c > 200);
                dark || white
            })
            .count();
        println!("pointer pixels: {marked}, far away: {:?}", at(100, 600));
        assert!(marked > 10, "the pointer is in the frame");
        let _ = std::fs::remove_file(&path);
    }

    /// VID-045 in the real engine: a red highlight around the pointer wherever it is
    /// (the mouse is not moved). It records the real screen: run it by hand.
    #[test]
    #[ignore]
    fn highlight_is_recorded() {
        use crate::effects::{Effects, Highlight};
        use windows::Win32::Foundation::POINT;
        use windows::Win32::Graphics::Gdi::{
            GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST,
        };
        use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
        unsafe {
            let _ = SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        }
        let mut at = POINT::default();
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        unsafe {
            let _ = GetCursorPos(&mut at);
            let _ = GetMonitorInfoW(MonitorFromPoint(at, MONITOR_DEFAULTTONEAREST), &mut info);
        }
        let m = info.rcMonitor;
        let (w, h) = (400, 300);
        let x = (at.x - w / 2).clamp(m.left, m.right - w);
        let y = (at.y - h / 2).clamp(m.top, m.bottom - h);
        let dir = std::env::temp_dir().join("xs-video-test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("highlight.mp4");
        let r = Recorder::start(Options {
            area: Area {
                x,
                y,
                width: w as u32,
                height: h as u32,
            },
            scale: 1.0,
            quality: Quality::High,
            max_size: MaxSize::Original,
            limit: None,
            software_only: false,
            cursor: true,
            system_audio: false,
            window: None,
            effects: Effects {
                highlight: Some(Highlight {
                    color: [255, 0, 0],
                    diameter: 80.0,
                    opacity: 0.9,
                    ring: false,
                }),
                ..Default::default()
            },
            path: path.clone(),
            recovery: None,
            microphone: None,
        })
        .unwrap();
        std::thread::sleep(Duration::from_millis(600));
        let _ = r.stop().unwrap();
        let (fw, _fh, px) = first_frame(&path).unwrap();
        let pixel = |dx: i32, dy: i32| {
            let (px_x, px_y) = ((at.x - x + dx) as u32, (at.y - y + dy) as u32);
            let i = ((px_y * fw + px_x) * 4) as usize;
            [px[i + 2], px[i + 1], px[i]]
        };
        // Left of and above the tip: inside the circle, outside the arrow.
        let around = [pixel(-30, 0), pixel(0, -30), pixel(-20, -20)];
        println!("pointer at {at:?}, around the tip: {around:?}");
        for p in around {
            assert!(p[0] > 150 && p[1] < 100 && p[2] < 100, "red highlight: {p:?}");
        }
        let _ = std::fs::remove_file(&path);
    }

    /// Records two seconds of the primary monitor's top-left corner and reads the file
    /// back. It records the real screen: run it by hand (`--ignored`).
    #[test]
    #[ignore]
    fn records_and_reads_back() {
        let dir = std::env::temp_dir().join("xs-video-test");
        let _ = std::fs::create_dir_all(&dir);
        for software in [false, true] {
            let path = dir.join(if software { "sw.mp4" } else { "hw.mp4" });
            let r = Recorder::start(Options {
                area: Area {
                    x: 0,
                    y: 0,
                    width: 1280,
                    height: 720,
                },
                scale: 1.0,
                quality: Quality::High,
                max_size: MaxSize::Original,
                limit: Some(Duration::from_secs(2)),
                software_only: software,
                cursor: false,
                system_audio: false,
                window: None,
                effects: Default::default(),
                path: path.clone(),
                recovery: None,
                microphone: None,
            })
            .unwrap();
            std::thread::sleep(Duration::from_millis(2500));
            let s = r.stop().unwrap();
            println!("{s:?}");
            assert!(s.frames >= 1 && s.bytes > 0);
            let (w, h, frames, last) = probe(&path).unwrap();
            println!("read back: {w}x{h}, {frames} frames, last at {last:.2}s");
            assert_eq!((w, h), (1280, 720));
            assert!(frames >= 1);
            assert!((1.5..2.6).contains(&last), "{last}");
            let (fw, fh, px) = first_frame(&path).unwrap();
            let n = (px.len() / 4).max(1) as f64;
            let mean = px
                .chunks(4)
                .map(|p| p[0] as f64 + p[1] as f64 + p[2] as f64)
                .sum::<f64>()
                / n
                / 3.0;
            let distinct: std::collections::HashSet<[u8; 3]> = px
                .chunks(4)
                .step_by(97)
                .map(|p| [p[0] / 16, p[1] / 16, p[2] / 16])
                .collect();
            println!(
                "first frame {fw}x{fh}: mean {mean:.1}, {} distinct colors",
                distinct.len()
            );
            assert!(mean > 8.0, "not black");
            let _ = std::fs::remove_file(&path);
        }
    }
}
