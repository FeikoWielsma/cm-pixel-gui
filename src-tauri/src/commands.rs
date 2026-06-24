use tauri::State;

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
pub fn set_mode(engine: EngineState<'_>, mode: Mode) {
    engine.lock().unwrap().settings.mode = mode;
}

#[tauri::command]
pub fn set_brightness(engine: EngineState<'_>, pct: u8) {
    engine.lock().unwrap().settings.brightness = pct.min(100);
}

#[tauri::command]
pub fn set_rotation(engine: EngineState<'_>, deg: u16) {
    engine.lock().unwrap().settings.rotation = deg % 360;
}

#[tauri::command]
pub fn set_fps(engine: EngineState<'_>, v: u16) {
    engine.lock().unwrap().settings.fps = v.max(1).min(60);
}

#[tauri::command]
pub fn set_autostart(engine: EngineState<'_>, on: bool) {
    engine.lock().unwrap().settings.autostart = on;
}

#[tauri::command]
pub fn list_animations() -> Vec<AnimInfo> {
    crate::anim::list().to_vec()
}

#[tauri::command]
pub fn pick_file(_kind: String) -> Option<String> {
    // T5/T7: replace with tauri_plugin_dialog file picker
    None
}

#[tauri::command]
pub fn stop_cm_service() -> Result<(), String> {
    // T9: elevate and run Stop-Service MasterCTRLService
    Ok(())
}
