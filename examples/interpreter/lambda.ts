#!/usr/bin/env -S deno --allow-read

// This is a simple lambda calculus interpreter based on the WALC format.

// Copyright (c) 2025-2026 Mark Lagodych
// SPDX-License-Identifier: MIT

type Expr = Var | Fun | Call
class Var { constructor(public name: string) {} }
class Fun { constructor(public variable: string, public body: Expr) {} }
class Call { constructor(public left: Expr, public right: Expr) {} }

function parse_expr(next_token: () => string): Expr {
    const token = next_token()

    if (token == "[") {
        const variable = next_token()
        const body = parse_expr(next_token)
        if (next_token() != "]") throw new Error("Expected ']'")
        return new Fun(variable, body)
    }

    if (token == "(") {
        const left = parse_expr(next_token)
        const right = parse_expr(next_token)
        if (next_token() != ")") throw new Error("Expected ')'")
        return new Call(left, right)
    }

    return new Var(token)
}

function parse(input: string): Expr {
    // ( whitespace | comment )* ( "(" | ")" | "[" | "]" | <identifier> )
    const token_regex = /(\s|(;.*\n))*(?<token>[()\[\]]|\w+)/y

    function next_token(): string {
        const match = token_regex.exec(input)
        if (!match) throw new Error("Expected token")
        return match.groups!.token
    }

    return parse_expr(next_token)
}

// Represents a suspended computation of an expression
class Closure {
    constructor(public env: Env | null, public expr: Expr) {}
}
// Represents a variable binding in a function
class Env {
    constructor(
        public parent: Env | null, // Next environment in the chain
        public name: string, // Variable name
        public value: Closure, // Variable value (may be updated when computed)
        public computed?: boolean // If undefined, the value will be updated
    ) {}
}

function run(closure: Closure): Closure {
    // Based on Krivine machine.
    // Implements an optimization to avoid re-computing variable values
    // by updating the env when a variable is computed for the first time.

    const stack: Closure[] = []

    // Stores envs whose values need to be computed and updated
    const uncomputed_envs: Env[] = []
    // Positions in the stack of when those envs were scheduled
    const stack_locations: number[] = []

    while (true) {
        if (closure.expr instanceof Call) {
            stack.push(new Closure(closure.env, closure.expr.right))
            closure = new Closure(closure.env, closure.expr.left)
        }
        else if (closure.expr instanceof Fun) {
            // Optimization: update the values of the corresponding envs
            while (stack_locations.at(-1) === stack.length) {
                stack_locations.pop()!
                const env = uncomputed_envs.pop()!
                env.value = closure // Assigns the computed (optimized) value
                env.computed = true
            }

            if (stack.length == 0)
                break

            const argument = stack.pop()!
            const env = new Env(closure.env, closure.expr.variable, argument)
            closure = new Closure(env, closure.expr.body)
        }
        else if (closure.expr instanceof Var) {
            // Find the environment where the variable is defined
            let env = closure.env
            while (env && env.name != closure.expr.name) env = env.parent

            if (!env) throw new Error(`Unbound variable: ${closure.expr.name}`)

            // Optimization: schedule the env for a value update
            if (!env.computed) {
                uncomputed_envs.push(env)
                stack_locations.push(stack.length)
            }

            closure = env.value
        }
    }

    return closure
}

// The text ensures that the variable will be undefined and thus cause an error
const unreachable = new Var("unreachable ¯\\_(ツ)_/¯")

const bit0 = parse("[x0[x1 x0]]")
const bit1 = parse("[x0[x1 x1]]")

const bit_getter = (bit_index: number): Expr =>
    parse(`[7[6[5[4[3[2[1[0 ${bit_index}]]]]]]]]`)

function decode_bit(closure: Closure): number {
    const expr = new Call(new Call(closure.expr, bit0), bit1)
    const result = run(new Closure(closure.env, expr))
    switch (result.expr) {
        case bit0: return 0
        case bit1: return 1
        default: throw new Error("Expected bit")
    }
}

function encode_bit(bit: number): Expr {
    switch (bit) {
        case 0: return bit0
        case 1: return bit1
        default: throw new Error("Expected bit")
    }
}

function decode_byte(closure: Closure): number {
    let result = 0
    for (let i = 0; i <= 7; i++) {
        const bit_expr = new Call(closure.expr, bit_getter(i))
        const bit = decode_bit(new Closure(closure.env, bit_expr))
        result |= bit << i
    }

    return result
}

function encode_byte(byte: number): Expr {
    let expr: Expr = new Var("x")
    for (let i = 7; i >= 0; i--) {
        const bit = (byte >> i) & 1
        expr = new Call(expr, encode_bit(bit))
    }

    return new Fun("x", expr)
}

function decode_pair(closure: Closure): [Closure, Closure] {
    // Avoid duplicate computations when evaluating the items
    closure = run(closure)

    const item0 = new Closure(closure.env, new Call(closure.expr, bit0))
    const item1 = new Closure(closure.env, new Call(closure.expr, bit1))
    return [item0, item1]
}

function encode_pair(expr0: Expr, expr1: Expr): Expr {
    return new Fun("p", new Call(new Call(new Var("p"), expr0), expr1))
}

function decode_optional(closure: Closure): Closure | null {
    const [has_data, data] = decode_pair(closure)
    return decode_bit(has_data) ? data : null
}

function encode_optional(expr: Expr | null): Expr {
    return expr
        ? encode_pair(encode_bit(1), expr)
        : encode_pair(encode_bit(0), unreachable)
}


import process from "node:process"
import * as fs from "node:fs"

const args = process.argv.slice(2)

function read_file(path: string): string {
    return fs.readFileSync(path, "utf-8")
}

function write_byte(byte: number) {
    fs.writeSync(process.stdout.fd, new Uint8Array([byte]))
}

function read_byte(): number | null {
    const buffer = new Uint8Array(1)
    const length = fs.readSync(process.stdin.fd, buffer)
    return length == 1 ? buffer[0] : null
}

const help_message =
`Lambda calculus interpreter based on WALC format
Run with 'deno --allow-read' (default), 'tsx', 'bun', etc.:
$ [<TYPESCRIPT_INTERPRETER>] ./lambda.ts <FILENAME>`

function main() {
    if (args.length != 1 || args[0] == "--help") {
        console.log(help_message)
        return
    }

    const source = read_file(args[0])
    let program = new Closure(null, parse(source))

    while (true) {
        const command = decode_optional(program)
        if (!command) return

        const [is_input, data] = decode_pair(command)

        if (decode_bit(is_input) == 1) {
            // Input
            const byte = read_byte()
            const input_expr = (byte == null) ? null : encode_byte(byte)
            const input = encode_optional(input_expr)
            program = new Closure(data.env, new Call(data.expr, input))
        } else {
            // Output
            const [output, next_program] = decode_pair(data)
            write_byte(decode_byte(output))
            program = next_program
        }
    }
}

main()
