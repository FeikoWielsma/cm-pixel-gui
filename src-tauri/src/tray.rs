use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};
use crate::engine::{Engine, Mode};

/// Build and attach the system tray icon + menu (T6 implementation).
pub fn setup_tray(app: &AppHandle) {
    let show_hide = MenuItemBuilder::with_id("show_hide", "Show / Hide").build(app).unwrap();
    let mode_off = MenuItemBuilder::with_id("mode_off", "Turn Off").build(app).unwrap();
    let mode_red = MenuItemBuilder::with_id("mode_red", "Solid Red").build(app).unwrap();
    let mode_green = MenuItemBuilder::with_id("mode_green", "Solid Green").build(app).unwrap();
    let mode_blue = MenuItemBuilder::with_id("mode_blue", "Solid Blue").build(app).unwrap();
    let mode_rubik = MenuItemBuilder::with_id("mode_rubik", "Rubik's Cube").build(app).unwrap();
    let quit = MenuItemBuilder::with_id("quit", "Quit").build(app).unwrap();

    let menu = MenuBuilder::new(app)
        .item(&show_hide)
        .separator()
        .item(&mode_off)
        .item(&mode_red)
        .item(&mode_green)
        .item(&mode_blue)
        .item(&mode_rubik)
        .separator()
        .item(&quit)
        .build()
        .unwrap();

    let mut builder = TrayIconBuilder::new()
        .menu(&menu)
        .on_menu_event(move |app, event| {
            let engine = app.state::<std::sync::Mutex<Engine>>();
            match event.id().as_ref() {
                "show_hide" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let is_visible = window.is_visible().unwrap_or(false);
                        if is_visible {
                            let _ = window.hide();
                        } else {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
                "mode_off" => {
                    engine.lock().unwrap().set_mode(app, Mode::Off);
                }
                "mode_red" => {
                    engine.lock().unwrap().set_mode(app, Mode::Solid { rgb: [255, 0, 0] });
                }
                "mode_green" => {
                    engine.lock().unwrap().set_mode(app, Mode::Solid { rgb: [0, 255, 0] });
                }
                "mode_blue" => {
                    engine.lock().unwrap().set_mode(app, Mode::Solid { rgb: [0, 0, 255] });
                }
                "mode_rubik" => {
                    engine.lock().unwrap().set_mode(app, Mode::Anim { id: "rubik".to_string(), speed: 1.0, params: None });
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button, button_state, .. } = event {
                if button == MouseButton::Left && button_state == MouseButtonState::Up {
                    let app = tray.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        let is_visible = window.is_visible().unwrap_or(false);
                        if is_visible {
                            let _ = window.hide();
                        } else {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
            }
        });

    let _ = builder
        .icon(tauri::include_image!("icons/tray.png"))
        .build(app);
}
