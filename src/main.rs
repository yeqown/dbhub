use clap::{ArgAction, Parser, Subcommand, ValueHint};
use color_eyre::eyre::Result;

mod config;
mod tools;
mod template;
mod embedded;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

fn get_all_aliases() -> Vec<&'static str> {
    vec!["aliases"]
}

#[derive(Subcommand)]
enum Commands {
    /// Connect to a database using environment and database name
    Connect {
        /// Connection alias
        #[arg(
            value_hint = ValueHint::Other,
        )]
        alias: Option<String>,
    },
    /// Manage database connection contexts
    Context {
        /// List all available connections
        #[arg(long, action = ArgAction::SetTrue)]
        list: bool,

        /// Apply default config file
        #[arg(long, action = ArgAction::SetTrue)]
        apply: bool,

        /// Environment name
        #[arg(short, long)]
        env: Option<String>,

        /// Database type (mysql, mongodb, redis, redis-sentinel)
        #[arg(short = 't', long)]
        db_type: Option<String>,
    },
}

fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    // load config from a file
    let cfg: config::Config = config::loads()?;

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
                tools::connect(db, &cfg)?
            } else {
                return Err(color_eyre::eyre::eyre!("Either alias or both env and db must be specified"));
            }
        }
        Commands::Context { list, apply, env, db_type } => {
            if apply {
                config::apply_default_config()?;
                return Ok(());
            }

            _ = list;
            config::list_connections(&cfg, env, db_type);
        }
    }

    Ok(())
}