-- $HOME/.dbhub/sample.lua
-- Using LUA 5.4
-- The `dbhub` would extract all variables from the database dsn string those
-- match the template pattern:
--
--   mysql://<user>:<password>@<host>:<port>/<database>
--
-- and then pass those variables to the script. The variables includes the dsn itself
-- in the `variables` table, and all metadata key-value pairs but those are has the `meta_` prefix.
--
-- Example:
--   dsn: mysql://root:password@localhost:3306/mydb"
--   metadata:
--     key1: value1
--     key2: value2
--
-- Then the script would receive the following variables:
-- dbhub {
--   count => number
--   variables => table(hashmap)
--   last_output_lines => table(array)
-- }
--

print("dbhub #?", dbhub)

local variables = dbhub.variables

local args = string.format("sample-cli -h %s -P %d -u %s -p%s --database=%s",
    variables.host, variables.port, variables.user, variables.password, variables.database)

return {
    command_with_args = args,
    again = false
}
