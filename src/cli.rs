use crate::config::Config;
use crate::tools;
use clap::{Args, Parser, Subcommand, ValueHint};
use color_eyre::eyre::Result;

/// 替换为实际的包描述
#[derive(Parser)]
#[command(author, version, about)]
#[command(name = "dbhub")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    /// Connect to a database using environment and database name
    #[command(alias = "c")]
    Connect {
        /// Connection alias
        #[arg(value_hint = ValueHint::Other, required = true)]
        alias: String,
    },
    /// Manage database connection contexts
    #[command(alias = "e")]
    Context(ContextArgs),
}


#[derive(Args)]
pub struct ContextArgs {
    /// Generate default config file
    #[arg(long, action)]
    pub generate: bool,
    /// Environment name
    #[arg(long, num_args = 1)]
    pub filter_env: Option<String>,
    /// Database type (mysql, mongodb, redis, redis-sentinel)
    #[arg(long, num_args = 1)]
    pub filter_db_type: Option<String>,
    /// Alias name
    #[arg(long, num_args = 1)]
    pub filter_alias: Option<String>,
    /// Output format control: with_dsn
    #[arg(long)]
    pub with_dsn: bool,
    /// Output format control: with_annotations
    #[arg(long)]
    pub with_annotations: bool,
}

pub fn handle_connect(cfg: &Config, alias: &String) -> Result<()> {
    let db = cfg.aliases.get(alias).ok_or_else(|| {
        color_eyre::eyre::eyre!("Alias '{}' not found", alias)
    })?;

    tools::connect(db, cfg)
}
