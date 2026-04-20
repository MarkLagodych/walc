#!/usr/bin/env -S deno --allow-read

// This is a simple lambda calculus interpreter based on the WALC format.

// Version 1.0

// Copyright (c) 2025-2026 Mark Lagodych
// SPDX-License-Identifier: MIT

type Expr = Var | Abs | Apply
class Var { constructor(public name: string) {} }
class Abs { constructor(public variable: string, public body: Expr) {} }
class Apply { constructor(public left: Expr, public right: Expr) {} }

function parse_expr(next_token: () => string): Expr {
    const token = next_token()

    if (token == "[") {
        const variable = next_token()
        const body = parse_expr(next_token)
        if (next_token() != "]") throw new Error("Expected ']'")
        return new Abs(variable, body)
    }

    if (token == "(") {
        const left = parse_expr(next_token)
        const right = parse_expr(next_token)
        if (next_token() != ")") throw new Error("Expected ')'")
        return new Apply(left, right)
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

function run(closure: Closure, recursion_depth: number = 0): Closure {
    /* Based on the Krivine machine with a mixed computation strategy:

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
        that might never be actually used, but it is surely the simplest one. */

    const stack: Closure[] = []

    // (1)
    // Stores envs whose values need to be computed and updated
    const uncomputed_envs: Env[] = []
    // Positions in the stack of when those envs were scheduled
    const stack_locations: number[] = []

    while (true) {
        if (closure.expr instanceof Var) {
            // Find the environment where the variable is defined
            let env = closure.env
            while (env && env.name != closure.expr.name) env = env.parent

            if (!env) throw new Error(`Unbound variable: ${closure.expr.name}`)

            // (1) Schedule the variable for an update
            if (!env.computed) {
                uncomputed_envs.push(env)
                stack_locations.push(stack.length)
            }

            closure = env.value
        }
        else if (closure.expr instanceof Abs) {
            // (1) Update variable values
            while (stack_locations.at(-1) === stack.length) {
                stack_locations.pop()!
                const env = uncomputed_envs.pop()!
                env.value = closure // Assign the computed value
                env.computed = true
            }

            if (stack.length == 0)
                break

            const argument = stack.pop()!
            const env = new Env(closure.env, closure.expr.variable, argument)
            closure = new Closure(env, closure.expr.body)
        }
        else if (closure.expr instanceof Apply) {
            let right = new Closure(closure.env, closure.expr.right)

            // (2) Preemptively compute the right if it is a variable,
            // but limit the recursion depth (10 worked well during tests).
            if (right.expr instanceof Var && recursion_depth < 10)
                right = run(right, recursion_depth + 1)

            stack.push(right)
            closure = new Closure(closure.env, closure.expr.left)
        }
    }

    return closure
}

// The text in the variable name ensures that it cannot be defined by the
// program. It is wrapped in an abstraction to prevent it from being
// preemptively computed by the interpreter.
const unreachable = new Abs("〜⁠(⁠꒪⁠꒳⁠꒪⁠)⁠〜", new Var("unreachable (⊙＿⊙')"))

const bit0 = parse("[x0[x1 x0]]")
const bit1 = parse("[x0[x1 x1]]")

function decode_bit(closure: Closure): number {
    const expr = new Apply(new Apply(closure.expr, bit0), bit1)
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

function decode_pair(closure: Closure): [Closure, Closure] {
    // Avoid duplicate computations when evaluating the items
    closure = run(closure)

    const item0 = new Closure(closure.env, new Apply(closure.expr, bit0))
    const item1 = new Closure(closure.env, new Apply(closure.expr, bit1))
    return [item0, item1]
}

function encode_pair(expr0: Expr, expr1: Expr): Expr {
    return new Abs("p", new Apply(new Apply(new Var("p"), expr0), expr1))
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

function decode_list(closure: Closure): Closure[] {
    const node = decode_optional(closure);
    if (!node) return [];

    const [head, tail] = decode_pair(node);
    return [head, ...decode_list(tail)];
}

function encode_list(items: Expr[]): Expr {
    const node = items.length == 0
        ? null
        : encode_pair(items[0], encode_list(items.slice(1)))

    return encode_optional(node)
}

function decode_byte(closure: Closure): number {
    const bits = decode_list(closure)

    let byte = 0
    for (const bit of bits) {
        byte <<= 1
        byte |= decode_bit(bit)
    }

    return byte
}

function encode_byte(byte: number): Expr {
    const bits = []

    for (let i = 0; i < 8; i++) {
        bits.push(encode_bit(byte & 1))
        byte >>= 1
    }

    return encode_list(bits.reverse())
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
            program = new Closure(data.env, new Apply(data.expr, input))
        } else {
            // Output
            const [output, next_program] = decode_pair(data)
            write_byte(decode_byte(output))
            program = next_program
        }
    }
}

main()
