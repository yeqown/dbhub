use color_eyre::eyre::Result;
use tracing::warn;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

mod config;
mod tools;
mod template;
mod embedded;
mod cli;

fn main() -> Result<()> {
    color_eyre::install()?;

    Registry::default()
        .with(
            EnvFilter::builder()
                .with_default_directive(tracing::Level::WARN.into()) // Set the default log level to INFO
                .from_env_lossy() // Load log level from environment variables
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_level(true)
                .without_time()
        )
        .init();

    // tracing_subscriber::fmt::init();

    let matches = cli::build_cli().get_matches();

    // load config from a file
    let cfg = config::loads();
    if cfg.is_err() {
        warn!("Could not load config file, please check or run `dbhub context --apply` to create.");
    }

    // handle subcommands
    let command = matches.subcommand();

    match command {
        Some(("connect", sub_matches)) => {
            cli::handle_connect(&cfg?, sub_matches)?;
        }
        Some(("context", sub_matches)) => {
            let list = sub_matches.get_flag("list");
            let apply = sub_matches.get_flag("apply");
            let env = sub_matches.get_one::<String>("env").cloned();
            let db_type = sub_matches.get_one::<String>("db_type").cloned();

            if apply {
                config::apply_default_config()?;
                return Ok(());
            }

            _ = list;
            config::list_connections(&cfg?, env, db_type);
        }
        _ => {
            // TODO: support default command
            let default_command = cli::build_default_command();
            let default_matches = default_command.get_matches();
            cli::handle_connect(&cfg?, &default_matches)?;
        }
    }

    Ok(())
}