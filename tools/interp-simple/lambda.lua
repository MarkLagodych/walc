#!/usr/bin/env lua

-- This is a simple lambda calculus interpreter of the WALC format.

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



---@class Token
---@field type "λ"|"("|")"|"id"
---@field value string|nil

---@param file file*
---@return Token|nil
local function read_token(file)
    local c = file:read(1)

    -- Skip whitespaces (including comments and dots)
    while c and c:find("[ \t\v\r\n#.]") do
        if c == "#" then
            while c and c ~= "\n" do c = file:read(1) end
        end

        c = file:read(1)
    end

    if c == nil then return nil end -- EOF
    if c == "λ" or c == "\\" then return { type = "λ" } end
    if c == "(" or c == ")" then return { type = c } end

    local identifier = ""
    while c and c:find("[a-zA-Z0-9_]") do
        identifier = identifier .. c
        c = file:read(1)
    end

    assert(identifier ~= "", "Unexpected character '" .. c .. "'")

    file:seek("cur", -1) -- Put the last character back

    return { type = "id", value = identifier }
end


local function insert_to_lambda(lambda, item)
    if lambda.type == "application" then
        if not lambda.left then
            lambda.left = item
        elseif not lambda.right then
            lambda.right = item
        else
            error("Application already has two arguments")
        end
    elseif lambda.type == "abstraction" then
        if not lambda.body then
            lambda.body = item
        else
            error("Abstraction already has a body")
        end
    end
end

---@param file file*
---@return Lambda|nil
local function parse(file)
    -- Begin with a dummy term to hold the result
    local stack = { { type = "abstraction", variable = "" } }

    local function new_initial_term() return { type = "application" } end
    local function is_initial_term(term) return #term == 1 end

    while true do
        local token = read_token(file)
        if not token then break end

        if token.type == "(" then
            table.insert(stack, new_initial_term())
        elseif token.type == "λ" then
            assert(is_initial_term(stack[#stack]), "Unexpected lambda")

            token = read_token(file)
            assert(token and token.type == "id", "Expected variable")
            stack[#stack] = { type = "abstraction", variable = token.value }
        elseif token.type == ")" then
            assert(#stack > 1, "Unexpected closing parenthesis")

            local subterm = table.remove(stack)
            insert_to_lambda(stack[#stack], subterm)
        else
            local subterm = { type = "variable", name = token.value }
            insert_to_lambda(stack[#stack], subterm)
        end
    end

    assert(#stack == 1, "Expected closing parenthesis")

    -- The body can be nil if the file is empty
    return stack[1].body
end


---@param lambda Lambda|nil
local function dump(lambda)
    if not lambda then
        io.write("nil")
        return
    end

    if lambda.type == "variable" then
        io.write(lambda.name)
    elseif lambda.type == "abstraction" then
        io.write("λ" .. lambda.variable .. " ")
        dump(lambda.body)
    elseif lambda.type == "application" then
        io.write("(")
        dump(lambda.left)
        io.write(" ")
        dump(lambda.right)
        io.write(")")
    end
end


---@class Value
---@field env Definition
---@field lambda Lambda

---@class Definition
---@field name string
---@field value Value


---@param input Lambda
local function eval(input)
    local lambda = input ---@type Lambda
    local env = {} ---@type Definition[]
    local stack = {} ---@type Value[]

    while true do
        print("Processing: ")
        dump(lambda)
        io.write("\n")

        if lambda.type == "application" then
            ---@cast lambda Lambda.application
            table.insert(stack, {
                env = env,
                lambda = lambda.right,
            })
            lambda = lambda.left
        elseif lambda.type == "abstraction" then
            ---@cast lambda Lambda.abstraction
            if #stack == 0 then
                -- No more applications to process, we're done
                io.write("Result: ")
                dump(lambda)
                io.write("\n")
                return
            end

            local arg = table.remove(stack) ---@type Value
            -- Extend the environment
            table.insert(env, {
                variable = lambda.variable,
                value = arg,
            })
            lambda = lambda.body
        elseif lambda.type == "variable" then
            ---@cast lambda Lambda.variable
            -- Look up the variable in the environment
            while true do
                assert(#env > 0, "Unbound variable: " .. lambda.name)

                local top = table.remove(env) ---@type Definition

                if top.name == lambda.name then
                    lambda = top.value.lambda
                    env = top.value.env
                    break
                end
            end
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
    eval(lambda)
end

main()
