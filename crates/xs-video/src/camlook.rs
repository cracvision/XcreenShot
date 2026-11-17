//! Where and how the camera shows in a recording (VID-032 to VID-036): in a corner of
//! the area (picture in picture) or filling the video, its size, shape, proportion,
//! mirror and border. The geometry here is the same for the video and the preview.

use std::sync::Arc;
use std::time::Instant;

use crate::effects::Rgb;

/// A frame of the camera, top row first, BGRA.
#[derive(Debug)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub bgra: Vec<u8>,
    pub at: Instant,
}

/// Where a recording gets the camera's latest frame (the camera read for the preview,
/// or pictures made up in the tests).
pub type Feed = Arc<dyn Fn() -> Option<Arc<Frame>> + Send + Sync>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Placement {
    /// In a corner, over the screen (VID-032).
    #[default]
    Pip,
    /// Filling the video, without the screen (VID-034).
    Fullscreen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    #[default]
    BottomRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Shape {
    Circle,
    #[default]
    RoundedRect,
    Rect,
    Square,
    RoundedSquare,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Border {
    #[default]
    None,
    Solid,
    /// From the first colour to the second, at `angle` degrees (0: left to right).
    Gradient,
}

/// The camera's look; each part can change while recording.
#[derive(Debug, Clone, PartialEq)]
pub struct Look {
    pub placement: Placement,
    pub corner: Corner,
    /// Its shorter side, as a share of the area's shorter side: 0.10 to 0.50 (VID-032).
    pub size: f32,
    pub shape: Shape,
    /// 16:9 (true) or 4:3, for the rectangles (VID-033).
    pub wide: bool,
    /// Left and right swapped, as in a mirror (VID-036).
    pub mirror: bool,
    pub border: Border,
    pub border_colors: (Rgb, Rgb),
    pub border_angle: f32,
    /// In logical pixels: 2 to 16 (VID-036).
    pub border_width: f32,
}

impl Default for Look {
    fn default() -> Self {
        Look {
            placement: Placement::Pip,
            corner: Corner::BottomRight,
            size: 0.25,
            shape: Shape::RoundedRect,
            wide: true,
            mirror: true,
            border: Border::None,
            border_colors: ([255, 255, 255], [47, 107, 255]),
            border_angle: 0.0,
            border_width: 4.0,
        }
    }
}

/// The margin between the camera and the area's edges, in logical pixels (VID-032).
pub const MARGIN: f32 = 24.0;
/// The sizes of the menu: Small, Medium and Large (VID-032).
pub const SIZES: [f32; 3] = [0.15, 0.25, 0.40];

/// A box: left, top, width and height.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Look {
    /// The camera's width over its height.
    pub fn ratio(&self) -> f32 {
        match self.shape {
            Shape::Circle | Shape::Square | Shape::RoundedSquare => 1.0,
            Shape::Rect | Shape::RoundedRect if self.wide => 16.0 / 9.0,
            Shape::Rect | Shape::RoundedRect => 4.0 / 3.0,
        }
    }

    /// Where the camera goes in an area of `w` × `h` pixels, `scale` pixels to a
    /// logical one: the whole area at full screen, else its corner with the margin.
    pub fn rect(&self, w: f32, h: f32, scale: f32) -> Rect {
        if self.placement == Placement::Fullscreen {
            return Rect { x: 0.0, y: 0.0, w, h };
        }
        let margin = MARGIN * scale;
        let short = w.min(h) * self.size.clamp(0.10, 0.50);
        let ratio = self.ratio();
        let (mut cw, mut ch) = if ratio >= 1.0 {
            (short * ratio, short)
        } else {
            (short, short / ratio)
        };
        // It never leaves the area, margins kept.
        let room = ((w - 2.0 * margin) / cw)
            .min((h - 2.0 * margin) / ch)
            .clamp(0.1, 1.0);
        cw *= room;
        ch *= room;
        let (left, top) = (margin, margin);
        let (right, bottom) = (w - margin - cw, h - margin - ch);
        let (x, y) = match self.corner {
            Corner::TopLeft => (left, top),
            Corner::TopRight => (right, top),
            Corner::BottomLeft => (left, bottom),
            Corner::BottomRight => (right, bottom),
        };
        Rect { x, y, w: cw, h: ch }
    }

    /// The radius of the corners of `r`, for its shape.
    pub fn radius(&self, r: Rect) -> f32 {
        match self.shape {
            Shape::Circle => r.w.min(r.h) / 2.0,
            Shape::RoundedRect | Shape::RoundedSquare if self.placement == Placement::Pip => {
                r.w.min(r.h) * 0.16
            }
            _ => 0.0,
        }
    }

    /// The border is drawn: only in a corner (VID-036).
    pub fn has_border(&self) -> bool {
        self.placement == Placement::Pip && self.border != Border::None && self.border_width > 0.0
    }
}

/// The part of a `sw` × `sh` picture that fills a `dw` × `dh` box without stretching:
/// cut evenly from both sides (VID-033, VID-034).
pub fn cover(sw: f32, sh: f32, dw: f32, dh: f32) -> Rect {
    let (sr, dr) = (sw / sh, dw / dh);
    if sr > dr {
        let w = sh * dr;
        Rect {
            x: (sw - w) / 2.0,
            y: 0.0,
            w,
            h: sh,
        }
    } else {
        let h = sw / dr;
        Rect {
            x: 0.0,
            y: (sh - h) / 2.0,
            w: sw,
            h,
        }
    }
}

/// The corner nearest to a point of an area (VID-032: let go, the camera snaps there).
pub fn nearest_corner(x: f32, y: f32, w: f32, h: f32) -> Corner {
    match (x < w / 2.0, y < h / 2.0) {
        (true, true) => Corner::TopLeft,
        (false, true) => Corner::TopRight,
        (true, false) => Corner::BottomLeft,
        (false, false) => Corner::BottomRight,
    }
}

/// The settings' words for these, and back.
impl Corner {
    pub fn parse(s: &str) -> Corner {
        match s {
            "topLeft" => Corner::TopLeft,
            "topRight" => Corner::TopRight,
            "bottomLeft" => Corner::BottomLeft,
            _ => Corner::BottomRight,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Corner::TopLeft => "topLeft",
            Corner::TopRight => "topRight",
            Corner::BottomLeft => "bottomLeft",
            Corner::BottomRight => "bottomRight",
        }
    }
}

impl Shape {
    pub fn parse(s: &str) -> Shape {
        match s {
            "circle" => Shape::Circle,
            "rect" => Shape::Rect,
            "square" => Shape::Square,
            "roundedSquare" => Shape::RoundedSquare,
            _ => Shape::RoundedRect,
        }
    }
}

impl Border {
    pub fn parse(s: &str) -> Border {
        match s {
            "solid" => Border::Solid,
            "gradient" => Border::Gradient,
            _ => Border::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_camera_sits_in_its_corner_with_the_margin() {
        let look = Look::default();
        // A 1920 × 1080 area at 100 %: a quarter of 1080 high, 16:9, bottom right.
        let r = look.rect(1920.0, 1080.0, 1.0);
        assert_eq!(r.h, 270.0);
        assert!((r.w - 480.0).abs() < 0.01);
        assert_eq!((r.x + r.w, r.y + r.h), (1920.0 - 24.0, 1080.0 - 24.0));
        let tl = Look {
            corner: Corner::TopLeft,
            ..look.clone()
        }
        .rect(1920.0, 1080.0, 1.5);
        assert_eq!((tl.x, tl.y), (36.0, 36.0), "the margin follows the scale");
        // Sizes are held between 10 % and 50 %.
        let big = Look {
            size: 0.9,
            shape: Shape::Circle,
            ..look.clone()
        }
        .rect(1000.0, 1000.0, 1.0);
        assert_eq!((big.w, big.h), (500.0, 500.0));
        // At full screen it is the whole area.
        let full = Look {
            placement: Placement::Fullscreen,
            ..look.clone()
        };
        assert_eq!(
            full.rect(1280.0, 720.0, 1.0),
            Rect {
                x: 0.0,
                y: 0.0,
                w: 1280.0,
                h: 720.0
            }
        );
        assert!(!full.has_border());
    }

    #[test]
    fn a_4_3_camera_keeps_its_proportion_and_is_not_stretched() {
        let look = Look {
            wide: false,
            shape: Shape::Rect,
            ..Look::default()
        };
        let r = look.rect(1920.0, 1080.0, 1.0);
        assert!((r.w / r.h - 4.0 / 3.0).abs() < 0.01);
        // A 16:9 camera in a 4:3 box: cut evenly at the sides.
        let c = cover(1280.0, 720.0, r.w, r.h);
        assert_eq!((c.y, c.h), (0.0, 720.0));
        assert!((c.w - 960.0).abs() < 0.01 && (c.x - 160.0).abs() < 0.01);
        // In a 16:9 box: all of it.
        assert_eq!(
            cover(1280.0, 720.0, 480.0, 270.0),
            Rect {
                x: 0.0,
                y: 0.0,
                w: 1280.0,
                h: 720.0
            }
        );
    }

    #[test]
    fn let_go_it_snaps_to_the_nearest_corner() {
        assert_eq!(nearest_corner(100.0, 80.0, 1920.0, 1080.0), Corner::TopLeft);
        assert_eq!(nearest_corner(1800.0, 900.0, 1920.0, 1080.0), Corner::BottomRight);
        assert_eq!(Corner::parse(Corner::TopRight.name()), Corner::TopRight);
    }
}
