use clap::Parser;
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

    let cli = cli::Cli::parse();

    // Handle completion command early to avoid config loading
    if let cli::Commands::Completion { shell } = &cli.command {
        cli::handle_completion(*shell)?;
        return Ok(());
    }

    // load config from a file
    let cfg = config::loads();
    if cfg.is_err() {
        warn!("Could not load config file, please check or run `dbhub context --generate` to create.");
    }

    match cli.command {
        cli::Commands::Connect { ref alias } => {
            cli::handle_connect(&cfg?, alias)?;
        }
        cli::Commands::Context(args) => {
            if args.generate {
                config::generate_default_config()?;
                return Ok(());
            }

            let opts = config::ListOptions::from_args(&args);
            config::list_connections(&cfg?, &opts);
        }
        cli::Commands::Completion { .. } => {
            // Already handled above
            unreachable!()
        }
    }

    Ok(())
}