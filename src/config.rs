use crate::tools;
use color_eyre::eyre::{eyre, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{info, warn};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    // all database configs.
    pub databases: Vec<Database>,
    // templates for different database types.
    // key: db_type, value: connection url template.
    // e.g. mysql: mysql://{user}:{password}@{host}:{port}/{database}
    pub templates: HashMap<String, Template>,

    // Loaded from config file to help CLI usage. Only saved in MEMORY.
    // key: alias, value: Database config instance.
    #[serde(skip)]
    pub aliases: HashMap<String, Database>,
    #[serde(skip)]
    pub environments: HashMap<String, Vec<Database>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Template {
    pub dsn: String, // represent the connection url template.
    pub cli: String, // represent the command line template.
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Database {
    pub db_type: String,
    pub dsn: String,
    pub env: String,
    pub alias: String,
    // Optionally fields
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
            tools::MYSQL.to_string(),
            Template {
                dsn: "mysql://{user}:{password}@{host}:{port}/{database}".to_string(),
                cli: "mysql -h {host} -P {port} -u {user} -p{password} {database}".to_string(),
            },
        );
        templates.insert(
            tools::MONGODB.to_string(),
            Template {
                dsn: "mongodb://{user}:{password}@{host}:{port}/{database}".to_string(),
                cli: "mongosh mongodb://{user}:{password}@{host}:{port}/{database}".to_string(),
            },
        );
        templates.insert(
            tools::REDIS.to_string(),
            Template {
                dsn: "redis://{user}:{password}@{host}:{port}/{database}".to_string(),
                cli: "redis-cli -h {host} -p {port} -a {password}".to_string(),
            },
        );

        Self {
            databases: Vec::new(),
            templates: HashMap::new(),
            aliases: HashMap::new(),
            environments: HashMap::new(),
        }
    }

    pub fn validate_connection_string(&self, db_type: &str, url: &str) -> Result<()> {
        let template = self.templates.get(db_type).ok_or_else(|| {
            eyre!("No template found for database type: {}", db_type)
        })?;

        // TODO(@yeqown): verify the connection string format with the template.dsn.
        let required_components = vec!["host", "port"];
        for component in required_components {
            if !template.dsn.contains(component) {
                return Err(eyre!(
                    "Template missing required component: {}",
                    component
                ));
            }
        }

        // verify the url format, whether it matches the template format.
        if !url.starts_with(&format!("{}", db_type)) {
            return Err(eyre!(
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
        let mut config: Config = serde_yaml::from_str(&content)?;

        // Populate aliases and environments
        for db in &config.databases {
            config.aliases.insert(db.alias.clone(), db.clone());
            config
                .environments
                .entry(db.env.clone())
                .or_default()
                .push(db.clone());
        }
        Ok(config)
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

impl Database {
    // Parse the variables from the connection string. Including
    // the metadata, and the connection url itself as dsn.
    pub fn variables(&self, dsn_template: &str) -> HashMap<String, String> {
        let mut variables = HashMap::new();

        // Parse the connection url and extract the variables.
        // e.g. mysql://{user}:{password}@{host}:{port}/{database}
        // return a HashMap of variables.
        // E.g. { "user": "root", "password": "root", "host": "localhost", "port": "3306", "database": "test" }
        let vars = crate::template::parse_variables(&dsn_template, &self.dsn);
        if let Some(vars) = vars {
            for (key, value) in vars {
                variables.insert(key, value);
            }
        } else {
            warn!(
                "Could not parse variables from connection string: {}, Check please!!!",
                self.dsn,
            )
        }

        // Add metadata to the variables, but the key starts with "meta_".
        if let Some(metadata) = &self.metadata {
            for (key, value) in metadata {
                let meta_key = format!("meta_{}", key);
                variables.insert(meta_key, value.clone());
            }
        }

        // Add the connection url itself as dsn.
        variables.insert("dsn".to_string(), self.dsn.clone());

        variables
    }
}

// list_connections function to list all available connections.
// if env is specified, only list connections in that env.
// if db_type is specified, only list connections of that type.
pub fn list_connections(
    config: &Config,
    env_filter: Option<String>,
    db_type_filter: Option<String>,
) {
    println!("Databases:");

    let mut found_databases: u8 = 0;

    // Print grouped databases by env.
    for (env, db_list) in &config.environments {
        // if env is specified, only list connections in that env.
        if let Some(ref specified_env) = env_filter {
            if env.ne(specified_env) {
                continue;
            }
        }

        println!("  Environment: {}", env);
        for db in db_list {
            // if db_type is specified, only list connections of that type.
            if let Some(ref specified_db_type) = db_type_filter {
                if db.db_type.ne(specified_db_type) {
                    continue;
                }
            }

            found_databases += 1;
            println!("\t⭐️ Alias: {} [Type: {}] \n\t📜 Desc: {}",
                     db.alias,
                     db.db_type,
                     db.description.clone().unwrap_or(String::from("No description")),
            );
        }
    }

    if found_databases == 0 {
        println!("No databases found.");
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
        dsn: url.to_string(),
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
