use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::anim::AnimInfo;
use crate::engine::{Engine, Mode, Settings, Status};

type EngineState<'a> = State<'a, std::sync::Mutex<Engine>>;

#[tauri::command]
pub fn get_status(engine: EngineState<'_>) -> Status {
    engine.lock().unwrap().status()
}

#[tauri::command]
pub fn get_settings(engine: EngineState<'_>) -> Settings {
    engine.lock().unwrap().settings.clone()
}

#[tauri::command]
pub fn set_mode(app: AppHandle, engine: EngineState<'_>, mode: Mode) {
    engine.lock().unwrap().set_mode(&app, mode);
}

#[tauri::command]
pub fn set_brightness(app: AppHandle, engine: EngineState<'_>, pct: u8) {
    engine.lock().unwrap().set_brightness(&app, pct);
}

#[tauri::command]
pub fn set_rotation(app: AppHandle, engine: EngineState<'_>, deg: u16) {
    engine.lock().unwrap().set_rotation(&app, deg);
}

#[tauri::command]
pub fn set_fps(app: AppHandle, engine: EngineState<'_>, v: u16) {
    engine.lock().unwrap().set_fps(&app, v);
}

#[tauri::command]
pub fn set_autostart(app: AppHandle, engine: EngineState<'_>, on: bool) {
    engine.lock().unwrap().set_autostart(&app, on);
    // Sync with tauri-plugin-autostart
    use tauri_plugin_autostart::ManagerExt;
    let autostart_manager = app.autolaunch();
    if on {
        let _ = autostart_manager.enable();
    } else {
        let _ = autostart_manager.disable();
    }
}

#[tauri::command]
pub fn set_minimize_to_tray(app: AppHandle, engine: EngineState<'_>, on: bool) {
    engine.lock().unwrap().set_minimize_to_tray(&app, on);
}

#[tauri::command]
pub fn set_start_minimized(app: AppHandle, engine: EngineState<'_>, on: bool) {
    engine.lock().unwrap().set_start_minimized(&app, on);
}

#[tauri::command]
pub fn list_animations() -> Vec<AnimInfo> {
    crate::anim::list().to_vec()
}

#[tauri::command]
pub async fn pick_file(app: AppHandle, kind: String) -> Result<Option<String>, String> {
    let mut builder = app.dialog().file();
    if kind == "image" {
        builder = builder.add_filter("Images", &["png", "jpg", "jpeg", "webp"]);
    } else if kind == "gif" {
        builder = builder.add_filter("GIFs", &["gif"]);
    }

    let file_handle = builder.blocking_pick_file();
    match file_handle {
        Some(fp) => match fp.into_path() {
            Ok(pb) => Ok(Some(pb.to_string_lossy().to_string())),
            Err(e) => Err(e.to_string()),
        },
        None => Ok(None),
    }
}

#[tauri::command]
pub fn stop_cm_service() -> Result<(), String> {
    use std::process::Command;
    let status = Command::new("powershell")
        .args(&[
            "-Command",
            "Start-Process powershell -ArgumentList 'Stop-Service -Name MasterCTRLService -Force' -Verb RunAs -Wait",
        ])
        .status();

    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => Err(format!("Command exited with status: {}", s)),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn set_raw_frame(engine: EngineState<'_>, px: Vec<[u8; 3]>) {
    // ponytail: update raw frame buffer for live-streaming (e.g. Doom)
    if px.len() == 1024 {
        let eng = engine.lock().unwrap();
        let mut rf = eng.raw_frame.lock().unwrap();
        for y in 0..32 {
            for x in 0..32 {
                rf.px[y][x] = px[y * 32 + x];
            }
        }
    }
}

#[tauri::command]
pub fn toggle_favorite(app: AppHandle, engine: EngineState<'_>, kind: String, path: String) -> Settings {
    let mut eng = engine.lock().unwrap();
    let list = if kind == "image" {
        &mut eng.settings.recent_images
    } else {
        &mut eng.settings.recent_gifs
    };
    
    if let Some(item) = list.iter_mut().find(|i| i.path == path) {
        item.favorite = !item.favorite;
    }
    
    eng.trigger_update(&app);
    eng.settings.clone()
}

#[tauri::command]
pub fn delete_history(app: AppHandle, engine: EngineState<'_>, kind: String, path: String) -> Settings {
    let mut eng = engine.lock().unwrap();
    let list = if kind == "image" {
        &mut eng.settings.recent_images
    } else {
        &mut eng.settings.recent_gifs
    };
    
    list.retain(|i| i.path != path);
    
    eng.trigger_update(&app);
    eng.settings.clone()
}

#[tauri::command]
pub fn get_anim_previews_data() -> std::collections::HashMap<String, Vec<[[[u8; 3]; 32]; 32]>> {
    let mut previews = std::collections::HashMap::new();
    let fps = 15.0;
    let dt = 1.0 / fps;

    for anim_info in crate::anim::list() {
        let id = anim_info.id;
        let mut anim = crate::anim::make(id, 1.0, None).unwrap_or_else(|| {
            Box::new(crate::anim::procedural::Rainbow::new(1.0, 0.0, "rainbow".to_string()))
        });

        // Vary frame counts based on the animation's nature to prevent synchronized resets
        let frame_count = match id {
            "plasma" | "julia" | "aurora" | "icecube" | "rubik" | "fireworks" | "starfield" | "breathe" => 90, // 6s loops
            "ripple" | "metaballs" | "matrix" | "pinwheel" | "torus" => 75, // 5s loops
            "rainbow" | "swirl" | "tunnel" | "comet" | "interference" | "chroma_life" | "soccer" => 60, // 4s loops
            _ => 45, // 3s loops (sparkle, fire, bounce, twinkle)
        };

        // Pre-warm animations that need time to build up (like fire and matrix)
        let pre_warm_frames = match id {
            "fire" | "matrix" => 60, // 4 seconds at 15 FPS
            _ => 0,
        };
        for _ in 0..pre_warm_frames {
            let _ = anim.render(0.0, dt);
        }

        let mut frames = Vec::with_capacity(frame_count);
        for f in 0..frame_count {
            let t = f as f32 * dt;
            let canvas = anim.render(t, dt);
            frames.push(canvas.px);
        }
        previews.insert(id.to_string(), frames);
    }

    previews
}
