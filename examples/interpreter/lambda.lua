#!/usr/bin/env lua

-- This is a simple lambda calculus interpreter based on the WALC format.
-- Runs on Lua 5.1 (released in 2006) and LuaJIT 2.1.

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

    if not c or c == "(" or c == ")" or c == "\\" or c == "." then
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
        assert(id and not id:find("[\\%(%)%.]"), "Not a variable name: " .. id)

        token = read_token(file)
        assert(token == ".", "Expected dot")

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


---@param lambda Lambda
---@param max_depth number|nil
local function dump(lambda, max_depth)
    max_depth = max_depth or 1000000

    assert(lambda, "Got nil lambda expression")

    if lambda.type == "variable" then
        return lambda.name
    else
        if max_depth <= 0 then return "[...]" end

        if lambda.type == "abstraction" then
            return "\\" .. lambda.variable .. "."
                .. dump(lambda.body, max_depth - 1)
        elseif lambda.type == "application" then
            return "(" .. dump(lambda.left, max_depth - 1) .. " "
                .. dump(lambda.right, max_depth - 1) .. ")"
        end
    end
end


--- Represents a delayed computation of a lambda expression.
---@class Value
---@field env Environment
---@field expression Lambda

--- Represents a variable binding in an abstraction.
---@class Environment
---@field parent Environment|nil
---@field name string
---@field value Value
---@field computed boolean|nil Optimization: if true, do not recompute the value


---@param value Value
---@return Value
local function eval(value)
    -- Based on Krivine machine, but with an optimization to avoid
    -- recomputing values for variables whose values have already been computed.
    -- Without the optimization programs can become very slow.

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


-- '$' is added to highlight that the terms are inserted by the interpreter
local unreachable = abstract("$unreachable", var("$unreachable"))
local bit0 = abstract("$x0", abstract("$x1", var("$x0")))
local bit1 = abstract("$x0", abstract("$x1", var("$x1")))
local pair = function(left, right)
    return abstract("$g", apply(apply(var("$g"), left), right))
end


---@param value Value
---@return number
local function into_bit(value)
    local expr = apply(apply(value.expression, bit0), bit1)
    local res = eval({ env = value.env, expression = expr })
    if res.expression == bit0 then return 0 end
    if res.expression == bit1 then return 1 end
    error("Unexpected value when evaluating bit: " .. dump(res.expression))
end

---@param bit number
---@return Lambda
local function from_bit(bit)
    if bit == 0 then
        return bit0
    else
        return bit1
    end
end

---@param value Value
---@return Value, Value
local function into_pair(value)
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
local function from_pair(first, second)
    return pair(first, second)
end

---@param value Value
---@return Value|nil
local function into_optional(value)
    local first
    local second
    first, second = into_pair(value)

    local has_value = into_bit(first)
    if has_value == 0 then return nil end
    return second
end

---@param opt_lambda Lambda|nil
---@return Lambda
local function from_optional(opt_lambda)
    if opt_lambda then
        return from_pair(from_bit(1), opt_lambda)
    else
        return from_pair(from_bit(0), unreachable)
    end
end

---@param value Value
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

---@param list table
---@return Lambda
local function from_list(list)
    local result = from_optional(nil)
    for i = #list, 1, -1 do
        result = from_optional(from_pair(list[i], result))
    end

    return result
end

---@param value Value
---@return number
local function into_byte(value)
    local bits = into_list(value)
    assert(#bits == 8, "Expected 8 bits for a byte, got " .. #bits)

    local byte = 0
    for i = 8, 1, -1 do
        byte = (byte * 2) + into_bit(bits[i])
    end

    return byte
end

---@param byte number
---@return Lambda
local function from_byte(byte)
    local bits = {}
    for i = 1, 8 do
        table.insert(bits, 1, from_bit(byte % 2))
        byte = math.floor(byte / 2)
    end

    return from_list(bits)
end

---@param value Value
---@return string
local function into_string(value)
    local chars = into_list(value)
    local str = ""
    for _, char_value in ipairs(chars) do
        local byte = into_byte(char_value)
        str = str .. string.char(byte)
    end

    return str
end

---@param str string
---@return Lambda
local function from_string(str)
    local chars = {}
    for i = 1, #str do
        local byte = string.byte(str, i)
        table.insert(chars, from_byte(byte))
    end

    return from_list(chars)
end

---@param value Value
---@return string, Value
local function into_output(value)
    local out_string, next_value
    out_string, next_value = into_pair(value)
    return into_string(out_string), next_value
end

---@param input string
---@param value Value
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
        io.flush()
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
        print("Usage: lambda.lua <filename>")
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
