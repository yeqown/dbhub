use crate::embedded::Configs;
use color_eyre::eyre::{eyre, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path,
};

use tracing::{debug, info, warn};

/// Initialization status for configuration directory
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum InitStatus {
    /// Config directory exists with at least one valid config file
    AlreadyExists,
    /// Config directory does not exist
    NotInitialized,
    /// Config directory exists but contains no valid config files
    NoValidConfig,
}

/// Result of checking initialization status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitResult {
    pub status: InitStatus,
    pub config_dir: std::path::PathBuf,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// All database configurations.
    pub databases: Vec<Database>,
    /// Templates for different database types.
    /// Key: db_type, value: connection URL template.
    /// e.g., mysql: mysql://{user}:{password}@{host}:{port}/{database}
    pub templates: Option<HashMap<String, Template>>,

    // Runtime indices - only in memory, not serialized
    /// Key: alias, value: Database index in databases.
    #[serde(skip)]
    pub aliases: HashMap<String, usize>,
    /// Key: environment, value: list of database indices.
    #[serde(skip)]
    pub environments: HashMap<String, Vec<usize>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Template {
    /// Connection URL template.
    pub dsn: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Database {
    /// Database type (mysql, mongo, redis, etc.)
    pub db_type: String,
    /// Connection string matching the template.
    pub dsn: String,
    /// Environment name (local, staging, prod, etc.)
    pub env: String,
    /// Unique alias for this connection.
    pub alias: String,
    /// Human-readable description.
    pub description: Option<String>,
    /// Additional metadata for the connection.
    /// e.g., { "region": "us-west-1", "version": "8.0.32" }
    pub annotations: Option<HashMap<String, String>>,
}

impl Config {
    /// Get mutable reference to templates.
    fn get_mut_templates(&mut self) -> &mut HashMap<String, Template> {
        self.templates.as_mut()
            .expect("templates must be initialized; call loads() first")
    }

    /// Get templates for database types.
    pub fn get_templates(&self) -> &HashMap<String, Template> {
        self.templates.as_ref()
            .expect("templates must be initialized; call loads() first")
    }

    /// Get a database by its index.
    pub fn get_database_by_index(&self, index: usize) -> Option<&Database> {
        self.databases.get(index)
    }

    /// Get a database by its alias.
    pub fn get_database_by_alias(&self, alias: &str) -> Option<&Database> {
        self.aliases.get(alias).and_then(|&idx| self.databases.get(idx))
    }

    /// Get all unique environment names.
    pub fn get_environments(&self) -> Vec<&str> {
        self.environments.keys().map(|s| s.as_str()).collect()
    }

    /// Get all aliases.
    pub fn get_aliases(&self) -> Vec<&str> {
        self.aliases.keys().map(|s| s.as_str()).collect()
    }
}

/// DBHUB_CONFIG environment variable name.
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
            .filter_map(expand_config_path)
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
        return expand_config_path(DEFAULT_CONFIG_PATH).into_iter().collect();
    }

    // Scan for all .yml and .yaml files
    let mut config_files = scan_config_directory(&config_dir);

    // Sort files alphabetically for consistent ordering
    config_files.sort();

    if config_files.is_empty() {
        info!("No config files found in {:?}, using default: {:?}", config_dir, DEFAULT_CONFIG_PATH);
        expand_config_path(DEFAULT_CONFIG_PATH).into_iter().collect()
    } else {
        info!("Found {} config file(s) in {:?}: {:?}", config_files.len(), config_dir, config_files);
        config_files
    }
}

/// Scan the configuration directory for valid config files.
fn scan_config_directory(config_dir: &path::PathBuf) -> Vec<path::PathBuf> {
    let mut config_files = Vec::new();

    let entries = match std::fs::read_dir(config_dir) {
        Ok(entries) => entries,
        Err(e) => {
            warn!("Failed to read config directory: {:?}", e);
            return Vec::new();
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            continue;
        }

        let file_name = match path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => continue,
        };

        // Skip hidden, temporary, and backup files
        if file_name.starts_with('.') || file_name.starts_with('#') || file_name.ends_with('~') {
            debug!("Skipping hidden/temp file: {}", file_name);
            continue;
        }

        if file_name.ends_with(".bak") || file_name.ends_with(".backup") {
            debug!("Skipping backup file: {}", file_name);
            continue;
        }

        // Only accept .yml and .yaml files
        if !file_name.ends_with(".yml") && !file_name.ends_with(".yaml") {
            continue;
        }

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
fn is_valid_config_file(path: &path::Path) -> bool {
    match std::fs::read_to_string(path) {
        Ok(content) => serde_yaml::from_str::<Config>(&content).is_ok(),
        Err(_) => false,
    }
}

/// Expand ~ in config path to home directory.
fn expand_config_path(path: &str) -> Option<path::PathBuf> {
    if let Some(ref home) = dirs::home_dir() {
        #[cfg(target_os = "macos")]
        if path.starts_with("~") {
            let relative_path = path.strip_prefix("~/").unwrap_or("");
            return Some(home.join(relative_path));
        }

        #[cfg(target_os = "windows")]
        if path.starts_with("%USERPROFILE%") {
            let relative_path = path.strip_prefix("%USERPROFILE%/").unwrap_or("");
            return Some(home.join(relative_path));
        }
    }

    Some(path::PathBuf::from(path))
}

/// Generate default configuration file.
pub fn generate_default_config() -> Result<()> {
    let config_path = expand_config_path(DEFAULT_CONFIG_PATH)
        .ok_or_else(|| eyre!("Failed to determine config path"))?;

    if config_path.exists() {
        return Err(eyre!("Config file already exists: {:?}", config_path));
    }

    let sample_config = Configs::get(SAMPLE_CONFIG_FILE_PATH)
        .ok_or_else(|| eyre!("Sample config not found in embedded resources"))?;

    let parent = config_path.parent()
        .ok_or_else(|| eyre!("Invalid config path: no parent directory"))?;

    std::fs::create_dir_all(parent)?;
    std::fs::write(&config_path, sample_config.data)?;

    info!("Created default config file at: {:?}", config_path);
    Ok(())
}

/// Check configuration initialization status.
///
/// This is a pure inspection function - no files are created or modified.
pub fn check_init_status() -> InitResult {
    let config_dir = if let Some(ref home) = dirs::home_dir() {
        home.join(".dbhub")
    } else {
        return InitResult {
            status: InitStatus::NotInitialized,
            config_dir: std::path::PathBuf::from("~/.dbhub"),
            message: Some("Cannot determine home directory".to_string()),
        };
    };

    if !config_dir.exists() {
        return InitResult {
            status: InitStatus::NotInitialized,
            config_dir: config_dir.clone(),
            message: Some(format!("Config directory does not exist: {config_dir:?}")),
        };
    }

    let config_files = scan_config_directory(&config_dir);
    if config_files.is_empty() {
        return InitResult {
            status: InitStatus::NoValidConfig,
            config_dir: config_dir.clone(),
            message: Some(format!("Config directory exists but contains no valid config files: {config_dir:?}")),
        };
    }

    InitResult {
        status: InitStatus::AlreadyExists,
        config_dir,
        message: None,
    }
}

/// Load and merge all configuration files.
pub fn loads() -> Result<Config> {
    let config_paths = get_config_paths();
    if config_paths.is_empty() {
        return Err(eyre!(
            "No configuration files found. \
             Run 'dbhub context --generate' to create a default configuration, \
             or set DBHUB_CONFIG environment variable to specify config paths."
        ));
    }

    let mut config = Config {
        databases: Vec::new(),
        templates: Some(HashMap::new()),
        aliases: HashMap::new(),
        environments: HashMap::new(),
    };

    for config_path in &config_paths {
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

    // Build runtime indices
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
    let config_path = config_path.as_ref();

    let content = std::fs::read_to_string(config_path)
        .map_err(|e| eyre!("Failed to read config file '{}': {}", config_path.display(), e))?;

    serde_yaml::from_str(&content)
        .map_err(|e| eyre!("Failed to parse config file '{}': {}", config_path.display(), e))
}

impl Database {
    /// Parse variables from the connection string using a template.
    ///
    /// Returns (variables, annotations) tuple.
    pub fn variables(&self, dsn_template: &str) -> Result<(HashMap<String, String>, HashMap<String, String>)> {
        let mut variables = HashMap::new();

        let parsed = crate::template::parse_variables(dsn_template, &self.dsn)
            .ok_or_else(|| eyre!("Could not parse variables from DSN: {}", self.dsn))?;

        variables.extend(parsed);
        variables.insert("dsn".to_string(), self.dsn.clone());

        let annotations = self.annotations.clone().unwrap_or_default();

        Ok((variables, annotations))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_init_status_returns_result() {
        let result = check_init_status();
        assert!(result.config_dir.ends_with(".dbhub"));
    }
}
