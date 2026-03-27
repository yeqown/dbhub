-- $HOME/.dbhub/mysql.lua
-- Using LUA 5.4

assert(dbhub ~= nil, "dbhub is not defined")
assert(dbhub.variables ~= nil, "dbhub.variables is not defined")
assert(dbhub.count ~= nil, "dbhub.count is not defined")
assert(dbhub.last_output_lines ~= nil, "dbhub.last_output_lines is not defined")

local variables = dbhub.variables

local optional_password = ""
if variables.password and variables.password ~= "" then
    optional_password = string.format("-a %s", variables.password)
end

local database = ""
if variables.database and variables.database ~= "" then
    database = string.format("-n %s", variables.database)
end

local args = string.format("redis-cli -h %s -p %d %s %s", variables.host, variables.port, optional_password, database)

return {
    command_with_args = args,
    again = false
}
