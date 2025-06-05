-- $HOME/.dbhub/mysql.lua
-- Using LUA 5.4

local optional_password = ""
if variables.password and variables.password ~= "" then
    optional_password = string.format("-a %s", variables.password)
end

local database = "0"
if variables.database and variables.database ~= "" then
    database = variables.database
end

return string.format("redis-cli -h %s -p %d %s -n %d", variables.host, variables.port, optional_password, database)
