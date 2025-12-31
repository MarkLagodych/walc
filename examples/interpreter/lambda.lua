#!/usr/bin/env lua

-- This is a simple lambda calculus interpreter based on the WALC format.
-- Runs on LuaJIT 2.1 / Lua 5.1.

-- Copyright (c) 2025 Mark Lagodych
-- SPDX-License-Identifier: MIT

local function Var(name)
    return { type = "variable", name = name }
end

local function Fun(variable, body)
    return { type = "function", variable = variable, body = body }
end

local function Call(left, right)
    return { type = "call", left = left, right = right }
end


local function parse_expr(next_token)
    local token = next_token()

    if token == "?" then
        return Fun(next_token(), parse_expr(next_token))
    elseif token == "!" then
        return Call(parse_expr(next_token), parse_expr(next_token))
    else
        return Var(token)
    end
end

local function parse(str)
    -- Replace comments with spaces
    str = str:gsub("#[^\n]*\n", " ")

    -- Add spaces around tokens
    str = str:gsub("!", " ! ")
    str = str:gsub("%?", " ? ")

    -- Split by spaces
    local next_token = str:gmatch("%S+")

    return parse_expr(next_token)
end


-- Represents a delayed computation of a lambda expression.
function Closure(env, expr)
    return { env = env, expr = expr }
end

-- Represents a variable binding in a function.
-- * parent: next environment in the chain
-- * name: variable name
-- * value: Closure assigned to the variable
-- * computed: whether the value has been computed and assigned (can be nil)
function Env(parent, name, value, computed)
    return { parent = parent, name = name, value = value, computed = computed }
end

local function run(closure)
    -- Based on Krivine machine
    -- Implements an optimization to avoid re-computing variable values
    -- by updating the env when a variable is computed for the first time.

    local stack = {}

    -- Stores envs whose values need to be computed and updated
    local uncomputed_envs = {}
    -- Positions in the stack of when those envs were scheduled
    local compute_locations = {}

    while true do
        if closure.expr.type == "call" then
            table.insert(stack, Closure(closure.env, closure.expr.right))
            closure = Closure(closure.env, closure.expr.left)
        elseif closure.expr.type == "function" then
            -- Optimization: assign the value to the corresponding variables
            while compute_locations[#compute_locations] == #stack do
                table.remove(compute_locations)
                local env = table.remove(uncomputed_envs)
                env.value = closure -- Assigns the computed (optimized) value
                env.computed = true
            end

            if #stack == 0 then
                break
            end

            local argument = table.remove(stack)
            local env = Env(closure.env, closure.expr.variable, argument)
            closure = Closure(env, closure.expr.body)
        elseif closure.expr.type == "variable" then
            -- Find the environment where the variable is defined
            local env = closure.env
            while closure.env and env.name ~= closure.expr.name do
                env = env.parent
            end

            assert(env, "Unbound variable: " .. closure.expr.name)

            -- Optimization: assign a computed value to the environment later
            if not env.computed then
                table.insert(uncomputed_envs, env)
                table.insert(compute_locations, #stack)
            end

            closure = env.value
        end
    end

    return closure
end

-- The text ensures that the variable cannot be defined
local unreachable = Var("unreachable ¯\\_(ツ)_/¯")

local bit0 = parse("?x0?x1 x0")
local bit1 = parse("?x0?x1 x1")

local function bit_getter(bit_index)
    return parse("?0?1?2?3?4?5?6?7 " .. bit_index)
end

local function decode_bit(closure)
    local expr = Call(Call(closure.expr, bit0), bit1)
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

local function decode_byte(closure)
    local result = 0
    for i = 0, 7 do
        local bit_expr = Call(closure.expr, bit_getter(i))
        local bit = decode_bit(Closure(closure.env, bit_expr))
        result = result + (bit * 2 ^ i)
    end

    return result
end

local function encode_byte(byte)
    local expr = Var("x")
    for i = 0, 7 do
        local bit = math.floor(byte / 2 ^ i) % 2
        expr = Call(expr, encode_bit(bit))
    end

    return Fun("x", expr)
end

local function decode_pair(closure)
    -- Avoid duplicate computations when evaluating the items
    closure = run(closure)

    local item0 = Closure(closure.env, Call(closure.expr, bit0))
    local item1 = Closure(closure.env, Call(closure.expr, bit1))
    return item0, item1
end

local function encode_pair(expr0, expr1)
    return Fun("p", Call(Call(Var("p"), expr0), expr1))
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

local help_message =
[[Lambda calculus interpreter based on WALC format
Run with 'lua' (default), 'luajit', etc.:
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
            program = Closure(data.env, Call(data.expr, input))
        else
            -- Output
            local output, next_program = decode_pair(data)
            io.write(string.char(decode_byte(output)))
            program = next_program
        end
    end
end

main()
