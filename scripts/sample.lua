-- $HOME/.dbhub/sample.lua
-- Using LUA 5.4

-- dbhub is a global variable that is injected by the script runner
-- it contains the following fields:
--   - variables: a table of variables that are defined in the script.
--                It is a table of hashmap, keys are equal the template variables.
--   - count: the number of times the script has been run.
--           It is a number.
--   - last_output_lines: the last output lines of this script provided command before the script is run again.
--                       It is a table of array of strings.
--   - annotations: a table of annotations that are defined in the script.
--                  It is a table of hashmap. Each table contains the following fields:

assert(dbhub ~= nil, "dbhub is not defined")
assert(dbhub.variables ~= nil, "dbhub.variables is not defined")
assert(dbhub.count ~= nil, "dbhub.count is not defined")
assert(dbhub.last_output_lines ~= nil, "dbhub.last_output_lines is not defined")
assert(dbhub.anotations ~= nil, "dbhub.annotations is not defined")

print("dbhub #?", dbhub)

local variables = dbhub.variables

local args = string.format("sample-cli -h %s -P %d -u %s -p%s --database=%s",
    variables.host, variables.port, variables.user, variables.password, variables.database)

return {
    command_with_args = args, -- the command to run
    again = false             -- indicates whether to run the script again
}
