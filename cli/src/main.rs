use clap::Parser;
use color_eyre::eyre::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use dbhub_core::{cli, config};

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

    // Check and initialize configuration
    let init_result = dbhub_core::check_init_status();
    match init_result.status {
        dbhub_core::InitStatus::AlreadyExists => {
            tracing::info!("Configuration exists: {:?}", init_result.config_dir);
        }
        dbhub_core::InitStatus::NotInitialized | dbhub_core::InitStatus::NoValidConfig => {
            tracing::info!("Configuration not initialized, creating default configuration...");
            match dbhub_core::config::generate_default_config() {
                Ok(()) => {
                    println!("✓ Default configuration created: {:?}",
                        init_result.config_dir.join("config.yml"));
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

    let cli = cli::Cli::parse();

    // load config from a file
    match cli.command {
        cli::Commands::Connect { ref alias, passthrough_args: ref script_args } => {
            let cfg = config::loads()?;
            cli::handle_connect(&cfg, alias, script_args)?;
        }
        cli::Commands::Context(args) => {
            if args.generate {
                config::generate_default_config()?;
                return Ok(());
            }

            let cfg = config::loads()?;
            let opts = config::ListOptions::from_args(&args);
            config::list_connections(&cfg, &opts);
        }
        cli::Commands::Completion { shell } => {
            // Already handled above
            cli::handle_completion(shell)?;
            return Ok(());
        }
        cli::Commands::CompletionSuggestions { ref suggestion_type } => {
            let cfg = config::loads()?;
            cli::handle_completion_suggestions(&cfg, suggestion_type)?;
            return Ok(());
        }
    }

    Ok(())
}