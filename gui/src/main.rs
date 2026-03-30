#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

use dbhub_core::check_init_status;
use tauri::{
    image::Image,
    menu::{IconMenuItemBuilder, Menu, MenuItem, PredefinedMenuItem, SubmenuBuilder},
    tray::TrayIconBuilder,
    Manager, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

/// Get icon for database type
fn get_db_icon(db_type: &str) -> Option<Image<'static>> {
    let bytes: &'static [u8] = match db_type.to_lowercase().as_str() {
        "mysql" => include_bytes!("../icons/mysql.png"),
        "mongo" | "mongodb" => include_bytes!("../icons/mongodb.png"),
        "redis" | "redis-sentinel" => include_bytes!("../icons/redis.png"),
        "memcached" => include_bytes!("../icons/memcached.png"),
        "doris" => include_bytes!("../icons/doris.png"),
        "postgres" | "postgresql" => include_bytes!("../icons/postgres.png"),
        _ => return None,
    };
    Image::from_bytes(bytes).ok()
}

/// Handle async connect call
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

/// Handle async config file opening
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

/// Handle first-run initialization with confirmation dialog
fn initial_config_file(app: &tauri::AppHandle, needs_init: bool) {
    if !needs_init {
        return;
    }

    println!("[GUI-Init] Showing native init dialog...");

    let app_handle = app.clone();
    app.dialog()
        .message("No configuration file found. Would you like to create a default configuration?\n\nConfiguration location: ~/.dbhub/config.yml")
        .title("Welcome to DB Hub!")
        .buttons(MessageDialogButtons::OkCancel)
        .show(move |confirmed| {
            if confirmed {
                println!("[GUI-Init] User confirmed, creating config...");
                match dbhub_core::config::generate_default_config() {
                    Ok(()) => {
                        println!("[GUI-Init] Config created successfully");
                        let _ = app_handle.dialog()
                            .message("Configuration file created successfully!\n\nThe application will now restart to load the new configuration.")
                            .title("Success")
                            .show(|_| {
                                println!("[GUI-Init] Restarting application...");
                                let exe = std::env::current_exe().unwrap();
                                std::process::Command::new(exe)
                                    .spawn()
                                    .expect("Failed to restart application");
                                std::process::exit(0);
                            });
                    }
                    Err(e) => {
                        eprintln!("[GUI-Init] Failed to create config: {}", e);
                        let error_msg = format!("Failed to create configuration: {}", e);
                        let _ = app_handle.dialog()
                            .message(error_msg)
                            .title("Error")
                            .kind(MessageDialogKind::Error)
                            .show(|_| {
                                std::process::exit(1);
                            });
                    }
                }
            } else {
                println!("[GUI-Init] User cancelled, exiting...");
                std::process::exit(0);
            }
        });
}

fn main() {
    // Check initialization status before starting GUI
    println!("[GUI-Init] Checking initialization status...");
    let init_result = check_init_status();
    println!("[GUI-Init] Init status: {:?}", init_result.status);
    println!("[GUI-Init] Config dir: {:?}", init_result.config_dir);
    if let Some(ref msg) = init_result.message {
        println!("[GUI-Init] Message: {}", msg);
    }

    // Check if initialization is needed
    let needs_init = init_result.status == dbhub_core::InitStatus::NotInitialized
        || init_result.status == dbhub_core::InitStatus::NoValidConfig;

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(init_result)
        .setup(move |app| {
            // Show init dialog if needed
            initial_config_file(app.handle(), needs_init);

            // Build static menu items
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let about = MenuItem::with_id(app, "about", "About", true, None::<&str>)?;

            // Build connect submenu
            let connect_submenu = {
                // Try to load connections synchronously
                let connections_result = std::panic::catch_unwind(|| {
                    let runtime = tokio::runtime::Runtime::new().unwrap();
                    runtime.block_on(async { commands::get_connections(None).await })
                });

                let mut builder = SubmenuBuilder::new(app, "Connect");

                match connections_result {
                    Ok(Ok(connections)) if !connections.is_empty() => {
                        println!("Loaded {} connection groups for submenu", connections.len());
                        let mut environments: Vec<_> = connections.keys().collect();
                        environments.sort();

                        for (index, env) in environments.iter().enumerate() {
                            let databases = connections.get(*env).unwrap();

                            // Add environment header (disabled item)
                            let header_text = format!("{} ({})", env, databases.len());
                            builder = builder.item(&MenuItem::with_id(
                                app,
                                &format!("env-header-{env}"),
                                &header_text,
                                false,
                                None::<&str>,
                            )?);

                            // Add connections with database-type icons
                            for db in databases {
                                let item_id = format!("connect-{}", db.alias);
                                let display_name = db.alias.clone();

                                // Try to create IconMenuItem with database-type icon
                                if let Some(icon) = get_db_icon(&db.db_type) {
                                    let icon_item = IconMenuItemBuilder::new(&display_name)
                                        .id(&item_id)
                                        .icon(icon)
                                        .build(app)?;
                                    builder = builder.item(&icon_item);
                                } else {
                                    // Fallback to regular MenuItem
                                    let item = MenuItem::with_id(
                                        app,
                                        &item_id,
                                        &display_name,
                                        true,
                                        None::<&str>,
                                    )?;
                                    builder = builder.item(&item);
                                }
                            }

                            // Add separator between environments (except last)
                            if index < environments.len() - 1 {
                                builder = builder.item(&PredefinedMenuItem::separator(app)?);
                            }
                        }
                    }
                    Ok(Ok(_)) => {
                        // No connections
                        builder = builder.item(&MenuItem::with_id(
                            app,
                            "connect-empty",
                            "No connections configured",
                            true,
                            None::<&str>,
                        )?);
                    }
                    Ok(Err(e)) => {
                        eprintln!("Failed to load connections: {e}");
                        builder = builder.item(&MenuItem::with_id(
                            app,
                            "connect-error",
                            "Error loading connections",
                            true,
                            None::<&str>,
                        )?);
                    }
                    Err(_) => {
                        eprintln!("Panic while loading connections");
                        builder = builder.item(&MenuItem::with_id(
                            app,
                            "connect-error",
                            "Error loading connections",
                            true,
                            None::<&str>,
                        )?);
                    }
                }

                builder.build()?
            };

            // Build config submenu
            let config_submenu = {
                let config_files_result = std::panic::catch_unwind(|| {
                    let runtime = tokio::runtime::Runtime::new().unwrap();
                    runtime.block_on(async { commands::get_config_files().await })
                });

                let mut builder = SubmenuBuilder::new(app, "Config");

                match config_files_result {
                    Ok(Ok(config_files)) if !config_files.is_empty() => {
                        println!("Loaded {} config files for submenu", config_files.len());
                        for file in &config_files {
                            let item_id = format!("config-{}", file.path);
                            let display_name = &file.name;
                            let item =
                                MenuItem::with_id(app, &item_id, display_name, true, None::<&str>)?;
                            builder = builder.item(&item);
                        }
                    }
                    Ok(Ok(_)) => {
                        builder = builder.item(&MenuItem::with_id(
                            app,
                            "config-empty",
                            "No config files",
                            true,
                            None::<&str>,
                        )?);
                    }
                    Ok(Err(e)) => {
                        eprintln!("Failed to load config files: {e}");
                        builder = builder.item(&MenuItem::with_id(
                            app,
                            "config-error",
                            "Error loading config",
                            true,
                            None::<&str>,
                        )?);
                    }
                    Err(_) => {
                        eprintln!("Panic while loading config files");
                        builder = builder.item(&MenuItem::with_id(
                            app,
                            "config-error",
                            "Error loading config",
                            true,
                            None::<&str>,
                        )?);
                    }
                }

                builder.build()?
            };

            // Build main tray menu
            let tray_menu = Menu::with_items(
                app,
                &[
                    &connect_submenu,
                    &config_submenu,
                    &PredefinedMenuItem::separator(app)?,
                    &about,
                    &PredefinedMenuItem::separator(app)?,
                    &quit,
                ],
            )?;

            // Create system tray with explicit icon
            let tray_icon = Image::from_bytes(include_bytes!("../icons/menubaricon.png"))?;
            let _tray = TrayIconBuilder::new()
                .icon(tray_icon)
                .icon_as_template(true)
                .menu(&tray_menu)
                .on_tray_icon_event(|tray, _event| {
                    // When tray icon is clicked, ensure the app is focused
                    let _ = tray.app_handle().set_activation_policy(tauri::ActivationPolicy::Accessory);
                })
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "about" => {
                        if let Some(about_window) = app.get_webview_window("about") {
                            let _ = about_window.show();
                            let _ = about_window.set_focus();
                        } else {
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
                    }
                    id if id.starts_with("connect-") => {
                        let alias = id[8..].to_string();
                        handle_connect(app, alias);
                    }
                    id if id.starts_with("config-") => {
                        let path = id[7..].to_string();
                        handle_open_config(app, path);
                    }
                    _ => {}
                })
                .show_menu_on_left_click(true)
                .build(app)?;

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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
