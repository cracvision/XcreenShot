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
