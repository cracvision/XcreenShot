//! Narrated slides (VID-071 to VID-074): the video's picture comes from images the app
//! hands over, not from the screen. Each slide is shown whole and centred on a
//! background colour, at its own size if it fits (or enlarged, if asked) and smaller if
//! it does not; it can be zoomed and moved, smoothly in the video.

use std::sync::Arc;
use std::time::Duration;

use crate::camlook::Rect;
use crate::effects::Rgb;

/// How a slide is framed: its zoom over the starting fit (1: as it starts), and which
/// point of the image (0 to 1 across and down) is at the video's middle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct View {
    pub zoom: f32,
    pub center: (f32, f32),
}

impl Default for View {
    fn default() -> Self {
        View {
            zoom: 1.0,
            center: (0.5, 0.5),
        }
    }
}

/// The slide showing now.
#[derive(Clone)]
pub struct Slide {
    /// Which slide, and which version of it: a new one is a cut, not a movement.
    pub id: u64,
    pub width: u32,
    pub height: u32,
    /// Its picture, top row first, BGRA.
    pub bgra: Arc<Vec<u8>>,
    pub background: Rgb,
    /// Small images are enlarged to fill (VID-074; off by default).
    pub enlarge: bool,
    pub view: View,
}

impl std::fmt::Debug for Slide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Slide")
            .field("id", &self.id)
            .field("size", &(self.width, self.height))
            .field("view", &self.view)
            .finish()
    }
}

/// Where a recording gets the slide showing now.
pub type SlideFeed = Arc<dyn Fn() -> Option<Slide> + Send + Sync>;

/// A recording of slides instead of the screen (VID-071).
#[derive(Clone)]
pub struct Source(pub SlideFeed);

impl std::fmt::Debug for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("slides")
    }
}

/// The image's place in a `w` × `h` video for `view`.
pub fn place(w: f32, h: f32, iw: f32, ih: f32, enlarge: bool, view: View) -> Rect {
    let fit = (w / iw).min(h / ih);
    // Bigger than the video: smaller until it fits; smaller: as it is, unless enlarged.
    let base = if fit < 1.0 || enlarge { fit } else { 1.0 };
    let s = base * view.zoom.clamp(1.0, 8.0);
    let (dw, dh) = (iw * s, ih * s);
    Rect {
        x: w / 2.0 - view.center.0 * dw,
        y: h / 2.0 - view.center.1 * dh,
        w: dw,
        h: dh,
    }
}

/// How long a change of zoom takes in the video (VID-074: a smooth movement).
pub const GLIDE: Duration = Duration::from_millis(300);

/// The view `t` of the way (0 to 1) from `a` to `b`, eased in and out.
pub fn glide(a: View, b: View, t: f32) -> View {
    let t = t.clamp(0.0, 1.0);
    let e = t * t * (3.0 - 2.0 * t);
    let mix = |x: f32, y: f32| x + (y - x) * e;
    View {
        zoom: mix(a.zoom, b.zoom),
        center: (mix(a.center.0, b.center.0), mix(a.center.1, b.center.1)),
    }
}

/// The video sizes offered (VID-074): 1920 × 1080 first.
pub const SIZES: [(u32, u32); 6] = [
    (1920, 1080),
    (1280, 720),
    (2560, 1440),
    (3840, 2160),
    (1080, 1080),
    (1080, 1920),
];

/// A custom size, made even and held between 320 and 3840 (VID-074).
pub fn custom_size(w: u32, h: u32) -> (u32, u32) {
    let even = |v: u32| (v.clamp(320, 3840) / 2) * 2;
    (even(w), even(h))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_slide_is_shown_whole_and_centred() {
        // 800 × 600 in 1920 × 1080: as it is, in the middle.
        let r = place(1920.0, 1080.0, 800.0, 600.0, false, View::default());
        assert_eq!(
            r,
            Rect {
                x: 560.0,
                y: 240.0,
                w: 800.0,
                h: 600.0
            }
        );
        // 2560 × 1600 in 1280 × 720: down to 1152 × 720, centred.
        let r = place(1280.0, 720.0, 2560.0, 1600.0, false, View::default());
        assert_eq!((r.w, r.h), (1152.0, 720.0));
        assert_eq!((r.x, r.y), (64.0, 0.0));
        // Enlarged to fill, when asked.
        let r = place(1920.0, 1080.0, 800.0, 600.0, true, View::default());
        assert_eq!((r.w, r.h), (1440.0, 1080.0));
    }

    #[test]
    fn a_zoom_brings_a_point_to_the_middle_and_glides_there() {
        let near = View {
            zoom: 2.0,
            center: (0.25, 0.25),
        };
        let r = place(1920.0, 1080.0, 800.0, 600.0, false, near);
        assert_eq!((r.w, r.h), (1600.0, 1200.0));
        // The image's point (200, 150) is at the video's middle.
        assert_eq!((r.x + 0.25 * r.w, r.y + 0.25 * r.h), (960.0, 540.0));
        let start = View::default();
        assert_eq!(glide(start, near, 0.0), start);
        assert_eq!(glide(start, near, 1.0), near);
        let half = glide(start, near, 0.5);
        assert!((half.zoom - 1.5).abs() < 1e-6);
    }

    #[test]
    fn a_custom_size_is_even_and_held() {
        assert_eq!(custom_size(1001, 99), (1000, 320));
        assert_eq!(custom_size(5000, 2161), (3840, 2160));
    }
}
