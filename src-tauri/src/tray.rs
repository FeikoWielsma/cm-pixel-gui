use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};
use crate::engine::{Engine, Mode};

pub fn setup_tray(app: &AppHandle) {
    // ── Animations submenu ────────────────────────────────────────────────
    let anims: &[(&str, &str)] = &[
        ("anim_plasma",      "Plasma"),
        ("anim_fire",        "Fire"),
        ("anim_aurora",      "Aurora"),
        ("anim_starfield",   "Starfield"),
        ("anim_matrix",      "Matrix"),
        ("anim_fireworks",   "Fireworks"),
        ("anim_julia",       "Julia Fractal"),
        ("anim_chroma_life", "Chroma Life"),
        ("anim_rubik",       "Rubik's Cube"),
        ("anim_icecube",     "Ice Cube"),
        ("anim_soccer",      "Soccer"),
    ];
    let anim_items: Vec<_> = anims.iter()
        .map(|(id, label)| MenuItemBuilder::with_id(*id, *label).build(app).unwrap())
        .collect();
    let mut anim_builder = SubmenuBuilder::new(app, "Animations");
    for item in &anim_items { anim_builder = anim_builder.item(item); }
    let anim_sub = anim_builder.build().unwrap();

    // ── Brightness submenu ────────────────────────────────────────────────
    let bright_items: &[(&str, &str)] = &[
        ("bright_25",  "25%"),
        ("bright_50",  "50%"),
        ("bright_75",  "75%"),
        ("bright_100", "100%"),
    ];
    let b_items: Vec<_> = bright_items.iter()
        .map(|(id, label)| MenuItemBuilder::with_id(*id, *label).build(app).unwrap())
        .collect();
    let mut bright_builder = SubmenuBuilder::new(app, "Brightness");
    for item in &b_items { bright_builder = bright_builder.item(item); }
    let bright_sub = bright_builder.build().unwrap();

    // ── Top-level items ───────────────────────────────────────────────────
    let show    = MenuItemBuilder::with_id("show",     "Show Window").build(app).unwrap();
    let off     = MenuItemBuilder::with_id("mode_off", "Turn Off").build(app).unwrap();
    let stop    = MenuItemBuilder::with_id("stop_svc", "Stop CM Service").build(app).unwrap();
    let quit    = MenuItemBuilder::with_id("quit",     "Quit").build(app).unwrap();

    let menu = MenuBuilder::new(app)
        .item(&show)
        .separator()
        .item(&off)
        .item(&anim_sub)
        .separator()
        .item(&bright_sub)
        .separator()
        .item(&stop)
        .separator()
        .item(&quit)
        .build()
        .unwrap();

    let _ = TrayIconBuilder::new()
        .icon(tauri::include_image!("icons/tray.png"))
        .menu(&menu)
        .on_menu_event(|app, event| {
            let engine = app.state::<std::sync::Mutex<Engine>>();
            let id = event.id().as_ref();
            match id {
                "show" => {
                    if let Some(w) = app.get_webview_window("main") {
                        let _ = w.show();
                        let _ = w.set_focus();
                    }
                }
                "mode_off" => {
                    engine.lock().unwrap().set_mode(app, Mode::Off);
                }
                "stop_svc" => {
                    let _ = std::process::Command::new("powershell")
                        .args(["-Command",
                            "Start-Process powershell \
                             -ArgumentList 'Stop-Service -Name MasterCTRLService -Force' \
                             -Verb RunAs -Wait"])
                        .spawn();
                }
                "quit" => app.exit(0),
                // brightness presets
                "bright_25"  => { engine.lock().unwrap().set_brightness(app, 25); }
                "bright_50"  => { engine.lock().unwrap().set_brightness(app, 50); }
                "bright_75"  => { engine.lock().unwrap().set_brightness(app, 75); }
                "bright_100" => { engine.lock().unwrap().set_brightness(app, 100); }
                // any "anim_*" item: strip prefix, use remainder as animation id
                id if id.starts_with("anim_") => {
                    let anim_id = id["anim_".len()..].to_string();
                    engine.lock().unwrap().set_mode(app, Mode::Anim {
                        id: anim_id,
                        speed: 1.0,
                        params: None,
                    });
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            // Left-click toggles the window
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                let app = tray.app_handle();
                if let Some(w) = app.get_webview_window("main") {
                    if w.is_visible().unwrap_or(false) {
                        let _ = w.hide();
                    } else {
                        let _ = w.show();
                        let _ = w.set_focus();
                    }
                }
            }
        })
        .build(app);
}
