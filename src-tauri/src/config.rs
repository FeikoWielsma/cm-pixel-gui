use crate::engine::Settings;

/// Load persisted settings from the Tauri store (T6 implementation).
pub fn load(_app: &tauri::AppHandle) -> Settings {
    Settings::default()
}

/// Persist current settings to the Tauri store (T6 implementation).
pub fn save(_app: &tauri::AppHandle, _settings: &Settings) {}
