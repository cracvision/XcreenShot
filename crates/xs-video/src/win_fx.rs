//! Drawing the cursor and keyboard effects on Windows (VID-045 to VID-047): Direct2D
//! and DirectWrite on the frame's texture, low-level mouse and keyboard hooks that
//! only watch (they never block or delay the input), and UI Automation to keep
//! password fields out.

use std::sync::mpsc;
use std::sync::Mutex;
use std::time::Instant;

use windows::core::{w, Interface};
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Direct2D::Common::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::Direct3D11::ID3D11Texture2D;
use windows::Win32::Graphics::DirectWrite::*;
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM;
use windows::Win32::Graphics::Dxgi::IDXGISurface;
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
use windows::Win32::UI::Accessibility::{CUIAutomation, IUIAutomation};
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::camlook::{cover, Border, Frame, Look};
use crate::effects::{wave_progress, Effects, KeyPlace, Live, Rgb, WAVE};

fn color(c: Rgb, a: f32) -> D2D1_COLOR_F {
    D2D1_COLOR_F {
        r: c[0] as f32 / 255.0,
        g: c[1] as f32 / 255.0,
        b: c[2] as f32 / 255.0,
        a,
    }
}

fn ellipse(x: f32, y: f32, r: f32) -> D2D1_ELLIPSE {
    D2D1_ELLIPSE {
        point: windows_numerics::Vector2 { X: x, Y: y },
        radiusX: r,
        radiusY: r,
    }
}

/// Draws the effects on a frame texture.
pub struct Painter {
    target: ID2D1RenderTarget,
    brush: ID2D1SolidColorBrush,
    write: IDWriteFactory,
    /// Text format, for the size it was made with.
    text: Mutex<Option<(f32, IDWriteTextFormat)>>,
}

impl Painter {
    pub fn new(texture: &ID3D11Texture2D) -> Result<Painter, String> {
        let e = |c: &str, e: windows::core::Error| format!("{c}: {}", e.message());
        // SAFETY: Direct2D and DirectWrite creation on this thread's texture.
        unsafe {
            let factory: ID2D1Factory =
                D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None).map_err(|x| e("Direct2D", x))?;
            let surface: IDXGISurface = texture.cast().map_err(|x| e("surface", x))?;
            let props = D2D1_RENDER_TARGET_PROPERTIES {
                r#type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: DXGI_FORMAT_B8G8R8A8_UNORM,
                    alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                },
                dpiX: 96.0,
                dpiY: 96.0,
                usage: D2D1_RENDER_TARGET_USAGE_GDI_COMPATIBLE,
                minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
            };
            let target = factory
                .CreateDxgiSurfaceRenderTarget(&surface, &props)
                .map_err(|x| e("Direct2D target", x))?;
            let brush = target
                .CreateSolidColorBrush(&color([0, 0, 0], 1.0), None)
                .map_err(|x| e("brush", x))?;
            let write: IDWriteFactory =
                DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED).map_err(|x| e("DirectWrite", x))?;
            Ok(Painter {
                target,
                brush,
                write,
                text: Mutex::new(None),
            })
        }
    }

    fn format(&self, size: f32) -> Option<IDWriteTextFormat> {
        let mut slot = self.text.lock().unwrap();
        if let Some((s, f)) = slot.as_ref() {
            if (*s - size).abs() < 0.1 {
                return Some(f.clone());
            }
        }
        // SAFETY: a plain DirectWrite call.
        let f = unsafe {
            self.write.CreateTextFormat(
                w!("Segoe UI"),
                None,
                DWRITE_FONT_WEIGHT_SEMI_BOLD,
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                size,
                w!(""),
            )
        }
        .ok()?;
        *slot = Some((size, f.clone()));
        Some(f)
    }

    /// The width a text takes in `format`.
    fn width(&self, text: &[u16], format: &IDWriteTextFormat) -> f32 {
        // SAFETY: a plain DirectWrite call; the metrics are an out-parameter.
        unsafe {
            let Ok(layout) = self.write.CreateTextLayout(text, format, 4000.0, 400.0) else {
                return text.len() as f32 * 10.0;
            };
            let mut m = DWRITE_TEXT_METRICS::default();
            if layout.GetMetrics(&mut m).is_ok() {
                m.widthIncludingTrailingWhitespace
            } else {
                text.len() as f32 * 10.0
            }
        }
    }

    /// Draws the effects: the highlight and the waves around `pointer`, the key
    /// labels at the bottom of `area` (both in the texture's pixels). `scale` turns
    /// logical sizes into pixels.
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &self,
        fx: &Effects,
        live: &Live,
        pointer: Option<(f32, f32)>,
        area: (f32, f32, f32, f32),
        origin: (f32, f32),
        scale: f32,
        now: Instant,
    ) {
        let t = &self.target;
        let b = &self.brush;
        // SAFETY: drawing between BeginDraw and EndDraw on this thread's target.
        unsafe {
            t.BeginDraw();
            if let (Some(h), Some((x, y))) = (&fx.highlight, pointer) {
                let r = h.diameter * scale / 2.0;
                b.SetColor(&color(h.color, h.opacity));
                if h.ring {
                    let width = 4.0 * scale;
                    t.DrawEllipse(&ellipse(x, y, r - width / 2.0), b, width, None);
                } else {
                    t.FillEllipse(&ellipse(x, y, r), b);
                }
            }
            if let Some(c) = &fx.clicks {
                for wave in &live.waves {
                    let age = now.saturating_duration_since(wave.at);
                    if age >= WAVE {
                        continue;
                    }
                    let p = wave_progress(age);
                    let col = if wave.secondary { c.secondary } else { c.primary };
                    let r = (c.diameter * scale / 2.0) * (0.25 + 0.75 * p);
                    let (x, y) = (wave.x as f32 - origin.0, wave.y as f32 - origin.1);
                    b.SetColor(&color(col, 0.25 * (1.0 - p)));
                    t.FillEllipse(&ellipse(x, y, r), b);
                    b.SetColor(&color(col, 0.9 * (1.0 - p)));
                    t.DrawEllipse(&ellipse(x, y, r), b, 3.0 * scale, None);
                }
                if let (Some(secondary), Some((x, y))) = (live.dragging(now), pointer) {
                    let col = if secondary { c.secondary } else { c.primary };
                    b.SetColor(&color(col, 0.6));
                    t.FillEllipse(&ellipse(x, y, 7.0 * scale), b);
                }
            }
            if let Some(k) = &fx.keys {
                let size = k.size * scale;
                if let Some(format) = self.format(size) {
                    let pad = size * 0.55;
                    let gap = size * 0.4;
                    let height = size * 1.7;
                    let texts: Vec<Vec<u16>> = live
                        .labels
                        .iter()
                        .map(|(s, _)| s.encode_utf16().collect())
                        .collect();
                    let widths: Vec<f32> = texts.iter().map(|w| self.width(w, &format) + 2.0 * pad).collect();
                    let total = widths.iter().sum::<f32>() + gap * widths.len().saturating_sub(1) as f32;
                    let (ax, ay, aw, ah) = area;
                    let margin = 16.0 * scale;
                    let mut x = match k.place {
                        KeyPlace::Center => ax + (aw - total) / 2.0,
                        KeyPlace::Left => ax + margin,
                        KeyPlace::Right => ax + aw - margin - total,
                    };
                    let y = ay + ah - margin - height;
                    for (text, w) in texts.iter().zip(&widths) {
                        let rect = D2D_RECT_F {
                            left: x,
                            top: y,
                            right: x + w,
                            bottom: y + height,
                        };
                        b.SetColor(&color([20, 22, 28], 0.82));
                        t.FillRoundedRectangle(
                            &D2D1_ROUNDED_RECT {
                                rect,
                                radiusX: 8.0 * scale,
                                radiusY: 8.0 * scale,
                            },
                            b,
                        );
                        b.SetColor(&color([255, 255, 255], 1.0));
                        let text_rect = D2D_RECT_F {
                            left: x + pad,
                            top: y + (height - size * 1.33) / 2.0,
                            right: x + w,
                            bottom: y + height,
                        };
                        t.DrawText(
                            text,
                            &format,
                            &text_rect,
                            b,
                            D2D1_DRAW_TEXT_OPTIONS_NONE,
                            DWRITE_MEASURING_MODE_NATURAL,
                        );
                        x += w + gap;
                    }
                }
            }
            let _ = t.EndDraw(None, None);
        }
    }

    /// Draws the camera (VID-032 to VID-036) in `area` (the texture's pixels): in its
    /// corner or filling it, in its shape, mirrored if asked, with its border.
    pub fn draw_camera(
        &self,
        cache: &mut Option<CameraBitmap>,
        frame: &Frame,
        look: &Look,
        area: (f32, f32, f32, f32),
        scale: f32,
    ) {
        draw_camera(&self.target, cache, frame, look, area, scale);
    }
}

/// Draws a slide on the video's picture (VID-071, VID-074): its background, then the
/// image where its view puts it.
pub struct SlidePainter {
    target: ID2D1RenderTarget,
    bitmap: Option<(u64, ID2D1Bitmap)>,
}

impl SlidePainter {
    pub fn new(texture: &ID3D11Texture2D) -> Result<SlidePainter, String> {
        let e = |c: &str, e: windows::core::Error| format!("{c}: {}", e.message());
        // SAFETY: Direct2D creation on this thread's texture.
        unsafe {
            let factory: ID2D1Factory =
                D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None).map_err(|x| e("Direct2D", x))?;
            let surface: IDXGISurface = texture.cast().map_err(|x| e("surface", x))?;
            let props = D2D1_RENDER_TARGET_PROPERTIES {
                r#type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: DXGI_FORMAT_B8G8R8A8_UNORM,
                    alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                },
                dpiX: 96.0,
                dpiY: 96.0,
                usage: D2D1_RENDER_TARGET_USAGE_NONE,
                minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
            };
            let target = factory
                .CreateDxgiSurfaceRenderTarget(&surface, &props)
                .map_err(|x| e("Direct2D target", x))?;
            Ok(SlidePainter { target, bitmap: None })
        }
    }

    /// Draws `slide` framed by `view` on a `size` picture.
    pub fn draw(&mut self, slide: &crate::slides::Slide, view: crate::slides::View, size: (u32, u32)) {
        let t = &self.target;
        let (iw, ih) = (slide.width, slide.height);
        // SAFETY: Direct2D calls on this thread's target; the slide's pixels hold `ih`
        // rows of `iw * 4` bytes and outlive the copy.
        unsafe {
            if self.bitmap.as_ref().is_none_or(|(id, _)| *id != slide.id)
                && iw > 0
                && ih > 0
                && slide.bgra.len() >= (iw * ih * 4) as usize
            {
                let props = D2D1_BITMAP_PROPERTIES {
                    pixelFormat: D2D1_PIXEL_FORMAT {
                        format: DXGI_FORMAT_B8G8R8A8_UNORM,
                        alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                    },
                    dpiX: 96.0,
                    dpiY: 96.0,
                };
                self.bitmap = t
                    .CreateBitmap(
                        D2D_SIZE_U {
                            width: iw,
                            height: ih,
                        },
                        Some(slide.bgra.as_ptr().cast()),
                        iw * 4,
                        &props,
                    )
                    .ok()
                    .map(|b| (slide.id, b));
            }
            t.BeginDraw();
            t.Clear(Some(&color(slide.background, 1.0)));
            if let Some((_, b)) = &self.bitmap {
                let r = crate::slides::place(
                    size.0 as f32,
                    size.1 as f32,
                    iw as f32,
                    ih as f32,
                    slide.enlarge,
                    view,
                );
                let dest = D2D_RECT_F {
                    left: r.x,
                    top: r.y,
                    right: r.x + r.w,
                    bottom: r.y + r.h,
                };
                t.DrawBitmap(b, Some(&dest), 1.0, D2D1_BITMAP_INTERPOLATION_MODE_LINEAR, None);
            }
            let _ = t.EndDraw(None, None);
        }
    }
}

/// The camera's last frame on a Direct2D target, kept until a newer one comes.
pub struct CameraBitmap {
    at: Instant,
    size: (u32, u32),
    bitmap: ID2D1Bitmap,
}

/// Draws the camera on `t` in `area`, where its look places it.
pub fn draw_camera(
    t: &ID2D1RenderTarget,
    cache: &mut Option<CameraBitmap>,
    frame: &Frame,
    look: &Look,
    area: (f32, f32, f32, f32),
    scale: f32,
) {
    let r = look.rect(area.2, area.3, scale);
    let r = crate::camlook::Rect {
        x: area.0 + r.x,
        y: area.1 + r.y,
        ..r
    };
    draw_camera_at(t, cache, frame, look, r, scale);
}

/// Draws the camera on `t` in the box `r`, in its shape, mirrored if asked, with its
/// border around (shared by the video and the preview, so both look alike).
pub fn draw_camera_at(
    t: &ID2D1RenderTarget,
    cache: &mut Option<CameraBitmap>,
    frame: &Frame,
    look: &Look,
    r: crate::camlook::Rect,
    scale: f32,
) {
    let (fw, fh) = (frame.width, frame.height);
    if fw == 0 || fh == 0 || frame.bgra.len() < (fw * fh * 4) as usize {
        return;
    }
    // SAFETY: Direct2D calls on this thread's target; the frame's pixels outlive the
    // copy, which reads `fh` rows of `fw * 4` bytes.
    unsafe {
        let fresh = cache.as_ref().is_none_or(|c| c.at != frame.at);
        if cache.as_ref().is_some_and(|c| c.size != (fw, fh)) {
            *cache = None;
        }
        if cache.is_none() {
            let props = D2D1_BITMAP_PROPERTIES {
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: DXGI_FORMAT_B8G8R8A8_UNORM,
                    alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                },
                dpiX: 96.0,
                dpiY: 96.0,
            };
            let Ok(bitmap) = t.CreateBitmap(
                D2D_SIZE_U {
                    width: fw,
                    height: fh,
                },
                Some(frame.bgra.as_ptr().cast()),
                fw * 4,
                &props,
            ) else {
                return;
            };
            *cache = Some(CameraBitmap {
                at: frame.at,
                size: (fw, fh),
                bitmap,
            });
        } else if fresh {
            if let Some(c) = cache.as_mut() {
                let _ = c.bitmap.CopyFromMemory(None, frame.bgra.as_ptr().cast(), fw * 4);
                c.at = frame.at;
            }
        }
        let Some(c) = cache.as_ref() else { return };
        let (x, y) = (r.x, r.y);
        let src = cover(fw as f32, fh as f32, r.w, r.h);
        let k = r.w / src.w;
        let props = D2D1_BITMAP_BRUSH_PROPERTIES {
            extendModeX: D2D1_EXTEND_MODE_CLAMP,
            extendModeY: D2D1_EXTEND_MODE_CLAMP,
            interpolationMode: D2D1_BITMAP_INTERPOLATION_MODE_LINEAR,
        };
        let Ok(brush) = t.CreateBitmapBrush(&c.bitmap, Some(&props), None) else {
            return;
        };
        // The cut of the picture onto the box; the mirror turns it around its middle.
        let m = if look.mirror {
            windows_numerics::Matrix3x2 {
                M11: -k,
                M12: 0.0,
                M21: 0.0,
                M22: k,
                M31: x + r.w + src.x * k,
                M32: y - src.y * k,
            }
        } else {
            windows_numerics::Matrix3x2 {
                M11: k,
                M12: 0.0,
                M21: 0.0,
                M22: k,
                M31: x - src.x * k,
                M32: y - src.y * k,
            }
        };
        brush.SetTransform(&m);
        let radius = look.radius(r);
        let rounded = |grow: f32| D2D1_ROUNDED_RECT {
            rect: D2D_RECT_F {
                left: x - grow,
                top: y - grow,
                right: x + r.w + grow,
                bottom: y + r.h + grow,
            },
            radiusX: (radius + grow).max(0.0),
            radiusY: (radius + grow).max(0.0),
        };
        t.BeginDraw();
        t.FillRoundedRectangle(&rounded(0.0), &brush);
        if look.has_border() {
            let width = look.border_width.clamp(2.0, 16.0) * scale;
            let ring = rounded(width / 2.0);
            let (a, b) = look.border_colors;
            if look.border == Border::Gradient {
                let stops = [
                    D2D1_GRADIENT_STOP {
                        position: 0.0,
                        color: color(a, 1.0),
                    },
                    D2D1_GRADIENT_STOP {
                        position: 1.0,
                        color: color(b, 1.0),
                    },
                ];
                let (cx, cy) = (x + r.w / 2.0, y + r.h / 2.0);
                let (dx, dy) = (
                    look.border_angle.to_radians().cos(),
                    look.border_angle.to_radians().sin(),
                );
                let reach = (r.w.abs() * dx.abs() + r.h.abs() * dy.abs()) / 2.0 + width;
                if let Ok(collection) =
                    t.CreateGradientStopCollection(&stops, D2D1_GAMMA_2_2, D2D1_EXTEND_MODE_CLAMP)
                {
                    let line = D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES {
                        startPoint: windows_numerics::Vector2 {
                            X: cx - dx * reach,
                            Y: cy - dy * reach,
                        },
                        endPoint: windows_numerics::Vector2 {
                            X: cx + dx * reach,
                            Y: cy + dy * reach,
                        },
                    };
                    if let Ok(g) = t.CreateLinearGradientBrush(&line, None, &collection) {
                        t.DrawRoundedRectangle(&ring, &g, width, None);
                    }
                }
            } else if let Ok(s) = t.CreateSolidColorBrush(&color(a, 1.0), None) {
                t.DrawRoundedRectangle(&ring, &s, width, None);
            }
        }
        let _ = t.EndDraw(None, None);
    }
}

/// Mouse and keyboard as they happen, from the hooks.
#[derive(Debug, Clone, Copy)]
pub enum Input {
    Button {
        down: bool,
        secondary: bool,
        x: i32,
        y: i32,
        at: Instant,
    },
    Key {
        vk: u32,
        down: bool,
        at: Instant,
    },
}

/// Where the hooks send what they see; one recording at a time.
static SINK: Mutex<Option<mpsc::Sender<Input>>> = Mutex::new(None);

fn send(i: Input) {
    if let Ok(sink) = SINK.lock() {
        if let Some(tx) = sink.as_ref() {
            let _ = tx.send(i);
        }
    }
}

unsafe extern "system" fn mouse_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        // SAFETY: for a mouse hook, lparam points at an MSLLHOOKSTRUCT.
        let m = unsafe { &*(lparam.0 as *const MSLLHOOKSTRUCT) };
        let button = match wparam.0 as u32 {
            WM_LBUTTONDOWN => Some((true, false)),
            WM_LBUTTONUP => Some((false, false)),
            WM_RBUTTONDOWN => Some((true, true)),
            WM_RBUTTONUP => Some((false, true)),
            _ => None,
        };
        if let Some((down, secondary)) = button {
            send(Input::Button {
                down,
                secondary,
                x: m.pt.x,
                y: m.pt.y,
                at: Instant::now(),
            });
        }
    }
    // SAFETY: passes the event on, unchanged.
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

unsafe extern "system" fn key_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        // SAFETY: for a keyboard hook, lparam points at a KBDLLHOOKSTRUCT.
        let k = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
        let down = match wparam.0 as u32 {
            WM_KEYDOWN | WM_SYSKEYDOWN => Some(true),
            WM_KEYUP | WM_SYSKEYUP => Some(false),
            _ => None,
        };
        if let Some(down) = down {
            send(Input::Key {
                vk: k.vkCode,
                down,
                at: Instant::now(),
            });
        }
    }
    // SAFETY: passes the event on, unchanged.
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

/// The hooks, on their own thread with its message loop; dropping them removes them.
pub struct Hooks {
    thread: u32,
    join: Option<std::thread::JoinHandle<()>>,
    pub events: mpsc::Receiver<Input>,
    pub keys: bool,
}

impl Hooks {
    /// The mouse hook, and the keyboard one with `keys`.
    pub fn start(keys: bool) -> Option<Hooks> {
        let (tx, rx) = mpsc::channel();
        *SINK.lock().ok()? = Some(tx);
        let (id_tx, id_rx) = mpsc::channel();
        let join = std::thread::Builder::new()
            .name("xs-record-input".into())
            .spawn(move || {
                // SAFETY: hooks installed and removed on this thread, with its loop.
                unsafe {
                    use windows::Win32::System::Threading::GetCurrentThreadId;
                    use windows::Win32::UI::HiDpi::{
                        SetThreadDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
                    };
                    let _ = SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
                    let mouse = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook), None, 0).ok();
                    let keyboard = if keys {
                        SetWindowsHookExW(WH_KEYBOARD_LL, Some(key_hook), None, 0).ok()
                    } else {
                        None
                    };
                    let _ = id_tx.send(GetCurrentThreadId());
                    let mut msg = MSG::default();
                    while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                        let _ = TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }
                    if let Some(h) = mouse {
                        let _ = UnhookWindowsHookEx(h);
                    }
                    if let Some(h) = keyboard {
                        let _ = UnhookWindowsHookEx(h);
                    }
                }
            })
            .ok()?;
        let thread = id_rx.recv_timeout(std::time::Duration::from_secs(2)).ok()?;
        Some(Hooks {
            thread,
            join: Some(join),
            events: rx,
            keys,
        })
    }
}

impl Drop for Hooks {
    fn drop(&mut self) {
        // SAFETY: asks the hook thread to end its loop; it removes its hooks.
        unsafe {
            let _ = PostThreadMessageW(self.thread, WM_QUIT, WPARAM(0), LPARAM(0));
        }
        if let Some(j) = self.join.take() {
            let _ = j.join();
        }
        if let Ok(mut sink) = SINK.lock() {
            *sink = None;
        }
    }
}

/// Whether the focused field is a password (keys are then never shown, VID-047).
pub struct Focus {
    uia: Option<IUIAutomation>,
}

impl Focus {
    pub fn new() -> Focus {
        // SAFETY: COM creation on this thread (initialized by the recording).
        let uia = unsafe { CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER) }.ok();
        Focus { uia }
    }

    /// True when keys may be shown: the focus is not a password field. Without UI
    /// Automation nothing is shown, to be safe.
    pub fn shows_keys(&self) -> bool {
        let Some(uia) = &self.uia else { return false };
        // SAFETY: plain UI Automation queries.
        unsafe {
            match uia.GetFocusedElement() {
                Ok(e) => !e.CurrentIsPassword().map(|b| b.as_bool()).unwrap_or(true),
                // Nothing focused: nothing typed into a field.
                Err(_) => true,
            }
        }
    }
}
