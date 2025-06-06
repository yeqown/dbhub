-- $HOME/.dbhub/mysql.lua
-- Using LUA 5.4

local variables = dbhub.variables

local args = string.format("mongosh %s", variables.dsn)

return {
    command_with_args = args,
    again = false
}
