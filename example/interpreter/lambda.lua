#!/usr/bin/env lua

-- This is a simple lambda calculus interpreter using the WALC format.

---@alias Lambda Lambda.variable|Lambda.abstraction|Lambda.application

---@class Lambda.variable
---@field type "variable"
---@field name string

---@class Lambda.abstraction
---@field type "abstraction"
---@field variable string
---@field body Lambda

---@class Lambda.application
---@field type "application"
---@field left Lambda
---@field right Lambda


---@param file file*
---@return "λ"|"("|")"|string|nil
local function read_token(file)
    local c = file:read(1)

    -- Skip whitespaces (including comments and dots)
    while c and c:find("[ \t\v\r\n#.]") do
        if c == "#" then
            while c and c ~= "\n" do
                c = file:read(1)
            end
        end

        c = file:read(1)
    end

    if c == nil then return nil end

    if c == "(" or c == ")" then return c end

    if c == "\\" then return "λ" end
    if c == "\xCE" then
        c = file:read(1)
        assert(c == "\xBB", "Unexpected character: \xCE" .. c)
        return "λ"
    end

    local identifier = ""
    while c and c:find("[a-zA-Z0-9_]") do
        identifier = identifier .. c
        c = file:read(1)
    end

    assert(identifier ~= "", "Unexpected character: " .. c)

    -- The last character did not belong to the identifier, put it back
    file:seek("cur", -1)

    return identifier
end


---@param file file*
---@return Lambda|nil
local function parse(file)
    local token = read_token(file)
    if not token then return nil end

    if token == "λ" then
        token = read_token(file)
        assert(token and not token:find("[λ%(%)]"), "Expected variable name")

        local body = parse(file)
        assert(body, "Expected abstraction body")

        return { type = "abstraction", variable = token, body = body }
    end

    if token == "(" then
        local left = parse(file)
        assert(left, "Expected left application term")

        local right = parse(file)
        assert(right, "Expected right application term")

        token = read_token(file)
        assert(token == ")", "Expected closing parenthesis")

        return { type = "application", left = left, right = right }
    end

    return { type = "variable", name = token }
end


---@param lambda Lambda|nil
local function dump(lambda)
    if not lambda then return "nil" end

    if lambda.type == "variable" then
        return lambda.name
    elseif lambda.type == "abstraction" then
        return "λ" .. lambda.variable .. " " .. dump(lambda.body)
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

    local value = { env = nil, expression = lambda }
    value = eval(value)
    print(dump(value.expression))
end

main()
