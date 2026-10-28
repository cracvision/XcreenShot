//! Pixel effects (EDT-040). They work on premultiplied RGBA, row by row.

/// Blurs with three box passes per direction (close to a Gaussian of `radius`).
pub fn blur(data: &mut [u8], width: usize, height: usize, radius: usize) {
    if radius == 0 || width == 0 || height == 0 {
        return;
    }
    let mut tmp = vec![0u8; data.len()];
    for _ in 0..3 {
        box_pass(data, &mut tmp, width, height, radius, true);
        box_pass(&tmp, data, width, height, radius, false);
    }
}

/// One box blur along rows (`horizontal`) or columns, edges clamped.
fn box_pass(src: &[u8], dst: &mut [u8], width: usize, height: usize, r: usize, horizontal: bool) {
    let (lines, len) = if horizontal {
        (height, width)
    } else {
        (width, height)
    };
    let at = |line: usize, i: usize| {
        if horizontal {
            (line * width + i) * 4
        } else {
            (i * width + line) * 4
        }
    };
    let span = (2 * r + 1) as u32;
    for line in 0..lines {
        let mut sum = [0u32; 4];
        for k in 0..=2 * r {
            let i = (k as isize - r as isize).clamp(0, len as isize - 1) as usize;
            let p = at(line, i);
            for c in 0..4 {
                sum[c] += src[p + c] as u32;
            }
        }
        for i in 0..len {
            let p = at(line, i);
            for c in 0..4 {
                dst[p + c] = ((sum[c] + span / 2) / span) as u8;
            }
            let out = at(line, (i as isize - r as isize).max(0) as usize);
            let inn = at(line, (i + r + 1).min(len - 1));
            for c in 0..4 {
                sum[c] = sum[c] + src[inn + c] as u32 - src[out + c] as u32;
            }
        }
    }
}

/// Replaces each `block` × `block` square with its average color. Squares are aligned
/// to (`origin_x`, `origin_y`) in the buffer's coordinates.
pub fn pixelate(data: &mut [u8], width: usize, height: usize, block: usize, origin_x: i64, origin_y: i64) {
    let block = block.max(1);
    let start = |origin: i64| -(origin.rem_euclid(block as i64)) as isize;
    let mut y0 = start(origin_y);
    while y0 < height as isize {
        let mut x0 = start(origin_x);
        while x0 < width as isize {
            let ys = y0.max(0) as usize..((y0 + block as isize) as usize).min(height);
            let xs = x0.max(0) as usize..((x0 + block as isize) as usize).min(width);
            let mut sum = [0u64; 4];
            let mut n = 0u64;
            for y in ys.clone() {
                for x in xs.clone() {
                    let p = (y * width + x) * 4;
                    for c in 0..4 {
                        sum[c] += data[p + c] as u64;
                    }
                    n += 1;
                }
            }
            if n > 0 {
                let avg = sum.map(|s| ((s + n / 2) / n) as u8);
                for y in ys.clone() {
                    for x in xs.clone() {
                        let p = (y * width + x) * 4;
                        data[p..p + 4].copy_from_slice(&avg);
                    }
                }
            }
            x0 += block as isize;
        }
        y0 += block as isize;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checker(w: usize, h: usize) -> Vec<u8> {
        let mut v = Vec::with_capacity(w * h * 4);
        for y in 0..h {
            for x in 0..w {
                let on = (x + y) % 2 == 0;
                let c = if on { 255 } else { 0 };
                v.extend_from_slice(&[c, c, c, 255]);
            }
        }
        v
    }

    #[test]
    fn blur_averages_fine_detail_away() {
        let mut v = checker(40, 40);
        blur(&mut v, 40, 40, 4);
        let p = (20 * 40 + 20) * 4;
        assert!((v[p] as i32 - 128).abs() < 10, "{}", v[p]);
        assert_eq!(v[p + 3], 255);
    }

    #[test]
    fn pixelate_makes_flat_blocks() {
        let mut v = checker(16, 16);
        pixelate(&mut v, 16, 16, 8, 0, 0);
        let px = |x: usize, y: usize| v[(y * 16 + x) * 4];
        assert_eq!(px(0, 0), px(7, 7));
        assert!((px(3, 4) as i32 - 128).abs() <= 1);
    }
}
