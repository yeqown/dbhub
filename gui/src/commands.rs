use dbhub_core::{config, Database, InitResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
pub struct DatabaseDto {
    pub alias: String,
    pub db_type: String,
    pub dsn: String,
    pub env: String,
    pub description: Option<String>,
    pub annotations: Option<HashMap<String, String>>,
}

impl From<Database> for DatabaseDto {
    fn from(db: Database) -> Self {
        Self {
            alias: db.alias,
            db_type: db.db_type,
            dsn: db.dsn,
            env: db.env,
            description: db.description,
            annotations: db.annotations,
        }
    }
}

impl From<DatabaseDto> for Database {
    fn from(dto: DatabaseDto) -> Self {
        Self {
            db_type: dto.db_type,
            dsn: dto.dsn,
            env: dto.env,
            alias: dto.alias,
            description: dto.description,
            annotations: dto.annotations,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilterParams {
    pub env: Option<String>,
    pub db_type: Option<String>,
    pub search: Option<String>,
}

#[tauri::command]
pub async fn get_connections(
    filter: Option<FilterParams>,
) -> Result<HashMap<String, Vec<DatabaseDto>>, String> {
    println!("[DEBUG] get_connections called with filter: {filter:?}");

    let config = config::loads().map_err(|e| {
        eprintln!("[ERROR] Failed to load config: {e}");
        e.to_string()
    })?;

    println!(
        "[DEBUG] Loaded {} databases from config",
        config.databases.len()
    );
    let mut grouped: HashMap<String, Vec<DatabaseDto>> = HashMap::new();

    // Collect all databases first
    let mut all_databases: Vec<Database> = config.databases;

    // Apply filters if provided
    if let Some(filter_params) = filter {
        all_databases.retain(|db| {
            // Environment filter
            if let Some(ref env_filter) = filter_params.env {
                if &db.env != env_filter {
                    return false;
                }
            }

            // Database type filter
            if let Some(ref type_filter) = filter_params.db_type {
                if &db.db_type != type_filter {
                    return false;
                }
            }

            // Search filter
            if let Some(ref search_term) = filter_params.search {
                let search_lower = search_term.to_lowercase();
                let search_in = format!(
                    "{}{}{}{}{}",
                    db.alias,
                    db.db_type,
                    db.env,
                    db.description.as_ref().unwrap_or(&String::new()),
                    db.dsn
                )
                .to_lowercase();

                if !search_in.contains(&search_lower) {
                    return false;
                }
            }

            true
        });
    }

    // Group by environment
    for db in all_databases {
        grouped.entry(db.env.clone()).or_default().push(db.into());
    }

    Ok(grouped)
}

#[tauri::command]
pub async fn connect(alias: String, runtime_args: Option<String>) -> Result<(), String> {
    use std::process::Command;

    let config = config::loads().map_err(|e| e.to_string())?;

    // Get database by alias
    let db_index = config
        .aliases
        .get(&alias)
        .ok_or_else(|| format!("Database not found: {alias}"))?;

    let _db = config
        .get_database_by_index(*db_index)
        .ok_or_else(|| format!("Database not found: {alias}"))?;

    // Build dbhub CLI command
    let mut cmd = format!("dbhub connect {alias}");
    if let Some(args) = runtime_args {
        cmd.push_str(" -- ");
        cmd.push_str(&args);
    }

    // Execute in new Terminal window and activate it
    let escaped_cmd = cmd.replace('\\', "\\\\").replace('"', "\\\"");

    Command::new("osascript")
        .arg("-e")
        .arg("tell application \"Terminal\" to activate")
        .arg("-e")
        .arg(format!(
            "tell application \"Terminal\" to do script \"{escaped_cmd}\""
        ))
        .spawn()
        .map_err(|e| format!("Failed to open terminal: {e}"))?;

    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct ConfigFile {
    pub path: String,
    pub name: String,
}

#[tauri::command]
pub async fn get_config_files() -> Result<Vec<ConfigFile>, String> {
    let config_paths = dbhub_core::get_config_paths();

    let config_files: Vec<ConfigFile> = config_paths
        .iter()
        .map(|path| {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.to_string_lossy().to_string());
            ConfigFile {
                path: path.to_string_lossy().to_string(),
                name,
            }
        })
        .collect();

    Ok(config_files)
}

#[tauri::command]
pub async fn open_config_editor(path: String) -> Result<(), String> {
    use std::process::Command;

    // Determine the default text editor for macOS
    // Try common editors in order: TextEdit, VS Code, Sublime Text, default open command
    if cfg!(target_os = "macos") {
        // On macOS, use 'open' command with default text editor
        Command::new("open")
            .args(["-t", &path])
            .spawn()
            .map_err(|e| format!("Failed to open editor: {e}"))?;
    } else {
        // On Linux, try xdg-open
        Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open editor: {e}"))?;
    }

    Ok(())
}

#[tauri::command]
pub async fn open_repository(url: String) -> Result<(), String> {
    use std::process::Command;

    // Open URL in default browser
    if cfg!(target_os = "macos") {
        Command::new("open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("Failed to open URL: {e}"))?;
    } else {
        Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("Failed to open URL: {e}"))?;
    }

    Ok(())
}

#[tauri::command]
pub async fn initialize_config() -> Result<String, String> {
    match dbhub_core::config::generate_default_config() {
        Ok(()) => Ok("Configuration created successfully".to_string()),
        Err(e) => {
            let error_msg = format!("Failed to create configuration: {}", e);
            eprintln!("{}", error_msg);
            Err(error_msg)
        }
    }
}

#[tauri::command]
pub async fn get_init_status(
    init_result: tauri::State<'_, InitResult>,
) -> Result<InitResult, String> {
    Ok(init_result.inner().clone())
}

#[tauri::command]
pub async fn read_config_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|e| format!("Failed to read config file: {e}"))
}

#[tauri::command]
pub async fn save_config_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, &content).map_err(|e| format!("Failed to save config file: {e}"))
}

#[tauri::command]
pub async fn create_config_file(name: String) -> Result<String, String> {
    let config_dir = dbhub_core::get_config_dir()
        .ok_or_else(|| "Cannot determine config directory".to_string())?;

    // Ensure the directory exists
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir)
            .map_err(|e| format!("Failed to create config directory: {e}"))?;
    }

    // Ensure the name ends with .yml
    let filename = if name.ends_with(".yml") || name.ends_with(".yaml") {
        name
    } else {
        format!("{}.yml", name)
    };

    let file_path = config_dir.join(&filename);

    // Check if file already exists
    if file_path.exists() {
        return Err(format!("Config file already exists: {}", filename));
    }

    // Create empty config file with basic structure
    let default_content = "databases: ";
    std::fs::write(&file_path, default_content)
        .map_err(|e| format!("Failed to create config file: {e}"))?;

    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn delete_config_file(path: String) -> Result<(), String> {
    // Safety check: don't delete if it's the only config file
    let config_paths = dbhub_core::get_config_paths();
    if config_paths.len() <= 1 {
        return Err("Cannot delete the last config file".to_string());
    }

    std::fs::remove_file(&path).map_err(|e| format!("Failed to delete config file: {e}"))
}
