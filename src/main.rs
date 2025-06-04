use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::info;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Config file path
    #[arg(short, long, default_value = "~/.db-hub/config.yml")]
    config: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Connect to a database using environment and database name
    Connect {
        /// Environment name
        #[arg(short, long)]
        env: Option<String>,

        /// Database name
        #[arg(short, long)]
        db: Option<String>,

        /// Connection alias
        #[arg(short, long)]
        alias: Option<String>,
    },
    /// List all available connections
    List,
    /// Add a new connection
    Add {
        /// Environment name
        #[arg(short, long)]
        env: String,

        /// Database name
        #[arg(short, long)]
        name: String,

        /// Database type (mysql, mongodb, documentdb, doris, redis)
        #[arg(short = 't', long)]
        db_type: String,

        /// Database connection string
        #[arg(short, long)]
        url: String,

        /// Connection alias (must be unique)
        #[arg(short, long)]
        alias: Option<String>,
    },
    /// Install database client tools
    Install {
        /// Tool to install (mycli, mongosh, redis-cli)
        #[arg(short, long)]
        tool: String,
    },
    /// Add or update connection string template
    Template {
        /// Database type
        #[arg(short = 't', long)]
        db_type: String,

        /// Connection string template
        #[arg(short, long)]
        template: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    environments: std::collections::HashMap<String, Environment>,
    templates: std::collections::HashMap<String, String>,
    aliases: std::collections::HashMap<String, ConnectionRef>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Environment {
    databases: std::collections::HashMap<String, Database>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Database {
    db_type: String,
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ConnectionRef {
    env: String,
    db: String,
}

impl Config {
    fn new() -> Self {
        let mut templates = std::collections::HashMap::new();
        templates.insert("mysql".to_string(), "mysql://{user}:{password}@{host}:{port}/{database}".to_string());
        templates.insert("mongodb".to_string(), "mongodb://{user}:{password}@{host}:{port}/{database}".to_string());
        templates.insert("redis".to_string(), "redis://{user}:{password}@{host}:{port}/{database}".to_string());
        
        Self {
            environments: std::collections::HashMap::new(),
            templates,
            aliases: std::collections::HashMap::new(),
        }
    }

    fn validate_connection_string(&self, db_type: &str, url: &str) -> Result<()> {
        let template = self.templates.get(db_type).ok_or_else(|| {
            color_eyre::eyre::eyre!("No template found for database type: {}", db_type)
        })?;

        // 简单验证：检查必要的组件是否存在
        let required_components = vec!["host", "port"];
        for component in required_components {
            if !template.contains(component) {
                return Err(color_eyre::eyre::eyre!("Template missing required component: {}", component));
            }
        }

        // 验证URL格式
        if !url.starts_with(&format!("{}", db_type)) {
            return Err(color_eyre::eyre::eyre!("Invalid URL format for {}", db_type));
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    
    // 确保配置目录存在
    if let Some(parent) = cli.config.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // 读取或创建配置文件
    let mut config: Config = if cli.config.exists() {
        let content = std::fs::read_to_string(&cli.config)?;
        serde_yaml::from_str(&content)?
    } else {
        Config::new()
    };

    match cli.command {
        Commands::Connect { env, db, alias } => {
            if let Some(alias) = alias {
                let conn_ref = config.aliases.get(&alias).ok_or_else(|| {
                    color_eyre::eyre::eyre!("Alias '{}' not found", alias)
                })?;
                connect(&config, &conn_ref.env, &conn_ref.db)?
            } else if let (Some(env), Some(db)) = (env, db) {
                connect(&config, &env, &db)?
            } else {
                return Err(color_eyre::eyre::eyre!("Either alias or both env and db must be specified"));
            }
        }
        Commands::List => {
            list_connections(&config);
        }
        Commands::Add { env, name, db_type, url, alias } => {
            config.validate_connection_string(&db_type, &url)?;
            add_connection(&cli.config, &mut config, &env, &name, &db_type, &url, alias.as_deref())?;
        }
        Commands::Install { tool } => {
            install_tool(&tool)?;
        }
        Commands::Template { db_type, template } => {
            config.templates.insert(db_type.clone(), template);
            let content = serde_yaml::to_string(&config)?;
            std::fs::write(&cli.config, content)?;
            info!("Updated template for {}", db_type);
        }
    }

    Ok(())
}

fn connect(config: &Config, env: &str, db: &str) -> Result<()> {
    let environment = config.environments.get(env).ok_or_else(|| {
        color_eyre::eyre::eyre!("Environment '{}' not found", env)
    })?;

    let database = environment.databases.get(db).ok_or_else(|| {
        color_eyre::eyre::eyre!("Database '{}' not found in environment '{}'", db, env)
    })?;

    let command = match database.db_type.as_str() {
        "mysql" | "doris" => "mycli",
        "mongodb" | "documentdb" => "mongosh",
        "redis" => "redis-cli",
        _ => return Err(color_eyre::eyre::eyre!("Unsupported database type: {}", database.db_type)),
    };

    // 检查命令是否存在
    which::which(command).map_err(|_| {
        color_eyre::eyre::eyre!("Command '{}' not found. Use 'db-hub install -t {}' to install it.", command, command)
    })?;

    let args = shell_words::split(&database.url)?;
    let status = std::process::Command::new(command)
        .args(args)
        .status()?;

    if !status.success() {
        return Err(color_eyre::eyre::eyre!("Failed to connect to database"));
    }

    Ok(())
}

fn list_connections(config: &Config) {
    println!("Environments:");
    for (env_name, env) in &config.environments {
        println!("  {}:", env_name);
        for (db_name, db) in &env.databases {
            println!("    {} ({}):", db_name, db.db_type);
            println!("      URL: {}", db.url);
            // 显示别名
            for (alias, conn_ref) in &config.aliases {
                if &conn_ref.env == env_name && &conn_ref.db == db_name {
                    println!("      Alias: {}", alias);
                }
            }
        }
    }

    println!("\nTemplates:");
    for (db_type, template) in &config.templates {
        println!("  {}: {}", db_type, template);
    }
}

fn add_connection(
    config_path: &PathBuf,
    config: &mut Config,
    env: &str,
    name: &str,
    db_type: &str,
    url: &str,
    alias: Option<&str>,
) -> Result<()> {
    let environment = config.environments.entry(env.to_string())
        .or_insert_with(|| Environment {
            databases: std::collections::HashMap::new(),
        });

    environment.databases.insert(name.to_string(), Database {
        db_type: db_type.to_string(),
        url: url.to_string(),
    });

    if let Some(alias) = alias {
        // 检查别名是否已存在
        if config.aliases.contains_key(alias) {
            return Err(color_eyre::eyre::eyre!("Alias '{}' already exists", alias));
        }
        config.aliases.insert(alias.to_string(), ConnectionRef {
            env: env.to_string(),
            db: name.to_string(),
        });
    }

    let content = serde_yaml::to_string(&config)?;
    std::fs::write(config_path, content)?;

    info!("Added database '{}' to environment '{}'.", name, env);
    Ok(())
}

fn install_tool(tool: &str) -> Result<()> {
    match tool {
        "mycli" => install_mycli()?,
        "mongosh" => install_mongosh()?,
        "redis-cli" => install_redis_cli()?,
        _ => return Err(color_eyre::eyre::eyre!("Unsupported tool: {}", tool)),
    }
    Ok(())
}

fn install_mycli() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("pip3")
            .args(["install", "--user", "mycli"])
            .status()?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("pip3")
            .args(["install", "--user", "mycli"])
            .status()?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("pip3")
            .args(["install", "--user", "mycli"])
            .status()?;
    }
    Ok(())
}

fn install_mongosh() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("brew")
            .args(["install", "mongosh"])
            .status()?;
    }
    #[cfg(target_os = "linux")]
    {
        // 为 Linux 添加 MongoDB 仓库并安装 mongosh
        std::process::Command::new("curl")
            .args(["-fsSL", "https://www.mongodb.org/static/pgp/server-6.0.asc", "-o", "/tmp/mongodb.asc"])
            .status()?;
        std::process::Command::new("sudo")
            .args(["apt-key", "add", "/tmp/mongodb.asc"])
            .status()?;
        std::process::Command::new("echo")
            .arg("deb [ arch=amd64,arm64 ] https://repo.mongodb.org/apt/ubuntu focal/mongodb-org/6.0 multiverse")
            .stdout(std::process::Stdio::piped())
            .spawn()?
            .stdout
            .ok_or_else(|| color_eyre::eyre::eyre!("Failed to get stdout"))?;
        std::process::Command::new("sudo")
            .args(["apt-get", "update"])
            .status()?;
        std::process::Command::new("sudo")
            .args(["apt-get", "install", "-y", "mongodb-mongosh"])
            .status()?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("choco")
            .args(["install", "mongodb-shell", "-y"])
            .status()?;
    }
    Ok(())
}

fn install_redis_cli() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("brew")
            .args(["install", "redis"])
            .status()?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("sudo")
            .args(["apt-get", "install", "-y", "redis-tools"])
            .status()?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("choco")
            .args(["install", "redis-cli", "-y"])
            .status()?;
    }
    Ok(())
}
