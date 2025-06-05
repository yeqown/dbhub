-- $HOME/.dbhub/mysql.lua
-- Using LUA 5.4

return string.format("mongosh %s", variables.dsn)
