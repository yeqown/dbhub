-- $HOME/.dbhub/mysql.lua
-- Using LUA 5.4

assert(dbhub ~= nil, "dbhub is not defined")
assert(dbhub.variables ~= nil, "dbhub.variables is not defined")
assert(dbhub.count ~= nil, "dbhub.count is not defined")
assert(dbhub.last_output_lines ~= nil, "dbhub.last_output_lines is not defined")
assert(dbhub.runtime_args ~= nil, "dbhub.runtime_args is not defined")

local variables = dbhub.variables

local db = variables.database
if db == nil or db == "" then
    db = "admin"
end

-- if runtime_args specified db, use it.
-- .e.g. ["--db=test"]
if #dbhub.runtime_args > 0 then
    for _, arg in ipairs(dbhub.runtime_args) do
        if arg:find("--db=") then
            db = arg:sub(6)
            break
        end
    end
end

-- mongodb://{user}:{password}@{host}:{port}/{database}?{query}
local args = string.format("mongosh mongodb://%s:%s@%s:%s/%s?%s", variables.user, variables.password, variables.host, variables.port, db, variables.query)

return {
    command_with_args = args,
    again = false
}
