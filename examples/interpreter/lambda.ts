#!/usr/bin/env -S npx tsx

/*  This is a simple lambda calculus interpreter based on the WALC format.
    Run with:
    - Node.js and TSX: $ npx tsx lambda.ts <FILENAME>
    - Deno: $ deno run --allow-read lambda.ts <FILENAME>
    - Bun: $ bun run lambda.ts <FILENAME>

    Copyright (c) 2025 Mark Lagodych
    SPDX-License-Identifier: MIT */

type Expr = Var | Fun | Call
class Var { constructor(public name: string) {} }
class Fun { constructor(public variable: string, public body: Expr) {} }
class Call { constructor(public left: Expr, public right: Expr) {} }

const token_regex = /(\s|(#.*\n))*(?<token>\.|\(|\)|\\|\w+)/y

function tokenize(input: string): string {
    const match = token_regex.exec(input)
    if (!match) throw new Error("Expected token")
    return match.groups!.token
}

function parse_expr(input: string): Expr {
    const token = tokenize(input)

    if (token == "\\") {
        const variable = tokenize(input)
        tokenize(input) // "."
        const body = parse_expr(input)
        return new Fun(variable, body)
    }

    if (token == "(") {
        const left = parse_expr(input)
        const right = parse_expr(input)
        tokenize(input) // ")"
        return new Call(left, right)
    }

    return new Var(token)
}

function parse(input: string): Expr {
    token_regex.lastIndex = 0 // Reset the regex
    return parse_expr(input)
}

// Represents a suspended computation of an expression
class Value {
    constructor(public env: Env | null, public expr: Expr) {}
}
// Represents a variable binding in a function
class Env {
    constructor(
        public parent: Env | null, // Next environment in the chain
        public name: string, // Variable name
        public value: Value, // Variable value (may be updated when computed)
        public computed?: boolean // If undefined, the value will be updated
    ) {}
}

function run(value: Value): Value {
    /*  Based on Krivine machine.
        Implements an optimization to avoid re-computing variable values
        by updating the env when a variable is computed for the first time. */

    const stack: Value[] = []

    // Stores envs whose values need to be computed and updated
    const uncomputed_envs: Env[] = []
    // Positions in the stack of when those envs were scheduled
    const stack_locations: number[] = []

    while (true) {
        if (value.expr instanceof Call) {
            stack.push(new Value(value.env, value.expr.right))
            value = new Value(value.env, value.expr.left)
        }
        else if (value.expr instanceof Fun) {
            // Optimization: update the values of the corresponding envs
            while (stack_locations.at(-1) === stack.length) {
                stack_locations.pop()!
                const env = uncomputed_envs.pop()!
                env.value = value // Assigns the computed (optimized) value
                env.computed = true
            }

            if (stack.length == 0) return value

            const argument = stack.pop()!
            const env = new Env(value.env, value.expr.variable, argument)
            value = new Value(env, value.expr.body)
        }
        else if (value.expr instanceof Var) {
            // Find the environment where the variable is defined
            let env = value.env
            while (env && env.name != value.expr.name) env = env.parent

            if (!env) throw new Error(`Unbound variable: ${value.expr.name}`)

            // Optimization: schedule the env for a value update
            if (!env.computed) {
                uncomputed_envs.push(env)
                stack_locations.push(stack.length)
            }

            value = env.value
        }
    }
}

// The text ensures that the variable cannot be defined
const unreachable = new Var("unreachable ¯\\_(ツ)_/¯")

const bit0 = parse("\\x0.\\x1.x0")
const bit1 = parse("\\x0.\\x1.x1")

const bit_getter = (bit_index: number): Expr =>
    parse(`\\0.\\1.\\2.\\3.\\4.\\5.\\6.\\7. ${bit_index}`)

function decode_bit(value: Value): number {
    const expr = new Call(new Call(value.expr, bit0), bit1)
    const result = run(new Value(value.env, expr))
    switch (result.expr) {
        case bit0: return 0
        case bit1: return 1
        default: throw new Error("Expected bit")
    }
}

function encode_bit(x: number): Expr {
    switch (x) {
        case 0: return bit0
        case 1: return bit1
        default: throw new Error("Expected bit")
    }
}

function decode_byte(value: Value): number {
    let result = 0
    for (let i = 0; i <= 7; i++) {
        const bit_expr = new Call(value.expr, bit_getter(i))
        const bit = decode_bit(new Value(value.env, bit_expr))
        result |= bit << i
    }

    return result
}

function encode_byte(x: number): Expr {
    let expr: Expr = new Var("x")
    for (let i = 7; i >= 0; i--) {
        const bit = (x >> i) & 1
        expr = new Call(expr, encode_bit(bit))
    }

    return new Fun("x", expr)
}

function decode_pair(value: Value): [Value, Value] {
    value = run(value) // Avoid duplicate computations when evaluating the items

    const item0 = new Value(value.env, new Call(value.expr, bit0))
    const item1 = new Value(value.env, new Call(value.expr, bit1))
    return [item0, item1]
}

function encode_pair(item0: Expr, item1: Expr): Expr {
    return new Fun("p", new Call(new Call(new Var("p"), item0), item1))
}

function decode_optional(value: Value): Value | null {
    const [has_data, data] = decode_pair(value)
    return decode_bit(has_data) ? data : null
}

function encode_optional(data: Expr | null): Expr {
    if (!data)
        return encode_pair(encode_bit(0), unreachable)

    return encode_pair(encode_bit(1), data)
}


import process from "node:process"
import * as fs from "node:fs"

function main(args: string[]) {
    if (args.length != 1 || args[0] == "--help") {
        console.log("Lambda calculus interpreter based on WALC format")
        console.log("Usage: ./lambda.ts <FILENAME>")
        return
    }

    const source = fs.readFileSync(args[0], "utf-8")
    let program = new Value(null, parse(source))

    while (true) {
        const result = decode_optional(program)
        if (!result) return

        const [command, payload] = decode_pair(result)

        if (decode_bit(command) == 0) {
            // Output
            const [output, next_program] = decode_pair(payload)
            const byte = decode_byte(output)
            fs.writeSync(process.stdout.fd, new Uint8Array([byte]))
            program = next_program
        } else {
            // Input
            const buffer = new Uint8Array(1)
            const read_result = fs.readSync(process.stdin.fd, buffer)
            const input_byte = read_result ? encode_byte(buffer[0]) : null
            const input = encode_optional(input_byte)
            program = new Value(payload.env, new Call(payload.expr, input))
        }
    }
}

main(process.argv.slice(2))
