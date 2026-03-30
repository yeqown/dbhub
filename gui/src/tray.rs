//! System tray setup and event handling.

use tauri::{
    image::Image,
    menu::Menu,
    tray::TrayIconBuilder,
    Manager, WebviewUrl, WebviewWindowBuilder,
};

use crate::commands;

/// Setup system tray with menu.
pub fn setup(
    app: &tauri::AppHandle,
    tray_menu: Menu<tauri::Wry>,
) -> Result<(), Box<dyn std::error::Error>> {
    let tray_icon = Image::from_bytes(include_bytes!("../icons/menubaricon.png"))?;

    TrayIconBuilder::new()
        .icon(tray_icon)
        .icon_as_template(true)
        .menu(&tray_menu)
        .on_tray_icon_event(|tray, _| {
            let _ = tray.app_handle().set_activation_policy(tauri::ActivationPolicy::Accessory);
        })
        .on_menu_event(handle_menu_event)
        .show_menu_on_left_click(true)
        .build(app)?;

    Ok(())
}

fn handle_menu_event(app: &tauri::AppHandle, event: tauri::menu::MenuEvent) {
    match event.id().as_ref() {
        "quit" => app.exit(0),
        "about" => show_about_window(app),
        "config-manage" => show_manage_window(app),
        id if id.starts_with("connect-") => {
            let alias = id[8..].to_string();
            handle_connect(app, alias);
        }
        id if id.starts_with("config-") => {
            let path = id[7..].to_string();
            handle_open_config(app, path);
        }
        _ => {}
    }
}

fn show_about_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("about") {
        let _ = window.show();
        let _ = window.set_focus();
        return;
    }

    let _ = WebviewWindowBuilder::new(
        app,
        "about",
        WebviewUrl::App("about.html".into()),
    )
    .title("About")
    .inner_size(400.0, 320.0)
    .resizable(false)
    .center()
    .always_on_top(true)
    .build();
}

fn handle_connect(_app: &tauri::AppHandle, alias: String) {
    tauri::async_runtime::spawn(async move {
        match commands::connect(alias.clone(), None).await {
            Ok(_) => println!("[DEBUG] Connected to {alias}"),
            Err(e) => eprintln!("[ERROR] Failed to connect to {alias}: {e}"),
        }
    });
}

fn handle_open_config(_app: &tauri::AppHandle, path: String) {
    tauri::async_runtime::spawn(async move {
        match commands::open_config_editor(path.clone()).await {
            Ok(_) => println!("[DEBUG] Opened config: {path}"),
            Err(e) => eprintln!("[ERROR] Failed to open config {path}: {e}"),
        }
    });
}

fn show_manage_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("manage") {
        let _ = window.show();
        let _ = window.set_focus();
        return;
    }

    let _ = WebviewWindowBuilder::new(
        app,
        "manage",
        WebviewUrl::App("manage.html".into()),
    )
    .title("Manage Configurations")
    .inner_size(900.0, 600.0)
    .min_inner_size(700.0, 400.0)
    .resizable(true)
    .center()
    .build();
}
