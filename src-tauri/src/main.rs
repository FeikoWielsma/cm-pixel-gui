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
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
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
        .setup(|app| {
            // Initialize and start the engine
            let engine_state = app.state::<Mutex<engine::Engine>>();
            engine_state.lock().unwrap().start(app.handle().clone());

            // Initialize system tray
            tray::setup_tray(app.handle());

            // Show window if not started minimized/hidden on boot
            let start_minimized = std::env::args().any(|arg| arg == "--minimized" || arg == "--hidden");
            if !start_minimized {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
