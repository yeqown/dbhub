use crate::embedded::Configs;
use color_eyre::eyre::{eyre, Result};
use console::{style, StyledObject};
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

pub const MYSQL: &str = "mysql";
pub const MYSQL_DSN_TEMPLATE: &str = "mysql://{user}:{password}@{host}:{port}/{database}";

pub const MONGO: &str = "mongo";

pub const MONGO_DSN_TEMPLATE: &str = "mongodb://{user}:{password}@{host}:{port}/{database}";

pub const REDIS: &str = "redis";

pub const REDIS_DSN_TEMPLATE: &str = "redis://{user}:{password}@{host}:{port}/{database}";

impl Config {
    pub fn new() -> Self {
        let mut templates = HashMap::new();
        templates.insert(
            MYSQL.to_string(),
            Template {
                dsn: MYSQL_DSN_TEMPLATE.to_string(),
            },
        );
        templates.insert(
            MONGO.to_string(),
            Template {
                dsn: MONGO_DSN_TEMPLATE.to_string(),
            },
        );
        templates.insert(
            REDIS.to_string(),
            Template {
                dsn: REDIS_DSN_TEMPLATE.to_string(),
            },
        );

        Self {
            databases: Vec::new(),
            templates: HashMap::new(),
            aliases: HashMap::new(),
            environments: HashMap::new(),
        }
    }
}

pub fn load_or_create(config_path: &PathBuf) -> Result<Config> {
    if !config_path.exists() {
        // Copy configs/sample.yml to `config_path`
        let sample_config = Configs::get("sample.yml");
        if sample_config.is_none() {
            return Err(eyre!("Sample config not found"));
        }

        let sample_config = sample_config.unwrap();
        std::fs::create_dir_all(config_path.parent().unwrap())?;
        std::fs::write(config_path, sample_config.data)?;

        info!("No config file found, apply the sample config file to: {:?}", config_path);
    }

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
}

impl Database {
    // Parse the variables from the connection string. Including
    // the metadata, and the connection url itself as dsn.
    pub fn variables(&self, dsn_template: &str) -> Result<HashMap<String, String>> {
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
            return Err(eyre!("Could not parse variables: {} !!!",self.dsn));
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

        Ok(variables)
    }

    pub fn validate_connection_string(&self, cfg: &Config) -> Result<bool> {
        let template = cfg.templates.get(&self.db_type).ok_or_else(|| {
            eyre!("No template found for database type: {}", self.db_type)
        })?;

        let vars = self.variables(&template.dsn);
        if let Err(e) = vars {
            return Err(e);
        }

        Ok(true)
    }
}


pub fn list_connections(
    config: &Config,
    env_filter: Option<String>,
    db_type_filter: Option<String>,
) {
    println!("{}", style("Databases:").bold());

    let mut found_databases = 0;

    // group the databases by env and db_type.
    let mut grouped_databases: std::collections::BTreeMap<&str, std::collections::BTreeMap<&str, Vec<&Database>>> = std::collections::BTreeMap::new();
    for (env, db_list) in &config.environments {
        // if env is specified, only list connections in that env.
        if let Some(ref specified_env) = env_filter {
            if env != specified_env {
                continue;
            }
        }

        for db in db_list {
            // if db_type is specified, only list connections of that type.
            if let Some(ref specified_db_type) = db_type_filter {
                if db.db_type != *specified_db_type {
                    continue;
                }
            }

            grouped_databases
                .entry(env)
                .or_insert_with(std::collections::BTreeMap::new)
                .entry(&db.db_type)
                .or_insert_with(Vec::new)
                .push(db);
        }
    }

    for (env, db_type_map) in grouped_databases {
        let styled_env: StyledObject<&str> = style(env).blue().bold();
        println!("  {}:", styled_env);

        for (db_type, db_list) in db_type_map {
            let styled_db_type: StyledObject<&str> = style(db_type).green().bold();
            println!("    {}:", styled_db_type);

            for db in db_list {
                found_databases += 1;
                let alias = format!("⭐️ Alias: {}", style(&db.alias).bold());
                let desc = format!("📜 Desc : {}", style(db.description.clone().unwrap_or_else(|| "No description".to_string())).dim());
                println!("      {} \n      {}", alias, desc);
            }
        }
    }

    if found_databases == 0 {
        println!("{}", style("No databases found.").red());
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
    let db = Database {
        db_type: db_type.to_string(),
        dsn: url.to_string(),
        env: env.to_string(),
        alias: alias.to_string(),
        description: Option::from(description.unwrap().to_string()),
        metadata: None,
    };

    // Make sure the alias is unique
    if config.aliases.contains_key(alias) {
        return Err(eyre!("Alias '{}' already exists", alias));
    }

    if let Err(e) = db.validate_connection_string(config) {
        return Err(e);
    }

    // save the connection config to the config file.
    config.databases.push(db.clone());

    let content = serialize_config(&config)?;
    std::fs::write(config_path, content)?;

    info!("Added new database connection with type '{}'.", db_type);
    Ok(())
}

pub fn serialize_config(config: &Config) -> Result<String> {
    Ok(serde_yaml::to_string(config)?)
}
