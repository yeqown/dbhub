//! Interactive connection execution.

use super::command::run_lua_iterative;
use crate::config::{Config, Database};
use color_eyre::eyre::Result;
use tracing::info;

/// Connect to a database interactively.
///
/// This function runs the connection command in the current terminal,
/// inheriting stdin/stdout/stderr for interactive sessions.
///
/// # Arguments
///
/// * `db` - The database connection information
/// * `cfg` - The configuration containing templates
/// * `passthrough_args` - Additional arguments to pass to the command
///
/// # Returns
///
/// Returns `Ok(())` on successful connection completion.
pub fn connect(db: &Database, cfg: &Config, passthrough_args: &[String]) -> Result<()> {
    let (command, args) = run_lua_iterative(db, cfg, passthrough_args)?;

    // Execute interactively
    std::process::Command::new(&command)
        .args(&args)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()?
        .wait()?;

    info!("Connection closed.");
    Ok(())
}
