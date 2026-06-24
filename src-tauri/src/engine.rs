use serde::{Deserialize, Serialize};

use crate::canvas::Rgb;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Mode {
    Off,
    Solid { rgb: Rgb },
    Image { path: String },
    Gif { path: String },
    Text { text: String, rgb: Rgb, scroll: bool },
    Anim { id: String, speed: f32 },
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Off
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Settings {
    pub mode: Mode,
    pub brightness: u8,
    pub rotation: u16,
    pub fps: u16,
    pub autostart: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            mode: Mode::Off,
            brightness: 100,
            rotation: 0,
            fps: 15,
            autostart: false,
        }
    }
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct Status {
    pub device_present: bool,
    pub running: bool,
    pub mode: Mode,
    pub conflict: bool,
}

/// Render-loop engine (T5). Owns the device and the current mode.
/// Receives settings changes over a channel and emits preview frames to the webview.
pub struct Engine {
    pub settings: Settings,
}

impl Engine {
    pub fn new() -> Self {
        Engine { settings: Settings::default() }
    }

    pub fn status(&self) -> Status {
        Status {
            device_present: crate::device::PixelDevice::is_present(),
            running: false, // T5: update when render loop is running
            mode: self.settings.mode.clone(),
            conflict: false,
        }
    }
}
