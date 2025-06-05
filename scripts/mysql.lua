-- $HOME/.dbhub/mysql.lua
-- Using LUA 5.4

local optional_password = ""
if variables.password and variables.password ~= "" then
    optional_password = string.format("-p%s", variables.password)
end

local optional_database = ""
if variables.database and variables.database ~= "" then
    optional_database = string.format("--database=%s", variables.database)
end

return string.format("mysqlsh -h %s -P %d -u %s %s --database=%s",
variables.host, variables.port, variables.user, optional_password, optional_database)
