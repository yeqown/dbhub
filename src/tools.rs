use crate::config::{Config, Database};
use crate::embedded::Scripts;
use color_eyre::eyre::{eyre, Result};
use dirs;
use mlua::{FromLua, Lua, Value};
use shell_words;
use std::collections::HashMap;
use std::path::PathBuf;
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
    let variables = db.variables(tpl.dsn.as_str())?;
    let mut last_output_lines: Vec<String> = vec![];
    let lua_script_path = locate_lua_script(db.db_type.as_str())?;

    let mut counter = 0;
    while counter < 5 {
        let output = run_lua_with(&lua_script_path, &variables, &last_output_lines, counter)?;
        info!("#{} Running command: \n\n\t💻 -> {}\n", counter, output.command_with_args);
        let args = shell_words::split(&output.command_with_args)?;
        let command = args.first().ok_or_else(|| eyre!("No command provided"))?;
        which(command).map_err(|_| {
            eyre!("Command `{}` not found, please install it or check yout PATH.", command)
        })?;

        last_output_lines.clear();

        if output.again {
            let output = Command::new(command)
                .args(&args[1..])
                .spawn()?
                .wait_with_output()?;
            // split stdout by line
            let output_lines = String::from_utf8(output.stdout)?;
            for line in output_lines.trim().split("\n").collect::<Vec<_>>() {
                last_output_lines.push(String::from(line));
            }
        } else {
            // DONE(@yeqown): open another shell to execute the interactive command.
            //  and then exit the current shell.
            Command::new(command)
                .args(&args[1..])
                .stdin(std::process::Stdio::inherit())
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .spawn()?
                .wait()?;

            info!("The connection has been closed.");

            return Ok(());
        }

        counter += 1;
    }


    Err(eyre!("Script execution over 5 times"))
}

fn locate_lua_script(db_type: &str) -> Result<PathBuf> {
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

    Ok(lua_script_path)
}

struct LuaOutput {
    command_with_args: String, // The command line needs to be executed.
    again: bool, // Whether to run the script again.
}

impl<'lua> FromLua for LuaOutput {
    fn from_lua(lua_value: Value, _: &Lua) -> mlua::Result<Self> {
        match lua_value {
            Value::Table(table) => {
                let command_with_args: String = table.get("command_with_args")?;
                let again: bool = table.get("again")?;
                Ok(LuaOutput {
                    command_with_args,
                    again,
                })
            }
            _ => Err(mlua::Error::FromLuaConversionError {
                from: lua_value.type_name(),
                to: String::from("LuaOutput"),
                message: Some("Expected a table".to_string()),
            }),
        }
    }
}


fn run_lua_with(
    lua_script_path: &PathBuf,
    variables: &HashMap<String, String>,
    last_output_lines: &Vec<String>,
    count: usize,
) -> Result<LuaOutput> {

    let lua_state = LuaState {
        count,
        variables: variables.clone(),
        last_output_lines: last_output_lines.clone(),
    };

    // Use the lua script to generate the command at first.
    try_execute_lua(lua_script_path, &lua_state)
}

struct LuaState {
    count: usize,
    variables: HashMap<String, String>,
    last_output_lines: Vec<String>,
}

fn try_execute_lua(lua_script_path: &PathBuf, state: &LuaState) -> Result<LuaOutput> {
    let lua = Lua::new();
    let globals = lua.globals();

    match lua.create_table() {
        Ok(lua_state) => {
            // Set the count in the table
            if let Err(e) = lua_state.set("count", state.count) {
                warn!("Runtime error: could not set 'count' to lua table: {}", e);
            }

            // Set the variables in the table
            match lua.create_table() {
                Ok(lua_variables) => {
                    for (key, value) in &state.variables {
                        if let Err(e) = lua_variables.set(key.clone(), value.clone()) {
                            warn!("Runtime error: could not set '{}' to lua variables table: {}", key, e);
                        }
                    }
                    if let Err(e) = lua_state.set("variables", lua_variables) {
                        warn!("Runtime error: could not set 'variables' to lua state table: {}", e);
                    }
                }
                Err(e) => {
                    warn!("Runtime error: could not create lua variables table: {}", e);
                }
            }

            // Set the last_output_lines in the table
            match lua.create_table() {
                Ok(lua_last_output_lines) => {
                    for (i, line) in state.last_output_lines.iter().enumerate() {
                        if let Err(e) = lua_last_output_lines.set(i + 1, line.clone()) {
                            warn!("Runtime error: could not set line {} to lua last_output_lines table: {}", i, e);
                        }
                    }
                    if let Err(e) = lua_state.set("last_output_lines", lua_last_output_lines) {
                        warn!("Runtime error: could not set 'last_output_lines' to lua state table: {}", e);
                    }
                }
                Err(e) => {
                    warn!("Runtime error: could not create lua last_output_lines table: {}", e);
                }
            }

            if let Err(e) = globals.set("dbhub", lua_state) {
                warn!("Runtime error: could not set 'dbhub' to lua globals: {}", e);
            }
        }
        Err(e) => {
            return Err(eyre!("Runtime error: could not create lua table: {}", e));
        }
    }

    match lua.load(lua_script_path.clone()).eval() {
        Ok(result) => {
            Ok(result)
        }
        Err(err) => {
            Err(eyre!("Runtime error: could not execute lua script: {}", err))
        }
    }
}