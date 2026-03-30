use clap::{Args, CommandFactory, Parser, Subcommand, ValueHint};
use clap_complete::{generate, Shell};
use color_eyre::eyre::Result;
use std::io;

/// DB Hub - A CLI tool for managing multi-environment database connections
#[derive(Parser)]
#[command(author, version, about)]
#[command(name = "dbhub")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Connect to a database using environment and database name
    #[command(alias = "c")]
    Connect {
        /// Connection alias
        #[arg(value_hint = ValueHint::Other, required = true)]
        alias: String,

        /// Trail
        #[arg(trailing_var_arg = true)]
        #[arg(allow_hyphen_values = true)]
        passthrough_args: Vec<String>,
    },
    /// Manage database connection contexts
    #[command(alias = "e")]
    Context(ContextArgs),
    /// Generate shell completion scripts
    #[command(alias = "comp")]
    Completion {
        /// Shell type
        #[arg(value_enum)]
        shell: Shell,
    },
    /// Internal command for completion suggestions (hidden)
    #[command(hide = true)]
    CompletionSuggestions {
        /// Type of suggestions to generate
        #[arg()]
        suggestion_type: String,
    },
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

pub fn handle_completion(shell: Shell) -> Result<()> {
    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "dbhub", &mut io::stdout());
    Ok(())
}
