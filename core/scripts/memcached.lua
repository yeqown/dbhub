-- $HOME/.dbhub/sample.lua
-- Using LUA 5.4

assert(dbhub ~= nil, "dbhub is not defined")
assert(dbhub.variables ~= nil, "dbhub.variables is not defined")
assert(dbhub.count ~= nil, "dbhub.count is not defined")
assert(dbhub.last_output_lines ~= nil, "dbhub.last_output_lines is not defined")

assert(dbhub.annotations ~= nil, "dbhub.annotations is not defined")

local variables = dbhub.variables
local annotations = dbhub.annotations

local hash_distribution_key = "memcached/hash-distribution"

local hd = ""
if annotations[hash_distribution_key] ~= nil then
    hd = string.format("--hash %s", annotations[hash_distribution_key])
end

-- memcached-cli: https://github.com/yeqown/memcached/tree/main/cmd/memcached-cli
--
local args = string.format("memcached-cli --servers %s %s", variables.servers, hd)

return {
    command_with_args = args,
    again = false
}