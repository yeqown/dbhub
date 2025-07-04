use crate::config::Config;
use crate::tools;
use clap::{Arg, ArgAction, ArgMatches, Command, ValueHint};
use color_eyre::eyre::Result;

pub fn build_cli() -> Command {
    let connect_subcommand = Command::new("connect")
        .about("Connect to a database using environment and database name")
        .alias("c")
        .arg(
            Arg::new("alias")
                .help("Connection alias")
                .value_hint(ValueHint::Other)
                .num_args(1)
                .required(true),
        );

    let context_subcommand = Command::new("context")
        .about("Manage database connection contexts")
        .alias("e")
        .arg(
            Arg::new("generate")
                .long("generate")
                .help("Generate default config file")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("filter-env")
                .long("env")
                .help("Environment name")
                .num_args(1)
                .required(false),
        )
        .arg(
            Arg::new("filter-db-type")
                .long("db-type")
                .help("Database type (mysql, mongodb, redis, redis-sentinel)")
                .num_args(1)
                .required(false),
        );

    Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand(connect_subcommand)
        .subcommand(context_subcommand)
}

pub fn build_default_command() -> Command {
    Command::new("dbhub")
        .arg(
            Arg::new("alias")
                .num_args(1)
        )
}

pub fn handle_connect(cfg: &Config, matched: &ArgMatches) -> Result<()> {
    if let Some(alias) = matched.get_one::<String>("alias") {
        let db = cfg.aliases.get(alias).ok_or_else(|| {
            color_eyre::eyre::eyre!("Alias '{}' not found", alias)
        })?;

        return tools::connect(db, cfg);
    }

    Err(color_eyre::eyre::eyre!("Either alias or both env and db must be specified"))
}
