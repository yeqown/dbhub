use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    // all database configs.
    pub databases: Vec<Database>,
    // templates for different database types.
    // key: db_type, value: connection url template.
    // e.g. mysql: mysql://{user}:{password}@{host}:{port}/{database}
    pub templates: HashMap<String, String>,

    // Loaded from config file to help CLI usage. only saved in MEMORY.
    // key: alias, value: Database config instance.
    #[serde(skip)]
    pub aliases: HashMap<String, Database>,
    #[serde(skip)]
    pub environments: HashMap<String, Vec<Database>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Database {
    pub db_type: String,
    pub url: String,
    pub env: String,
    pub alias: String,
    // Optional fields
    // Description of the connection for human-readable.
    pub description: Option<String>,
    // metadata for the connection.
    // e.g. { "region": "us-west-1", "account_id": "123456789012" }
    // e.g. { "redis-sentinel": "1", "master-name": "my-master" } for redis sentinel.
    pub metadata: Option<HashMap<String, String>>,
}

impl Config {
    pub fn new() -> Self {
        let mut templates = HashMap::new();
        templates.insert(
            "mysql".to_string(),
            "mysql://{user}:{password}@{host}:{port}/{database}".to_string(),
        );
        templates.insert(
            "mongodb".to_string(),
            "mongodb://{user}:{password}@{host}:{port}/{database}".to_string(),
        );
        templates.insert(
            "redis".to_string(),
            "redis://{user}:{password}@{host}:{port}/{database}".to_string(),
        );

        Self {
            databases: Vec::new(),
            templates,
            aliases: HashMap::new(),
            environments: HashMap::new(),
        }
    }

    pub fn validate_connection_string(&self, db_type: &str, url: &str) -> Result<()> {
        let template = self.templates.get(db_type).ok_or_else(|| {
            color_eyre::eyre::eyre!("No template found for database type: {}", db_type)
        })?;

        // 简单验证：检查必要的组件是否存在
        let required_components = vec!["host", "port"];
        for component in required_components {
            if !template.contains(component) {
                return Err(color_eyre::eyre::eyre!(
                    "Template missing required component: {}",
                    component
                ));
            }
        }

        // verify the url format, whether it matches the template format.
        if !url.starts_with(&format!("{}", db_type)) {
            return Err(color_eyre::eyre::eyre!(
                "Invalid URL format for {}",
                db_type
            ));
        }

        Ok(())
    }
}


pub fn load_or_create(config_path: &PathBuf) -> Result<Config> {
    if config_path.exists() {
        let content = std::fs::read_to_string(config_path)?;
        Ok(serde_yaml::from_str(&content)?)
    } else {
        let config = Config::new();
        let content = serde_yaml::to_string(&config)?;
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(config_path, content)?;
        Ok(config)
    }
}


pub fn list_connections(config: &Config) {
    println!("Databases:");
    for (index, db) in config.databases.iter().enumerate() {
        println!(
            "  {}: {} ({}), Env: {}, Alias: {:?}",
            index + 1,
            db.db_type,
            db.url,
            db.env,
            db.alias
        );
    }

    println!("\nDB Connection Templates:");
    for (db_type, template) in &config.templates {
        println!("  {}: {}", db_type, template);
    }
}
pub fn add_connection(
    config_path: &PathBuf,
    config: &mut Config,
    db_type: &str,
    url: &str,
    env: &str,
    alias: &str,
    description: Option<String>,
) -> Result<()> {
    // validate the connection string
    config.validate_connection_string(db_type, url)?;

    // save the connection config to the config file.
    let new_db_index = config.environments.len();
    config.databases.push(Database {
        db_type: db_type.to_string(),
        url: url.to_string(),
        env: env.to_string(),
        alias: alias.to_string(),
        description: Option::from(description.unwrap().to_string()),
        metadata: None,
    });
    _ = new_db_index;

    // Make sure the alias is unique
    if config.aliases.contains_key(alias) {
        return Err(color_eyre::eyre::eyre!("Alias '{}' already exists", alias));
    }

    let content = serialize_config(&config)?;
    std::fs::write(config_path, content)?;

    info!("Added new database connection with type '{}'.", db_type);
    Ok(())
}

pub fn serialize_config(config: &Config) -> Result<String> {
    Ok(serde_yaml::to_string(config)?)
}
