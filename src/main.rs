use color_eyre::eyre::Result;
use tracing::warn;

mod config;
mod tools;
mod template;
mod embedded;
mod cli;

fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();


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