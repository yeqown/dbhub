-- $HOME/.dbhub/mysql.lua
-- Using LUA 5.4

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
