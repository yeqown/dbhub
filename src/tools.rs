use crate::config::{Config, Database, Template};
use crate::embedded::Scripts;
use color_eyre::eyre::{eyre, Result};
use dirs;
use mlua::Lua;
use shell_words;
use std::collections::HashMap;
use std::process::Command;
use tracing::{debug, info, warn};
use which::which;


/// Connect to a database using environment and database name
///
/// # Arguments
///
/// * `db` - The database connection information
///
/// # Returns
///
/// * `Result<()>` - Returns `Ok(())` if the connection is successful, otherwise returns an error
pub fn connect(db: &Database, cfg: &Config) -> Result<()> {
    let template = cfg.templates.get(db.db_type.as_str());
    if template.is_none() {
        return Err(eyre!("No template found for database type: {}", db.db_type));
    }
    let tpl = template.unwrap();

    let args = build_cli_command(db, tpl)?;
    info!("Running command: \n\n\t💻 -> {}\n", args.join(" "));

    let mut commanded_cli = "dbhub";

    // Extract the command from the first argument
    if let Some(template_cli) = args.first() {
        commanded_cli = template_cli;
    }

    // Whether the command exists
    which(commanded_cli).map_err(|_| {
        eyre!("Command '{}' not found. please install it or check your PATH.", commanded_cli)
    })?;

    // DONE(@yeqown): open another shell to execute the interactive command.
    //  and then exit the current shell.
    Command::new(commanded_cli).args(&args[1..])
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()?
        .wait()?;

    info!("The connection has been closed.");

    Ok(())
}

fn build_cli_command(db: &Database, template: &Template) -> Result<Vec<String>> {
    let variables = db.variables(template.dsn.as_str())?;

    // Use the lua script to generate the command at first.
    let lua_command = try_execute_lua(&db.db_type, &variables);
    match lua_command {
        Ok(lua_command) => {
            Ok(shell_words::split(&lua_command)?)
        }
        Err(e) => {
            // Use the template to generate the command.
            Err(e)
        }
    }
}

fn try_execute_lua(db_type: &str, variables: &HashMap<String, String>) -> Result<String> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| eyre!("Failed to get home directory"))?;

    let lua_script = format!("{}.lua", db_type);
    let lua_script_path = home_dir.join(".dbhub").join(lua_script.clone());

    if !lua_script_path.exists() {
        debug!("lua script not found, use the default script. now copy it to the lua_script_path.");
        // Use the default script in embedded::Scripts
        // Copy the script to the lua_script_path if it doesn't exist.
        // Only fail if the script doesn't exist in embedded::Scripts either.
        let file = Scripts::get(format!("{}", lua_script).as_str());
        if file.is_none() {
            return Err(eyre!("No lua script found for database type: {}", db_type));
        }

        std::fs::create_dir_all(lua_script_path.parent().unwrap())?;
        std::fs::write(&lua_script_path, file.unwrap().data.as_ref())?;

        info!("No lua script found, apply the default script to: {:?}", lua_script_path);
    }

    let lua = Lua::new();
    let globals = lua.globals();

    match lua.create_table() {
        Ok(lua_variables) => {
            // Set the variables in the table
            for (key, value) in variables {
                if let Err(e) = lua_variables.set(key.clone(), value.clone()) {
                    warn!("Runtime error: could not set to lua table '{}': {}", key, e);
                }
            }

            if let Err(e) = globals.set("variables", lua_variables) {
                warn!("Runtime error: could not set to lua table 'variables': {}", e);
            }
        }
        Err(e) => {
            return Err(eyre!("Runtime error: could not create lua table: {}", e));
        }
    }

    match lua.load(lua_script_path).eval() {
        Ok(result) => {
            Ok(result)
        }
        Err(err) => {
            Err(eyre!("Runtime error: could not execute lua script: {}", err))
        }
    }
}