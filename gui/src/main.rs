#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::get_connections,
            commands::connect,
            commands::add_database,
            commands::update_database,
            commands::delete_database,
            commands::get_config,
            commands::save_config_dto,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
