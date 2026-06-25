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
            commands::set_minimize_to_tray,
            commands::set_start_minimized,
            commands::list_animations,
            commands::pick_file,
            commands::stop_cm_service,
            commands::set_raw_frame,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let app = window.app_handle();
                let engine = app.state::<Mutex<engine::Engine>>();
                let minimize_to_tray = engine.lock().unwrap().settings.minimize_to_tray;
                if minimize_to_tray {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .setup(|app| {
            // Initialize and start the engine (loads persisted settings)
            let engine_state = app.state::<Mutex<engine::Engine>>();
            engine_state.lock().unwrap().start(app.handle().clone());

            // Initialize system tray
            tray::setup_tray(app.handle());

            // Show window unless --minimized CLI arg or start_minimized setting is on
            let cli_minimized = std::env::args().any(|arg| arg == "--minimized" || arg == "--hidden");
            let setting_minimized = engine_state.lock().unwrap().settings.start_minimized;
            if !cli_minimized && !setting_minimized {
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
