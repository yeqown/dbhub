use clap::{Parser, Subcommand, ArgAction};
use color_eyre::eyre::Result;
use std::path::{Path,PathBuf};
use tracing::info;
use dirs;

mod config;
mod tools;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Config file path
    #[arg(short, long)]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Connect to a database using environment and database name
    Connect {
        /// Connection alias
        #[arg(short, long)]
        alias: Option<String>,
    },
    /// Manage database connection contexts
    Context {
        /// List all available connections
        #[arg(long, action = ArgAction::SetTrue)]
        list: bool,

        /// Add a new connection context
        #[arg(long, action = ArgAction::SetTrue)]
        add: bool,

        /// Environment name
        #[arg(short, long)]
        env: Option<String>,

        /// Database type (mysql, mongodb, documentdb, doris, redis)
        #[arg(short = 't', long)]
        db_type: Option<String>,

        /// Database connection string
        #[arg(short, long)]
        url: Option<String>,

        /// Connection alias (must be unique)
        #[arg(short, long)]
        alias: Option<String>,

        /// Description of the connection
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Install database client tools
    Install {
        /// Tool to install (mysql, mongosh, redis-cli)
        #[arg(short, long)]
        tool: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let mut cli = Cli::parse();

    // Make sure the config directory exists
    let config_path = cli.config.take().unwrap_or(PathBuf::from("~/.dbhub/config.yml"));
    let abs_config_path = if let Some(home) = dirs::home_dir() {
        if config_path.starts_with("~") {
            let relative_path = config_path.strip_prefix("~").unwrap_or(Path::new(""));
            home.join(relative_path)
        } else {
            config_path
        }
    } else {
        config_path
    };

    cli.config = Some(abs_config_path.clone());

    // load config from file
    let mut cfg: config::Config = config::load_or_create(&abs_config_path)?;
    info!("Loaded config from: {:?}", &abs_config_path);

    // handle subcommands
    let command = cli.command.unwrap_or(Commands::Connect {
        alias: None,
    });

    match command {
        Commands::Connect { alias } => {
            if let Some(alias) = alias {
                let db = cfg.aliases.get(&alias).ok_or_else(|| {
                    color_eyre::eyre::eyre!("Alias '{}' not found", alias)
                })?;
                tools::connect(db)?
            } else {
                return Err(color_eyre::eyre::eyre!("Either alias or both env and db must be specified"));
            }
        }
        Commands::Context { list, add, env, db_type, url, alias, description } => {
            if list {
                config::list_connections(&cfg);
                return Ok(());
            } else if add {
                if let (Some(alias), Some(env), Some(db_type), Some(url), Some(description)) = (alias, env, db_type, url, description) {
                    config::add_connection(&abs_config_path, &mut cfg, &env, &db_type, &url, &alias, Some(description))?;
                } else {
                    return Err(color_eyre::eyre::eyre!("When using the context sub - command for adding, env, name, db_type and url must be specified"));
                }
                return Ok(());
            }

            config::list_connections(&cfg);
        }
        Commands::Install { tool } => {
            tools::install_tool(&tool)?;
        }
    }

    Ok(())
}