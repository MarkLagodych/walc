#!/usr/bin/env lua

-- This is a simple lambda calculus interpreter based on the WALC format.
-- Runs on LuaJIT 2.1 / Lua 5.1.

-- Copyright (c) 2025 Mark Lagodych
-- SPDX-License-Identifier: MIT

---@alias Lambda Lambda.variable|Lambda.abstraction|Lambda.application

---@class Lambda.variable
---@field type "variable"
---@field name string

---@param name string
---@return Lambda
local function var(name)
    return { type = "variable", name = name }
end

---@class Lambda.abstraction
---@field type "abstraction"
---@field variable string
---@field body Lambda

---@param variable string
---@param body Lambda
---@return Lambda
local function abstract(variable, body)
    return { type = "abstraction", variable = variable, body = body }
end

---@class Lambda.application
---@field type "application"
---@field left Lambda
---@field right Lambda

---@param left Lambda
---@param right Lambda
---@return Lambda
local function apply(left, right)
    return { type = "application", left = left, right = right }
end


---@param file file*
---@return "\\"|"."|"("|")"|string|nil
local function read_token(file)
    local c = file:read(1)

    -- Skip whitespaces (including comments)
    while c and c:find("[ \t\v\f\r\n#]") do
        if c == "#" then
            while c and c ~= "\n" do
                c = file:read(1)
            end
        end

        c = file:read(1)
    end

    if not c or c:find("[\\.()]") then
        return c
    end

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

    if token == "\\" then
        local id = read_token(file)
        assert(id and id:find("^[a-zA-Z0-9_]"), "Expected variable identifier")

        token = read_token(file)
        assert(token == ".", "Expected dot after abstraction variable")

        local body = parse(file)
        assert(body, "Expected abstraction body")

        return abstract(id, body)
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


--- Represents a delayed computation of a lambda expression.
---@class Value
---@field env Environment
---@field expression Lambda

--- Represents a variable binding in an abstraction.
---@class Environment
---@field parent Environment|nil Next item in the environment chain
---@field name string Variable name
---@field value Value Variable value
---@field computed boolean|nil Optimization: if true, value is not recomputed


---@param value Value
---@return Value
local function eval(value)
    -- Based on Krivine machine, but with an optimization to avoid
    -- recomputing values for variables whose values have already been computed.
    -- Without the optimization evaluation becomes very slow very quickly.

    local stack = {} ---@type Value[] Data stack

    -- Optimization: stores environments whose values have not been computed yet
    local uncomputed_envs = {} ---@type Environment[]
    local compute_locations = {} ---@type number[] Stack locations (sizes)

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
            -- Optimization: assign the value to the corresponding variables
            while compute_locations[#compute_locations] == #stack do
                table.remove(compute_locations)
                local env = table.remove(uncomputed_envs)
                env.value = value
                env.computed = true
            end

            if #stack == 0 then
                break
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

            -- Optimization: assign a computed value to the environment later
            if not value.env.computed then
                table.insert(uncomputed_envs, value.env)
                table.insert(compute_locations, #stack)
            end

            value = value.env.value
        end
    end

    return value
end


-- '$' is added to avoid naming conflicts
local unreachable = abstract("$unreachable", var("$unreachable"))
local bit0 = abstract("$x0", abstract("$x1", var("$x0")))
local bit1 = abstract("$x0", abstract("$x1", var("$x1")))

---@param value Value
---@return number
local function decode_bit(value)
    local expr = apply(apply(value.expression, bit0), bit1)
    local res = eval({ env = value.env, expression = expr })
    if res.expression == bit0 then return 0 end
    if res.expression == bit1 then return 1 end
    error("Decoding error: not a bit")
end

---@param bit number
---@return Lambda
local function encode_bit(bit)
    if bit == 0 then return bit0 else return bit1 end
end

---@param value Value
---@return Value first, Value second
local function decode_pair(value)
    value = eval(value) -- Pre-compute as much as possible for both items

    local first_expression = apply(value.expression, bit0)
    local first = eval({ env = value.env, expression = first_expression })

    local second_expression = apply(value.expression, bit1)
    local second = eval({ env = value.env, expression = second_expression })

    return first, second
end

---@param first Lambda
---@param second Lambda
---@return Lambda
local function encode_pair(first, second)
    return abstract("$g", apply(apply(var("$g"), first), second))
end

---@param value Value
---@return Value|nil
local function decode_optional(value)
    local has_data, data = decode_pair(value)
    if decode_bit(has_data) == 1 then return data end
    return nil
end

---@param lambda Lambda|nil
---@return Lambda
local function encode_optional(lambda)
    if lambda then
        return encode_pair(encode_bit(1), lambda)
    else
        return encode_pair(encode_bit(0), unreachable)
    end
end

---@param value Value
---@return Value[]
local function decode_list(value)
    local list = {}
    local current_node = value
    while true do
        local node = decode_optional(current_node)
        if not node then break end

        local item, next_node = decode_pair(node)
        table.insert(list, item)

        current_node = next_node
    end

    return list
end

---@param list Lambda[]
---@return Lambda
local function encode_list(list)
    local result = encode_optional(nil)
    for i = #list, 1, -1 do
        result = encode_optional(encode_pair(list[i], result))
    end

    return result
end

---@param value Value
---@return number
local function decode_number(value)
    local bits = decode_list(value)

    local number = 0
    for i = #bits, 1, -1 do
        number = (number * 2) + decode_bit(bits[i])
    end

    return number
end

---@param number number
---@return Lambda
local function encode_number(number)
    local bits = {}
    for i = 1, 8 do
        table.insert(bits, encode_bit(number % 2))
        number = math.floor(number / 2)
    end

    return encode_list(bits)
end

---@param value Value
---@return string
local function decode_string(value)
    local chars = decode_list(value)
    local str = ""
    for i = 1, #chars do
        str = str .. string.char(decode_number(chars[i]))
    end

    return str
end

---@param program Value
---@return number command, Value payload
local function decode_command(program)
    local command, payload = decode_pair(program)
    return decode_number(command), payload
end

---@param payload Value
---@return string output, Value continuation
local function decode_output(payload)
    local output, continuation = decode_pair(payload)
    return decode_string(output), continuation
end

---@param input string|nil
---@param continuation Value
---@return Value program
local function encode_input(input, continuation)
    local input_lambda
    if input then
        input_lambda = encode_optional(encode_number(input:byte()))
    else
        input_lambda = encode_optional(nil)
    end

    return {
        env = continuation.env,
        expression = apply(continuation.expression, input_lambda)
    }
end


local function main()
    if #arg ~= 1 or arg[1] == "--help" then
        print("Lambda calculus interpreter based on WALC format\n")
        print("Usage: lambda.lua <FILENAME>")
        return
    end

    local file = io.open(arg[1])
    assert(file, "Cannot open file " .. arg[1])

    local lambda = parse(file)
    file:close()

    if not lambda then return end -- Empty/blank file

    local program = { env = nil, expression = lambda }
    while true do
        local command, payload = decode_command(program)

        if command == 0 then
            local output = decode_string(payload)
            io.write(output)
            io.flush()
        end

        if not continuation then break end

        program = encode_input(io.read(1), continuation)
    end
end

main()
