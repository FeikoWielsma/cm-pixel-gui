use serde::{Deserialize, Serialize};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

use crate::canvas::{Canvas, Rgb};
use crate::device::PixelDevice;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Mode {
    Off,
    Solid { rgb: Rgb },
    Image { path: String },
    Gif { path: String },
    Text { text: String, rgb: Rgb, scroll: bool },
    Anim { id: String, speed: f32, params: Option<serde_json::Value> },
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

enum ConnectionResult {
    Success(PixelDevice),
    Conflict,
    Disconnected,
}

/// Render-loop engine (T5). Owns the device and the current mode.
/// Receives settings changes over a channel and emits preview frames to the webview.
pub struct Engine {
    pub settings: Settings,
    tx: Option<mpsc::Sender<Settings>>,
    status: Arc<Mutex<Status>>,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            settings: Settings::default(),
            tx: None,
            status: Arc::new(Mutex::new(Status::default())),
        }
    }

    pub fn start(&mut self, app: AppHandle) {
        let (tx, rx) = mpsc::channel::<Settings>();
        self.tx = Some(tx.clone());

        // Load initial settings
        let settings = crate::config::load(&app);
        self.settings = settings.clone();

        let status = self.status.clone();
        {
            let mut st = status.lock().unwrap();
            st.mode = settings.mode.clone();
            st.conflict = false;
            st.device_present = false;
            st.running = false;
        }

        let (req_tx, req_rx) = mpsc::channel::<()>();
        let (result_tx, result_rx) = mpsc::channel::<ConnectionResult>();

        // Background worker thread for connection management to keep UI loop 100% responsive
        thread::spawn(move || {
            while req_rx.recv().is_ok() {
                loop {
                    if PixelDevice::is_present() {
                        match PixelDevice::open() {
                            Ok(dev) => {
                                let _ = result_tx.send(ConnectionResult::Success(dev));
                                break;
                            }
                            Err(e) => {
                                let err_str = e.to_string();
                                if err_str.contains("MasterCTRL") || err_str.contains("access denied") || err_str.contains("sharing violation") {
                                    let _ = result_tx.send(ConnectionResult::Conflict);
                                } else {
                                    let _ = result_tx.send(ConnectionResult::Disconnected);
                                }
                            }
                        }
                    } else {
                        let _ = result_tx.send(ConnectionResult::Disconnected);
                    }
                    thread::sleep(Duration::from_secs(2));
                    // Drain any piled up connection requests
                    while req_rx.try_recv().is_ok() {}
                }
            }
        });

        thread::spawn(move || {
            let mut current_settings = settings;
            let mut device: Option<PixelDevice> = None;
            let mut active_anim: Option<Box<dyn crate::anim::Animation>> = None;
            let mut last_anim_mode: Option<Mode> = None;

            let start_time = Instant::now();
            let mut last_frame_time = Instant::now();

            let mut present = false;
            let mut conflict = false;
            let mut connection_requested = false;

            loop {
                let loop_start = Instant::now();

                // 1. Receive settings updates
                while let Ok(updated) = rx.try_recv() {
                    current_settings = updated;
                }

                // 2. Manage device connection non-blockingly
                if device.is_none() {
                    if !connection_requested {
                        let _ = req_tx.send(());
                        connection_requested = true;
                    }
                    while let Ok(res) = result_rx.try_recv() {
                        match res {
                            ConnectionResult::Success(dev) => {
                                device = Some(dev);
                                present = true;
                                conflict = false;
                                connection_requested = false;
                            }
                            ConnectionResult::Conflict => {
                                present = false;
                                conflict = true;
                            }
                            ConnectionResult::Disconnected => {
                                present = false;
                                conflict = false;
                            }
                        }
                    }
                } else {
                    present = true;
                    conflict = false;
                    connection_requested = false;
                }

                let is_running = device.is_some();
                {
                    let mut st = status.lock().unwrap();
                    let prev_present = st.device_present;
                    let prev_conflict = st.conflict;
                    let prev_running = st.running;
                    let prev_mode_kind = std::mem::discriminant(&st.mode);
                    let curr_mode_kind = std::mem::discriminant(&current_settings.mode);

                    st.device_present = present;
                    st.conflict = conflict;
                    st.running = is_running;
                    st.mode = current_settings.mode.clone();

                    if prev_present != present || prev_conflict != conflict || prev_running != is_running || prev_mode_kind != curr_mode_kind {
                        let status_val = st.clone();
                        let _ = app.emit("status", status_val);
                    }
                }

                // 3. Render next frame
                let now = Instant::now();
                let t = now.duration_since(start_time).as_secs_f32();
                let mut dt = now.duration_since(last_frame_time).as_secs_f32();
                last_frame_time = now;

                // Cap dt to prevent massive frame jumps/catch-up speed issues on startup or delay
                if dt > 0.2 {
                    dt = 0.2;
                }

                let rebuild_anim = match (&last_anim_mode, &current_settings.mode) {
                    (Some(Mode::Anim { id: id1, speed: s1, params: p1 }), Mode::Anim { id: id2, speed: s2, params: p2 }) => {
                        id1 != id2 || s1 != s2 || p1 != p2
                    }
                    (Some(Mode::Text { text: t1, rgb: c1, scroll: s1 }), Mode::Text { text: t2, rgb: c2, scroll: s2 }) => t1 != t2 || c1 != c2 || s1 != s2,
                    (Some(Mode::Image { path: p1 }), Mode::Image { path: p2 }) => p1 != p2,
                    (Some(Mode::Gif { path: p1 }), Mode::Gif { path: p2 }) => p1 != p2,
                    (Some(m1), m2) => std::mem::discriminant(m1) != std::mem::discriminant(m2),
                    (None, _) => true,
                };

                if rebuild_anim {
                    active_anim = match &current_settings.mode {
                        Mode::Off | Mode::Solid { .. } => None,
                        Mode::Image { path } => {
                            match crate::anim::media::StaticImage::new(path) {
                                Ok(img) => Some(Box::new(img)),
                                Err(e) => {
                                    eprintln!("Error loading image: {}", e);
                                    None
                                }
                            }
                        }
                        Mode::Gif { path } => {
                            match crate::anim::media::GifPlayer::new(path, 1.0) {
                                Ok(player) => Some(Box::new(player)),
                                Err(e) => {
                                    eprintln!("Error loading gif: {}", e);
                                    None
                                }
                            }
                        }
                        Mode::Text { text, rgb, scroll } => {
                            Some(Box::new(crate::anim::text::TextScroll::new(
                                text.clone(),
                                *rgb,
                                1.0,
                                *scroll,
                             )))
                        }
                        Mode::Anim { id, speed, params } => {
                            crate::anim::make(id, *speed, params.as_ref())
                        }
                    };
                    last_anim_mode = Some(current_settings.mode.clone());
                }

                let mut canvas = Canvas::black();
                match &current_settings.mode {
                    Mode::Off => {}
                    Mode::Solid { rgb } => {
                        canvas.fill(*rgb);
                    }
                    _ => {
                        if let Some(ref mut anim) = active_anim {
                            canvas = anim.render(t, dt);
                        }
                    }
                }

                // 4. Send frame to device
                if let Some(ref mut dev) = device {
                    let processed = canvas
                        .rotated(current_settings.rotation)
                        .with_brightness(current_settings.brightness);
                    if let Err(e) = dev.send(&processed) {
                        eprintln!("Error sending to device: {}", e);
                        device = None;
                        present = false;
                        conflict = false;
                        // Request connection again
                        let _ = req_tx.send(());
                        connection_requested = true;
                    }
                }

                // 5. Emit preview
                let _ = app.emit("preview", canvas.px);

                // 6. Sleep to match target FPS
                let elapsed = loop_start.elapsed();
                let frame_duration = Duration::from_secs_f32(1.0 / current_settings.fps.max(1) as f32);
                if elapsed < frame_duration {
                    thread::sleep(frame_duration - elapsed);
                }
            }
        });
    }

    pub fn status(&self) -> Status {
        self.status.lock().unwrap().clone()
    }

    pub fn set_mode(&mut self, app: &AppHandle, mode: Mode) {
        self.settings.mode = mode;
        self.trigger_update(app);
    }

    pub fn set_brightness(&mut self, app: &AppHandle, pct: u8) {
        self.settings.brightness = pct.min(100);
        self.trigger_update(app);
    }

    pub fn set_rotation(&mut self, app: &AppHandle, deg: u16) {
        self.settings.rotation = deg % 360;
        self.trigger_update(app);
    }

    pub fn set_fps(&mut self, app: &AppHandle, v: u16) {
        self.settings.fps = v.max(1).min(60);
        self.trigger_update(app);
    }

    pub fn set_autostart(&mut self, app: &AppHandle, on: bool) {
        self.settings.autostart = on;
        self.trigger_update(app);
    }

    fn trigger_update(&mut self, app: &AppHandle) {
        crate::config::save(app, &self.settings);
        if let Some(ref tx) = self.tx {
            let _ = tx.send(self.settings.clone());
        }
    }
}
