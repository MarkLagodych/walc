#!/usr/bin/env lua

-- This is a simple lambda calculus interpreter using the WALC format.


---@alias Lambda Lambda.variable|Lambda.abstraction|Lambda.application

---@class Lambda.variable
---@field type "variable"
---@field name string

local function var(name)
    return { type = "variable", name = name }
end

---@class Lambda.abstraction
---@field type "abstraction"
---@field variable string
---@field body Lambda

local function abstract(variable, body)
    return { type = "abstraction", variable = variable, body = body }
end

---@class Lambda.application
---@field type "application"
---@field left Lambda
---@field right Lambda

local function apply(left, right)
    return { type = "application", left = left, right = right }
end


---@param file file*
---@return "^"|"("|")"|string|nil
local function read_token(file)
    local c = file:read(1)

    -- Skip whitespaces (including comments)
    while c and c:find("[ \t\v\r\n#]") do
        if c == "#" then
            while c and c ~= "\n" do
                c = file:read(1)
            end
        end

        c = file:read(1)
    end

    if not c or c == "(" or c == ")" or c == "^" then return c end

    local identifier = ""
    while c and c:find("[a-zA-Z0-9_]") do
        identifier = identifier .. c
        c = file:read(1)
    end
    file:seek("cur", -1)

    assert(identifier ~= "", "Unexpected character: " .. c)
    return identifier
end


---@param file file*
---@return Lambda|nil
local function parse(file)
    local token = read_token(file)
    if not token then return nil end

    if token == "^" then
        token = read_token(file)
        assert(token and not token:find("[%^%(%)]"), "Expected variable name")

        local body = parse(file)
        assert(body, "Expected abstraction body")

        return abstract(token, body)
    end

    if token == "(" then
        local left = parse(file)
        assert(left, "Expected left application term")

        local right = parse(file)
        assert(right, "Expected right application term")

        token = read_token(file)
        assert(token == ")", "Expected closing parenthesis")

        return apply(left, right)
    end

    return var(token)
end


---@param lambda Lambda|nil
local function dump(lambda)
    if not lambda then return "nil" end

    if lambda.type == "variable" then
        return lambda.name
    elseif lambda.type == "abstraction" then
        return "^" .. lambda.variable .. " " .. dump(lambda.body)
    elseif lambda.type == "application" then
        return "(" .. dump(lambda.left) .. " " .. dump(lambda.right) .. ")"
    end
end


---@class Value
---@field env Environment
---@field expression Lambda

---@class Environment
---@field parent Environment|nil
---@field name string
---@field value Value

---@param value Value
---@return Value
local function eval(value)
    local stack = {} ---@type Value[]

    -- Based on Krivine's K-machine.
    while true do
        if value.expression.type == "application" then
            table.insert(stack, {
                env = value.env,
                expression = value.expression.right
            })

            value = {
                env = value.env,
                expression = value.expression.left
            }
        elseif value.expression.type == "abstraction" then
            if #stack == 0 then
                return value
            end

            value = {
                env = {
                    parent = value.env,
                    name = value.expression.variable,
                    value = table.remove(stack)
                },
                expression = value.expression.body
            }
        elseif value.expression.type == "variable" then
            while value.env and value.env.name ~= value.expression.name do
                value = {
                    env = value.env.parent,
                    expression = value.expression
                }
            end

            assert(value.env, "Unbound variable: " .. value.expression.name)

            value = value.env.value
        end
    end
end



local bit0 = abstract("x", abstract("y", var("x")))
local bit1 = abstract("x", abstract("y", var("y")))


---@return number
local function into_bit(value)
    local expr = apply(apply(value.expression, bit0), bit1)
    local res = eval({ env = value.env, expression = expr })
    if res.expression == bit0 then return 0 end
    if res.expression == bit1 then return 1 end
    error("Expected bit value")
end

---@return Value
local function from_bit(bit)
    if bit == 0 then
        return { env = nil, expression = bit0 }
    elseif bit == 1 then
        return { env = nil, expression = bit1 }
    end

    error("Invalid bit value: " .. tostring(bit))
end

---@param v Value
---@return Value, Value
local function into_pair(v)
    local first = eval({ env = v.env, expression = apply(v.expression, bit0) })
    local second = eval({ env = v.env, expression = apply(v.expression, bit1) })
    return first, second
end

---@return Value
local function from_pair(first, second)
    local expr = abstract("g", apply(apply(var("g"), first), second))
    return { env = nil, expression = expr }
end

---@return Value|nil
local function into_optional(value)
    local first
    local second
    first, second = into_pair(value)

    local has_value = into_bit(first)
    if has_value == 0 then return nil end
    return second
end

---@return Value
local function from_optional(opt_value)
    if opt_value then
        return from_pair(from_bit(1), opt_value)
    else
        return from_pair(from_bit(0), from_bit(0))
    end
end

---@return Value[]
local function into_list(value)
    local list = {}
    local current = value
    while true do
        local node = into_optional(current)
        if not node then break end

        local item
        item, current = into_pair(node)
        table.insert(list, item)
    end

    return list
end

---@return Value
local function from_list(list)
    local result = from_optional(nil)
    for i = #list, 1, -1 do
        result = from_optional(from_pair(list[i], result))
    end

    return result
end

---@return number
local function into_byte(value)
    local bits = into_list(value)
    assert(#bits == 8, "Expected 8 bits for a byte")

    local byte = 0
    for i = 8, 1, -1 do
        byte = (byte << 1) | into_bit(bits[i])
    end

    return byte
end

---@return Value
local function from_byte(byte)
    local bits = {}
    for i = 1, 8 do
        table.insert(bits, 1, from_bit(byte & 1))
        byte = byte >> 1
    end

    return from_list(bits)
end

local function into_string(value)
    local chars = into_list(value)
    local str = ""
    for _, char_value in ipairs(chars) do
        local byte = into_byte(char_value)
        str = str .. string.char(byte)
    end

    return str
end

---@return Value
local function from_string(str)
    local chars = {}
    for i = 1, #str do
        local byte = string.byte(str, i)
        table.insert(chars, from_byte(byte))
    end

    return from_list(chars)
end

---@return string, Value
local function into_output(value)
    local out_string, next_value
    out_string, next_value = into_pair(value)
    return into_string(out_string), next_value
end

---@param input string
---@return Value
local function from_input(input, value)
    return {
        env = value.env,
        expression = apply(value.expression, from_string(input))
    }
end


---@param output string
---@return string
local function execute_command(output)
    local command = string.byte(output, 1)
    local data = string.sub(output, 2)

    if command == 0 then
        io.write(data)
        return ""
    end

    if command == 1 then
        return io.read(1) or ""
    end

    if command == 2 then
        return io.read("*all") or ""
    end

    error("Unknown command " .. command)
end


local function main()
    if #arg == 0 or arg[1] == "--help" then
        print("Usage: run.lua <filename>")
        return
    end

    local file_name = arg[1]
    assert(file_name, "Expected a file name")
    local file = io.open(file_name, "r")
    assert(file, "Cannot open file " .. file_name)

    local lambda = parse(file)
    file:close()

    if not lambda then return end -- Empty file

    local program = { env = nil, expression = lambda }

    while true do
        local output
        output, program = into_output(program)

        if output == "" then break end

        local input = execute_command(output)

        program = from_input(input, program)
    end
end

main()
