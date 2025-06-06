-- $HOME/.dbhub/mysql.lua
-- Using LUA 5.4

local variables = dbhub.variables

-- step1: get master from sentinel
local function get_master()
    -- hosts recv from variables: host1, host2, host3
    local hosts = {}
    local ports = {}
    for i = 1, 3 do
        local host = variables["host" .. i]
        local port = variables["port".. i]
        if host and host ~= "" then
            table.insert(hosts, host)
        end

        if port and port ~= "" then
            table.insert(ports, port)
        end
    end

    local host = ""
    local port = ""

    if #hosts > 0 then
    -- randomly select a host
        local index = math.random(1, #hosts)
        host = hosts[index]
        port = ports[index]
    end

    local master_name = "unknown"

    if variables.meta_master_name and variables.meta_master_name ~= "" then
        master_name = variables.meta_master_name
    end

    return string.format("redis-cli -h %s -p %s sentinel get-master-addr-by-name %s", host, port, master_name)
end

-- step2: generate redis-cli command line
local function command()
    -- TODO(@yeqown): use the last output to generate.
    print("debugging last_output_lines: ")
    for i, line in ipairs(dbhub.last_output_lines) do
         print(string.format("  Line %d: %s", i, line))
    end

    return "redis-cli -h 127.0.0.1 -p 6379 -n 0"
end

local again = false

if dbhub.count < 1 then
    again = true
    args = get_master()
end

if dbhub.count >= 1 then
    again = false
    args = command()
end

return {
    command_with_args = args,
    again = false
}
