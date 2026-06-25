use crate::canvas::Canvas;
use crate::anim::Animation;
use rand::seq::SliceRandom;

/// Helper: hsv_to_rgb for scalar h, s, v in [0, 1] -> [u8; 3]
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [u8; 3] {
    let h = h.rem_euclid(1.0) * 6.0;
    let i = h.floor() as i32;
    let f = h - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);

    let (r, g, b) = match i % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        5 => (v, p, q),
        _ => (0.0, 0.0, 0.0),
    };

    [
        (r.clamp(0.0, 1.0) * 255.0).round() as u8,
        (g.clamp(0.0, 1.0) * 255.0).round() as u8,
        (b.clamp(0.0, 1.0) * 255.0).round() as u8,
    ]
}

pub fn sample_palette_cyclic(colors: &[[u8; 3]], t: f32) -> [u8; 3] {
    let t = t.rem_euclid(1.0); // Wrap around 0..1
    let n = colors.len();
    if n == 0 {
        return [0, 0, 0];
    }
    if n == 1 {
        return colors[0];
    }
    let scaled = t * n as f32;
    let idx = scaled.floor() as usize;
    let fract = scaled - idx as f32;
    let c1 = colors[idx % n];
    let c2 = colors[(idx + 1) % n];
    [
        (c1[0] as f32 * (1.0 - fract) + c2[0] as f32 * fract) as u8,
        (c1[1] as f32 * (1.0 - fract) + c2[1] as f32 * fract) as u8,
        (c1[2] as f32 * (1.0 - fract) + c2[2] as f32 * fract) as u8,
    ]
}

pub fn evaluate_palette(name: &str, t: f32) -> [u8; 3] {
    match name {
        "catppuccin" => {
            let colors = &[
                [198, 160, 246], // Mauve
                [138, 173, 244], // Blue
                [139, 213, 202], // Teal
                [166, 218, 149], // Green
                [238, 212, 159], // Yellow
                [245, 169, 127], // Peach
                [237, 135, 150], // Red
                [245, 189, 230], // Pink
            ];
            sample_palette_cyclic(colors, t)
        }
        "everforest" => {
            let colors = &[
                [167, 192, 128],
                [218, 223, 172],
                [245, 122, 85],
                [219, 190, 106],
                [127, 187, 179],
                [230, 126, 124],
            ];
            sample_palette_cyclic(colors, t)
        }
        "nord" => {
            let colors = &[
                [143, 188, 187],
                [136, 192, 208],
                [129, 161, 193],
                [94, 129, 172],
                [191, 97, 106],
                [163, 190, 140],
                [235, 203, 139],
            ];
            sample_palette_cyclic(colors, t)
        }
        "dracula" => {
            let colors = &[
                [189, 147, 249],
                [255, 121, 198],
                [139, 233, 253],
                [80, 250, 123],
                [255, 184, 108],
                [241, 250, 140],
                [255, 85, 85],
            ];
            sample_palette_cyclic(colors, t)
        }
        "sunset" => {
            let colors = &[
                [114, 9, 183],
                [181, 23, 158],
                [247, 37, 133],
                [252, 76, 2],
                [255, 186, 8],
            ];
            sample_palette_cyclic(colors, t)
        }
        "cyberpunk" => {
            let colors = &[
                [255, 220, 0],  // neon yellow
                [255, 180, 0],  // amber/gold
                [0, 200, 255],  // ice blue
                [0, 255, 240],  // cyan
                [255, 240, 100], // pale yellow
            ];
            sample_palette_cyclic(colors, t)
        }
        "toxic" => {
            let colors = &[
                [0, 40, 0],
                [0, 120, 20],
                [0, 220, 50],
                [100, 255, 80],
                [200, 255, 180],
            ];
            sample_palette_cyclic(colors, t)
        }
        "ice" => {
            let colors = &[
                [0, 30, 100],
                [0, 100, 200],
                [0, 200, 255],
                [100, 255, 255],
                [200, 255, 255],
            ];
            sample_palette_cyclic(colors, t)
        }
        "ice_fire" => {
            let colors = &[
                [0, 30, 120],
                [0, 100, 255],
                [150, 0, 150],
                [255, 50, 0],
                [255, 210, 100],
            ];
            sample_palette_cyclic(colors, t)
        }
        "magma" => {
            let colors = &[
                [50, 0, 0],
                [180, 20, 0],
                [255, 100, 0],
                [255, 210, 0],
                [255, 255, 180],
            ];
            sample_palette_cyclic(colors, t)
        }
        "synthwave" => {
            let colors = &[
                [43, 0, 128],
                [255, 0, 128],
                [0, 242, 254],
                [255, 224, 0],
            ];
            sample_palette_cyclic(colors, t)
        }
        "forest" => {
            let colors = &[
                [20, 50, 20],
                [40, 100, 50],
                [100, 160, 80],
                [200, 210, 120],
                [230, 240, 190],
            ];
            sample_palette_cyclic(colors, t)
        }
        _ => {
            // "rainbow" or default
            hsv_to_rgb(t, 1.0, 1.0)
        }
    }
}

// ---------------------------------------------------------
// 1. Plasma
// ---------------------------------------------------------
pub struct Plasma {
    speed: f32,
    palette: String,
    t: f32,
}

impl Plasma {
    pub fn new(speed: f32, palette: String) -> Self {
        Self { speed, palette, t: 0.0 }
    }
}

impl Animation for Plasma {
    fn render(&mut self, _t: f32, dt: f32) -> Canvas {
        self.t += dt * 15.0 * 0.15 * self.speed;
        let mut canvas = Canvas::black();
        let cx = 15.5f32;
        let cy = 15.5f32;
        for y in 0..32 {
            let dy = y as f32 - cy;
            for x in 0..32 {
                let dx = x as f32 - cx;
                let r = (dx * dx + dy * dy).sqrt();
                let v = (x as f32 / 4.0 + self.t).sin()
                    + (y as f32 / 3.0 - self.t).sin()
                    + ((x as f32 + y as f32) / 4.0 + self.t).sin()
                    + (r / 3.0 - self.t).sin();
                let h = (v + 4.0) / 8.0;
                canvas.set(x, y, evaluate_palette(&self.palette, h));
            }
        }
        canvas
    }
}

// ---------------------------------------------------------
// 2. Rainbow
// ---------------------------------------------------------
pub struct Rainbow {
    speed: f32,
    angle: f32,
    palette: String,
    t: f32,
}

impl Rainbow {
    pub fn new(speed: f32, angle: f32, palette: String) -> Self {
        Self { speed, angle, palette, t: 0.0 }
    }
}

impl Animation for Rainbow {
    fn render(&mut self, _t: f32, dt: f32) -> Canvas {
        self.t += dt * 15.0 * 0.02 * self.speed;
        let mut canvas = Canvas::black();
        let rad = self.angle * std::f32::consts::PI / 180.0;
        let dx = rad.cos();
        let dy = rad.sin();
        let cx = 15.5f32;
        let cy = 15.5f32;
        for y in 0..32 {
            let y_diff = y as f32 - cy;
            for x in 0..32 {
                let x_diff = x as f32 - cx;
                let base = x_diff * dx + y_diff * dy;
                let h = base / 22.0 + self.t;
                canvas.set(x, y, evaluate_palette(&self.palette, h));
            }
        }
        canvas
    }
}

// ---------------------------------------------------------
// 3. Swirl
// ---------------------------------------------------------
pub struct Swirl {
    speed: f32,
    palette: String,
    t: f32,
}

impl Swirl {
    pub fn new(speed: f32, palette: String) -> Self {
        Self { speed, palette, t: 0.0 }
    }
}

impl Animation for Swirl {
    fn render(&mut self, _t: f32, dt: f32) -> Canvas {
        self.t += dt * 15.0 * 0.05 * self.speed;
        let mut canvas = Canvas::black();
        let cx = 15.5f32;
        let cy = 15.5f32;
        for y in 0..32 {
            let dy = y as f32 - cy;
            for x in 0..32 {
                let dx = x as f32 - cx;
                let r = (dx * dx + dy * dy).sqrt();
                let ang = dy.atan2(dx);
                let h = (ang / (2.0 * std::f32::consts::PI)) + r / 16.0 + self.t;
                canvas.set(x, y, evaluate_palette(&self.palette, h));
            }
        }
        canvas
    }
}

// ---------------------------------------------------------
// 4. Ripple
// ---------------------------------------------------------
pub struct Ripple {
    speed: f32,
    palette: String,
    t: f32,
}

impl Ripple {
    pub fn new(speed: f32, palette: String) -> Self {
        Self { speed, palette, t: 0.0 }
    }
}

impl Animation for Ripple {
    fn render(&mut self, _t: f32, dt: f32) -> Canvas {
        self.t += dt * 15.0 * 0.3 * self.speed;
        let mut canvas = Canvas::black();
        let cx = 15.5f32;
        let cy = 15.5f32;
        for y in 0..32 {
            let dy = y as f32 - cy;
            for x in 0..32 {
                let dx = x as f32 - cx;
                let r = (dx * dx + dy * dy).sqrt();
                let wave = ((r - self.t).sin() + 1.0) / 2.0;
                let h = r / 16.0 - self.t / 6.0;
                let base_col = evaluate_palette(&self.palette, h);
                let c = [
                    (base_col[0] as f32 * wave).clamp(0.0, 255.0) as u8,
                    (base_col[1] as f32 * wave).clamp(0.0, 255.0) as u8,
                    (base_col[2] as f32 * wave).clamp(0.0, 255.0) as u8,
                ];
                canvas.set(x, y, c);
            }
        }
        canvas
    }
}

// ---------------------------------------------------------
// 5. Breathe
// ---------------------------------------------------------
pub struct Breathe {
    speed: f32,
    palette: String,
    fixed_color: [u8; 3],
    t: f32,
}

impl Breathe {
    pub fn new(speed: f32, palette: String, fixed_color: [u8; 3]) -> Self {
        Self {
            speed,
            palette,
            fixed_color,
            t: 0.0,
        }
    }
}

impl Animation for Breathe {
    fn render(&mut self, _t: f32, dt: f32) -> Canvas {
        self.t += dt * 15.0 * 0.08 * self.speed;
        let v = (self.t.sin() + 1.0) / 2.0;

        let base_col = if self.palette == "fixed" {
            self.fixed_color
        } else {
            let hue = (self.t * 0.1).rem_euclid(1.0);
            evaluate_palette(&self.palette, hue)
        };

        let c = [
            (base_col[0] as f32 * v).clamp(0.0, 255.0) as u8,
            (base_col[1] as f32 * v).clamp(0.0, 255.0) as u8,
            (base_col[2] as f32 * v).clamp(0.0, 255.0) as u8,
        ];
        let mut canvas = Canvas::black();
        canvas.fill(c);
        canvas
    }
}

// ---------------------------------------------------------
// 6. Sparkle
// ---------------------------------------------------------
pub struct Sparkle {
    speed: f32,
    color: [f32; 3],
    fade: f32,
    rate: usize,
    buf: [[[f32; 3]; 32]; 32],
    accumulator: f32,
}

impl Sparkle {
    pub fn new(speed: f32, color: [u8; 3], fade: f32, rate: usize) -> Self {
        Self {
            speed,
            color: [color[0] as f32, color[1] as f32, color[2] as f32],
            fade,
            rate,
            buf: [[[0.0; 3]; 32]; 32],
            accumulator: 0.0,
        }
    }
}

impl Animation for Sparkle {
    fn render(&mut self, _t: f32, dt: f32) -> Canvas {
        self.accumulator += dt * 15.0;
        let limit = 10;
        let mut steps = 0;
        let mut rng = rand::thread_rng();
        let leds = crate::layout::active_leds();

        while self.accumulator >= 1.0 && steps < limit {
            // Apply fade
            for y in 0..32 {
                for x in 0..32 {
                    for c in 0..3 {
                        self.buf[y][x][c] *= self.fade;
                    }
                }
            }

            // Add sparkles
            if !leds.is_empty() {
                let num_sparks = (self.rate as f32 * self.speed) as usize;
                for _ in 0..num_sparks {
                    if let Some(&(x, y)) = leds.choose(&mut rng) {
                        self.buf[y][x] = self.color;
                    }
                }
            }

            self.accumulator -= 1.0;
            steps += 1;
        }

        // Convert to canvas
        let mut canvas = Canvas::black();
        for y in 0..32 {
            for x in 0..32 {
                let r = self.buf[y][x][0].clamp(0.0, 255.0) as u8;
                let g = self.buf[y][x][1].clamp(0.0, 255.0) as u8;
                let b = self.buf[y][x][2].clamp(0.0, 255.0) as u8;
                canvas.set(x, y, [r, g, b]);
            }
        }
        canvas
    }
}

// ---------------------------------------------------------
// 7. Fire
// ---------------------------------------------------------
pub struct Fire {
    speed: f32,
    intensity: f32,
    heat: [[f32; 32]; 32],
    palette: [[u8; 3]; 256],
    accumulator: f32,
}

impl Fire {
    pub fn new(speed: f32, intensity: f32, palette_name: String) -> Self {
        let stops = match palette_name.as_str() {
            "ice" | "nord" => [
                [0.0, 0.0, 0.0],
                [0.0, 50.0, 150.0],
                [0.0, 140.0, 255.0],
                [0.0, 255.0, 255.0],
                [180.0, 255.0, 255.0],
            ],
            "ice_fire" | "synthwave" => [
                [0.0, 0.0, 0.0],
                [0.0, 60.0, 180.0],
                [110.0, 0.0, 160.0],
                [255.0, 50.0, 0.0],
                [255.0, 210.0, 100.0],
            ],
            "cyberpunk" => [
                [0.0, 0.0, 0.0],
                [180.0, 140.0, 0.0],
                [255.0, 220.0, 0.0],
                [0.0, 180.0, 255.0],
                [180.0, 255.0, 255.0],
            ],
            "toxic" | "green" | "everforest" | "forest" => [
                [0.0, 0.0, 0.0],
                [0.0, 100.0, 0.0],
                [0.0, 220.0, 50.0],
                [120.0, 255.0, 100.0],
                [200.0, 255.0, 180.0],
            ],
            "dracula" | "purple" => [
                [0.0, 0.0, 0.0],
                [60.0, 0.0, 120.0],
                [160.0, 0.0, 220.0],
                [255.0, 80.0, 180.0],
                [255.0, 220.0, 255.0],
            ],
            "sunset" | "catppuccin" => [
                [0.0, 0.0, 0.0],
                [100.0, 0.0, 80.0],
                [200.0, 30.0, 0.0],
                [255.0, 150.0, 0.0],
                [255.0, 230.0, 150.0],
            ],
            _ => [ // "classic", "magma", "rainbow", default
                [0.0, 0.0, 0.0],
                [120.0, 0.0, 0.0],
                [255.0, 80.0, 0.0],
                [255.0, 190.0, 0.0],
                [255.0, 255.0, 180.0],
            ],
        };
        let mut palette = [[0u8; 3]; 256];
        for i in 0..256 {
            let t = (i as f32 / 255.0) * 4.0;
            let idx = (t.floor() as usize).min(3);
            let frac = t - idx as f32;
            let c0 = stops[idx];
            let c1 = stops[idx + 1];
            palette[i] = [
                (c0[0] + (c1[0] - c0[0]) * frac).clamp(0.0, 255.0).round() as u8,
                (c0[1] + (c1[1] - c0[1]) * frac).clamp(0.0, 255.0).round() as u8,
                (c0[2] + (c1[2] - c0[2]) * frac).clamp(0.0, 255.0).round() as u8,
            ];
        }
        Self {
            speed,
            intensity,
            heat: [[0.0; 32]; 32],
            palette,
            accumulator: 0.0,
        }
    }
}

impl Animation for Fire {
    fn render(&mut self, _t: f32, dt: f32) -> Canvas {
        self.accumulator += dt * 15.0 * self.speed;
        let limit = 10;
        let mut steps = 0;
        let mut rng = rand::thread_rng();
        use rand::Rng;

        while self.accumulator >= 1.0 && steps < limit {
            let mut next_heat = [[0.0f32; 32]; 32];

            // Smooth rows 0–30 upward from the row below (no wrap)
            for y in 0..31 {
                let y_up = y + 1;
                for x in 0..32 {
                    let x_left = (x + 31) % 32;
                    let x_right = (x + 1) % 32;

                    let mut val = self.heat[y_up][x] * 0.42
                        + self.heat[y][x] * 0.30
                        + self.heat[y_up][x_left] * 0.14
                        + self.heat[y_up][x_right] * 0.14;

                    // Decay: low at bottom (y≈30), high at top (y≈0)
                    let y_factor = y as f32 / 30.0;
                    let decay = rng.gen_range(1.5..3.8) + (1.0 - y_factor) * 4.8;
                    val = (val - decay).max(0.0);

                    next_heat[y][x] = val;
                }
            }

            // Seed the bottom row with fresh sparks after smoothing so they are never smoothed away
            for x in 0..32 {
                let r: f32 = rng.gen();
                let bias = r.powf(0.35);
                next_heat[31][x] = (bias * 255.0 * self.intensity).clamp(0.0, 255.0);
            }

            self.heat = next_heat;
            self.accumulator -= 1.0;
            steps += 1;
        }

        let mut canvas = Canvas::black();
        for y in 0..32 {
            for x in 0..32 {
                let heat_val = self.heat[y][x];
                let idx = (heat_val as usize).min(255);
                canvas.set(x, y, self.palette[idx]);
            }
        }
        canvas
    }
}

// ---------------------------------------------------------
// 8. Starfield
// ---------------------------------------------------------
#[derive(Clone, Copy)]
pub struct Star {
    x: f32,
    y: f32,
    z: f32,
}

pub struct Starfield {
    speed: f32,
    density: f32,
    trail_len: f32,
    stars: Vec<Star>,
    accumulator: f32,
}

impl Starfield {
    pub fn new(speed: f32, density: f32, trail_len: f32) -> Self {
        Self {
            speed,
            density: density * 1.5, // increase density slightly for better warp visual
            trail_len,
            stars: Vec::new(),
            accumulator: 0.0,
        }
    }
}

impl Animation for Starfield {
    fn render(&mut self, _t: f32, dt: f32) -> Canvas {
        self.accumulator += dt * 15.0;
        let limit = 10;
        let mut steps = 0;
        let mut rng = rand::thread_rng();
        use rand::Rng;

        let cx = 15.5f32;
        let cy = 15.5f32;

        while self.accumulator >= 1.0 && steps < limit {
            // Spawn new stars
            if rng.gen::<f32>() < self.density * 8.0 {
                self.stars.push(Star {
                    x: rng.gen_range(-cx..cx),
                    y: rng.gen_range(-cy..cy),
                    z: 16.0,
                });
            }

            // Move and filter stars (warp: move faster and closer to screen)
            let mut alive = Vec::with_capacity(self.stars.len());
            for mut star in self.stars.drain(..) {
                star.z -= 0.8 * self.speed;
                if star.z > 0.4 {
                    alive.push(star);
                }
            }

            // Keep at most 200 stars
            if alive.len() > 200 {
                let start_idx = alive.len() - 200;
                self.stars = alive[start_idx..].to_vec();
            } else {
                self.stars = alive;
            }

            self.accumulator -= 1.0;
            steps += 1;
        }

        // Draw to canvas as streaking lines radiating from center
        let mut canvas = Canvas::black();
        for star in &self.stars {
            let streak_len = self.trail_len * self.speed;
            let steps = 6;
            for i in 0..steps {
                let z_val = star.z + (i as f32 / steps as f32) * streak_len;
                let px = (cx + star.x * 8.0 / z_val) as i32;
                let py = (cy + star.y * 8.0 / z_val) as i32;
                if px >= 0 && px < 32 && py >= 0 && py < 32 {
                    let alpha = 1.0 - (i as f32 / steps as f32);
                    let intensity = (255.0 * (1.0 - star.z / 16.0) * alpha).clamp(0.0, 255.0) as u8;

                    let existing = canvas.px[py as usize][px as usize];
                    let r = existing[0].max(intensity);
                    let g = existing[1].max(intensity);
                    let b = existing[2].max(intensity);
                    canvas.set(px as usize, py as usize, [r, g, b]);
                }
            }
        }

        canvas
    }
}

// ---------------------------------------------------------
// 9. Tunnel
// ---------------------------------------------------------
pub struct Tunnel {
    speed: f32,
    palette: String,
    t: f32,
}

impl Tunnel {
    pub fn new(speed: f32, palette: String) -> Self {
        Self { speed, palette, t: 0.0 }
    }
}

impl Animation for Tunnel {
    fn render(&mut self, _t: f32, dt: f32) -> Canvas {
        self.t += dt * 15.0 * 0.05 * self.speed;
        let mut canvas = Canvas::black();
        let cx = 15.5f32;
        let cy = 15.5f32;
        for y in 0..32 {
            let dy = y as f32 - cy;
            for x in 0..32 {
                let dx = x as f32 - cx;
                let r = (dx * dx + dy * dy).sqrt();
                let ang = dy.atan2(dx);

                let v_val = 1.0 / (r + 0.6);
                let u_val = ang / (2.0 * std::f32::consts::PI);
                let vig = 1.0 - (r / (32.0 * 0.6)).clamp(0.0, 1.0);

                let stripes = 0.5 + 0.5 * (v_val * 18.0 - self.t * 4.0).sin();
                let hue = (v_val * 1.5 + u_val + self.t * 0.05).rem_euclid(1.0);
                let val = stripes * (0.3 + 0.7 * vig);

                let base_col = evaluate_palette(&self.palette, hue);
                let c = [
                    (base_col[0] as f32 * val).clamp(0.0, 255.0) as u8,
                    (base_col[1] as f32 * val).clamp(0.0, 255.0) as u8,
                    (base_col[2] as f32 * val).clamp(0.0, 255.0) as u8,
                ];
                canvas.set(x, y, c);
            }
        }
        canvas
    }
}

// ---------------------------------------------------------
// 10. Metaballs
// ---------------------------------------------------------
pub struct Metaballs {
    speed: f32,
    n: usize,
    ball_size: f32,
    palette: String,
    ph: Vec<[f32; 2]>,
    t: f32,
}

impl Metaballs {
    pub fn new(speed: f32, n: usize, ball_size: f32, palette: String) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut ph = Vec::with_capacity(n);
        for _ in 0..n {
            ph.push([
                rng.gen::<f32>() * 2.0 * std::f32::consts::PI,
                rng.gen::<f32>() * 2.0 * std::f32::consts::PI,
            ]);
        }
        Self { speed, n, ball_size, palette, ph, t: 0.0 }
    }
}

impl Animation for Metaballs {
    fn render(&mut self, _t: f32, dt: f32) -> Canvas {
        self.t += dt * 15.0 * 0.05 * self.speed;
        let mut canvas = Canvas::black();

        // Calculate center for each metaball
        let mut centers = Vec::with_capacity(self.n);
        for i in 0..self.n {
            let cx = 16.0 + 32.0 * 0.34 * (self.t * 0.6 + self.ph[i][0]).sin();
            let cy = 16.0 + 32.0 * 0.34 * (self.t * 0.5 + self.ph[i][1]).cos();
            centers.push((cx, cy));
        }

        for y in 0..32 {
            for x in 0..32 {
                let mut field = 0.0f32;
                for i in 0..self.n {
                    let cx = centers[i].0;
                    let cy = centers[i].1;
                    let dx = x as f32 - cx;
                    let dy = y as f32 - cy;
                    field += (30.0 * self.ball_size) / (dx * dx + dy * dy + 8.0);
                }
                let hue = (field * 0.3 + self.t * 0.05).rem_euclid(1.0);
                let val = (field * 0.9).clamp(0.0, 1.0);
                let base_col = evaluate_palette(&self.palette, hue);
                let c = [
                    (base_col[0] as f32 * val).clamp(0.0, 255.0) as u8,
                    (base_col[1] as f32 * val).clamp(0.0, 255.0) as u8,
                    (base_col[2] as f32 * val).clamp(0.0, 255.0) as u8,
                ];
                canvas.set(x, y, c);
            }
        }

        canvas
    }
}

// ---------------------------------------------------------
// 11. Matrix
// ---------------------------------------------------------
pub struct Matrix {
    speed: f32,
    fade: f32,
    buf: [[[f32; 3]; 32]; 32],
    heads: [f32; 32],
    spd: [f32; 32],
    active: [bool; 32],
    accumulator: f32,
}

impl Matrix {
    pub fn new(speed: f32, fade: f32) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut spd = [0.0f32; 32];
        let mut active = [false; 32];
        for x in 0..32 {
            spd[x] = 0.3 + rng.gen::<f32>() * 0.7;
            active[x] = rng.gen::<f32>() < 0.4;
        }
        Self {
            speed,
            fade,
            buf: [[[0.0; 3]; 32]; 32],
            heads: [0.0; 32],
            spd,
            active,
            accumulator: 0.0,
        }
    }
}

impl Animation for Matrix {
    fn render(&mut self, _t: f32, dt: f32) -> Canvas {
        self.accumulator += dt * 15.0;
        let limit = 10;
        let mut steps = 0;
        let mut rng = rand::thread_rng();
        use rand::Rng;

        while self.accumulator >= 1.0 && steps < limit {
            // Apply fade
            for y in 0..32 {
                for x in 0..32 {
                    self.buf[y][x][0] *= self.fade;
                    self.buf[y][x][1] *= self.fade;
                    self.buf[y][x][2] *= self.fade;
                }
            }

            for x in 0..32 {
                if !self.active[x] {
                    if rng.gen::<f32>() < 0.03 {
                        self.active[x] = true;
                        self.heads[x] = 0.0;
                    }
                    continue;
                }

                let y = self.heads[x] as i32;
                if y >= 0 && y < 32 {
                    self.buf[y as usize][x] = [170.0, 255.0, 170.0];
                }

                self.heads[x] += self.spd[x] * self.speed;
                if self.heads[x] > 36.0 { // GRID_H + 4
                    self.active[x] = false;
                }
            }

            self.accumulator -= 1.0;
            steps += 1;
        }

        let mut canvas = Canvas::black();
        for y in 0..32 {
            for x in 0..32 {
                let r = self.buf[y][x][0].clamp(0.0, 255.0) as u8;
                let g = self.buf[y][x][1].clamp(0.0, 255.0) as u8;
                let b = self.buf[y][x][2].clamp(0.0, 255.0) as u8;
                canvas.set(x, y, [r, g, b]);
            }
        }
        canvas
    }
}

// ---------------------------------------------------------
// 12. Fireworks
// ---------------------------------------------------------
pub struct FireworkParticle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    color: [f32; 3],
}

pub struct Fireworks {
    speed: f32,
    gravity: f32,
    palette: String,
    parts: Vec<FireworkParticle>,
    buf: [[[f32; 3]; 32]; 32],
    accumulator: f32,
}

impl Fireworks {
    pub fn new(speed: f32, gravity: f32, palette: String) -> Self {
        Self {
            speed,
            gravity,
            palette,
            parts: Vec::new(),
            buf: [[[0.0; 3]; 32]; 32],
            accumulator: 0.0,
        }
    }
}

impl Animation for Fireworks {
    fn render(&mut self, _t: f32, dt: f32) -> Canvas {
        self.accumulator += dt * 15.0;
        let limit = 10;
        let mut steps = 0;
        let mut rng = rand::thread_rng();
        use rand::Rng;

        while self.accumulator >= 1.0 && steps < limit {
            // Apply fade
            for y in 0..32 {
                for x in 0..32 {
                    self.buf[y][x][0] *= 0.82;
                    self.buf[y][x][1] *= 0.82;
                    self.buf[y][x][2] *= 0.82;
                }
            }

            // Maybe spawn new explosion
            if rng.gen::<f32>() < 0.06 * self.speed || self.parts.is_empty() {
                let cx = rng.gen_range(8.0..24.0);
                let cy = rng.gen_range(6.0..18.0);
                let hue = rng.gen::<f32>();
                let rgb_u8 = evaluate_palette(&self.palette, hue);
                let col = [rgb_u8[0] as f32, rgb_u8[1] as f32, rgb_u8[2] as f32];
                let n = rng.gen_range(18..30);
                for k in 0..n {
                    let a = 2.0 * std::f32::consts::PI * k as f32 / n as f32;
                    let sp = rng.gen_range(0.4..1.2);
                    self.parts.push(FireworkParticle {
                        x: cx,
                        y: cy,
                        vx: a.cos() * sp,
                        vy: a.sin() * sp,
                        life: 1.0,
                        color: col,
                    });
                }
            }

            // Move and update particles
            let mut alive = Vec::with_capacity(self.parts.len());
            for mut p in self.parts.drain(..) {
                p.x += p.vx * self.speed;
                p.y += p.vy * self.speed;
                p.vy += self.gravity * self.speed;
                p.life -= 0.02 * self.speed;

                if p.life > 0.0 {
                    let xi = p.x.round() as i32;
                    let yi = p.y.round() as i32;
                    if xi >= 0 && xi < 32 && yi >= 0 && yi < 32 {
                        let y_idx = yi as usize;
                        let x_idx = xi as usize;
                        self.buf[y_idx][x_idx][0] = p.color[0] * p.life;
                        self.buf[y_idx][x_idx][1] = p.color[1] * p.life;
                        self.buf[y_idx][x_idx][2] = p.color[2] * p.life;
                    }
                    alive.push(p);
                }
            }
            self.parts = alive;

            self.accumulator -= 1.0;
            steps += 1;
        }

        let mut canvas = Canvas::black();
        for y in 0..32 {
            for x in 0..32 {
                let r = self.buf[y][x][0].clamp(0.0, 255.0) as u8;
                let g = self.buf[y][x][1].clamp(0.0, 255.0) as u8;
                let b = self.buf[y][x][2].clamp(0.0, 255.0) as u8;
                canvas.set(x, y, [r, g, b]);
            }
        }
        canvas
    }
}

// ---------------------------------------------------------
// 13. Aurora
// ---------------------------------------------------------
pub struct Aurora {
    speed: f32,
    size: f32,
    t: f32,
}

impl Aurora {
    pub fn new(speed: f32, size: f32) -> Self {
        Self { speed, size, t: 0.0 }
    }
}

impl Animation for Aurora {
    fn render(&mut self, _t: f32, dt: f32) -> Canvas {
        self.t += dt * 15.0 * 0.06 * self.speed;
        let mut canvas = Canvas::black();
        let width = 18.0 * self.size;

        for y in 0..32 {
            let y_f = y as f32;
            for x in 0..32 {
                let x_f = x as f32;
                let center = 16.0 + 6.0 * (x_f * 0.3 + self.t).sin() + 3.0 * (x_f * 0.7 - self.t * 1.3).sin();
                let dy = y_f - center;
                let val = (-(dy * dy) / width).exp().clamp(0.0, 1.0);
                let hue = (0.33 + 0.18 * (x_f * 0.2 + self.t * 0.5).sin() + y_f * 0.012).rem_euclid(1.0);
                canvas.set(x, y, hsv_to_rgb(hue, 0.85, val));
            }
        }
        canvas
    }
}

// ---------------------------------------------------------
// 14. Comet
// ---------------------------------------------------------
pub struct Comet {
    speed: f32,
    buf: [[[f32; 3]; 32]; 32],
    t: f32,
    accumulator: f32,
}

impl Comet {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            buf: [[[0.0; 3]; 32]; 32],
            t: 0.0,
            accumulator: 0.0,
        }
    }
}

impl Animation for Comet {
    fn render(&mut self, _t: f32, dt: f32) -> Canvas {
        self.accumulator += dt * 15.0;
        let limit = 10;
        let mut steps = 0;

        while self.accumulator >= 1.0 && steps < limit {
            // Apply fade
            for y in 0..32 {
                for x in 0..32 {
                    self.buf[y][x][0] *= 0.80;
                    self.buf[y][x][1] *= 0.80;
                    self.buf[y][x][2] *= 0.80;
                }
            }

            let r = 32.0 * 0.32;
            let cx = 16.0f32;
            let cy = 16.0f32;
            let px = cx + r * self.t.cos();
            let py = cy + r * self.t.sin();

            let hue = (self.t * 0.1).rem_euclid(1.0);
            let col = hsv_to_rgb(hue, 1.0, 1.0);

            let r_head = 2.2f32;
            let x_start = (px - r_head).floor() as i32;
            let x_end = (px + r_head).ceil() as i32;
            let y_start = (py - r_head).floor() as i32;
            let y_end = (py + r_head).ceil() as i32;
            for y_curr in y_start..=y_end {
                for x_curr in x_start..=x_end {
                    if x_curr >= 0 && x_curr < 32 && y_curr >= 0 && y_curr < 32 {
                        let dx = x_curr as f32 - px;
                        let dy = y_curr as f32 - py;
                        let dist = (dx * dx + dy * dy).sqrt();
                        if dist <= r_head {
                            let intensity = (1.0 - (dist / r_head).powf(1.8)).clamp(0.0, 1.0);
                            let y_idx = y_curr as usize;
                            let x_idx = x_curr as usize;
                            self.buf[y_idx][x_idx][0] = self.buf[y_idx][x_idx][0].max(col[0] as f32 * intensity);
                            self.buf[y_idx][x_idx][1] = self.buf[y_idx][x_idx][1].max(col[1] as f32 * intensity);
                            self.buf[y_idx][x_idx][2] = self.buf[y_idx][x_idx][2].max(col[2] as f32 * intensity);
                        }
                    }
                }
            }

            self.t += 0.15 * self.speed;
            self.accumulator -= 1.0;
            steps += 1;
        }

        let mut canvas = Canvas::black();
        for y in 0..32 {
            for x in 0..32 {
                let r = self.buf[y][x][0].clamp(0.0, 255.0) as u8;
                let g = self.buf[y][x][1].clamp(0.0, 255.0) as u8;
                let b = self.buf[y][x][2].clamp(0.0, 255.0) as u8;
                canvas.set(x, y, [r, g, b]);
            }
        }
        canvas
    }
}

// ---------------------------------------------------------
// 15. Pinwheel
// ---------------------------------------------------------
pub struct Pinwheel {
    speed: f32,
    arms: f32,
    palette: String,
    t: f32,
}

impl Pinwheel {
    pub fn new(speed: f32, arms: usize, palette: String) -> Self {
        Self {
            speed,
            arms: arms as f32,
            palette,
            t: 0.0,
        }
    }
}

impl Animation for Pinwheel {
    fn render(&mut self, _t: f32, dt: f32) -> Canvas {
        self.t += dt * 15.0 * 0.05 * self.speed;
        let mut canvas = Canvas::black();
        let cx = 15.5f32;
        let cy = 15.5f32;

        for y in 0..32 {
            let dy = y as f32 - cy;
            for x in 0..32 {
                let dx = x as f32 - cx;
                let r = (dx * dx + dy * dy).sqrt();
                let ang = dy.atan2(dx);

                let val = 0.5 + 0.5 * (ang * self.arms + r * 0.5 - self.t * 3.0).sin();
                let hue = (ang / (2.0 * std::f32::consts::PI) + self.t * 0.1).rem_euclid(1.0);

                let base_col = evaluate_palette(&self.palette, hue);
                let v = val.powf(1.5);
                let c = [
                    (base_col[0] as f32 * v).clamp(0.0, 255.0) as u8,
                    (base_col[1] as f32 * v).clamp(0.0, 255.0) as u8,
                    (base_col[2] as f32 * v).clamp(0.0, 255.0) as u8,
                ];
                canvas.set(x, y, c);
            }
        }

        canvas
    }
}

// ---------------------------------------------------------
// 16. Bounce
// ---------------------------------------------------------
pub struct Bounce {
    speed: f32,
    palette: String,
    ball_size: f32,
    buf: [[[f32; 3]; 32]; 32],
    pos: [f32; 2],
    vel: [f32; 2],
    t: f32,
    accumulator: f32,
}

impl Bounce {
    pub fn new(speed: f32, ball_size: f32, palette: String) -> Self {
        Self {
            speed,
            palette,
            ball_size,
            buf: [[[0.0; 3]; 32]; 32],
            pos: [16.0, 16.0],
            vel: [0.7, 0.5],
            t: 0.0,
            accumulator: 0.0,
        }
    }
}

impl Animation for Bounce {
    fn render(&mut self, _t: f32, dt: f32) -> Canvas {
        self.accumulator += dt * 15.0;
        let limit = 10;
        let mut steps = 0;

        let layout_data = crate::layout::layout();

        while self.accumulator >= 1.0 && steps < limit {
            // Apply fade
            for y in 0..32 {
                for x in 0..32 {
                    self.buf[y][x][0] *= 0.75;
                    self.buf[y][x][1] *= 0.75;
                    self.buf[y][x][2] *= 0.75;
                }
            }

            // 1. Move ball
            self.pos[0] += self.vel[0] * self.speed;
            self.pos[1] += self.vel[1] * self.speed;

            // 2. Check collision at the new position
            let cx = self.pos[0];
            let cy = self.pos[1];

            let mut nx = 0.0f32;
            let mut ny = 0.0f32;
            let mut count = 0;

            // Check the ball area for outside-hexagon pixels
            let r_sq = self.ball_size * self.ball_size;
            let min_x = (cx - self.ball_size).floor() as i32;
            let max_x = (cx + self.ball_size).ceil() as i32;
            let min_y = (cy - self.ball_size).floor() as i32;
            let max_y = (cy + self.ball_size).ceil() as i32;

            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    let dx = x as f32 + 0.5 - cx;
                    let dy = y as f32 + 0.5 - cy;
                    if dx * dx + dy * dy <= r_sq {
                        let is_outside = if x < 0 || x >= 32 || y < 0 || y >= 32 {
                            true
                        } else {
                            layout_data[(y * 32 + x) as usize] == 0
                        };
                        if is_outside {
                            // Vector from outside pixel to ball center points away from the wall
                            nx += -dx;
                            ny += -dy;
                            count += 1;
                        }
                    }
                }
            }

            if count > 0 {
                // Collided! Reflect velocity relative to normal (nx, ny)
                let len = (nx*nx + ny*ny).sqrt();
                if len > 0.001 {
                    let nx = nx / len;
                    let ny = ny / len;
                    
                    let dot = self.vel[0] * nx + self.vel[1] * ny;
                    if dot < 0.0 {
                        // Reflect: v = v - 2 * (v . n) * n
                        self.vel[0] = self.vel[0] - 2.0 * dot * nx;
                        self.vel[1] = self.vel[1] - 2.0 * dot * ny;
                        
                        // Normalize velocity to keep speed constant (magnitude = 0.86, derived from original [0.7, 0.5])
                        let v_len = (self.vel[0]*self.vel[0] + self.vel[1]*self.vel[1]).sqrt();
                        if v_len > 0.001 {
                            self.vel[0] = (self.vel[0] / v_len) * 0.86;
                            self.vel[1] = (self.vel[1] / v_len) * 0.86;
                        }

                        // Add a tiny random angle perturbation to prevent infinite periodic orbit locks
                        use rand::Rng;
                        let mut rng = rand::thread_rng();
                        let angle: f32 = rng.gen_range(-0.15..0.15);
                        let c = angle.cos();
                        let s = angle.sin();
                        let vx = self.vel[0];
                        let vy = self.vel[1];
                        self.vel[0] = vx * c - vy * s;
                        self.vel[1] = vx * s + vy * c;
                    }
                }

                // Push out of wall (towards center) by a small step to prevent getting stuck
                let dx = 15.5 - self.pos[0];
                let dy = 15.5 - self.pos[1];
                let dist = (dx*dx + dy*dy).sqrt();
                if dist > 0.01 {
                    self.pos[0] += (dx / dist) * 0.6;
                    self.pos[1] += (dy / dist) * 0.6;
                } else {
                    self.pos = [15.5, 15.5];
                }
            }

            let cx = self.pos[0];
            let cy = self.pos[1];
            let r_sq = self.ball_size * self.ball_size;
            let min_x = (cx - self.ball_size).floor() as i32;
            let max_x = (cx + self.ball_size).ceil() as i32;
            let min_y = (cy - self.ball_size).floor() as i32;
            let max_y = (cy + self.ball_size).ceil() as i32;

            let hue = (self.t * 0.02).rem_euclid(1.0);
            let col = evaluate_palette(&self.palette, hue);

            let mut lit_any = false;
            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    if x >= 0 && x < 32 && y >= 0 && y < 32 {
                        let dx = x as f32 + 0.5 - cx;
                        let dy = y as f32 + 0.5 - cy;
                        if dx * dx + dy * dy <= r_sq {
                            self.buf[y as usize][x as usize] = [col[0] as f32, col[1] as f32, col[2] as f32];
                            lit_any = true;
                        }
                    }
                }
            }

            if !lit_any {
                let px = cx.round() as i32;
                let py = cy.round() as i32;
                if px >= 0 && px < 32 && py >= 0 && py < 32 {
                    self.buf[py as usize][px as usize] = [col[0] as f32, col[1] as f32, col[2] as f32];
                }
            }

            self.t += self.speed;
            self.accumulator -= 1.0;
            steps += 1;
        }

        let mut canvas = Canvas::black();
        for y in 0..32 {
            for x in 0..32 {
                let r = self.buf[y][x][0].clamp(0.0, 255.0) as u8;
                let g = self.buf[y][x][1].clamp(0.0, 255.0) as u8;
                let b = self.buf[y][x][2].clamp(0.0, 255.0) as u8;
                canvas.set(x, y, [r, g, b]);
            }
        }
        canvas
    }
}

// ---------------------------------------------------------
// 17. Interference
// ---------------------------------------------------------
pub struct Interference {
    speed: f32,
    palette: String,
    t: f32,
}

impl Interference {
    pub fn new(speed: f32, palette: String) -> Self {
        Self { speed, palette, t: 0.0 }
    }
}

impl Animation for Interference {
    fn render(&mut self, _t: f32, dt: f32) -> Canvas {
        self.t += dt * 15.0 * 0.05 * self.speed;
        let mut canvas = Canvas::black();

        let c1x = 16.0 + 10.0 * (self.t * 0.7).sin();
        let c1y = 16.0 + 10.0 * (self.t * 0.5).cos();
        let c2x = 16.0 + 10.0 * (self.t * 0.4 + 2.0).sin();
        let c2y = 16.0 + 10.0 * (self.t * 0.6 + 1.0).cos();

        for y in 0..32 {
            let dy1 = y as f32 - c1y;
            let dy2 = y as f32 - c2y;
            for x in 0..32 {
                let dx1 = x as f32 - c1x;
                let dx2 = x as f32 - c2x;

                let d1 = (dx1 * dx1 + dy1 * dy1).sqrt();
                let d2 = (dx2 * dx2 + dy2 * dy2).sqrt();

                let val = ((d1 - self.t * 3.0).sin() + (d2 - self.t * 3.0).sin()) / 2.0;
                let hue = (val * 0.5 + 0.5 + self.t * 0.03).rem_euclid(1.0);

                let base_col = evaluate_palette(&self.palette, hue);
                let v = val * 0.5 + 0.5;
                let c = [
                    (base_col[0] as f32 * v).clamp(0.0, 255.0) as u8,
                    (base_col[1] as f32 * v).clamp(0.0, 255.0) as u8,
                    (base_col[2] as f32 * v).clamp(0.0, 255.0) as u8,
                ];
                canvas.set(x, y, c);
            }
        }
        canvas
    }
}

// ---------------------------------------------------------
// 18. Twinkle
// ---------------------------------------------------------
struct TwinkleStar {
    x: usize,
    y: usize,
    hue: f32,
    phase: f32,
    phase_speed: f32,
}

pub struct Twinkle {
    speed: f32,
    n: usize,
    palette: String,
    stars: Vec<TwinkleStar>,
    accumulator: f32,
}

impl Twinkle {
    pub fn new(speed: f32, n: usize, palette: String) -> Self {
        Self {
            speed,
            n,
            palette,
            stars: Vec::new(),
            accumulator: 0.0,
        }
    }
}

impl Animation for Twinkle {
    fn render(&mut self, _t: f32, dt: f32) -> Canvas {
        self.accumulator += dt * 15.0;
        let limit = 10;
        let mut steps = 0;
        let mut rng = rand::thread_rng();
        use rand::Rng;

        let leds = crate::layout::active_leds();

        while self.accumulator >= 1.0 && steps < limit {
            // Maybe spawn new star
            if self.stars.len() < self.n && rng.gen::<f32>() < 0.6 {
                if !leds.is_empty() {
                    let &(x, y) = leds.choose(&mut rng).unwrap();
                    self.stars.push(TwinkleStar {
                        x,
                        y,
                        hue: rng.gen::<f32>(),
                        phase: 0.0,
                        phase_speed: 0.03 + rng.gen::<f32>() * 0.06,
                    });
                }
            }

            // Update stars
            let mut alive = Vec::with_capacity(self.stars.len());
            for mut s in self.stars.drain(..) {
                s.phase += s.phase_speed * self.speed;
                if s.phase < std::f32::consts::PI {
                    alive.push(s);
                }
            }
            self.stars = alive;

            self.accumulator -= 1.0;
            steps += 1;
        }

        // Draw to canvas
        let mut canvas = Canvas::black();
        for s in &self.stars {
            let val = s.phase.sin();
            let base_col = evaluate_palette(&self.palette, s.hue);
            let col = [
                (base_col[0] as f32 * val).clamp(0.0, 255.0) as u8,
                (base_col[1] as f32 * val).clamp(0.0, 255.0) as u8,
                (base_col[2] as f32 * val).clamp(0.0, 255.0) as u8,
            ];
            canvas.set(s.x, s.y, col);
        }
        canvas
    }
}

// ---------------------------------------------------------
// 19. Torus (3D Donut)
// ---------------------------------------------------------
pub struct Torus {
    speed: f32,
    palette: String,
    t: f32,
}

impl Torus {
    pub fn new(speed: f32, palette: String) -> Self {
        Self {
            speed,
            palette,
            t: 0.0,
        }
    }
}

impl Animation for Torus {
    fn render(&mut self, _t: f32, dt: f32) -> Canvas {
        self.t += dt * self.speed;
        let mut canvas = Canvas::black();
        let mut z_buf = [[0.0f32; 32]; 32];

        let a = self.t * 0.8; // rotation angle A
        let b = self.t * 0.5; // rotation angle B

        let cos_a = a.cos();
        let sin_a = a.sin();
        let cos_b = b.cos();
        let sin_b = b.sin();

        let r1 = 1.0f32; // tube radius
        let r2 = 2.0f32; // ring radius
        let d = 8.0f32;  // viewer distance
        let k1 = 35.0f32; // scale

        // Iterate theta (tube section) and phi (torus ring)
        let mut theta = 0.0f32;
        while theta < 2.0 * std::f32::consts::PI {
            let cos_t = theta.cos();
            let sin_t = theta.sin();

            let mut phi = 0.0f32;
            while phi < 2.0 * std::f32::consts::PI {
                let cos_p = phi.cos();
                let sin_p = phi.sin();

                // 3D coordinates before rotation
                let x0 = r2 + r1 * cos_t;
                let y0 = r1 * sin_t;

                let x = x0 * cos_p;
                let y = x0 * sin_p;
                let z = y0;

                // Rotation around X-axis (A)
                let rx = x;
                let ry = y * cos_a - z * sin_a;
                let rz = y * sin_a + z * cos_a;

                // Rotation around Y-axis (B)
                let rxx = rx * cos_b + rz * sin_b;
                let ryy = ry;
                let rzz = -rx * sin_b + rz * cos_b;

                // Perspective projection
                let ooz = 1.0 / (rzz + d);
                let xp = (16.0 + rxx * k1 * ooz) as i32;
                let yp = (16.0 - ryy * k1 * ooz) as i32;

                // Surface normal
                let nx = cos_t * cos_p;
                let ny = cos_t * sin_p;
                let nz = sin_t;

                // Rotate normal
                let rnx = nx;
                let rny = ny * cos_a - nz * sin_a;
                let rnz = ny * sin_a + nz * cos_a;

                let rnyy = rny;
                let rnzz = -rnx * sin_b + rnz * cos_b;

                // Lighting (light source at [0, 1, -1] normalized: [0, 0.707, -0.707])
                let l = rnyy * 0.707 - rnzz * 0.707;
                if l > 0.0 {
                    if xp >= 0 && xp < 32 && yp >= 0 && yp < 32 {
                        let ux = xp as usize;
                        let uy = yp as usize;
                        if ooz > z_buf[uy][ux] {
                            z_buf[uy][ux] = ooz;
                            // Map light intensity to palette
                            let col = evaluate_palette(&self.palette, l.min(1.0));
                            canvas.set(ux, uy, col);
                        }
                    }
                }

                phi += 0.05;
            }
            theta += 0.05;
        }

        canvas
    }
}

// ---------------------------------------------------------
// 20. Julia (Morphing Julia Set Fractal)
// ---------------------------------------------------------
pub struct Julia {
    speed: f32,
    zoom: f32,
    palette: String,
    t: f32,
}

impl Julia {
    pub fn new(speed: f32, zoom: f32, palette: String) -> Self {
        Self {
            speed,
            zoom,
            palette,
            t: 0.0,
        }
    }
}

impl Animation for Julia {
    fn render(&mut self, _t: f32, dt: f32) -> Canvas {
        self.t += dt * self.speed;
        let mut canvas = Canvas::black();

        // High dimensional orbit for Julia constant c
        let cx = -0.75 + 0.12 * (self.t * 0.4).cos();
        let cy = 0.15 + 0.12 * (self.t * 0.25).sin();

        let zoom_scale = 8.0 * self.zoom.max(0.1);

        for y in 0..32 {
            for x in 0..32 {
                let mut u = (x as f32 - 15.5) / zoom_scale;
                let mut v = (y as f32 - 15.5) / zoom_scale;

                let mut iter = 0;
                let max_iter = 25;
                while iter < max_iter {
                    let u2 = u * u;
                    let v2 = v * v;
                    if u2 + v2 > 4.0 {
                        break;
                    }
                    let next_v = 2.0 * u * v + cy;
                    let next_u = u2 - v2 + cx;
                    u = next_u;
                    v = next_v;
                    iter += 1;
                }

                if iter < max_iter {
                    let hue = (iter as f32) / (max_iter as f32);
                    let col = evaluate_palette(&self.palette, hue);
                    canvas.set(x, y, col);
                } else {
                    canvas.set(x, y, [0, 0, 0]);
                }
            }
        }

        canvas
    }
}

// ---------------------------------------------------------
// 21. ChromaLife (Conway's Game of Life with Color Aging)
// ---------------------------------------------------------
pub struct ChromaLife {
    speed: f32,
    palette: String,
    grid: [[u8; 32]; 32],
    buf: [[[f32; 3]; 32]; 32],
    accumulator: f32,
    prev_population: usize,
    stable_count: usize,
}

impl ChromaLife {
    pub fn new(speed: f32, palette: String) -> Self {
        let mut s = Self {
            speed,
            palette,
            grid: [[0; 32]; 32],
            buf: [[[0.0; 3]; 32]; 32],
            accumulator: 0.0,
            prev_population: 0,
            stable_count: 0,
        };
        s.seed_random();
        s
    }

    fn seed_random(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        for y in 0..32 {
            for x in 0..32 {
                if rng.gen_bool(0.20) {
                    self.grid[y][x] = 1;
                } else {
                    self.grid[y][x] = 0;
                }
            }
        }
        self.stable_count = 0;
    }
}

impl Animation for ChromaLife {
    fn render(&mut self, _t: f32, dt: f32) -> Canvas {
        self.accumulator += dt * 8.0 * self.speed; // speed controls evolution rate
        let limit = 10;
        let mut steps = 0;

        while self.accumulator >= 1.0 && steps < limit {
            // 1. Fade the trail buffer
            for y in 0..32 {
                for x in 0..32 {
                    self.buf[y][x][0] *= 0.88;
                    self.buf[y][x][1] *= 0.88;
                    self.buf[y][x][2] *= 0.88;
                }
            }

            // 2. Compute next generation
            let mut next_grid = [[0u8; 32]; 32];
            let mut population = 0;

            for y in 0..32 {
                for x in 0..32 {
                    // Count neighbors
                    let mut neighbors = 0;
                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            if dx == 0 && dy == 0 {
                                continue;
                            }
                            let ny = (y as i32 + dy).rem_euclid(32) as usize;
                            let nx = (x as i32 + dx).rem_euclid(32) as usize;
                            if self.grid[ny][nx] > 0 {
                                neighbors += 1;
                            }
                        }
                    }

                    let curr_age = self.grid[y][x];
                    if curr_age > 0 {
                        if neighbors == 2 || neighbors == 3 {
                            next_grid[y][x] = curr_age.saturating_add(1);
                            population += 1;
                        } else {
                            next_grid[y][x] = 0; // dies
                        }
                    } else {
                        if neighbors == 3 {
                            next_grid[y][x] = 1; // born
                            population += 1;
                        } else {
                            next_grid[y][x] = 0;
                        }
                    }
                }
            }

            self.grid = next_grid;

            // 3. Draw active cells to trail buffer with age gradient
            for y in 0..32 {
                for x in 0..32 {
                    let age = self.grid[y][x];
                    if age > 0 {
                        let hue = (age as f32 * 0.05).min(1.0);
                        let col = evaluate_palette(&self.palette, hue);
                        // Newborn glows brighter/white
                        if age == 1 {
                            self.buf[y][x] = [255.0, 255.0, 255.0];
                        } else {
                            self.buf[y][x] = [col[0] as f32, col[1] as f32, col[2] as f32];
                        }
                    }
                }
            }

            // 4. Entropy check to prevent static loops
            if population == self.prev_population {
                self.stable_count += 1;
            } else {
                self.stable_count = 0;
            }
            self.prev_population = population;

            // If population dies out or gets stuck in a static state/oscillator for ~60 frames
            if population < 4 || self.stable_count > 60 {
                self.seed_random();
            }

            self.accumulator -= 1.0;
            steps += 1;
        }

        let mut canvas = Canvas::black();
        for y in 0..32 {
            for x in 0..32 {
                let r = self.buf[y][x][0].clamp(0.0, 255.0) as u8;
                let g = self.buf[y][x][1].clamp(0.0, 255.0) as u8;
                let b = self.buf[y][x][2].clamp(0.0, 255.0) as u8;
                canvas.set(x, y, [r, g, b]);
            }
        }
        canvas
    }
}
