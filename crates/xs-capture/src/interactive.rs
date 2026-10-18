//! On-screen selection (CAP-009, CAP-013, CAP-025, CAP-027): freeze every monitor,
//! let the shell show the selection over the frozen frames, then cut the image out
//! of those same frames, so tooltips and menus that vanish meanwhile still appear.

use std::sync::Arc;
use std::time::Instant;

use image::RgbaImage;
use serde::Serialize;
use serde_json::{json, Value};
use xs_core::host::Frame;
use xs_core::i18n::tr;
use xs_core::{CommandError, CommandResult, Context, Core, ErrorCode, Lang};

use crate::{app_windows, failed, finish, monitors, platform, MonitorInfo, Options, Rect, Shot};

/// A window the selection can highlight and pick (CAP-016), topmost first.
#[derive(Debug, Clone, Serialize)]
pub struct WindowRect {
    pub id: u32,
    pub pid: u32,
    pub rect: Rect,
    pub title: String,
    pub app: String,
}

pub struct Frozen {
    pub monitors: Vec<MonitorInfo>,
    pub frames: Vec<Frame>,
    pub windows: Vec<WindowRect>,
    pub pointer: Option<(i32, i32)>,
}

/// Takes every monitor as it is right now (CAP-013, APP-048).
pub fn freeze(lang: Lang) -> Result<Frozen, CommandError> {
    let started = Instant::now();
    let pointer = platform::cursor_position();
    let list = monitors().map_err(|e| failed(lang, e))?;
    let mut frames = Vec::with_capacity(list.len());
    for (m, _) in &list {
        let image = m.capture_image().map_err(|e| failed(lang, e))?;
        frames.push(Frame {
            width: image.width(),
            height: image.height(),
            rgba: Arc::new(image.into_raw()),
        });
    }
    let captured_ms = started.elapsed().as_millis();
    let windows = app_windows()
        .into_iter()
        .filter_map(|w| {
            Some(WindowRect {
                id: w.id().ok()?,
                pid: w.pid().ok()?,
                rect: Rect {
                    x: w.x().ok()?,
                    y: w.y().ok()?,
                    width: w.width().ok()?,
                    height: w.height().ok()?,
                },
                title: w.title().unwrap_or_default(),
                app: w.app_name().unwrap_or_default(),
            })
        })
        .collect();
    tracing::info!(
        captured_ms,
        total_ms = started.elapsed().as_millis() as u64,
        monitors = list.len(),
        "screens frozen"
    );
    Ok(Frozen {
        monitors: list.into_iter().map(|(_, i)| i).collect(),
        frames,
        windows,
        pointer,
    })
}

/// RGBA to BGRA (the byte order GDI paints), for showing frozen screens natively.
pub fn to_bgra(rgba: &[u8]) -> Vec<u8> {
    let mut out = rgba.to_vec();
    for px in out.as_chunks_mut::<4>().0.iter_mut() {
        px.swap(0, 2);
    }
    out
}

/// Cuts `area` (desktop pixels) out of the frozen frames; uncovered parts stay
/// transparent (CAP-021).
pub fn crop_frames(monitors: &[MonitorInfo], frames: &[Frame], area: Rect) -> RgbaImage {
    let mut out = RgbaImage::new(area.width, area.height);
    let dst_stride = area.width as usize * 4;
    let buf: &mut [u8] = &mut out;
    for (info, frame) in monitors.iter().zip(frames) {
        let Some(part) = info.rect.intersect(&area) else {
            continue;
        };
        let src_stride = frame.width as usize * 4;
        let row_bytes = part.width as usize * 4;
        for row in 0..part.height as usize {
            let sy = (part.y - info.rect.y) as usize + row;
            let sx = (part.x - info.rect.x) as usize;
            let dy = (part.y - area.y) as usize + row;
            let dx = (part.x - area.x) as usize;
            let src = sy * src_stride + sx * 4;
            let dst = dy * dst_stride + dx * 4;
            if src + row_bytes <= frame.rgba.len() && dst + row_bytes <= buf.len() {
                buf[dst..dst + row_bytes].copy_from_slice(&frame.rgba[src..src + row_bytes]);
            }
        }
    }
    out
}

fn rect_from(v: &Value) -> Option<Rect> {
    Some(Rect {
        x: v.get("x")?.as_i64()? as i32,
        y: v.get("y")?.as_i64()? as i32,
        width: v.get("width")?.as_u64()?.max(1) as u32,
        height: v.get("height")?.as_u64()?.max(1) as u32,
    })
}

/// Runs an interactive capture: `mode` is `region`, `window` or `universal`.
pub fn run(core: &Core, ctx: &Context, opts: &Options, mode: &str, extra: Value) -> CommandResult {
    let lang = ctx.lang;
    let host = core.host();
    let hide = core.settings.get_bool("capture.hideOnCapture");
    if hide {
        host.hide_for_capture(true);
    }
    let frozen = freeze(lang);
    if hide {
        host.hide_for_capture(false);
    }
    let frozen = frozen?;
    let mut request = json!({
        "mode": mode,
        "monitors": frozen.monitors,
        "windows": frozen.windows,
        "pointer": frozen.pointer,
        "autoDetect": core.settings.get_bool("capture.autoDetect"),
        "showMagnifier": core.settings.get_bool("capture.showMagnifier"),
        "magnifierZoom": core.recall("capture.magnifierZoom").unwrap_or(json!(4)),
    });
    if let (Some(r), Some(e)) = (request.as_object_mut(), extra.as_object()) {
        r.extend(e.clone());
    }
    let outcome = host.select_on_screen(request, frozen.frames.clone())?;
    if let Some(zoom) = outcome.get("magnifierZoom").filter(|z| z.is_u64()) {
        core.remember("capture.magnifierZoom", zoom.clone());
    }
    match outcome.get("kind").and_then(Value::as_str) {
        Some("rect") => {
            let area = outcome
                .get("rect")
                .and_then(rect_from)
                .ok_or_else(|| CommandError::new(ErrorCode::InvalidParams, "rect"))?;
            let image = crop_frames(&frozen.monitors, &frozen.frames, area);
            let hit: Vec<&MonitorInfo> = frozen
                .monitors
                .iter()
                .filter(|m| m.rect.intersect(&area).is_some())
                .collect();
            let main = hit
                .iter()
                .max_by_key(|m| m.rect.intersect(&area).map(|r| r.area()).unwrap_or(0));
            // "Aplicación" of a region: the window covering most of it (CAP-056).
            let window = frozen
                .windows
                .iter()
                .filter_map(|w| Some((w.rect.intersect(&area)?.area(), w)))
                .max_by_key(|(a, _)| *a)
                .map(|(_, w)| w);
            let shot = Shot {
                image,
                rect: area,
                mode: if mode == "universal" {
                    "universal"
                } else {
                    "region"
                },
                scale: hit.iter().map(|m| m.scale).fold(1.0f32, f32::max),
                monitor: main.map(|m| m.name.clone()).unwrap_or_default(),
                app: window.map(|w| platform::app_info(w.pid, &w.app)),
                window_title: window.map(|w| w.title.clone()),
                window_id: None,
                warnings: Vec::new(),
            };
            finish(core, ctx, shot, opts)
        }
        Some("window") => {
            let id = outcome.get("id").and_then(Value::as_u64).unwrap_or(0) as u32;
            // The whole window, even if covered (CAP-027): taken live, not from the frames.
            let shot = crate::capture_window(&crate::WindowTarget::Id(id), lang)?;
            finish(core, ctx, shot, opts)
        }
        _ => Err(
            CommandError::new(ErrorCode::Cancelled, tr(lang, "errors.capture_cancelled", &[]))
                .with_details(json!({ "status": "cancelled" })),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(w: u32, h: u32, value: u8) -> Frame {
        Frame {
            width: w,
            height: h,
            rgba: Arc::new(vec![value; (w * h * 4) as usize]),
        }
    }

    fn monitor(x: i32, w: u32, h: u32) -> MonitorInfo {
        MonitorInfo {
            id: 0,
            name: "m".into(),
            rect: Rect {
                x,
                y: 0,
                width: w,
                height: h,
            },
            scale: 1.0,
            primary: x == 0,
        }
    }

    /// CAP-009: the image has exactly the selected size, even if odd.
    #[test]
    fn crop_has_the_exact_selected_size() {
        let img = crop_frames(
            &[monitor(0, 400, 300)],
            &[frame(400, 300, 7)],
            Rect {
                x: 10,
                y: 20,
                width: 301,
                height: 201,
            },
        );
        assert_eq!((img.width(), img.height()), (301, 201));
        assert!(img.as_raw().iter().all(|&b| b == 7));
    }

    #[test]
    fn crop_across_monitors_and_gaps() {
        let monitors = [monitor(0, 4, 4), monitor(4, 4, 2)];
        let frames = [frame(4, 4, 1), frame(4, 2, 2)];
        let img = crop_frames(
            &monitors,
            &frames,
            Rect {
                x: 2,
                y: 0,
                width: 4,
                height: 4,
            },
        );
        assert_eq!(img.get_pixel(1, 0).0, [1, 1, 1, 1]);
        assert_eq!(img.get_pixel(2, 0).0, [2, 2, 2, 2]);
        assert_eq!(
            img.get_pixel(3, 3).0,
            [0, 0, 0, 0],
            "below the short monitor is transparent"
        );
    }
}
