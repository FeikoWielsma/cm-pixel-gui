use crate::engine::Settings;
use tauri_plugin_store::StoreExt;

/// Load persisted settings from the Tauri store.
pub fn load(app: &tauri::AppHandle) -> Settings {
    if let Ok(store) = app.store("settings.json") {
        if let Some(val) = store.get("settings") {
            if let Ok(settings) = serde_json::from_value(val.clone()) {
                return settings;
            }
        }
    }
    Settings::default()
}

/// Persist current settings to the Tauri store.
pub fn save(app: &tauri::AppHandle, settings: &Settings) {
    if let Ok(store) = app.store("settings.json") {
        if let Ok(val) = serde_json::to_value(settings) {
            store.set("settings", val);
            let _ = store.save();
        }
    }
}
