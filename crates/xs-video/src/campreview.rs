//! The camera's preview on the screen (VID-031, VID-032): a window of its own over the
//! recorded area, where the camera goes in the video, drawn as the video draws it. It
//! never shows in the video (the system leaves it out of captures): the recording
//! draws the camera itself. It is dragged by its picture (let go, it snaps to the
//! nearest corner) and resized from its handle, the corner facing the area's middle.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use windows::core::w;
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM};
use windows::Win32::Graphics::Direct2D::Common::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::HiDpi::{SetThreadDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2};
use windows::Win32::UI::Input::KeyboardAndMouse::{ReleaseCapture, SetCapture};
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::camlook::{nearest_corner, Feed, Frame, Look, Placement, Rect};
use crate::win_fx::{draw_camera_at, CameraBitmap};

/// What the user did to the preview, for the settings and the recording.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Change {
    Corner(crate::camlook::Corner),
    Size(f32),
}

/// The area recorded, in virtual-screen pixels, and its monitor's scale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Area {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub scale: f32,
}

struct Shared {
    feed: Feed,
    look: Mutex<Look>,
    area: Mutex<Area>,
    dirty: AtomicBool,
    on_change: Box<dyn Fn(Change) + Send + Sync>,
}

/// The preview; dropping it closes its window.
pub struct Preview {
    shared: Arc<Shared>,
    thread: u32,
    join: Option<std::thread::JoinHandle<()>>,
}

/// The handle's size, in logical pixels.
const HANDLE: f32 = 14.0;

impl Preview {
    /// Shows the camera of `feed` over `area`, placed by `look`; `on_change` hears the
    /// corner or size the user gives it.
    pub fn show(
        feed: Feed,
        area: Area,
        look: Look,
        on_change: impl Fn(Change) + Send + Sync + 'static,
    ) -> Option<Preview> {
        let shared = Arc::new(Shared {
            feed,
            look: Mutex::new(look),
            area: Mutex::new(area),
            dirty: AtomicBool::new(true),
            on_change: Box::new(on_change),
        });
        let (tx, rx) = std::sync::mpsc::channel();
        let s = shared.clone();
        let join = std::thread::Builder::new()
            .name("xs-camera-preview".into())
            .spawn(move || run(s, tx))
            .ok()?;
        let thread = rx.recv().ok().flatten()?;
        Some(Preview {
            shared,
            thread,
            join: Some(join),
        })
    }

    /// Placed and drawn by `look` from now on.
    pub fn set_look(&self, look: Look) {
        *self.shared.look.lock().unwrap() = look;
        self.shared.dirty.store(true, Ordering::Relaxed);
    }

    /// Over `area` from now on (a window followed, VID-005).
    pub fn set_area(&self, area: Area) {
        *self.shared.area.lock().unwrap() = area;
        self.shared.dirty.store(true, Ordering::Relaxed);
    }
}

impl Drop for Preview {
    fn drop(&mut self) {
        // SAFETY: asks the preview's own thread to end its message loop.
        unsafe {
            let _ = PostThreadMessageW(self.thread, WM_QUIT, WPARAM(0), LPARAM(0));
        }
        if let Some(j) = self.join.take() {
            let _ = j.join();
        }
    }
}

/// The preview is always drawn in a corner: at full screen the video shows only the
/// camera, but the screen stays the user's (VID-034).
fn pip(look: &Look) -> Look {
    Look {
        placement: Placement::Pip,
        ..look.clone()
    }
}

/// How far the border reaches out of the camera's box, in pixels.
fn pad(look: &Look, scale: f32) -> i32 {
    let border = if pip(look).has_border() {
        look.border_width.clamp(2.0, 16.0) * scale
    } else {
        0.0
    };
    (border + 2.0).ceil() as i32
}

/// The camera's box on the screen.
fn camera_box(look: &Look, area: &Area) -> Rect {
    let r = pip(look).rect(area.width as f32, area.height as f32, area.scale);
    Rect {
        x: area.x as f32 + r.x,
        y: area.y as f32 + r.y,
        ..r
    }
}

enum Drag {
    Move { grab: POINT, from: RECT },
    Resize { anchor: (f32, f32) },
}

struct State {
    shared: Arc<Shared>,
    hwnd: HWND,
    target: ID2D1DCRenderTarget,
    dc: HDC,
    dib: Option<(HBITMAP, (i32, i32))>,
    cache: Option<CameraBitmap>,
    last: Option<Instant>,
    /// The box being dragged or resized, on the screen; the look while resizing.
    drag: Option<Drag>,
    live: Option<(Rect, Look)>,
    placeholder: Arc<Frame>,
}

fn run(shared: Arc<Shared>, ready: std::sync::mpsc::Sender<Option<u32>>) {
    // SAFETY: a window of this thread, made, driven and destroyed here; the state lives
    // in a Box whose pointer the window keeps until WM_DESTROY.
    unsafe {
        let _ = SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let Ok(module) = GetModuleHandleW(None) else {
            let _ = ready.send(None);
            return;
        };
        let class = w!("XcreenShotCameraPreview");
        let wc = WNDCLASSW {
            lpfnWndProc: Some(proc),
            hInstance: module.into(),
            lpszClassName: class,
            hCursor: LoadCursorW(None, IDC_SIZEALL).unwrap_or_default(),
            ..Default::default()
        };
        // Registering twice fails harmlessly.
        RegisterClassW(&wc);
        let Ok(hwnd) = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            class,
            w!("XcreenShot camera"),
            WS_POPUP,
            0,
            0,
            10,
            10,
            None,
            None,
            Some(module.into()),
            None,
        ) else {
            let _ = ready.send(None);
            return;
        };
        // Never in a capture: the recording draws the camera itself.
        let _ = SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE);
        let Ok(factory) = D2D1CreateFactory::<ID2D1Factory>(D2D1_FACTORY_TYPE_SINGLE_THREADED, None) else {
            let _ = DestroyWindow(hwnd);
            let _ = ready.send(None);
            return;
        };
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
        let Ok(target) = factory.CreateDCRenderTarget(&props) else {
            let _ = DestroyWindow(hwnd);
            let _ = ready.send(None);
            return;
        };
        let state = Box::new(State {
            shared,
            hwnd,
            target,
            dc: CreateCompatibleDC(None),
            dib: None,
            cache: None,
            last: None,
            drag: None,
            live: None,
            placeholder: Arc::new(Frame {
                width: 16,
                height: 9,
                bgra: [40u8, 40, 40, 255].repeat(16 * 9),
                at: Instant::now(),
            }),
        });
        let state = Box::into_raw(state);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, state as isize);
        let _ = SetTimer(Some(hwnd), 1, 33, None);
        (*state).render();
        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        let _ = ready.send(Some(GetCurrentThreadId()));
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
        let _ = DestroyWindow(hwnd);
        let state = Box::from_raw(state);
        if let Some((dib, _)) = state.dib {
            let _ = DeleteObject(dib.into());
        }
        let _ = DeleteDC(state.dc);
    }
}

impl State {
    /// The camera's box and look now: the one being resized, or the settings'.
    fn now(&self) -> (Rect, Look, Area) {
        let area = *self.shared.area.lock().unwrap();
        if let Some((r, l)) = &self.live {
            return (*r, l.clone(), area);
        }
        let look = self.shared.look.lock().unwrap().clone();
        (camera_box(&look, &area), look, area)
    }

    /// Draws the latest frame where the camera goes.
    fn render(&mut self) {
        let (r, look, area) = self.now();
        let p = pad(&look, area.scale);
        let (w, h) = ((r.w.round() as i32) + 2 * p, (r.h.round() as i32) + 2 * p);
        let (x, y) = (r.x.round() as i32 - p, r.y.round() as i32 - p);
        // SAFETY: GDI and Direct2D calls on this thread's DC, bitmap and window.
        unsafe {
            if self.dib.as_ref().is_none_or(|(_, s)| *s != (w, h)) {
                if let Some((old, _)) = self.dib.take() {
                    let _ = DeleteObject(old.into());
                }
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
                let Ok(dib) = CreateDIBSection(Some(self.dc), &info, DIB_RGB_COLORS, &mut bits, None, 0)
                else {
                    return;
                };
                SelectObject(self.dc, dib.into());
                self.dib = Some((dib, (w, h)));
            }
            let rect = RECT {
                left: 0,
                top: 0,
                right: w,
                bottom: h,
            };
            if self.target.BindDC(self.dc, &rect).is_err() {
                return;
            }
            self.target.BeginDraw();
            self.target.Clear(Some(&D2D1_COLOR_F::default()));
            let _ = self.target.EndDraw(None, None);
            let frame = (self.shared.feed)().unwrap_or_else(|| self.placeholder.clone());
            self.last = Some(frame.at);
            let inner = Rect {
                x: p as f32,
                y: p as f32,
                w: r.w,
                h: r.h,
            };
            let t: &ID2D1RenderTarget = &self.target;
            draw_camera_at(t, &mut self.cache, &frame, &pip(&look), inner, area.scale);
            // The handle: a small dot at the corner facing the area's middle.
            let (hx, hy) = self.handle(&r, &area);
            let size = HANDLE * area.scale / 2.0;
            self.target.BeginDraw();
            if let Ok(b) = self.target.CreateSolidColorBrush(
                &D2D1_COLOR_F {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 0.95,
                },
                None,
            ) {
                let e = D2D1_ELLIPSE {
                    point: windows_numerics::Vector2 {
                        X: hx - x as f32,
                        Y: hy - y as f32,
                    },
                    radiusX: size,
                    radiusY: size,
                };
                self.target.FillEllipse(&e, &b);
                b.SetColor(&D2D1_COLOR_F {
                    r: 0.1,
                    g: 0.1,
                    b: 0.12,
                    a: 0.9,
                });
                self.target.DrawEllipse(&e, &b, 1.5 * area.scale, None);
            }
            let _ = self.target.EndDraw(None, None);
            let blend = BLENDFUNCTION {
                BlendOp: AC_SRC_OVER as u8,
                BlendFlags: 0,
                SourceConstantAlpha: 255,
                AlphaFormat: AC_SRC_ALPHA as u8,
            };
            let _ = UpdateLayeredWindow(
                self.hwnd,
                None,
                Some(&POINT { x, y }),
                Some(&SIZE { cx: w, cy: h }),
                Some(self.dc),
                Some(&POINT { x: 0, y: 0 }),
                COLORREF(0),
                Some(&blend),
                ULW_ALPHA,
            );
            let _ = SetWindowPos(
                self.hwnd,
                Some(HWND_TOPMOST),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
        }
    }

    /// Where the handle is: the camera's corner facing the area's middle (screen).
    fn handle(&self, r: &Rect, area: &Area) -> (f32, f32) {
        let (cx, cy) = (
            area.x as f32 + area.width as f32 / 2.0,
            area.y as f32 + area.height as f32 / 2.0,
        );
        let hx = if r.x + r.w / 2.0 > cx { r.x } else { r.x + r.w };
        let hy = if r.y + r.h / 2.0 > cy { r.y } else { r.y + r.h };
        (hx, hy)
    }

    fn on_handle(&self, at: POINT) -> bool {
        let (r, _, area) = self.now();
        let (hx, hy) = self.handle(&r, &area);
        let reach = HANDLE * area.scale;
        (at.x as f32 - hx).abs() <= reach && (at.y as f32 - hy).abs() <= reach
    }

    fn press(&mut self, at: POINT) {
        let (r, look, area) = self.now();
        if self.on_handle(at) {
            // The corner across from the handle stays put.
            let (hx, hy) = self.handle(&r, &area);
            let anchor = (
                if hx == r.x { r.x + r.w } else { r.x },
                if hy == r.y { r.y + r.h } else { r.y },
            );
            self.drag = Some(Drag::Resize { anchor });
            self.live = Some((r, look));
        } else {
            let mut from = RECT::default();
            // SAFETY: reads this thread's window rectangle.
            unsafe {
                let _ = GetWindowRect(self.hwnd, &mut from);
            }
            self.drag = Some(Drag::Move { grab: at, from });
        }
        // SAFETY: the mouse stays with this window until it is let go.
        unsafe {
            SetCapture(self.hwnd);
        }
    }

    fn moved(&mut self, at: POINT) {
        match &self.drag {
            Some(Drag::Move { grab, from }) => {
                // SAFETY: moves this thread's window.
                unsafe {
                    let _ = SetWindowPos(
                        self.hwnd,
                        Some(HWND_TOPMOST),
                        from.left + at.x - grab.x,
                        from.top + at.y - grab.y,
                        0,
                        0,
                        SWP_NOSIZE | SWP_NOACTIVATE,
                    );
                }
            }
            Some(Drag::Resize { anchor }) => {
                let area = *self.shared.area.lock().unwrap();
                let Some((_, look)) = self.live.clone() else {
                    return;
                };
                let ratio = look.ratio();
                let (dx, dy) = ((at.x as f32 - anchor.0).abs(), (at.y as f32 - anchor.1).abs());
                // The short side the pointer asks for, held between 10 % and 50 %.
                let wanted = if ratio >= 1.0 {
                    dy.max(dx / ratio)
                } else {
                    dx.max(dy * ratio)
                };
                let short = (area.width.min(area.height)) as f32;
                let size = (wanted / short).clamp(0.10, 0.50);
                let sized = Look { size, ..look };
                let r = pip(&sized).rect(area.width as f32, area.height as f32, area.scale);
                // Grown from the anchor, towards the pointer.
                let x = if at.x as f32 >= anchor.0 {
                    anchor.0
                } else {
                    anchor.0 - r.w
                };
                let y = if at.y as f32 >= anchor.1 {
                    anchor.1
                } else {
                    anchor.1 - r.h
                };
                self.live = Some((Rect { x, y, w: r.w, h: r.h }, sized));
                self.render();
            }
            None => {}
        }
    }

    fn release(&mut self) {
        // SAFETY: lets the mouse go.
        unsafe {
            let _ = ReleaseCapture();
        }
        let area = *self.shared.area.lock().unwrap();
        match self.drag.take() {
            Some(Drag::Move { .. }) => {
                let mut now = RECT::default();
                // SAFETY: reads this thread's window rectangle.
                unsafe {
                    let _ = GetWindowRect(self.hwnd, &mut now);
                }
                let cx = (now.left + now.right) as f32 / 2.0 - area.x as f32;
                let cy = (now.top + now.bottom) as f32 / 2.0 - area.y as f32;
                let corner = nearest_corner(cx, cy, area.width as f32, area.height as f32);
                self.shared.look.lock().unwrap().corner = corner;
                (self.shared.on_change)(Change::Corner(corner));
            }
            Some(Drag::Resize { .. }) => {
                if let Some((_, look)) = self.live.take() {
                    self.shared.look.lock().unwrap().size = look.size;
                    (self.shared.on_change)(Change::Size(look.size));
                }
            }
            None => {}
        }
        self.render();
    }
}

fn point(lparam: LPARAM, hwnd: HWND) -> POINT {
    // Client coordinates to the screen's.
    let mut p = POINT {
        x: (lparam.0 & 0xFFFF) as i16 as i32,
        y: ((lparam.0 >> 16) & 0xFFFF) as i16 as i32,
    };
    // SAFETY: converts a point of this window.
    unsafe {
        let _ = ClientToScreen(hwnd, &mut p);
    }
    p
}

unsafe extern "system" fn proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // SAFETY: the pointer is the State this thread made for this window, or 0.
    let state = unsafe { (GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut State).as_mut() };
    if let Some(s) = state {
        match msg {
            WM_TIMER => {
                let newer = (s.shared.feed)().map(|f| f.at) != s.last;
                if s.drag.is_none() && (newer || s.shared.dirty.swap(false, Ordering::Relaxed)) {
                    s.render();
                }
                return LRESULT(0);
            }
            WM_MOUSEACTIVATE => return LRESULT(MA_NOACTIVATE as isize),
            WM_SETCURSOR => {
                let mut at = POINT::default();
                // SAFETY: plain cursor calls.
                unsafe {
                    let _ = GetCursorPos(&mut at);
                    let (r, _, area) = s.now();
                    let (hx, hy) = s.handle(&r, &area);
                    let diagonal = (hx == r.x) == (hy == r.y);
                    let cursor = if s.on_handle(at) {
                        if diagonal {
                            IDC_SIZENWSE
                        } else {
                            IDC_SIZENESW
                        }
                    } else {
                        IDC_SIZEALL
                    };
                    let _ = SetCursor(LoadCursorW(None, cursor).ok());
                }
                return LRESULT(1);
            }
            WM_LBUTTONDOWN => {
                s.press(point(lparam, hwnd));
                return LRESULT(0);
            }
            WM_MOUSEMOVE if s.drag.is_some() => {
                s.moved(point(lparam, hwnd));
                return LRESULT(0);
            }
            WM_LBUTTONUP if s.drag.is_some() => {
                s.release();
                return LRESULT(0);
            }
            WM_CAPTURECHANGED if s.drag.is_some() => {
                s.release();
                return LRESULT(0);
            }
            _ => {}
        }
    }
    // SAFETY: the default handling of everything else.
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Shows a preview of a made-up camera at the bottom right of a 1280 × 720 area for
    /// a second and checks it is where the camera goes (by hand: it is on the screen,
    /// and out of any capture).
    /// `cargo test -p xs-video -- --ignored --nocapture a_preview_shows_where_the_camera_goes`
    #[test]
    #[ignore]
    fn a_preview_shows_where_the_camera_goes() {
        let frame = Arc::new(Frame {
            width: 160,
            height: 90,
            bgra: [255u8, 0, 255, 255].repeat(160 * 90),
            at: Instant::now(),
        });
        let f = frame.clone();
        let feed: Feed = Arc::new(move || Some(f.clone()));
        let area = Area {
            x: 0,
            y: 0,
            width: 1280,
            height: 720,
            scale: 1.0,
        };
        let p = Preview::show(feed, area, Look::default(), |c| println!("{c:?}")).expect("a preview");
        std::thread::sleep(std::time::Duration::from_millis(1000));
        // SAFETY: finds the preview's window and reads its rectangle, in physical
        // pixels like the preview's.
        let r = unsafe {
            let _ = SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
            let hwnd = FindWindowW(w!("XcreenShotCameraPreview"), None).expect("its window");
            let mut r = RECT::default();
            let _ = GetWindowRect(hwnd, &mut r);
            r
        };
        drop(p);
        println!("{r:?}");
        // A quarter of 720 high, 16:9, 24 px from the bottom right, 2 px around it.
        assert_eq!((r.right, r.bottom), (1280 - 24 + 2, 720 - 24 + 2));
        assert_eq!(r.bottom - r.top, 180 + 4);
    }
}
