use dbhub_core::{config, Config, Database};
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

#[derive(Serialize, Deserialize)]
pub struct ConfigDto {
    pub databases: Vec<DatabaseDto>,
    pub templates: Option<HashMap<String, TemplateDto>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TemplateDto {
    pub dsn: String,
}

#[tauri::command]
pub async fn get_connections() -> Result<HashMap<String, Vec<DatabaseDto>>, String> {
    let config = config::loads().map_err(|e| e.to_string())?;
    let mut grouped: HashMap<String, Vec<DatabaseDto>> = HashMap::new();

    for db in config.databases {
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
        .ok_or_else(|| format!("Database not found: {}", alias))?;

    let _db = config
        .get_database_by_index(db_index)
        .ok_or_else(|| format!("Database not found: {}", alias))?;

    // Build dbhub CLI command
    let mut cmd = format!("dbhub connect {}", alias);
    if let Some(args) = runtime_args {
        cmd.push_str(" -- ");
        cmd.push_str(&args);
    }

    // Execute in new Terminal window
    let escaped_cmd = cmd.replace('\\', "\\\\").replace('"', "\\\"");

    Command::new("osascript")
        .arg("-e")
        .arg(format!(
            "tell application \"Terminal\" to do script \"{}\"",
            escaped_cmd
        ))
        .spawn()
        .map_err(|e| format!("Failed to open terminal: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn add_database(db: DatabaseDto) -> Result<(), String> {
    let mut config = config::loads().map_err(|e| e.to_string())?;

    // Check for duplicate alias
    if config.databases.iter().any(|d| d.alias == db.alias) {
        return Err(format!("Alias already exists: {}", db.alias));
    }

    config.databases.push(db.into());
    save_config(&config)?;
    Ok(())
}

#[tauri::command]
pub async fn update_database(alias: String, db: DatabaseDto) -> Result<(), String> {
    let mut config = config::loads().map_err(|e| e.to_string())?;

    let index = config
        .databases
        .iter()
        .position(|d| d.alias == alias)
        .ok_or_else(|| format!("Database not found: {}", alias))?;

    config.databases[index] = db.into();
    save_config(&config)?;
    Ok(())
}

#[tauri::command]
pub async fn delete_database(alias: String) -> Result<(), String> {
    let mut config = config::loads().map_err(|e| e.to_string())?;

    let index = config
        .databases
        .iter()
        .position(|d| d.alias == alias)
        .ok_or_else(|| format!("Database not found: {}", alias))?;

    config.databases.remove(index);
    save_config(&config)?;
    Ok(())
}

fn save_config(config: &Config) -> Result<(), String> {
    use std::fs;
    use std::io::Write;

    // Get config path - default to ~/.dbhub/config.yml
    let home = dirs::home_dir().ok_or("Cannot determine home directory")?;
    let config_path = home.join(".dbhub").join("config.yml");

    // Backup existing config
    if config_path.exists() {
        let backup_path = config_path.with_extension("yml.bak");
        fs::copy(&config_path, &backup_path).map_err(|e| e.to_string())?;
    }

    // Serialize and save
    let yaml = serde_yaml::to_string(config).map_err(|e| e.to_string())?;

    // Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let mut file = fs::File::create(&config_path).map_err(|e| e.to_string())?;
    file.write_all(yaml.as_bytes()).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_config() -> Result<ConfigDto, String> {
    let config = config::loads().map_err(|e| e.to_string())?;

    let templates = config.templates.map(|t| {
        t.into_iter()
            .map(|(k, v)| (k, TemplateDto { dsn: v.dsn }))
            .collect()
    });

    Ok(ConfigDto {
        databases: config.databases.into_iter().map(|d| d.into()).collect(),
        templates,
    })
}

#[tauri::command]
pub async fn save_config_dto(config: ConfigDto) -> Result<(), String> {
    let mut core_config = Config {
        databases: config.databases.into_iter().map(|d| d.into()).collect(),
        templates: config.templates.map(|t| {
            t.into_iter()
                .map(|(k, v)| (k, dbhub_core::Template { dsn: v.dsn }))
                .collect()
        }),
        aliases: HashMap::new(),
        environments: HashMap::new(),
    };

    // Rebuild indexes
    for (i, db) in core_config.databases.iter().enumerate() {
        core_config.aliases.insert(db.alias.clone(), i);
        core_config
            .environments
            .entry(db.env.clone())
            .or_default()
            .push(i);
    }

    save_config(&core_config)?;
    Ok(())
}
