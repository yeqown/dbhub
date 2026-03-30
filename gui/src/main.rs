#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod menu;
mod tray;

use dbhub_core::check_init_status;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

/// Handle first-run initialization with confirmation dialog
fn handle_init_dialog(app: &tauri::AppHandle, needs_init: bool) {
    if !needs_init {
        return;
    }

    let app_handle = app.clone();
    app.dialog()
        .message("No configuration file found. Would you like to create a default configuration?\n\nConfiguration location: ~/.dbhub/config.yml")
        .title("Welcome to DB Hub!")
        .buttons(MessageDialogButtons::OkCancel)
        .show(move |confirmed| {
            if confirmed {
                match dbhub_core::generate_default_config() {
                    Ok(()) => {
                        app_handle.dialog()
                            .message("Configuration created successfully!\n\nThe application will restart.")
                            .title("Success")
                            .show(|_| restart_app());
                    }
                    Err(e) => {
                        app_handle.dialog()
                            .message(format!("Failed to create configuration: {e}"))
                            .title("Error")
                            .kind(MessageDialogKind::Error)
                            .show(|_| std::process::exit(1));
                    }
                }
            } else {
                std::process::exit(0);
            }
        });
}

fn restart_app() {
    let exe = std::env::current_exe().unwrap();
    std::process::Command::new(exe)
        .spawn()
        .expect("Failed to restart application");
    std::process::exit(0);
}

fn main() {
    let init_result = check_init_status();
    let needs_init = init_result.status == dbhub_core::InitStatus::NotInitialized
        || init_result.status == dbhub_core::InitStatus::NoValidConfig;

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(init_result)
        .setup(move |app| {
            handle_init_dialog(app.handle(), needs_init);

            let tray_menu = menu::build(app.handle())?;
            tray::setup(app.handle(), tray_menu)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_connections,
            commands::connect,
            commands::get_config_files,
            commands::open_config_editor,
            commands::open_repository,
            commands::initialize_config,
            commands::get_init_status,
            commands::read_config_file,
            commands::save_config_file,
            commands::create_config_file,
            commands::delete_config_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
