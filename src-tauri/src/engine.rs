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
    Image {
        path: String,
        #[serde(default)]
        zoom: Option<f32>,
        #[serde(default)]
        pan_x: Option<f32>,
        #[serde(default)]
        pan_y: Option<f32>,
    },
    Gif {
        path: String,
        #[serde(default)]
        zoom: Option<f32>,
        #[serde(default)]
        pan_x: Option<f32>,
        #[serde(default)]
        pan_y: Option<f32>,
    },
    Text { text: String, rgb: Rgb, scroll: bool },
    Anim { id: String, speed: f32, params: Option<serde_json::Value> },
    Raw,
    Loop { ids: Vec<String>, interval_secs: f32 },
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Off
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct HistoryItem {
    pub path: String,
    pub favorite: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct ImageParams {
    pub zoom: f32,
    pub pan_x: f32,
    pub pan_y: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Settings {
    pub mode: Mode,
    pub brightness: u8,
    pub rotation: u16,
    pub fps: u16,
    pub autostart: bool,
    pub minimize_to_tray: bool,
    pub start_minimized: bool,
    #[serde(default)]
    pub recent_images: Vec<HistoryItem>,
    #[serde(default)]
    pub recent_gifs: Vec<HistoryItem>,
    #[serde(default)]
    pub image_parameters: std::collections::HashMap<String, ImageParams>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            mode: Mode::Off,
            brightness: 100,
            rotation: 0,
            fps: 15,
            autostart: false,
            minimize_to_tray: true,
            start_minimized: false,
            recent_images: Vec::new(),
            recent_gifs: Vec::new(),
            image_parameters: std::collections::HashMap::new(),
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

fn add_to_history(list: &mut Vec<HistoryItem>, path: String) {
    if let Some(pos) = list.iter().position(|item| item.path == path) {
        let item = list.remove(pos);
        list.insert(0, item);
    } else {
        list.insert(0, HistoryItem { path, favorite: false });
    }
    
    if list.len() > 10 {
        let mut i = list.len() - 1;
        let mut removed = 0;
        let target_to_remove = list.len() - 10;
        while i > 0 && removed < target_to_remove {
            if !list[i].favorite {
                list.remove(i);
                removed += 1;
            }
            if i > 0 {
                i -= 1;
            }
        }
    }
}

/// Render-loop engine (T5). Owns the device and the current mode.
/// Receives settings changes over a channel and emits preview frames to the webview.
pub struct Engine {
    pub settings: Settings,
    tx: Option<mpsc::Sender<Settings>>,
    status: Arc<Mutex<Status>>,
    pub raw_frame: Arc<Mutex<Canvas>>,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            settings: Settings::default(),
            tx: None,
            status: Arc::new(Mutex::new(Status::default())),
            raw_frame: Arc::new(Mutex::new(Canvas::black())),
        }
    }

    pub fn start(&mut self, app: AppHandle) {
        let (tx, rx) = mpsc::channel::<Settings>();
        self.tx = Some(tx.clone());

        // Load initial settings
        let settings = crate::config::load(&app);
        self.settings = settings.clone();

        let status = self.status.clone();
        let raw_frame = self.raw_frame.clone(); // ponytail: clone raw frame reference for render loop
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

            let mut loop_index = 0;
            let mut loop_timer = 0.0;
            let mut current_loop_id: Option<String> = None;

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

                let mut force_rebuild = false;
                if let Mode::Loop { ids, interval_secs } = &current_settings.mode {
                    if !ids.is_empty() {
                        let interval = *interval_secs;
                        let mut idx = loop_index;
                        if idx >= ids.len() {
                            idx = 0;
                        }
                        loop_timer += dt;
                        if loop_timer >= interval {
                            loop_timer = 0.0;
                            idx = (idx + 1) % ids.len();
                            force_rebuild = true;
                        }
                        loop_index = idx;
                        let target_id = &ids[loop_index];
                        if current_loop_id.as_ref() != Some(target_id) {
                            current_loop_id = Some(target_id.clone());
                            force_rebuild = true;
                        }
                    } else {
                        if current_loop_id.is_some() {
                            current_loop_id = None;
                            force_rebuild = true;
                        }
                    }
                } else {
                    current_loop_id = None;
                    loop_timer = 0.0;
                    loop_index = 0;
                }

                let rebuild_anim = force_rebuild || match (&last_anim_mode, &current_settings.mode) {
                    (Some(Mode::Anim { id: id1, speed: s1, params: p1 }), Mode::Anim { id: id2, speed: s2, params: p2 }) => {
                        id1 != id2 || s1 != s2 || p1 != p2
                    }
                    (Some(Mode::Text { text: t1, rgb: c1, scroll: s1 }), Mode::Text { text: t2, rgb: c2, scroll: s2 }) => t1 != t2 || c1 != c2 || s1 != s2,
                    (Some(Mode::Image { path: p1, .. }), Mode::Image { path: p2, .. }) => p1 != p2,
                    (Some(Mode::Gif { path: p1, .. }), Mode::Gif { path: p2, .. }) => p1 != p2,
                    (Some(Mode::Loop { ids: ids1, interval_secs: int1 }), Mode::Loop { ids: ids2, interval_secs: int2 }) => {
                        ids1 != ids2 || int1 != int2
                    }
                    (Some(m1), m2) => std::mem::discriminant(m1) != std::mem::discriminant(m2),
                    (None, _) => true,
                };

                if rebuild_anim {
                    active_anim = match &current_settings.mode {
                        Mode::Off | Mode::Solid { .. } | Mode::Raw => None, // ponytail: no animation rebuild for Raw mode
                        Mode::Image { path, zoom, pan_x, pan_y } => {
                            let z = zoom.unwrap_or(1.0);
                            let px = pan_x.unwrap_or(0.0);
                            let py = pan_y.unwrap_or(0.0);
                            match crate::anim::media::StaticImage::new(path, z, px, py) {
                                Ok(img) => Some(Box::new(img)),
                                Err(e) => {
                                    eprintln!("Error loading image: {}", e);
                                    None
                                }
                            }
                        }
                        Mode::Gif { path, zoom, pan_x, pan_y } => {
                            let z = zoom.unwrap_or(1.0);
                            let px = pan_x.unwrap_or(0.0);
                            let py = pan_y.unwrap_or(0.0);
                            match crate::anim::media::GifPlayer::new(path, 1.0, z, px, py) {
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
                        Mode::Loop { .. } => {
                            if let Some(ref id) = current_loop_id {
                                crate::anim::make(id, 1.0, None)
                            } else {
                                None
                            }
                        }
                    };
                    last_anim_mode = Some(current_settings.mode.clone());
                } else {
                    // Update active anim zoom/pan parameters without reloading from disk
                    if let Some(ref mut anim) = active_anim {
                        match &current_settings.mode {
                            Mode::Image { zoom, pan_x, pan_y, .. } | Mode::Gif { zoom, pan_x, pan_y, .. } => {
                                let z = zoom.unwrap_or(1.0);
                                let px = pan_x.unwrap_or(0.0);
                                let py = pan_y.unwrap_or(0.0);
                                anim.update_params(z, px, py);
                            }
                            _ => {}
                        }
                    }
                    last_anim_mode = Some(current_settings.mode.clone());
                }

                let mut canvas = Canvas::black();
                match &current_settings.mode {
                    Mode::Off => {}
                    Mode::Solid { rgb } => {
                        canvas.fill(*rgb);
                    }
                    Mode::Raw => {
                        // ponytail: read raw frame streamed from frontend
                        canvas = raw_frame.lock().unwrap().clone();
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
        match &mode {
            Mode::Image { path, zoom, pan_x, pan_y } => {
                if !path.is_empty() {
                    add_to_history(&mut self.settings.recent_images, path.clone());
                    if let (Some(z), Some(px), Some(py)) = (zoom, pan_x, pan_y) {
                        self.settings.image_parameters.insert(path.clone(), ImageParams { zoom: *z, pan_x: *px, pan_y: *py });
                    }
                }
            }
            Mode::Gif { path, zoom, pan_x, pan_y } => {
                if !path.is_empty() {
                    add_to_history(&mut self.settings.recent_gifs, path.clone());
                    if let (Some(z), Some(px), Some(py)) = (zoom, pan_x, pan_y) {
                        self.settings.image_parameters.insert(path.clone(), ImageParams { zoom: *z, pan_x: *px, pan_y: *py });
                    }
                }
            }
            _ => {}
        }
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

    pub fn set_minimize_to_tray(&mut self, app: &AppHandle, on: bool) {
        self.settings.minimize_to_tray = on;
        self.trigger_update(app);
    }

    pub fn set_start_minimized(&mut self, app: &AppHandle, on: bool) {
        self.settings.start_minimized = on;
        self.trigger_update(app);
    }

    pub fn trigger_update(&mut self, app: &AppHandle) {
        crate::config::save(app, &self.settings);
        if let Some(ref tx) = self.tx {
            let _ = tx.send(self.settings.clone());
        }
    }
}
