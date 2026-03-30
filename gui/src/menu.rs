//! Menu building logic.

use tauri::{
    image::Image,
    menu::{IconMenuItemBuilder, Menu, MenuItem, PredefinedMenuItem, SubmenuBuilder},
};

use crate::commands::{get_config_files, ConfigFile, DatabaseDto, get_connections};

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

/// Build the tray menu with connections and config files.
pub fn build(app: &tauri::AppHandle) -> Result<Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let about = MenuItem::with_id(app, "about", "About", true, None::<&str>)?;

    let connect_submenu = build_connect_submenu(app)?;
    let config_submenu = build_config_submenu(app)?;

    Menu::with_items(
        app,
        &[
            &connect_submenu,
            &config_submenu,
            &PredefinedMenuItem::separator(app)?,
            &about,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )
    .map_err(Into::into)
}

fn build_connect_submenu(
    app: &tauri::AppHandle,
) -> Result<tauri::menu::Submenu<tauri::Wry>, Box<dyn std::error::Error>> {
    let connections = load_connections()?;
    let mut builder = SubmenuBuilder::new(app, "Connect");

    if connections.is_empty() {
        builder = builder.item(&MenuItem::with_id(
            app,
            "connect-empty",
            "No connections configured",
            true,
            None::<&str>,
        )?);
        return Ok(builder.build()?);
    }

    let mut environments: Vec<_> = connections.keys().collect();
    environments.sort();

    for (index, env) in environments.iter().enumerate() {
        let databases = connections.get(*env).unwrap();

        // Environment header
        builder = builder.item(&MenuItem::with_id(
            app,
            format!("env-header-{env}"),
            format!("{env} ({})", databases.len()),
            false,
            None::<&str>,
        )?);

        // Database items with icons
        for db in databases {
            let item_id = format!("connect-{}", db.alias);

            if let Some(icon) = get_db_icon(&db.db_type) {
                let icon_item = IconMenuItemBuilder::new(&db.alias)
                    .id(&item_id)
                    .icon(icon)
                    .build(app)?;
                builder = builder.item(&icon_item);
            } else {
                let item = MenuItem::with_id(app, &item_id, &db.alias, true, None::<&str>)?;
                builder = builder.item(&item);
            }
        }

        // Separator between environments
        if index < environments.len() - 1 {
            builder = builder.item(&PredefinedMenuItem::separator(app)?);
        }
    }

    Ok(builder.build()?)
}

fn build_config_submenu(
    app: &tauri::AppHandle,
) -> Result<tauri::menu::Submenu<tauri::Wry>, Box<dyn std::error::Error>> {
    let config_files = load_config_files()?;
    let mut builder = SubmenuBuilder::new(app, "Config");

    // Add Manage menu item at the top
    let manage_item = MenuItem::with_id(app, "config-manage", "Manage...", true, None::<&str>)?;
    builder = builder.item(&manage_item);

    // Add separator
    builder = builder.item(&PredefinedMenuItem::separator(app)?);

    if config_files.is_empty() {
        builder = builder.item(&MenuItem::with_id(
            app,
            "config-empty",
            "No config files",
            true,
            None::<&str>,
        )?);
        return Ok(builder.build()?);
    }

    for file in &config_files {
        let item_id = format!("config-{}", file.path);
        let item = MenuItem::with_id(app, &item_id, &file.name, true, None::<&str>)?;
        builder = builder.item(&item);
    }

    Ok(builder.build()?)
}

fn load_connections() -> Result<std::collections::HashMap<String, Vec<DatabaseDto>>, Box<dyn std::error::Error>> {
    let runtime = tokio::runtime::Runtime::new()?;
    let result = runtime.block_on(async { get_connections(None).await })?;
    Ok(result)
}

fn load_config_files() -> Result<Vec<ConfigFile>, Box<dyn std::error::Error>> {
    let runtime = tokio::runtime::Runtime::new()?;
    let result = runtime.block_on(async { get_config_files().await })?;
    Ok(result)
}
