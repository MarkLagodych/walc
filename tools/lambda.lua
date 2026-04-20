#!/usr/bin/env luajit

-- This is a simple lambda calculus interpreter based on the WALC format.
-- Runs on LuaJIT 2.1 / Lua 5.1.

-- Version 1.0

-- Copyright (c) 2025-2026 Mark Lagodych
-- SPDX-License-Identifier: MIT

local function Var(name)
    return { type = "variable", name = name }
end

local function Abs(variable, body)
    return { type = "abstraction", variable = variable, body = body }
end

local function Apply(left, right)
    return { type = "application", left = left, right = right }
end


local function parse_expr(next_token)
    local token = next_token()

    if token == "[" then
        local variable = next_token()
        local body = parse_expr(next_token)
        assert(next_token() == "]", "Expected ']'")
        return Abs(variable, body)
    elseif token == "(" then
        local left = parse_expr(next_token)
        local right = parse_expr(next_token)
        assert(next_token() == ")", "Expected ')'")
        return Apply(left, right)
    else
        return Var(token)
    end
end

local function parse(str)
    -- Replace comments with spaces
    str = str:gsub(";[^\n]*\n", " ")

    -- Add spaces around brackets
    str = str:gsub("%(", " ( ")
    str = str:gsub("%)", " ) ")
    str = str:gsub("%[", " [ ")
    str = str:gsub("%]", " ] ")

    -- Split by spaces
    local next_token = str:gmatch("%S+")

    return parse_expr(next_token)
end


-- Represents a delayed computation of a lambda expression.
function Closure(env, expr)
    return { env = env, expr = expr }
end

-- Represents a value binding of an abstraction variable.
-- * parent: the next environment in the chain (can be nil)
-- * name: the variable name
-- * value: a closure assigned to the variable (may be updated when computed)
-- * computed: true if the value has been evaluated yet (can be nil)
function Env(parent, name, value, computed)
    return { parent = parent, name = name, value = value, computed = computed }
end

-- recursion_depth avoids stack overflow with too much recursion.
-- Pass nil when normally calling from other functions.
local function run(closure, recursion_depth)
    --[[ Based on the Krivine machine with a mixed computation strategy:

    (1) Call by need (weak) is used by default, meaning that most expressions
        are evaluated lazily and also, when a variable value is computed for
        the first time, its value is updated in its environment to avoid future
        re-computations.
        This prevents the program from slowing down with time.

    (2) Call by value (strict) is used when a function argument is a variable,
        meaning that in such a case the right side of an application is computed
        before the left side.
        This avoids building long environment chains with lots of unused
        variables, which are unavoidable with weak strategies.
        Effectively, this prevents the program from using more and more memory
        with time.

        This approach involves recursion. To prevent stack overflows,
        the recursion depth is limited, so computation just stops at some point.

        This might not be the most efficient approach as it computes things
        that might never be actually used, but it is surely the simplest one. ]]

    local stack = {}

    -- (1)
    -- Stores envs whose values need to be computed and updated
    local uncomputed_envs = {}
    -- Positions in the stack of when those envs were scheduled
    local stack_locations = {}

    recursion_depth = recursion_depth or 0

    while true do
        if closure.expr.type == "variable" then
            -- Find the environment where the variable is defined
            local env = closure.env
            while env and env.name ~= closure.expr.name do
                env = env.parent
            end

            assert(env, "Unbound variable: " .. closure.expr.name)

            -- (1) Schedule the variable for an update
            if not env.computed then
                table.insert(uncomputed_envs, env)
                table.insert(stack_locations, #stack)
            end

            closure = env.value
        elseif closure.expr.type == "abstraction" then
            -- (1) Update variable values
            while stack_locations[#stack_locations] == #stack do
                table.remove(stack_locations)
                local env = table.remove(uncomputed_envs)
                env.value = closure -- Assign the computed value
                env.computed = true
            end

            if #stack == 0 then
                break
            end

            local argument = table.remove(stack)
            local env = Env(closure.env, closure.expr.variable, argument)
            closure = Closure(env, closure.expr.body)
        elseif closure.expr.type == "application" then
            local right = Closure(closure.env, closure.expr.right)

            -- (2) Preemptively compute the right if it is a variable,
            -- but limit the recursion depth (10 worked well during tests).
            if right.expr.type == "variable" and recursion_depth < 10 then
                right = run(right, recursion_depth + 1)
            end

            table.insert(stack, right)
            closure = Closure(closure.env, closure.expr.left)
        end
    end

    return closure
end

-- The text in the variable name ensures that it cannot be defined by the
-- program. It is wrapped in an abstraction to prevent it from being
-- preemptively computed by the interpreter.
local unreachable = Abs("〜⁠(⁠꒪⁠꒳⁠꒪⁠)⁠〜", Var("unreachable (⊙＿⊙')"))

local bit0 = parse("[x0[x1 x0]]")
local bit1 = parse("[x0[x1 x1]]")

local function decode_bit(closure)
    local expr = Apply(Apply(closure.expr, bit0), bit1)
    local result = run(Closure(closure.env, expr))
    if result.expr == bit0 then
        return 0
    elseif result.expr == bit1 then
        return 1
    else
        error("Expected bit")
    end
end

local function encode_bit(bit)
    if bit == 0 then
        return bit0
    elseif bit == 1 then
        return bit1
    else
        error("Expected bit")
    end
end

local function decode_pair(closure)
    -- Avoid duplicate computations when evaluating the items
    closure = run(closure)

    local item0 = Closure(closure.env, Apply(closure.expr, bit0))
    local item1 = Closure(closure.env, Apply(closure.expr, bit1))
    return item0, item1
end

local function encode_pair(expr0, expr1)
    return Abs("p", Apply(Apply(Var("p"), expr0), expr1))
end

local function decode_optional(closure)
    local has_data, data = decode_pair(closure)
    return decode_bit(has_data) == 1 and data or nil
end

local function encode_optional(expr)
    return expr
        and encode_pair(encode_bit(1), expr)
        or encode_pair(encode_bit(0), unreachable)
end

local function decode_list(closure)
    local node = decode_optional(closure)
    if not node then return {} end

    local head, tail = decode_pair(node)
    local items = decode_list(tail)
    table.insert(items, 1, head)
    return items
end

local function encode_list(items)
    local node
    if #items == 0 then
        node = nil
    else
        local head = table.remove(items, 1)
        node = encode_pair(head, encode_list(items))
    end

    return encode_optional(node)
end

local function decode_byte(closure)
    local bits = decode_list(closure)

    local byte = 0
    for _, bit in ipairs(bits) do
        byte = byte * 2 + decode_bit(bit)
    end

    return byte
end

local function encode_byte(byte)
    local bits = {}

    for i = 0, 7 do
        table.insert(bits, 1, encode_bit(byte % 2))
        byte = math.floor(byte / 2)
    end

    return encode_list(bits)
end

local help_message =
[[Lambda calculus interpreter based on WALC format
Run with 'luajit' (default), 'lua', etc.:
$ [<LUA_INTERPRETER>] ./lambda.lua <FILENAME>]]

local function main()
    if #arg ~= 1 or arg[1] == "--help" then
        print(help_message)
        return
    end

    local source = io.open(arg[1]):read("*a")
    local program = Closure(nil, parse(source))

    while true do
        local command = decode_optional(program)
        if not command then return end

        local is_input, data = decode_pair(command)

        if decode_bit(is_input) == 1 then
            -- Input
            local input_byte = io.read(1)

            local input_expr = nil
            if input_byte then
                input_expr = encode_byte(string.byte(input_byte))
            end

            local input = encode_optional(input_expr)
            program = Closure(data.env, Apply(data.expr, input))
        else
            -- Output
            local output, next_program = decode_pair(data)
            io.write(string.char(decode_byte(output)))
            io.flush()
            program = next_program
        end
    end
end

main()
