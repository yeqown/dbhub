//! Lua script execution.

use color_eyre::eyre::{eyre, Result};
use std::{
    collections::HashMap,
    path,
};
use tracing::warn;

/// Context for Lua script execution.
pub struct LuaContext {
    pub count: usize,
    pub variables: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
    pub last_output_lines: Vec<String>,
    pub runtime_args: Vec<String>,
}

/// Output from Lua script execution.
pub struct LuaOutput {
    /// The command line to execute.
    pub command_with_args: String,
    /// Whether to run again with captured output.
    pub again: bool,
}

impl mlua::FromLua for LuaOutput {
    fn from_lua(lua_value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
        match lua_value {
            mlua::Value::Table(table) => {
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

/// Execute a Lua script with the given context.
pub fn execute_lua(lua_script_path: &path::Path, state: &LuaContext) -> Result<LuaOutput> {
    let lua = mlua::Lua::new();
    let globals = lua.globals();

    let lua_state = lua.create_table()
        .map_err(|e| eyre!("Could not create Lua table: {}", e))?;

    // Set count
    set_lua_table_value(
        &lua_state,
        mlua::Value::String(lua.create_string("count").unwrap()),
        mlua::Value::Integer(state.count as i64),
    );

    // Set variables
    if let Ok(lua_variables) = create_and_fill_lua_table(
        &lua,
        state.variables.iter().map(|(k, v)| {
            (
                mlua::Value::String(lua.create_string(k).unwrap()),
                mlua::Value::String(lua.create_string(v).unwrap()),
            )
        }),
    ) {
        set_lua_table_value(
            &lua_state,
            mlua::Value::String(lua.create_string("variables").unwrap()),
            mlua::Value::Table(lua_variables),
        );
    }

    // Set annotations
    if let Ok(lua_annotations) = create_and_fill_lua_table(
        &lua,
        state.annotations.iter().map(|(k, v)| {
            (
                mlua::Value::String(lua.create_string(k).unwrap()),
                mlua::Value::String(lua.create_string(v).unwrap()),
            )
        }),
    ) {
        set_lua_table_value(
            &lua_state,
            mlua::Value::String(lua.create_string("annotations").unwrap()),
            mlua::Value::Table(lua_annotations),
        );
    }

    // Set last_output_lines
    if let Ok(lua_last_output_lines) = create_and_fill_lua_table(
        &lua,
        state.last_output_lines.iter().enumerate().map(|(i, line)| {
            (
                mlua::Value::Integer((i + 1) as i64),
                mlua::Value::String(lua.create_string(line).unwrap()),
            )
        }),
    ) {
        set_lua_table_value(
            &lua_state,
            mlua::Value::String(lua.create_string("last_output_lines").unwrap()),
            mlua::Value::Table(lua_last_output_lines),
        );
    }

    // Set runtime_args
    if let Ok(lua_runtime_args) = create_and_fill_lua_table(
        &lua,
        state.runtime_args.iter().enumerate().map(|(i, arg)| {
            (
                mlua::Value::Integer((i + 1) as i64),
                mlua::Value::String(lua.create_string(arg).unwrap()),
            )
        }),
    ) {
        set_lua_table_value(
            &lua_state,
            mlua::Value::String(lua.create_string("runtime_args").unwrap()),
            mlua::Value::Table(lua_runtime_args),
        );
    }

    // Set dbhub global
    set_lua_table_value(
        &globals,
        mlua::Value::String(lua.create_string("dbhub").unwrap()),
        mlua::Value::Table(lua_state),
    );

    lua.load(lua_script_path).eval()
        .map_err(|e| eyre!("Lua script execution failed: {}", e))
}

fn set_lua_table_value(lua_table: &mlua::Table, key: mlua::Value, value: mlua::Value) {
    if let Err(e) = lua_table.set(key, value) {
        warn!("Failed to set Lua table value: {}", e);
    }
}

fn create_and_fill_lua_table(
    lua: &mlua::Lua,
    entries: impl IntoIterator<Item = (mlua::Value, mlua::Value)>,
) -> mlua::Result<mlua::Table> {
    let lua_table = lua.create_table()?;
    for (key, value) in entries {
        set_lua_table_value(&lua_table, key, value);
    }
    Ok(lua_table)
}
