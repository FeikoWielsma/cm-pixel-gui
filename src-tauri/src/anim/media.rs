use crate::canvas::Canvas;
use crate::anim::Animation;
use std::fs::File;
use image::AnimationDecoder;
use image::codecs::gif::GifDecoder;

pub struct StaticImage {
    img: image::RgbImage,
    zoom: f32,
    pan_x: f32,
    pan_y: f32,
    canvas: Canvas,
}

impl StaticImage {
    pub fn new(path: &str, zoom: f32, pan_x: f32, pan_y: f32) -> anyhow::Result<Self> {
        let img = image::open(path)?.to_rgb8();
        let mut this = Self {
            img,
            zoom,
            pan_x,
            pan_y,
            canvas: Canvas::black(),
        };
        this.recalculate_canvas();
        Ok(this)
    }

    fn recalculate_canvas(&mut self) {
        let (w, h) = self.img.dimensions();
        let s_base = w.min(h) as f32;
        let zoom = self.zoom.max(1.0);
        let s_crop = s_base / zoom;

        let half_crop = s_crop / 2.0;
        let min_cx = half_crop;
        let max_cx = w as f32 - half_crop;
        let min_cy = half_crop;
        let max_cy = h as f32 - half_crop;

        // self.pan_x and self.pan_y are normalized offsets (-0.5 to 0.5) from center
        let cx = (w as f32 / 2.0 - self.pan_x * s_base).clamp(min_cx, max_cx);
        let cy = (h as f32 / 2.0 - self.pan_y * s_base).clamp(min_cy, max_cy);

        let x_start = (cx - half_crop).max(0.0) as u32;
        let y_start = (cy - half_crop).max(0.0) as u32;
        let crop_size = s_crop.round() as u32;

        let crop_w = crop_size.min(w - x_start).max(1);
        let crop_h = crop_size.min(h - y_start).max(1);

        let cropped = image::imageops::crop_imm(&self.img, x_start, y_start, crop_w, crop_h).to_image();
        let resized = image::DynamicImage::ImageRgb8(cropped)
            .resize_exact(32, 32, image::imageops::FilterType::Nearest);
        let rgb_img = resized.to_rgb8();

        for y in 0..32 {
            for x in 0..32 {
                let p = rgb_img.get_pixel(x as u32, y as u32);
                self.canvas.set(x, y, [p[0], p[1], p[2]]);
            }
        }
    }
}

impl Animation for StaticImage {
    fn render(&mut self, _t: f32, _dt: f32) -> Canvas {
        self.canvas.clone()
    }

    fn update_params(&mut self, zoom: f32, pan_x: f32, pan_y: f32) {
        if self.zoom != zoom || self.pan_x != pan_x || self.pan_y != pan_y {
            self.zoom = zoom;
            self.pan_x = pan_x;
            self.pan_y = pan_y;
            self.recalculate_canvas();
        }
    }
}

pub struct GifFrame {
    img: image::RgbImage,
    delay_secs: f32,
    canvas: Canvas,
}

pub struct GifPlayer {
    frames: Vec<GifFrame>,
    current_frame: usize,
    accumulated_time: f32,
    speed: f32,
    zoom: f32,
    pan_x: f32,
    pan_y: f32,
}

impl GifPlayer {
    pub fn new(path: &str, speed: f32, zoom: f32, pan_x: f32, pan_y: f32) -> anyhow::Result<Self> {
        let file = File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let decoder = GifDecoder::new(reader)?;
        let decoded_frames = decoder.into_frames().collect_frames()?;

        let mut frames = Vec::new();
        for frame in decoded_frames {
            let (num, denom) = frame.delay().numer_denom_ms();
            let mut delay_secs = if denom == 0 {
                0.1
            } else {
                (num as f32 / denom as f32) / 1000.0
            };
            if delay_secs <= 0.0 {
                delay_secs = 0.1;
            }

            let rgba_img = frame.into_buffer();
            let dynamic_img = image::DynamicImage::ImageRgba8(rgba_img);
            let rgb_img = dynamic_img.to_rgb8();

            frames.push(GifFrame {
                img: rgb_img,
                delay_secs,
                canvas: Canvas::black(),
            });
        }

        if frames.is_empty() {
            return Err(anyhow::anyhow!("GIF contains no frames"));
        }

        let mut player = Self {
            frames,
            current_frame: 0,
            accumulated_time: 0.0,
            speed,
            zoom,
            pan_x,
            pan_y,
        };
        player.recalculate_canvases();
        Ok(player)
    }

    fn recalculate_canvases(&mut self) {
        for frame in &mut self.frames {
            let (w, h) = frame.img.dimensions();
            let s_base = w.min(h) as f32;
            let zoom = self.zoom.max(1.0);
            let s_crop = s_base / zoom;

            let half_crop = s_crop / 2.0;
            let min_cx = half_crop;
            let max_cx = w as f32 - half_crop;
            let min_cy = half_crop;
            let max_cy = h as f32 - half_crop;

            let cx = (w as f32 / 2.0 - self.pan_x * s_base).clamp(min_cx, max_cx);
            let cy = (h as f32 / 2.0 - self.pan_y * s_base).clamp(min_cy, max_cy);

            let x_start = (cx - half_crop).max(0.0) as u32;
            let y_start = (cy - half_crop).max(0.0) as u32;
            let crop_size = s_crop.round() as u32;

            let crop_w = crop_size.min(w - x_start).max(1);
            let crop_h = crop_size.min(h - y_start).max(1);

            let cropped = image::imageops::crop_imm(&frame.img, x_start, y_start, crop_w, crop_h).to_image();
            let resized = image::DynamicImage::ImageRgb8(cropped)
                .resize_exact(32, 32, image::imageops::FilterType::Nearest);
            let rgb_img = resized.to_rgb8();

            for y in 0..32 {
                for x in 0..32 {
                    let p = rgb_img.get_pixel(x as u32, y as u32);
                    frame.canvas.set(x, y, [p[0], p[1], p[2]]);
                }
            }
        }
    }
}

impl Animation for GifPlayer {
    fn render(&mut self, _t: f32, dt: f32) -> Canvas {
        if self.frames.is_empty() {
            return Canvas::black();
        }

        self.accumulated_time += dt * self.speed;

        while self.accumulated_time >= self.frames[self.current_frame].delay_secs {
            self.accumulated_time -= self.frames[self.current_frame].delay_secs;
            self.current_frame = (self.current_frame + 1) % self.frames.len();
        }

        self.frames[self.current_frame].canvas.clone()
    }

    fn update_params(&mut self, zoom: f32, pan_x: f32, pan_y: f32) {
        if self.zoom != zoom || self.pan_x != pan_x || self.pan_y != pan_y {
            self.zoom = zoom;
            self.pan_x = pan_x;
            self.pan_y = pan_y;
            self.recalculate_canvases();
        }
    }
}
