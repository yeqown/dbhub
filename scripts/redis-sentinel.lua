-- $HOME/.dbhub/mysql.lua
-- Using LUA 5.4

assert(dbhub ~= nil, "dbhub is not defined")
assert(dbhub.variables ~= nil, "dbhub.variables is not defined")
assert(dbhub.count ~= nil, "dbhub.count is not defined")
assert(dbhub.last_output_lines ~= nil, "dbhub.last_output_lines is not defined")

assert(dbhub.annotations ~= nil, "dbhub.annotations is not defined")

local variables = dbhub.variables
local annotations = dbhub.annotations

local master_name_key = "redis-sentinel/mastername"

-- step1: get master from sentinel, support 3 sentinels at most.
local function get_master()
    local hosts = {}
    for i = 1, 3 do
        local host = variables["host" .. i]
        local port = variables["port" .. i]
        if host and host ~= "" and port and port ~= "" then
            table.insert(hosts, { host = host, port = port })
        end
    end

    local selected = #hosts > 0 and hosts[math.random(#hosts)] or { host = "", port = "" }
    -- Get the master name from the `annotations.master_name` variable.
    -- If it's not set, use the default value "mymaster".
    local master_name = "mymaster"
    if annotations[master_name_key] and annotations[master_name_key] ~= "" then
        master_name = annotations[master_name_key]
    end
    
    return string.format("redis-cli -h %s -p %s sentinel get-master-addr-by-name %s", selected.host, selected.port, master_name)
end

-- step2: generate redis-cli command line
local function command()
    --print("debugging last_output_lines: ")
    --for i, line in ipairs(dbhub.last_output_lines) do
    --     print(string.format("  Line %d: %s", i, line))
    --end

    -- We expect the last output to be the master address like this:
    -- [1] = "127.0.0.1"
    -- [2] = "6379"

    assert(#dbhub.last_output_lines == 2, "Expected 2 lines of output from sentinel get-master-addr-by-name")
    local host = dbhub.last_output_lines[1]
    local port = dbhub.last_output_lines[2]

    local database = ""
    if variables.database and variables.database ~= "" then
        database = string.format("-n %s", variables.database)
    end

    return string.format("redis-cli -h %s -p %s %s", host, port, database)
end

-- again MUST be number so that it can be used in the return statement.
local again = dbhub.count < 1
local args = again and get_master() or command()

return {
    command_with_args = args,
    again = again
}
