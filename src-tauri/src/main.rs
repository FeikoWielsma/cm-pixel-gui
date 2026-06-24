#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod anim;
mod canvas;
mod commands;
mod config;
mod device;
mod engine;
mod layout;
mod tray;

use std::sync::Mutex;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {}))
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(Mutex::new(engine::Engine::new()))
        .invoke_handler(tauri::generate_handler![
            commands::get_status,
            commands::get_settings,
            commands::set_mode,
            commands::set_brightness,
            commands::set_rotation,
            commands::set_fps,
            commands::set_autostart,
            commands::list_animations,
            commands::pick_file,
            commands::stop_cm_service,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
