//! Geometry and compositing of multi-monitor captures (CAP-021). Pure functions, so
//! they are tested without a screen.

use image::RgbaImage;
use serde::Serialize;

/// Rectangle in physical pixels on the virtual desktop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub fn right(&self) -> i32 {
        self.x + self.width as i32
    }

    pub fn bottom(&self) -> i32 {
        self.y + self.height as i32
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
    }

    pub fn intersect(&self, other: &Rect) -> Option<Rect> {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());
        (right > x && bottom > y).then(|| Rect {
            x,
            y,
            width: (right - x) as u32,
            height: (bottom - y) as u32,
        })
    }

    pub fn area(&self) -> u64 {
        self.width as u64 * self.height as u64
    }

    /// Smallest rectangle containing all of `rects`.
    pub fn bounding(rects: &[Rect]) -> Option<Rect> {
        let first = rects.first()?;
        let (mut x, mut y, mut r, mut b) = (first.x, first.y, first.right(), first.bottom());
        for rect in &rects[1..] {
            x = x.min(rect.x);
            y = y.min(rect.y);
            r = r.max(rect.right());
            b = b.max(rect.bottom());
        }
        Some(Rect {
            x,
            y,
            width: (r - x) as u32,
            height: (b - y) as u32,
        })
    }
}

/// A piece of a capture: the pixels of `rect` (desktop coordinates).
pub struct Piece {
    pub rect: Rect,
    pub image: RgbaImage,
}

/// Places pieces into one image covering `area`. Uncovered pixels stay fully
/// transparent (gaps between monitors of different sizes, CAP-021).
pub fn compose(area: Rect, pieces: &[Piece]) -> RgbaImage {
    let mut out = RgbaImage::new(area.width, area.height);
    for piece in pieces {
        let Some(visible) = piece.rect.intersect(&area) else {
            continue;
        };
        for dy in 0..visible.height {
            for dx in 0..visible.width {
                let sx = (visible.x - piece.rect.x) as u32 + dx;
                let sy = (visible.y - piece.rect.y) as u32 + dy;
                if sx < piece.image.width() && sy < piece.image.height() {
                    let px = *piece.image.get_pixel(sx, sy);
                    out.put_pixel(
                        (visible.x - area.x) as u32 + dx,
                        (visible.y - area.y) as u32 + dy,
                        px,
                    );
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn solid(w: u32, h: u32, c: [u8; 4]) -> RgbaImage {
        RgbaImage::from_pixel(w, h, Rgba(c))
    }

    #[test]
    fn intersection_and_bounds() {
        let a = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 100,
        };
        let b = Rect {
            x: 50,
            y: 50,
            width: 100,
            height: 100,
        };
        assert_eq!(
            a.intersect(&b),
            Some(Rect {
                x: 50,
                y: 50,
                width: 50,
                height: 50
            })
        );
        let c = Rect {
            x: 100,
            y: 0,
            width: 10,
            height: 10,
        };
        assert_eq!(a.intersect(&c), None);
        assert_eq!(
            Rect::bounding(&[
                a,
                Rect {
                    x: -20,
                    y: 10,
                    width: 10,
                    height: 200
                }
            ]),
            Some(Rect {
                x: -20,
                y: 0,
                width: 120,
                height: 210
            })
        );
    }

    /// CAP-021: the whole desktop covers the bounding box, gaps are transparent.
    #[test]
    fn all_monitors_with_gap_is_transparent() {
        let left = Rect {
            x: 0,
            y: 0,
            width: 4,
            height: 4,
        };
        let right = Rect {
            x: 4,
            y: 0,
            width: 2,
            height: 2,
        };
        let area = Rect::bounding(&[left, right]).unwrap();
        let img = compose(
            area,
            &[
                Piece {
                    rect: left,
                    image: solid(4, 4, [255, 0, 0, 255]),
                },
                Piece {
                    rect: right,
                    image: solid(2, 2, [0, 0, 255, 255]),
                },
            ],
        );
        assert_eq!((img.width(), img.height()), (6, 4));
        assert_eq!(img.get_pixel(5, 1).0, [0, 0, 255, 255]);
        assert_eq!(
            img.get_pixel(5, 3).0[3],
            0,
            "gap below the small monitor is transparent"
        );
    }

    /// CAP-021: a region crossing two monitors is one continuous image.
    #[test]
    fn region_across_monitors_is_continuous() {
        let left = Rect {
            x: 0,
            y: 0,
            width: 4,
            height: 4,
        };
        let right = Rect {
            x: 4,
            y: 0,
            width: 4,
            height: 4,
        };
        let region = Rect {
            x: 2,
            y: 1,
            width: 4,
            height: 2,
        };
        let img = compose(
            region,
            &[
                Piece {
                    rect: left,
                    image: solid(4, 4, [255, 0, 0, 255]),
                },
                Piece {
                    rect: right,
                    image: solid(4, 4, [0, 255, 0, 255]),
                },
            ],
        );
        assert_eq!((img.width(), img.height()), (4, 2));
        assert_eq!(img.get_pixel(1, 0).0, [255, 0, 0, 255]);
        assert_eq!(img.get_pixel(2, 0).0, [0, 255, 0, 255]);
        assert!(img.pixels().all(|p| p.0[3] == 255));
    }
}
