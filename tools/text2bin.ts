#!/usr/bin/env -S deno --allow-read --allow-write

// Copyright (c) 2025-2026 Mark Lagodych
// SPDX-License-Identifier: MIT

type Expr = Var | Abs | Apply

const VAR_FLAGS = 0b10 << 30
const ABS_FLAGS = 0b11 << 30

class Var {
    constructor(public id: number) {}
    encode(): number { return this.id | VAR_FLAGS; }
}

class Abs {
    constructor(public var_id: number) {}
    encode(): number { return this.var_id | ABS_FLAGS; }
}

class Apply {
    // right_index is an index into the expression list, can be set later.
    constructor(public right_index: number = 0) {}
    encode(): number { return this.right_index; }
}


class VariableIdBinding {
    constructor(public name: string, public id: number) {}
}

class VariableIdGenerator {
    private bindings: VariableIdBinding[] = []

    private next_id: number = 0

    add_variable(variable_name: string) {
        this.bindings.push(new VariableIdBinding(variable_name, this.next_id))
        this.next_id++
    }

    pop_variable() {
        this.bindings.pop()
    }

    find_id(variable_name: string): number {
        for (let i = this.bindings.length - 1; i >= 0; i--) {
            if (this.bindings[i].name == variable_name) {
                return this.bindings[i].id
            }
        }

        throw new Error(`Undefined variable: ${variable_name}`)
    }

    get_total_count(): number {
        return this.next_id
    }
}

class Generator {
    private expressions: Expr[] = []

    // Stack of indexes into `expressions`
    private unfinished_applications: number[] = []

    // Stack of indexes into `expressions`
    private expression_starts: number[] = []

    private variables: VariableIdGenerator = new VariableIdGenerator()

    generate(): Uint8Array {
        const numbers = [
            this.expressions.length,
            this.variables.get_total_count(),
            ...this.expressions.map(expr => expr.encode())
        ]
        return new Uint8Array(new Uint32Array(numbers).buffer)
    }

    start_application() {
        this.expression_starts.push(this.expressions.length)
        this.unfinished_applications.push(this.expressions.length)
        this.expressions.push(new Apply())
    }

    end_application() {
        const app_index = this.unfinished_applications.pop()!
        const right_index = this.expression_starts.pop()!
        const _left_index = this.expression_starts.pop()!
        const app = this.expressions[app_index] as Apply
        app.right_index = right_index
    }

    start_abstraction(variable: string) {
        this.variables.add_variable(variable)
        this.expression_starts.push(this.expressions.length)
        this.expressions.push(new Abs(this.variables.find_id(variable)))
    }

    end_abstraction() {
        this.variables.pop_variable()
        const _body_index = this.expression_starts.pop()!
    }

    handle_variable(variable: string) {
        this.expressions.push(new Var(this.variables.find_id(variable)))
    }
}


class Parser {
    // ( whitespace | comment )* ( "(" | ")" | "[" | "]" | <identifier> )
    private token_regex = /(\s|(;.*\n))*(?<token>[()\[\]]|\w+)/y

    private generator: Generator = new Generator()

    constructor(private input: string) {}

    generate(): Uint8Array {
        return this.generator.generate()
    }

    private next_token(): string {
        const match = this.token_regex.exec(this.input)
        if (!match) throw new Error("Expected token")
        return match.groups!.token
    }

    private is_variable(token: string): boolean {
        return token != "(" && token != ")" && token != "[" && token != "]"
    }

    parse() {
        type ExpectedItem = ")" | "]" | "term"
        const stack: ExpectedItem[] = ["term"]

        while (stack.length > 0) {
            const token = this.next_token()

            switch (token) {
                case "(": {
                    if (stack.pop() != "term")
                        throw new Error("Unexpected '('")

                    stack.push(")")
                    stack.push("term")
                    stack.push("term")

                    this.generator.start_application()
                    break
                }
                case ")": {
                    if (stack.pop() != ")")
                        throw new Error("Unexpected ')'")

                    this.generator.end_application()
                    break
                }
                case "[": {
                    if (stack.pop() != "term")
                        throw new Error("Unexpected '['")

                    stack.push("]")
                    stack.push("term")

                    const variable = this.next_token()

                    if (!this.is_variable(variable))
                        throw new Error(`Expected variable, got: ${variable}`)

                    this.generator.start_abstraction(variable)
                    break
                }
                case "]": {
                    if (stack.pop() != "]")
                        throw new Error("Unexpected ']'")

                    this.generator.end_abstraction()
                    break
                }
                default: {
                    if (stack.pop() != "term")
                        throw new Error(`Unexpected variable: ${token}`)

                    this.generator.handle_variable(token)
                    break
                }
            }
        }
    }
}


import process from "node:process"
import * as fs from "node:fs"

const help_message =
`Converts lambda expressions from the WALC text format to the binary format.
The resulting files depend on machine endianness and are not portable.
Run with 'deno --allow-read --allow-write' (default), 'tsx', 'bun', etc.:
$ [<TS_INTERP>] text2bin.ts <INPUT_FILE>.walc <OUTPUT_FILE>.bin`

function main() {
    const args = process.argv.slice(2)

    if (args.length != 2) {
        console.log(help_message)
        return
    }

    const input_path = args[0]
    const output_path = args[1]

    const source = fs.readFileSync(input_path, "utf-8")

    const parser = new Parser(source)
    parser.parse()

    const result = parser.generate()

    fs.writeFileSync(output_path, result, "binary")
}

main()
