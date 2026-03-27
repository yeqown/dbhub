-- $HOME/.dbhub/mysql.lua
-- Using LUA 5.4

assert(dbhub ~= nil, "dbhub is not defined")
assert(dbhub.variables ~= nil, "dbhub.variables is not defined")
assert(dbhub.count ~= nil, "dbhub.count is not defined")
assert(dbhub.last_output_lines ~= nil, "dbhub.last_output_lines is not defined")

local variables = dbhub.variables

local password = "--no-password"
if variables.password and variables.password ~= "" then
    password = string.format("-p%s", variables.password)
end

local optional_database = ""
if variables.database and variables.database ~= "" then
    optional_database = string.format("--database=%s", variables.database)
end

local args = string.format("mysqlsh -h %s -P %d -u %s %s %s",
             variables.host, variables.port, variables.user, password, optional_database)

return {
    command_with_args = args,
    again = false
}
