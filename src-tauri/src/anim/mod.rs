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
];

pub fn list() -> &'static [AnimInfo] {
    ANIM_IDS
}

/// Construct an animation by id. Returns `None` if id is unknown.
/// Populated by T2 (procedural), T3 (rubik), T4 (text/media).
pub fn make(_id: &str, _speed: f32) -> Option<Box<dyn Animation>> {
    None
}
