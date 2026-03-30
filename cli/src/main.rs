use clap::Parser;
use color_eyre::eyre::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

mod cli;
mod display;
mod r#match;

use cli::{Cli, Commands};
use display::ListOptions;
use r#match::find_similar_alias;

fn main() -> Result<()> {
    color_eyre::install()?;

    Registry::default()
        .with(
            EnvFilter::builder()
                .with_default_directive(tracing::Level::WARN.into())
                .from_env_lossy(),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_level(true)
                .without_time(),
        )
        .init();

    // Check and initialize configuration
    let init_result = dbhub_core::check_init_status();
    match init_result.status {
        dbhub_core::InitStatus::AlreadyExists => {
            tracing::info!("Configuration exists: {:?}", init_result.config_dir);
        }
        dbhub_core::InitStatus::NotInitialized | dbhub_core::InitStatus::NoValidConfig => {
            tracing::info!("Configuration not initialized, creating default configuration...");
            match dbhub_core::generate_default_config() {
                Ok(()) => {
                    println!(
                        "✓ Default configuration created: {:?}",
                        init_result.config_dir.join("config.yml")
                    );
                    println!("Hint: Run 'dbhub context' to list database connections");
                }
                Err(e) => {
                    tracing::error!("Failed to create default configuration: {}", e);
                    tracing::error!("Please check file system permissions or create config manually");
                    std::process::exit(1);
                }
            }
        }
    };

    let cli = Cli::parse();

    match cli.command {
        Commands::Connect {
            ref alias,
            passthrough_args: ref script_args,
        } => {
            let cfg = dbhub_core::loads()?;
            handle_connect(&cfg, alias, script_args)?;
        }
        Commands::Context(args) => {
            if args.generate {
                dbhub_core::generate_default_config()?;
                return Ok(());
            }

            let cfg = dbhub_core::loads()?;
            let opts = ListOptions::from_args(&args);
            display::list_connections(&cfg, &opts);
        }
        Commands::Completion { shell } => {
            cli::handle_completion(shell)?;
        }
        Commands::CompletionSuggestions { ref suggestion_type } => {
            let cfg = dbhub_core::loads()?;
            handle_completion_suggestions(&cfg, suggestion_type)?;
        }
    }

    Ok(())
}

fn handle_connect(
    cfg: &dbhub_core::Config,
    alias: &str,
    passthrough_args: &[String],
) -> Result<()> {
    use color_eyre::eyre::eyre;

    let db_index = cfg.aliases.get(alias).ok_or_else(|| {
        let similar_alias = find_similar_alias(alias, &cfg.aliases.keys().cloned().collect::<Vec<_>>());
        eyre!("Alias '{}' not found, maybe {}?", alias, similar_alias)
    })?;

    let db = cfg.get_database_by_index(*db_index).unwrap();

    tracing::debug!("passthrough_args: {:?}", passthrough_args);

    dbhub_core::connect(db, cfg, passthrough_args)
}

fn handle_completion_suggestions(cfg: &dbhub_core::Config, suggestion_type: &str) -> Result<()> {
    match suggestion_type {
        "aliases" => {
            let mut aliases: Vec<_> = cfg.aliases.keys().cloned().collect();
            aliases.sort();
            for alias in aliases {
                println!("{alias}");
            }
        }
        _ => {
            // Unknown suggestion type, return empty
        }
    }
    Ok(())
}
