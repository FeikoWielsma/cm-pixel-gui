use crate::canvas::Canvas;
use crate::anim::Animation;
use std::fs::File;
use image::AnimationDecoder;
use image::codecs::gif::GifDecoder;

pub struct StaticImage {
    canvas: Canvas,
}

impl StaticImage {
    pub fn new(path: &str) -> anyhow::Result<Self> {
        let img = image::open(path)?;
        let resized = img.resize_to_fill(32, 32, image::imageops::FilterType::Nearest);
        let rgb_img = resized.to_rgb8();

        let mut canvas = Canvas::black();
        for y in 0..32 {
            for x in 0..32 {
                let p = rgb_img.get_pixel(x as u32, y as u32);
                canvas.set(x, y, [p[0], p[1], p[2]]);
            }
        }

        Ok(Self { canvas })
    }
}

impl Animation for StaticImage {
    fn render(&mut self, _t: f32, _dt: f32) -> Canvas {
        self.canvas.clone()
    }
}

pub struct GifFrame {
    canvas: Canvas,
    delay_secs: f32,
}

pub struct GifPlayer {
    frames: Vec<GifFrame>,
    current_frame: usize,
    accumulated_time: f32,
    speed: f32,
}

impl GifPlayer {
    pub fn new(path: &str, speed: f32) -> anyhow::Result<Self> {
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
                delay_secs = 0.1; // fallback to 100ms
            }

            let rgba_img = frame.into_buffer();
            let dynamic_img = image::DynamicImage::ImageRgba8(rgba_img);
            let resized = dynamic_img.resize_to_fill(32, 32, image::imageops::FilterType::Nearest);
            let rgb_img = resized.to_rgb8();

            let mut canvas = Canvas::black();
            for y in 0..32 {
                for x in 0..32 {
                    let p = rgb_img.get_pixel(x as u32, y as u32);
                    canvas.set(x, y, [p[0], p[1], p[2]]);
                }
            }

            frames.push(GifFrame {
                canvas,
                delay_secs,
            });
        }

        if frames.is_empty() {
            return Err(anyhow::anyhow!("GIF contains no frames"));
        }

        Ok(Self {
            frames,
            current_frame: 0,
            accumulated_time: 0.0,
            speed,
        })
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
}
