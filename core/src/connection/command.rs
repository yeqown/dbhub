//! Command generation from Lua scripts.

use crate::config::{Config, Database};
use crate::embedded::Scripts;
use color_eyre::eyre::{eyre, Result};
use std::{
    collections::HashMap,
    path,
};
use tracing::{debug, info};

use super::lua::{execute_lua, LuaContext, LuaOutput};

/// Maximum number of Lua script execution iterations.
/// This prevents infinite loops caused by Lua scripts returning `again = true`.
const MAX_LUA_ITERATIONS: usize = 5;

/// A parsed connection command ready for execution.
#[derive(Debug, Clone)]
pub struct ConnectCommand {
    /// The command to execute (e.g., "mysqlsh")
    pub command: String,
    /// Arguments to pass to the command
    pub args: Vec<String>,
}

/// Build a connection command for the given database.
///
/// This function runs the Lua script to generate the final command.
/// For simple cases, it returns a single command.
/// For multi-step connections (like Redis Sentinel), it handles the iteration.
///
/// # Arguments
///
/// * `db` - The database connection information
/// * `cfg` - The configuration containing templates
/// * `passthrough_args` - Additional arguments to pass to the command
///
/// # Returns
///
/// Returns the command to execute on success.
pub fn build_connect_command(
    db: &Database,
    cfg: &Config,
    passthrough_args: &[String],
) -> Result<ConnectCommand> {
    let template = cfg.get_templates().get(db.db_type.as_str())
        .ok_or_else(|| eyre!("No template found for database type: {}", db.db_type))?;

    let (variables, annotations) = db.variables(&template.dsn)?;
    let lua_script_path = locate_lua_script(db.db_type.as_str())?;

    // Execute Lua script to generate command
    let output = execute_lua_once(
        &lua_script_path,
        &variables,
        &annotations,
        &[],
        0,
        passthrough_args,
    )?;

    let args = shell_words::split(&output.command_with_args)?;
    let command = args.first()
        .ok_or_else(|| eyre!("No command provided by Lua script"))?
        .clone();

    Ok(ConnectCommand {
        command,
        args: args[1..].to_vec(),
    })
}

/// Execute Lua script once and return the output.
fn execute_lua_once(
    lua_script_path: &path::Path,
    variables: &HashMap<String, String>,
    annotations: &HashMap<String, String>,
    last_output_lines: &[String],
    count: usize,
    runtime_args: &[String],
) -> Result<LuaOutput> {
    let context = LuaContext {
        count,
        variables: variables.clone(),
        annotations: annotations.clone(),
        last_output_lines: last_output_lines.to_vec(),
        runtime_args: runtime_args.to_vec(),
    };

    execute_lua(lua_script_path, &context)
}

/// Run the Lua script iteratively until it returns `again = false` or reaches max iterations.
///
/// Returns the final command arguments.
pub(super) fn run_lua_iterative(
    db: &Database,
    cfg: &Config,
    passthrough_args: &[String],
) -> Result<(String, Vec<String>)> {
    let template = cfg.get_templates().get(db.db_type.as_str())
        .ok_or_else(|| eyre!("No template found for database type: {}", db.db_type))?;

    let (variables, annotations) = db.variables(&template.dsn)?;
    let lua_script_path = locate_lua_script(db.db_type.as_str())?;

    let mut last_output_lines: Vec<String> = vec![];
    let mut counter = 0;

    while counter < MAX_LUA_ITERATIONS {
        let output = execute_lua_once(
            &lua_script_path,
            &variables,
            &annotations,
            &last_output_lines,
            counter,
            passthrough_args,
        )?;

        info!("#{} Running command: \n\n\t💻 -> {}\n", counter, output.command_with_args);

        let args = shell_words::split(&output.command_with_args)?;
        let command = args.first()
            .ok_or_else(|| eyre!("No command provided"))?
            .clone();

        // Verify command exists
        which::which(&command).map_err(|_| {
            eyre!("Command `{}` not found, please install it or check PATH.", command)
        })?;

        counter += 1;
        last_output_lines.clear();

        if output.again {
            // Execute command and capture output for next iteration
            let exec_output = std::process::Command::new(&command)
                .args(&args[1..])
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()?
                .wait_with_output()?;

            if !exec_output.status.success() {
                return Err(eyre!(
                    "Command `{}` failed: {}",
                    command,
                    String::from_utf8(exec_output.stderr)?
                ));
            }

            debug!("Command `{}` output: \n{}", command,
                String::from_utf8(exec_output.stdout.clone())?);

            for line in String::from_utf8(exec_output.stdout)?.trim().split('\n') {
                last_output_lines.push(line.to_string());
            }
            continue;
        }

        return Ok((command, args[1..].to_vec()));
    }

    Err(eyre!("Script execution exceeded {} iterations", MAX_LUA_ITERATIONS))
}

/// Locate the Lua script for a database type.
///
/// If the script doesn't exist in ~/.dbhub/, it will be copied from embedded resources.
fn locate_lua_script(db_type: &str) -> Result<path::PathBuf> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| eyre!("Failed to get home directory"))?;

    let lua_script = format!("{db_type}.lua");
    let lua_script_path = home_dir.join(".dbhub").join(&lua_script);

    if !lua_script_path.exists() {
        debug!("Lua script not found at {:?}, copying from embedded resources", lua_script_path);

        let embedded_file = Scripts::get(&lua_script)
            .ok_or_else(|| eyre!("No embedded Lua script found for database type: {}", db_type))?;

        let parent_dir = lua_script_path.parent()
            .ok_or_else(|| eyre!("Invalid Lua script path: no parent directory"))?;

        std::fs::create_dir_all(parent_dir)?;
        std::fs::write(&lua_script_path, embedded_file.data.as_ref())?;

        info!("Created Lua script at: {:?}", lua_script_path);
    }

    Ok(lua_script_path)
}
