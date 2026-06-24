pub const W: usize = 32;
pub const H: usize = 32;
pub type Rgb = [u8; 3];

#[derive(Clone)]
pub struct Canvas {
    pub px: [[Rgb; W]; H],
}

impl Canvas {
    pub fn black() -> Self {
        Canvas { px: [[[0u8; 3]; W]; H] }
    }

    pub fn set(&mut self, x: usize, y: usize, c: Rgb) {
        self.px[y][x] = c;
    }

    pub fn fill(&mut self, c: Rgb) {
        for row in &mut self.px {
            for px in row.iter_mut() {
                *px = c;
            }
        }
    }

    /// Rotate the canvas by `deg` degrees CW (0 / 90 / 180 / 270).
    /// Matches `np.rot90(arr, k=(deg/90)%4)` in the Python driver (CCW for np.rot90,
    /// but the device wiring means visual CW = np.rot90 CCW so we match Python exactly).
    pub fn rotated(&self, deg: u16) -> Canvas {
        let k = (deg as usize / 90) % 4;
        let mut out = Canvas::black();
        for y in 0..H {
            for x in 0..W {
                let (ox, oy) = match k {
                    0 => (x, y),
                    1 => (H - 1 - y, x),
                    2 => (W - 1 - x, H - 1 - y),
                    3 => (y, W - 1 - x),
                    _ => unreachable!(),
                };
                out.px[oy][ox] = self.px[y][x];
            }
        }
        out
    }

    /// Scale all channel values linearly by `pct` / 100.
    pub fn with_brightness(&self, pct: u8) -> Canvas {
        let scale = pct.min(100) as u16;
        let mut out = self.clone();
        for row in &mut out.px {
            for px in row.iter_mut() {
                px[0] = ((px[0] as u16 * scale) / 100) as u8;
                px[1] = ((px[1] as u16 * scale) / 100) as u8;
                px[2] = ((px[2] as u16 * scale) / 100) as u8;
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_roundtrip() {
        let mut c = Canvas::black();
        c.fill([255, 0, 0]);
        assert_eq!(c.px[0][0], [255, 0, 0]);
        assert_eq!(c.px[31][31], [255, 0, 0]);
    }

    #[test]
    fn brightness_half() {
        let mut c = Canvas::black();
        c.fill([200, 100, 50]);
        let d = c.with_brightness(50);
        assert_eq!(d.px[0][0], [100, 50, 25]);
    }

    #[test]
    fn rotation_180_identity() {
        let mut c = Canvas::black();
        c.set(3, 5, [1, 2, 3]);
        let r180 = c.rotated(180);
        assert_eq!(r180.px[H - 1 - 5][W - 1 - 3], [1, 2, 3]);
    }
}
