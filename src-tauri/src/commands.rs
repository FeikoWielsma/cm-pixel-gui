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
