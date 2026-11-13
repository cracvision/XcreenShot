//! Freehand (CAP-032): an area of any shape, drawn on the frozen screens as a free
//! outline, a polygon or an ellipse. The image is the rectangle around the shape, with
//! the pixels outside it transparent.

use image::RgbaImage;

use crate::Rect;

/// The rectangle around `points` (desktop pixels), at least 1 × 1.
pub fn bounds(points: &[(f64, f64)]) -> Option<Rect> {
    if points.len() < 3 {
        return None;
    }
    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for &(x, y) in points {
        x0 = x0.min(x);
        y0 = y0.min(y);
        x1 = x1.max(x);
        y1 = y1.max(y);
    }
    let (x, y) = (x0.floor() as i32, y0.floor() as i32);
    Some(Rect {
        x,
        y,
        width: ((x1.ceil() as i32 - x).max(1)) as u32,
        height: ((y1.ceil() as i32 - y).max(1)) as u32,
    })
}

/// Makes the pixels of `image` (the rectangle `area` of the desktop) outside the
/// shape `points` transparent; the shape is closed back to its first point. Pixel
/// centers inside by the even-odd rule stay.
pub fn cut_out(image: &mut RgbaImage, area: Rect, points: &[(f64, f64)]) {
    let n = points.len();
    if n < 3 {
        return;
    }
    let local: Vec<(f64, f64)> = points
        .iter()
        .map(|&(x, y)| (x - area.x as f64, y - area.y as f64))
        .collect();
    let mut xs: Vec<f64> = Vec::with_capacity(16);
    for row in 0..image.height() {
        let cy = row as f64 + 0.5;
        xs.clear();
        for i in 0..n {
            let (ax, ay) = local[i];
            let (bx, by) = local[(i + 1) % n];
            if (ay <= cy && by > cy) || (by <= cy && ay > cy) {
                xs.push(ax + (cy - ay) * (bx - ax) / (by - ay));
            }
        }
        xs.sort_by(|a, b| a.total_cmp(b));
        // Pixels between pairs of crossings are inside; the rest go.
        let mut inside = vec![false; image.width() as usize];
        for pair in xs.chunks(2) {
            if let [a, b] = pair {
                let from = (a - 0.5).ceil().max(0.0) as usize;
                let to = ((b - 0.5).floor() as i64).min(image.width() as i64 - 1);
                if to >= from as i64 {
                    for cell in &mut inside[from..=to as usize] {
                        *cell = true;
                    }
                }
            }
        }
        for (col, keep) in inside.into_iter().enumerate() {
            if !keep {
                image.get_pixel_mut(col as u32, row).0[3] = 0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opaque(w: u32, h: u32) -> RgbaImage {
        RgbaImage::from_pixel(w, h, image::Rgba([10, 20, 30, 255]))
    }

    /// CAP-032: a five-click polygon gives a pentagon, transparent outside.
    #[test]
    fn a_pentagon() {
        let points: Vec<(f64, f64)> = (0..5)
            .map(|k| {
                let a = -std::f64::consts::FRAC_PI_2 + k as f64 * std::f64::consts::TAU / 5.0;
                (1000.0 + 100.0 * a.cos(), 500.0 + 100.0 * a.sin())
            })
            .collect();
        let area = bounds(&points).unwrap();
        assert!((190..=202).contains(&area.width));
        let mut img = opaque(area.width, area.height);
        cut_out(&mut img, area, &points);
        // The center stays, the corners go.
        let (cx, cy) = ((1000 - area.x) as u32, (500 - area.y) as u32);
        assert_eq!(img.get_pixel(cx, cy).0[3], 255);
        assert_eq!(img.get_pixel(0, 0).0[3], 0);
        assert_eq!(img.get_pixel(area.width - 1, area.height - 1).0[3], 0);
    }

    /// CAP-032: a star keeps its points and loses what is between them.
    #[test]
    fn a_star() {
        let points: Vec<(f64, f64)> = (0..10)
            .map(|k| {
                let a = -std::f64::consts::FRAC_PI_2 + k as f64 * std::f64::consts::PI / 5.0;
                let r = if k % 2 == 0 { 100.0 } else { 40.0 };
                (200.0 + r * a.cos(), 200.0 + r * a.sin())
            })
            .collect();
        let area = bounds(&points).unwrap();
        let mut img = opaque(area.width, area.height);
        cut_out(&mut img, area, &points);
        let at = |x: f64, y: f64| {
            img.get_pixel((x - area.x as f64) as u32, (y - area.y as f64) as u32)
                .0[3]
        };
        assert_eq!(at(200.0, 200.0), 255, "the middle");
        assert_eq!(at(200.0, 105.0), 255, "the top point");
        // Between the top point and the next one: outside.
        assert_eq!(at(240.0, 120.0), 0);
        let kept = img.pixels().filter(|p| p.0[3] == 255).count() as f64;
        let all = (area.width * area.height) as f64;
        assert!(kept / all > 0.25 && kept / all < 0.6);
    }

    #[test]
    fn too_few_points() {
        assert!(bounds(&[(0.0, 0.0), (5.0, 5.0)]).is_none());
    }
}
