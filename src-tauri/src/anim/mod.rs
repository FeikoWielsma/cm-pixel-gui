pub mod procedural;
pub mod rubik;
pub mod text;
pub mod media;

use crate::canvas::Canvas;

pub trait Animation: Send {
    /// Render the next frame. `t` = seconds since app start, `dt` = seconds since last call.
    fn render(&mut self, t: f32, dt: f32) -> Canvas;
}

#[derive(Clone, serde::Serialize)]
pub struct AnimInfo {
    pub id: &'static str,
    pub label: &'static str,
}

/// All available animation IDs (matches cm-pixel ids).
pub const ANIM_IDS: &[AnimInfo] = &[
    AnimInfo { id: "plasma",       label: "Plasma" },
    AnimInfo { id: "rainbow",      label: "Rainbow" },
    AnimInfo { id: "swirl",        label: "Swirl" },
    AnimInfo { id: "ripple",       label: "Ripple" },
    AnimInfo { id: "breathe",      label: "Breathe" },
    AnimInfo { id: "sparkle",      label: "Sparkle" },
    AnimInfo { id: "fire",         label: "Fire" },
    AnimInfo { id: "starfield",    label: "Starfield" },
    AnimInfo { id: "tunnel",       label: "Tunnel" },
    AnimInfo { id: "metaballs",    label: "Metaballs" },
    AnimInfo { id: "matrix",       label: "Matrix" },
    AnimInfo { id: "fireworks",    label: "Fireworks" },
    AnimInfo { id: "aurora",       label: "Aurora" },
    AnimInfo { id: "comet",        label: "Comet" },
    AnimInfo { id: "pinwheel",     label: "Pinwheel" },
    AnimInfo { id: "bounce",       label: "Bounce" },
    AnimInfo { id: "interference", label: "Interference" },
    AnimInfo { id: "twinkle",      label: "Twinkle" },
    AnimInfo { id: "rubik",        label: "Rubik" },
    AnimInfo { id: "torus",        label: "Torus 3D" },
    AnimInfo { id: "julia",        label: "Julia Fractal" },
    AnimInfo { id: "chroma_life",  label: "Chroma Life" },
];

pub fn list() -> &'static [AnimInfo] {
    ANIM_IDS
}

/// Construct an animation by id. Returns `None` if id is unknown.
/// Populated by T2 (procedural), T3 (rubik), T4 (text/media).
pub fn make(id: &str, speed: f32, params: Option<&serde_json::Value>) -> Option<Box<dyn Animation>> {
    let palette = params
        .and_then(|p| p.get("palette"))
        .and_then(|v| v.as_str())
        .unwrap_or("rainbow")
        .to_string();

    let angle = params
        .and_then(|p| p.get("angle"))
        .and_then(|v| v.as_f64())
        .map(|v| v as f32)
        .unwrap_or(0.0);

    let intensity = params
        .and_then(|p| p.get("intensity"))
        .and_then(|v| v.as_f64())
        .map(|v| v as f32)
        .unwrap_or(1.0);

    let count = params
        .and_then(|p| p.get("count"))
        .and_then(|v| v.as_f64())
        .map(|v| v as usize);

    let ball_size = params
        .and_then(|p| p.get("ball_size"))
        .and_then(|v| v.as_f64())
        .map(|v| v as f32)
        .unwrap_or(1.0);

    let size = params
        .and_then(|p| p.get("size"))
        .and_then(|v| v.as_f64())
        .map(|v| v as f32)
        .unwrap_or(1.0);

    let angle_x = params
        .and_then(|p| p.get("angle_x"))
        .and_then(|v| v.as_f64())
        .map(|v| v as f32)
        .unwrap_or(33.0);

    let angle_y = params
        .and_then(|p| p.get("angle_y"))
        .and_then(|v| v.as_f64())
        .map(|v| v as f32)
        .unwrap_or(45.0);

    let scale = params
        .and_then(|p| p.get("scale"))
        .and_then(|v| v.as_f64())
        .map(|v| v as f32)
        .unwrap_or(6.3);

    let zoom = params
        .and_then(|p| p.get("zoom"))
        .and_then(|v| v.as_f64())
        .map(|v| v as f32)
        .unwrap_or(1.0);

    let pan_x = params
        .and_then(|p| p.get("pan_x"))
        .and_then(|v| v.as_f64())
        .map(|v| v as f32)
        .unwrap_or(0.0);

    let pan_y = params
        .and_then(|p| p.get("pan_y"))
        .and_then(|v| v.as_f64())
        .map(|v| v as f32)
        .unwrap_or(0.0);

    let mut fixed_color = [0, 120, 255];
    if let Some(c_val) = params.and_then(|p| p.get("color")) {
        if let Some(arr) = c_val.as_array() {
            if arr.len() >= 3 {
                let r = arr[0].as_f64().unwrap_or(0.0) as u8;
                let g = arr[1].as_f64().unwrap_or(120.0) as u8;
                let b = arr[2].as_f64().unwrap_or(255.0) as u8;
                fixed_color = [r, g, b];
            }
        }
    }

    match id {
        "plasma" => Some(Box::new(procedural::Plasma::new(speed, palette))),
        "rainbow" => Some(Box::new(procedural::Rainbow::new(speed, angle, palette))),
        "swirl" => Some(Box::new(procedural::Swirl::new(speed, palette))),
        "ripple" => Some(Box::new(procedural::Ripple::new(speed, palette))),
        "breathe" => Some(Box::new(procedural::Breathe::new(speed, palette, fixed_color))),
        "sparkle" => Some(Box::new(procedural::Sparkle::new(speed, [255, 255, 255], 0.85, 6))),
        "fire" => Some(Box::new(procedural::Fire::new(speed, intensity))),
        "starfield" => {
            let mut density = 0.04;
            let mut trail_len = 1.5;
            if let Some(p) = params {
                if let Some(d) = p.get("density").and_then(|v| v.as_f64()) {
                    density = d as f32;
                }
                if let Some(t) = p.get("trail_len").and_then(|v| v.as_f64()) {
                    trail_len = t as f32;
                }
            }
            Some(Box::new(procedural::Starfield::new(speed, density, trail_len)))
        }
        "tunnel" => Some(Box::new(procedural::Tunnel::new(speed, palette))),
        "metaballs" => Some(Box::new(procedural::Metaballs::new(speed, count.unwrap_or(4), ball_size, palette))),
        "matrix" => Some(Box::new(procedural::Matrix::new(speed, 0.78))),
        "fireworks" => Some(Box::new(procedural::Fireworks::new(speed, 0.05, palette))),
        "aurora" => Some(Box::new(procedural::Aurora::new(speed, size))),
        "comet" => Some(Box::new(procedural::Comet::new(speed))),
        "pinwheel" => Some(Box::new(procedural::Pinwheel::new(speed, count.unwrap_or(5), palette))),
        "bounce" => Some(Box::new(procedural::Bounce::new(speed, ball_size, palette))),
        "interference" => Some(Box::new(procedural::Interference::new(speed, palette))),
        "twinkle" => Some(Box::new(procedural::Twinkle::new(speed, count.unwrap_or(45), palette))),
        "rubik" => Some(Box::new(rubik::Rubik::new(
            speed, palette, angle_x, angle_y, scale, pan_x, pan_y,
        ))),
        "torus" => Some(Box::new(procedural::Torus::new(speed, palette))),
        "julia" => Some(Box::new(procedural::Julia::new(speed, zoom, palette))),
        "chroma_life" => Some(Box::new(procedural::ChromaLife::new(speed, palette))),
        _ => None,
    }
}
