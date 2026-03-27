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

/// Initialization status for configuration directory
#[derive(Debug, PartialEq, Clone)]
pub enum InitStatus {
    /// Config directory exists with at least one valid config file
    AlreadyExists,
    /// Config directory does not exist
    NotInitialized,
    /// Config directory exists but contains no valid config files
    NoValidConfig,
}

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
    pub(super) fn get_all_aliases(&self) -> Vec<String> {
        self.aliases.keys().cloned().collect()
    }

    pub(super) fn get_mut_templates(&mut self) -> &mut HashMap<String, Template> {
        self.templates.as_mut().unwrap()
    }

    pub(super) fn get_templates(&self) -> &HashMap<String, Template> {
        self.templates.as_ref().unwrap()
    }

    pub fn get_database_by_index(&self, index: &usize) -> Option<&Database> {
        if index < &(0usize) || index > &self.databases.len() {
            return None;
        }
        
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

/// Get all configuration file paths.
///
/// Priority:
/// 1. If `DBHUB_CONFIG` environment variable is set, use it (for backward compatibility)
/// 2. Otherwise, auto-scan `~/.dbhub/` directory for all .yml/.yaml files
///
/// Returns a vector of configuration file paths sorted alphabetically.
pub fn get_config_paths() -> Vec<path::PathBuf> {
    // Priority 1: Check DBHUB_CONFIG environment variable (backward compatibility)
    if let Ok(paths) = std::env::var(DBHUB_CONFIG_ENV) {
        info!("Using DBHUB_CONFIG environment variable: {}", paths);
        return paths
            .split(':')
            .filter_map(deal_config_path)
            .collect();
    }

    // Priority 2: Auto-scan ~/.dbhub/ directory
    info!("No DBHUB_CONFIG environment variable found, scanning ~/.dbhub/ directory");

    let config_dir = if let Some(ref home) = dirs::home_dir() {
        home.join(".dbhub")
    } else {
        warn!("Cannot determine home directory");
        return vec![];
    };

    // If config directory doesn't exist, return default path
    if !config_dir.exists() {
        info!("Config directory does not exist: {:?}", config_dir);
        return deal_config_path(DEFAULT_CONFIG_PATH).into_iter().collect();
    }

    // Scan for all .yml and .yaml files
    let mut config_files = scan_config_directory(&config_dir);

    // Sort files alphabetically for consistent ordering
    config_files.sort();

    if config_files.is_empty() {
        info!("No config files found in {:?}, using default: {:?}", config_dir, DEFAULT_CONFIG_PATH);
        deal_config_path(DEFAULT_CONFIG_PATH).into_iter().collect()
    } else {
        info!("Found {} config file(s) in {:?}: {:?}", config_files.len(), config_dir, config_files);
        config_files
    }
}

/// Scan the configuration directory for valid config files.
///
/// Returns a vector of paths to .yml/.yaml files, excluding:
/// - Backup files (files ending with ~, .bak, .backup)
/// - Temporary files (files starting with . or #)
/// - Hidden files (files starting with .)
/// - Files that cannot be parsed as valid YAML
fn scan_config_directory(config_dir: &path::PathBuf) -> Vec<path::PathBuf> {
    let mut config_files = Vec::new();

    // Read directory entries
    let entries = match std::fs::read_dir(config_dir) {
        Ok(entries) => entries,
        Err(e) => {
            warn!("Failed to read config directory: {:?}", e);
            return Vec::new();
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();

        // Skip directories
        if path.is_dir() {
            continue;
        }

        // Get file name
        let file_name = match path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => continue,
        };

        // Skip hidden files (starting with .)
        if file_name.starts_with('.') {
            debug!("Skipping hidden file: {}", file_name);
            continue;
        }

        // Skip temporary files (starting with # or ending with ~)
        if file_name.starts_with('#') || file_name.ends_with('~') {
            debug!("Skipping temporary file: {}", file_name);
            continue;
        }

        // Skip backup files
        if file_name.ends_with(".bak") || file_name.ends_with(".backup") {
            debug!("Skipping backup file: {}", file_name);
            continue;
        }

        // Only accept .yml and .yaml files
        if !file_name.ends_with(".yml") && !file_name.ends_with(".yaml") {
            debug!("Skipping non-YAML file: {}", file_name);
            continue;
        }

        // Validate that the file is a valid YAML file
        if !is_valid_config_file(&path) {
            warn!("Skipping invalid config file: {}", file_name);
            continue;
        }

        debug!("Found valid config file: {:?}", path);
        config_files.push(path);
    }

    config_files
}

/// Check if a file is a valid configuration file by attempting to parse it.
///
/// Returns true if the file can be parsed as a valid Config, false otherwise.
fn is_valid_config_file(path: &path::Path) -> bool {
    match std::fs::read_to_string(path) {
        Ok(content) => {
            match serde_yaml::from_str::<Config>(&content) {
                Ok(_) => true,
                Err(e) => {
                    debug!("Failed to parse config file {:?}: {}", path, e);
                    false
                }
            }
        }
        Err(e) => {
            debug!("Failed to read config file {:?}: {}", path, e);
            false
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

    if config_path.exists() {
        return Err(eyre!("Config file already exists: {:?}", config_path));
    }

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
                warn!("Failed to load config file: {:?}, {}", config_path, e)
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
        let vars = crate::template::parse_variables(dsn_template, &self.dsn);
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
        ListFormat {
            with_dsn: args.with_dsn,
            with_annotations: args.with_annotations,
            ..Default::default()
        }
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
        ListOptions {
            filter: Filter::from_args(args),
            format: ListFormat::from_args(args),
        }
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