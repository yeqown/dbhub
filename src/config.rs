use crate::cli::ContextArgs;
use crate::embedded::Configs;
use color_eyre::eyre::{eyre, Result};
use console::{style, StyledObject};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    collections::HashMap,
    path,
};

use tracing::{debug, error, info, warn};


#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    // all database configs.
    pub databases: Vec<Database>,
    // templates for different database types.
    // key: db_type, value: connection url template.
    // e.g. mysql: mysql://{user}:{password}@{host}:{port}/{database}
    pub templates: Option<HashMap<String, Template>>,

    // Loaded from config file to help CLI usage. Only saved in MEMORY.
    // key: alias, value: Database index in databases.
    #[serde(skip)]
    pub aliases: HashMap<String, usize>,
    #[serde(skip)]
    pub environments: HashMap<String, Vec<usize>>,
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
    // annotations for the connection.
    // e.g. { "region": "us-west-1", "account_id": "123,456,789,012" }
    // e.g. { "redis-sentinel": "1", "master-name": "my-master" } for redis sentinel.
    pub annotations: Option<HashMap<String, String>>,
}

impl Config {
    #[allow(unused)]
    pub(super) fn get_all_aliases(&self) -> Vec<&str> {
        self.aliases.keys().map(|alias| alias.as_str()).collect()
    }
    
    pub(super) fn get_mut_templates(&mut self) -> &mut HashMap<String, Template> {
        self.templates.as_mut().unwrap()
    }

    pub(super) fn get_templates(&self) -> &HashMap<String, Template> {
        self.templates.as_ref().unwrap()
    }

    pub(super) fn get_database_by_index(&self, index: &usize) -> Option<&Database> {
        self.databases.get(*index)
    }
}

/// DBHUB_CONFIG environment variable.
/// If set, use the value as the config file path.
/// If not set, use the default config file path.
/// The default config file path is `~/.dbhub/config.yml`.
///
/// Environment variable MUST follow the following format:
///
/// ```bash
/// export DBHUB_CONFIG=~/.dbhub/config1.yml:~/.dbhub/config2.yml
/// ```
const DBHUB_CONFIG_ENV: &str = "DBHUB_CONFIG";
const DEFAULT_CONFIG_PATH: &str = "~/.dbhub/config.yml";
const SAMPLE_CONFIG_FILE_PATH: &str = "sample.yml";

fn get_config_paths() -> Vec<path::PathBuf> {
    match std::env::var(DBHUB_CONFIG_ENV) {
        Ok(paths) => {
            paths
                .split(':')
                .map(|path| -> path::PathBuf {
                    deal_config_path(path).unwrap()
                })
                .collect()
        }
        Err(_) => {
            info!("No DBHUB_CONFIG environment variable found, use the default config file path: {:?}", DEFAULT_CONFIG_PATH);
            match deal_config_path(DEFAULT_CONFIG_PATH) {
                Some(path) => vec![path],
                None => vec![],
            }
        }
    }
}

fn deal_config_path(path: &str) -> Option<path::PathBuf> {
    if let Some(ref home) = dirs::home_dir() {
        // Unix-like system (e.g., Linux, macOS)
        #[cfg(target_os = "macos")]
        if path.starts_with("~") {
            let relative_path = path.strip_prefix("~/").unwrap_or("");
            return Some(home.join(relative_path));
        }

        // Windows system
        #[cfg(target_os = "windows")]
        if path.starts_with("%USERPROFILE%") {
            let relative_path = path.strip_prefix("%USERPROFILE%/").unwrap_or(Path::new(""));
            return Some(home.join(PathBuf::from(relative_path)));
        }
    }

    Some(path::PathBuf::from(path))
}

pub fn generate_default_config() -> Result<()> {
    let config_path = deal_config_path(DEFAULT_CONFIG_PATH).unwrap();

    // Copy configs/sample.yml to `config_path`
    let sample_config = Configs::get(SAMPLE_CONFIG_FILE_PATH);
    if sample_config.is_none() {
        return Err(eyre!("Sample config not found"));
    }

    let sample_config = sample_config.unwrap();
    std::fs::create_dir_all(config_path.parent().unwrap())?;
    std::fs::write(&config_path, sample_config.data)?;

    info!("No config file found, apply the sample config file to: {:?}", config_path);
    Ok(())
}

pub fn loads() -> Result<Config> {
    let config_paths = get_config_paths();
    if config_paths.is_empty() {
        return Err(eyre!("No config file are set"));
    }

    // iterate all config files and merge them.
    let mut config = Config {
        databases: Vec::new(),
        templates: Some(HashMap::new()),
        aliases: HashMap::new(),
        environments: HashMap::new(),
    };

    for ref config_path in config_paths {
        match load_config(config_path) {
            Ok(incoming) => {
                config.databases.extend(incoming.databases);
                if let Some(templates) = incoming.templates {
                    config.get_mut_templates().extend(templates);
                }
            }
            Err(e) => {
                warn!("Failed to load config file: {:?}, error: {:?}", config_path, e)
            }
        }
    }

    // Populate aliases and environments, and warn about duplicates.
    for (i, db) in config.databases.iter().enumerate() {
        if config.aliases.contains_key(&db.alias) {
            warn!("Duplicate alias found: {}", &db.alias);
        }

        config.aliases.insert(db.alias.clone(), i);
        config.environments
            .entry(db.env.clone())
            .or_default()
            .push(i);
    }

    Ok(config)
}

fn load_config<P: AsRef<path::Path>>(config_path: P) -> Result<Config> {
    match std::fs::read_to_string(config_path.as_ref()) {
        Ok(content) => {
            match serde_yaml::from_str(&content) {
                Ok(config) => Ok(config),
                Err(e) => {
                    error!("Failed to parse config file: {:?}, error: {:?}", config_path.as_ref(), e);
                    Err(e.into())
                }
            }
        }
        Err(e) => {
            Err(e.into())
        }
    }
}

impl Database {
    // Parse the variables from the connection string. Including
    // the annotations, and the connection url itself as dsn.
    pub fn variables(&self, dsn_template: &str) -> Result<(HashMap<String, String>, HashMap<String, String>)> {
        // Parse the connection url and extract the variables.
        // e.g. mysql://{user}:{password}@{host}:{port}/{database}
        // return a HashMap of variables.
        // E.g. { "user": "root", "password": "root", "host": "localhost", "port": "3306", "database": "test" }
        let mut variables = HashMap::new();
        let vars = crate::template::parse_variables(&dsn_template, &self.dsn);
        if let Some(vars) = vars {
            for (key, value) in vars {
                variables.insert(key, value);
            }
        } else {
            return Err(eyre!("Could not parse variables: {} !!!",self.dsn));
        }
        // Add the connection url itself as dsn.
        variables.insert("dsn".to_string(), self.dsn.clone());

        let mut annotations = HashMap::new();
        // Add annotations to the variables, but the key starts with "meta_".
        if let Some(annos) = &self.annotations {
            for (key, value) in annos {
                annotations.insert(key.clone(), value.clone());
            }
        }

        Ok((variables, annotations))
    }

    // pub fn validate_connection_string(&self, cfg: &Config) -> Result<bool> {
    //     let template = cfg.templates.get(&self.db_type).ok_or_else(|| {
    //         eyre!("No template found for database type: {}", self.db_type)
    //     })?;
    //
    //     let vars = self.variables(&template.dsn);
    //     if let Err(e) = vars {
    //         return Err(e);
    //     }
    //
    //     Ok(true)
    // }
}


#[derive(Debug, Default)]
pub struct Filter {
    pub env: Option<String>,
    pub db_type: Option<String>,
    pub alias: Option<String>,
}

impl Filter {
    pub fn from_args(args: &ContextArgs) -> Self {
        let mut filter = Filter::default();
        if let Some(ref env) = args.filter_env {
            filter.env = Some(env.clone());
        }
        if let Some(ref db_type) = args.filter_db_type {
            filter.db_type = Some(db_type.clone());
        }
        if let Some(ref alias) = args.filter_alias {
            filter.alias = Some(alias.clone());
        }

        filter
    }
}

#[derive(Debug)]
struct ListFormat {
    pub with_desc: bool,
    pub with_dsn: bool,
    pub with_annotations: bool,
}

impl ListFormat {
    pub fn from_args(args: &ContextArgs) -> Self {
        let mut format = ListFormat::default();
        // format.with_desc = args.get_flag("with-desc");
        format.with_dsn = args.with_dsn;
        format.with_annotations = args.with_annotations;
        format
    }
}

impl Default for ListFormat {
    fn default() -> Self {
        Self {
            with_desc: true,
            with_dsn: false,
            with_annotations: false,
        }
    }
}

#[derive(Debug, Default)]
pub struct ListOptions {
    pub filter: Filter,
    format: ListFormat,
}

// impl Default for ListOptions {
//     fn default() -> Self {
//         Self {
//             filter: Filter::default(),
//             format: ListFormat::default(),
//         }
//     }
// }

impl ListOptions {
    pub fn from_args(args: &ContextArgs) -> Self {
        let mut options = ListOptions::default();
        options.filter = Filter::from_args(args);
        options.format = ListFormat::from_args(args);
        options
    }
}

pub fn list_connections(
    config: &Config,
    opts: &ListOptions,
) {
    println!("{}", style("Databases:").bold());

    debug!("list_connections with options: {:?}", opts);

    let mut found_databases = 0;

    // group the databases by env and db_type.
    let mut grouped_databases: std::collections::BTreeMap<&str, std::collections::BTreeMap<&str, Vec<&Database>>> = std::collections::BTreeMap::new();
    for (env, db_list) in config.environments.iter() {
        // if env is specified, only list connections in that env.
        if let Some(ref specified_env) = opts.filter.env {
            if env != specified_env {
                continue;
            }
        }

        for db_index in db_list {
            let db = config.get_database_by_index(db_index).unwrap();

            // if alias is specified, only list connections with that alias.
            if let Some(ref specified_alias) = opts.filter.alias {
                if &db.alias != specified_alias {
                    continue;
                }
            }

            // if db_type is specified, only list connections of that type.
            if let Some(ref specified_db_type) = opts.filter.db_type {
                if db.db_type != *specified_db_type {
                    continue;
                }
            }

            found_databases += 1;

            grouped_databases
                .entry(env)
                .or_default()
                .entry(&db.db_type)
                .or_default()
                .push(db);
        }
    }

    print_databases(grouped_databases, opts);

    if found_databases == 0 {
        println!("{}", style("No databases found.").red());
    }
}

fn print_databases(
    grouped_databases: BTreeMap<&str, BTreeMap<&str, Vec<&Database>>>,
    opts: &ListOptions,
) {
    for (env, db_type_map) in grouped_databases {
        let styled_env: StyledObject<&str> = style(env).blue().bold();
        println!("  {styled_env}");

        for (db_type, db_list) in db_type_map {
            let styled_db_type: StyledObject<&str> = style(db_type).green().bold();
            println!("    {styled_db_type}");

            let mut is_first = true;

            for db in db_list {
                if !is_first {
                    println!();
                }

                let alias = format!("🚀 Alias: {}", style(&db.alias).bold());
                println!("\t{alias}");

                if opts.format.with_desc {
                    let desc = format!("📜 Desc : {}", style(db.description.clone().unwrap_or_else(|| "No description".to_string())).dim());
                    println!("\t{desc}");
                }

                if opts.format.with_dsn {
                    let dsn = format!("🔗 DSN : {}", style(&db.dsn).dim());
                    println!("\t{dsn}");
                }

                if opts.format.with_annotations {
                    if let Some(annos) = &db.annotations {
                        println!("\t{}", style("📝 Annotations:").bold());
                        for (key, value) in annos {
                            let anno = format!("-> \"{key}\": {value}");
                            println!("\t\t{anno}");
                        }
                    }
                }

                is_first = false;
            }
        }
    }
}