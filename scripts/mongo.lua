-- $HOME/.dbhub/mysql.lua
-- Using LUA 5.4

assert(dbhub ~= nil, "dbhub is not defined")
assert(dbhub.variables ~= nil, "dbhub.variables is not defined")
assert(dbhub.count ~= nil, "dbhub.count is not defined")
assert(dbhub.last_output_lines ~= nil, "dbhub.last_output_lines is not defined")

local variables = dbhub.variables

local args = string.format("mongosh %s", variables.dsn)

return {
    command_with_args = args,
    again = false
}
