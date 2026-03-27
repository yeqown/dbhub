#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

use dbhub_core::{InitResult, check_init_status};
use tauri::{
    CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem,
    SystemTraySubmenu,
};

// Build connect submenu with environment-grouped connections (2 levels only)
fn build_connect_submenu() -> SystemTraySubmenu {
    // Try to load connections synchronously
    let connections_result = std::panic::catch_unwind(|| {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async { commands::get_connections(None).await })
    });

    let connections = match connections_result {
        Ok(Ok(conns)) => conns,
        Ok(Err(e)) => {
            eprintln!("Failed to load connections: {e}");
            let error_item = CustomMenuItem::new("connect-error", "Error loading connections");
            return SystemTraySubmenu::new("Connect", SystemTrayMenu::new().add_item(error_item));
        }
        Err(_) => {
            eprintln!("Panic while loading connections");
            let error_item = CustomMenuItem::new("connect-error", "Error loading connections");
            return SystemTraySubmenu::new("Connect", SystemTrayMenu::new().add_item(error_item));
        }
    };

    if connections.is_empty() {
        eprintln!("No connections found");
        let empty_item = CustomMenuItem::new("connect-empty", "No connections configured");
        return SystemTraySubmenu::new("Connect", SystemTrayMenu::new().add_item(empty_item));
    }

    println!("Loaded {} connection groups for submenu", connections.len());

    // Create the Connect submenu with all connections
    let mut connect_menu = SystemTrayMenu::new();
    let mut environments: Vec<_> = connections.keys().collect();
    environments.sort();

    for (index, env) in environments.iter().enumerate() {
        let databases = connections.get(*env).unwrap();

        // Add environment as a simple header item
        let header_text = format!("{} ({})", env, databases.len());
        let header_item =
            CustomMenuItem::new(format!("env-header-{env}"), header_text).disabled();
        connect_menu = connect_menu.add_item(header_item);

        // Add connections for this environment (just alias, no env prefix)
        for db in databases {
            let item_id = format!("connect-{}", db.alias);
            let display_name = format!("  {}", db.alias);
            let item = CustomMenuItem::new(item_id, display_name);
            connect_menu = connect_menu.add_item(item);
        }

        // Add separator between environments (except last)
        if index < environments.len() - 1 {
            connect_menu = connect_menu.add_native_item(SystemTrayMenuItem::Separator);
        }
    }

    SystemTraySubmenu::new("Connect", connect_menu)
}

// Build config submenu with all config files
fn build_config_submenu() -> SystemTraySubmenu {
    // Try to load config files synchronously
    let config_files_result = std::panic::catch_unwind(|| {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async { commands::get_config_files().await })
    });

    let config_files = match config_files_result {
        Ok(Ok(files)) => files,
        Ok(Err(e)) => {
            eprintln!("Failed to load config files: {e}");
            let error_item = CustomMenuItem::new("config-error", "Error loading config");
            return SystemTraySubmenu::new("Config", SystemTrayMenu::new().add_item(error_item));
        }
        Err(_) => {
            eprintln!("Panic while loading config files");
            let error_item = CustomMenuItem::new("config-error", "Error loading config");
            return SystemTraySubmenu::new("Config", SystemTrayMenu::new().add_item(error_item));
        }
    };

    if config_files.is_empty() {
        eprintln!("No config files found");
        let empty_item = CustomMenuItem::new("config-empty", "No config files");
        return SystemTraySubmenu::new("Config", SystemTrayMenu::new().add_item(empty_item));
    }

    println!("Loaded {} config files for submenu", config_files.len());

    // Create the Config submenu with all files
    let mut config_menu = SystemTrayMenu::new();

    for file in &config_files {
        let item_id = format!("config-{}", file.path);
        let display_name = &file.name;
        let item = CustomMenuItem::new(item_id, display_name.clone());
        config_menu = config_menu.add_item(item);
    }

    SystemTraySubmenu::new("Config", config_menu)
}

// Helper function to handle async connect call
fn handle_connect(_app: &tauri::AppHandle, alias: String) {
    println!("[DEBUG] Connect clicked for alias: {alias}");
    println!("[DEBUG] Starting async connect task...");
    tauri::async_runtime::spawn(async move {
        println!("[DEBUG] Connect task running for: {alias}");
        match commands::connect(alias.clone(), None).await {
            Ok(_) => {
                println!("[DEBUG] Successfully opened Terminal for {alias}");
            }
            Err(e) => {
                eprintln!("[ERROR] Failed to connect to {alias}: {e}");
                eprintln!("[ERROR] This might mean 'dbhub' CLI is not installed or not in PATH");
            }
        }
    });
}

// Helper function to handle async config file opening
fn handle_open_config(_app: &tauri::AppHandle, path: String) {
    println!("[DEBUG] Open config clicked for path: {path}");
    tauri::async_runtime::spawn(async move {
        match commands::open_config_editor(path.clone()).await {
            Ok(_) => {
                println!("[DEBUG] Successfully opened editor for {path}");
            }
            Err(e) => {
                eprintln!("[ERROR] Failed to open config file {path}: {e}");
            }
        }
    });
}

fn main() {
    // Check initialization status before starting GUI
    let init_result = check_init_status();

    // Create static menu items
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let about = CustomMenuItem::new("about".to_string(), "About");

    // Build connect submenu with environment-grouped connections
    let connect_submenu = build_connect_submenu();

    // Build config submenu with config files
    let config_submenu = build_config_submenu();

    let tray_menu = SystemTrayMenu::new()
        .add_submenu(connect_submenu)
        .add_submenu(config_submenu)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(about)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);

    let system_tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        .system_tray(system_tray)
        .manage(init_result)
        .on_system_tray_event(|app, event| if let SystemTrayEvent::MenuItemClick { id, .. } = event {
            match id.as_str() {
                "quit" => {
                    std::process::exit(0);
                }
                "about" => {
                    if let Some(about_window) = app.get_window("about") {
                        let _ = about_window.show();
                        let _ = about_window.set_focus();
                    } else {
                        let _ = tauri::WindowBuilder::new(
                            app,
                            "about",
                            tauri::WindowUrl::App("about.html".into())
                        )
                        .title("About")
                        .inner_size(400.0, 320.0)
                        .resizable(false)
                        .center()
                        .always_on_top(true)
                        .build();
                    }
                }
                id if id.starts_with("connect-") => {
                    let alias = id[8..].to_string(); // Remove "connect-" prefix
                    handle_connect(app, alias);
                }
                id if id.starts_with("config-") => {
                    let path = id[7..].to_string(); // Remove "config-" prefix
                    handle_open_config(app, path);
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_connections,
            commands::connect,
            commands::get_config_files,
            commands::open_config_editor,
            commands::open_repository,
            commands::initialize_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
