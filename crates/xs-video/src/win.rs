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
    // SAFETY: Media Foundation calls with arguments that live through each call.
    unsafe {
        let mut attrs = None;
        MFCreateAttributes(&mut attrs, 4).map_err(|e| err("attributes", e))?;
        let attrs: IMFAttributes = attrs.ok_or("no attributes")?;
        let set = |r: windows::core::Result<()>| r.map_err(|e| err("attributes", e));
        set(attrs.SetUINT32(&MF_SINK_WRITER_DISABLE_THROTTLING, 1))?;
        set(attrs.SetGUID(&MF_TRANSCODE_CONTAINERTYPE, &MFTranscodeContainerType_MPEG4))?;
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
            staging,
            width: size.0,
            height: size.1,
        })
    }
}

impl Writer {
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
fn snapshot(gpu: &Gpu, conv: &Converter, monitor: RECT) -> Result<(), String> {
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
                if copied.is_ok() {
                    // GDI leaves alpha at 0: the converter ignores it.
                    gpu.context
                        .UpdateSubresource(&conv.frame, 0, None, bits, (w * 4) as u32, 0);
                }
                SelectObject(mem, old);
                let _ = DeleteObject(dib.into());
                copied.map_err(|e| err("screen picture", e))
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
    let setup = (|| {
        let (adapter, output, mr) = find_output(options.area)?;
        let gpu = gpu(&adapter)?;
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
        let audio_rate = loopback.as_ref().map(|l| l.rate);
        let _ = std::fs::remove_file(&options.path);
        let w = if options.software_only {
            writer(&gpu, &options.path, size, fps, bitrate, false, audio_rate)?
        } else {
            match writer(&gpu, &options.path, size, fps, bitrate, true, audio_rate) {
                Ok(w) => w,
                Err(e) => {
                    tracing::info!(error = %e, "video: no hardware encoder, software instead");
                    let _ = std::fs::remove_file(&options.path);
                    writer(&gpu, &options.path, size, fps, bitrate, false, audio_rate)?
                }
            }
        };
        // The first picture, whatever Desktop Duplication does.
        snapshot(&gpu, &conv, mr)?;
        Ok((gpu, output, dup, conv, w, loopback, audio_rate))
    })();
    let (gpu, output, mut dup, conv, mut w, mut loopback, audio_rate) = match setup {
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
    // Sound frames written so far, and when a lost output was last looked for.
    let mut audio_frames = 0u64;
    let mut last_reopen = Instant::now();
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
    let now_recorded = || {
        shared
            .clock
            .lock()
            .unwrap()
            .recorded(Instant::now())
            .as_secs_f64()
    };

    // Writes the current picture at the recorded time. A hardware encoder that fails
    // on the first frame is replaced by software; one that fails later stops the
    // recording with what was written (continuing in software: still to do, VID-029).
    let write_frame = |w: &Writer,
                       last_frame: &mut Option<ID3D11Texture2D>,
                       last_time: &mut f64,
                       frames: &mut u64,
                       fx: &Effects,
                       live: &Live|
     -> Result<(), String> {
        let frame = conv.nv12(&gpu, shared.cursor.load(Ordering::Relaxed), fx, live)?;
        let t = now_recorded().max(*last_time + 0.001);
        w.write(&gpu, &frame, t, interval)?;
        *last_time = t;
        *last_frame = Some(frame);
        *frames += 1;
        Ok(())
    };

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
            if free_space(&options.path).is_some_and(crate::low_disk) {
                tracing::warn!("video: the disk is almost full, stopping");
                low_disk = true;
                break;
            }
        }
        let due_in = match last_write {
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
            Ok(true) => dirty = true,
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
        // The sound (VID-040): what played, placed at the moment it was heard, with
        // silence before it for the time nothing played.
        if let Some(l) = &loopback {
            match l.read() {
                Ok(samples) if shared.paused.load(Ordering::Relaxed) => drop(samples),
                Ok(samples) => {
                    let rate = l.rate as f64;
                    let now = now_recorded();
                    let length = samples.len() as f64 / 2.0 / rate;
                    // Without sound for a while, silence up to a moment ago.
                    let target = if samples.is_empty() {
                        now - 0.3
                    } else {
                        now - length
                    };
                    let written = audio_frames as f64 / rate;
                    if target - written > 0.02 {
                        let gap = ((target - written) * rate) as usize;
                        let silence = vec![0i16; gap * 2];
                        if w.write_audio(&silence, written).is_ok() {
                            audio_frames += gap as u64;
                        }
                    }
                    if !samples.is_empty() {
                        let at = audio_frames as f64 / rate;
                        if w.write_audio(&samples, at).is_ok() {
                            audio_frames += (samples.len() / 2) as u64;
                        }
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
        } else if audio_rate.is_some() && last_reopen.elapsed() > Duration::from_secs(1) {
            last_reopen = Instant::now();
            loopback = crate::audio::Loopback::open()
                .ok()
                .filter(|n| Some(n.rate) == audio_rate);
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
        // Waves, a drag or labels showing: frames keep coming.
        if live.tick(Instant::now()) {
            dirty = true;
        }
        if shared.paused.load(Ordering::Relaxed) {
            was_paused = true;
            continue;
        }
        if was_paused {
            // The picture at the moment it resumed.
            was_paused = false;
            dirty = true;
        }
        let due = last_write.is_none_or(|t| t.elapsed().as_secs_f64() >= interval);
        if dirty && due && !hold {
            match write_frame(&w, &mut last_frame, &mut last_time, &mut frames, &fx, &live) {
                Ok(()) => {
                    last_write = Some(Instant::now());
                    dirty = false;
                }
                Err(e) if w.hardware && frames == 0 => {
                    // The hardware encoder failed on the first frame: software.
                    tracing::warn!(error = %e, "video: hardware encoder failed, software instead");
                    let _ = std::fs::remove_file(&options.path);
                    match writer(
                        &gpu,
                        &options.path,
                        size,
                        fps,
                        options.quality.bitrate(size.0, size.1),
                        false,
                        audio_rate,
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
    if cancelled {
        drop(w);
        let _ = std::fs::remove_file(&options.path);
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
            cancelled: true,
        });
    }
    if failure.is_none() {
        // The last change, and the last picture held until the end.
        if dirty {
            let _ = write_frame(&w, &mut last_frame, &mut last_time, &mut frames, &fx, &live);
        }
        if let Some(f) = &last_frame {
            if duration - last_time > interval {
                let t = (duration - interval).max(last_time + 0.001);
                let _ = w.write(&gpu, f, t, interval);
            }
        }
    }
    if let (Some(rate), None) = (audio_rate, &failure) {
        let rate = rate as f64;
        let written = audio_frames as f64 / rate;
        if duration > written {
            let gap = ((duration - written) * rate) as usize;
            let _ = w.write_audio(&vec![0i16; gap * 2], written);
        }
    }
    let encoder = if w.hardware { "hardware" } else { "software" };
    let finished = if frames > 0 {
        w.finish()
    } else {
        Err("nothing was recorded".into())
    };
    drop(w);
    shared.set_state(State::Done);
    if let Some(e) = failure {
        if frames == 0 {
            let _ = std::fs::remove_file(&options.path);
            return Err(e);
        }
        tracing::warn!(error = %e, "video: stopped early");
    }
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
        cancelled: false,
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
