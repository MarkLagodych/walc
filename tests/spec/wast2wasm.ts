#!/usr/bin/env -S deno --allow-read --allow-write --allow-env --allow-run

import fs from 'node:fs'
import path from 'node:path'
import { spawnSync } from 'node:child_process'


const scriptDir = fs.realpathSync(import.meta.dirname ?? '.')
const binDir = `${scriptDir}/bin`
const cacheDir = `${binDir}/cache`


function main() {
    if (!fs.existsSync(binDir)) {
        console.log(`Creating "${binDir}"...`)
        fs.mkdirSync(binDir)
    }

    // This can be done once because the temp files don't change between runs
    if (!fs.existsSync(cacheDir)) {
        console.log(`Creating "${cacheDir}"...`)
        fs.mkdirSync(cacheDir)

        console.log('Compiling .wast files to .wasm/.json...')
        compileWastFiles()
        console.log('Removing malformedness tests...')
        removeMalformedWatFiles()
        console.log('Filtering WASM1/LIME1 tests...')
        filterWasm1Tests()
        console.log('Converting .wasm files to .wat...')
        convertWasmToWat()
    } else {
        console.log(`"${cacheDir}" already exists, skipping .wast compilation`)
    }

    console.log('Compiling test commands from .json to .wat...')
    compileTests()

    console.log('Converting .wat to .wasm...')
    convertWatToWasm()
}

function compileWastFiles() {
    const wastFiles = fs.readFileSync(`${scriptDir}/tests.txt`, 'utf-8')
        .split('\n')
        .map(file => file.trim())
        .filter(file => file.length > 0)


    for (const wastFile of wastFiles) {
        const baseName = path.basename(wastFile, '.wast')
        const jsonFile = `${cacheDir}/${baseName}.json`

        const result = spawnSync(
            'wasm-tools',
            [
                'json-from-wast',
                wastFile,
                '--wasm-dir',
                cacheDir,
                '-o',
                jsonFile
            ]
        )

        if (result.status !== 0) {
            throw new Error(`Failed to compile ${wastFile}: ${result.stderr}`)
        }
    }
}

function removeMalformedWatFiles() {
    const malformedWatFiles = fs.readdirSync(cacheDir)
        .filter(file => file.endsWith('.wat'))
        .map(file => `${cacheDir}/${file}`)

    for (const watFile of malformedWatFiles) {
        fs.rmSync(watFile)
    }
}

function filterWasm1Tests() {
    const wasmFiles = fs.readdirSync(cacheDir)
        .filter(file => file.endsWith('.wasm'))
        .map(file => `${cacheDir}/${file}`)

    for (const wasmFile of wasmFiles) {
        const result = spawnSync(
            'wasm-tools',
            [
                'validate',
                '--features',
                'wasm1,lime1',
                wasmFile
            ]
        )

        if (result.status !== 0) {
            fs.rmSync(wasmFile)
        }
    }
}

function convertWasmToWat() {
    const wasmFiles = fs.readdirSync(cacheDir)
        .filter(file => file.endsWith('.wasm'))
        .map(file => `${cacheDir}/${file}`)

    for (const wasmFile of wasmFiles) {
        const baseName = path.basename(wasmFile, '.wasm')
        const watFile = `${cacheDir}/${baseName}.wat`

        // We need --name-unnamed because we will be inserting additional
        // imports manually and so we need all function references
        // to be stable and not change when we add imports.
        const result = spawnSync(
            'wasm-tools',
            [
                'print',
                wasmFile,
                '-o',
                watFile,
                '--name-unnamed',
                // '--print-offsets'
            ]
        )

        if (result.status !== 0) {
            throw new Error(
                `Failed to convert ${wasmFile} to .wat: ${result.stderr}`
            )
        }

        fs.rmSync(wasmFile)
    }
}


// Represents a JSON generated from a .wast file
interface TestSet {
    source_filename: string,
    commands: TestCommand[]
}

interface TestCommand {
    type: string,
    line: number,
}

interface ModuleCommand extends TestCommand {
    type: "module",
    filename: string,
}

interface AssertReturnCommand extends TestCommand {
    type: "assert_return",
    action: TestAction,
    expected: TestValue[]
}

interface AssertTrapCommand extends TestCommand {
    type: "assert_trap",
    action: TestAction,
    text: string,
}

interface TestAction {
    type: "invoke",
    module: string | undefined,
    field: string,
    args: TestValue[] | undefined
}

interface TestValue {
    type: "i32" | "i64" | "f32" | "f64",
    value: string
}

// Represents a single test within a .wast test set that relates to a single
// module
interface Test {
    moduleCommand: ModuleCommand,
    commands: (AssertReturnCommand | AssertTrapCommand)[]
}

// Represents a .wat module. A .wast file can use multiple modules for testing
interface TestModule {
    filename: string,
    line: number,
    source: string,
    exports: Map<string, string>
}


function convertWatToWasm() {
    const watFiles = fs.readdirSync(binDir)
        .filter(file => file.endsWith('.wat'))
        .map(file => `${binDir}/${file}`)

    for (const watFile of watFiles) {
        const baseName = path.basename(watFile, '.wat')
        const wasmFile = `${binDir}/${baseName}.wasm`

        const result = spawnSync(
            'wasm-tools',
            [
                'parse',
                watFile,
                '-o',
                wasmFile,
            ]
        )

        if (result.status !== 0) {
            throw new Error(
                `Failed to convert ${watFile} to .wasm: ${result.stderr}`
            )
        }
    }
}

function compileTests() {
    const testSetFiles = fs.readdirSync(cacheDir)
        .filter(file => file.endsWith('.json'))
        .map(file => `${cacheDir}/${file}`)

    for (const testSetFile of testSetFiles) {
        console.log(`Compiling tests from ${testSetFile}...`)

        const testSetSource = fs.readFileSync(testSetFile, 'utf-8')
        const testSet = JSON.parse(testSetSource) as TestSet

        compileTestSet(testSet)
    }
}

function compileTestSet(testSet: TestSet) {
    console.log(`Compiling test set from ${testSet.source_filename}...`)

    const tests = splitTestSet(testSet)

    for (const test of tests) {
        compileTest(test)
    }
}

function splitTestSet(testSet: TestSet): Test[] {
    return testSet.commands.reduce(
        (tests, cmd) => {
            switch (cmd.type) {
                case 'module':
                    tests.push({
                        moduleCommand: cmd as ModuleCommand,
                        commands: []
                    })
                    return tests

                case 'assert_return':
                case 'assert_trap':
                    tests.at(-1)
                        ?.commands
                        .push(cmd as AssertReturnCommand | AssertTrapCommand)
                    return tests

                default:
                    return tests // Ignore other command types
            }
        },
        [] as Test[]
    )
}

const imports = `(import "walc" "output" (func $walc_output (param i32)))
`

const testerCodeStart =
`(global $walc_test_id (mut i32) (i32.const 0))
(global $walc_trap_is_expected (mut i32) (i32.const 0))
(global $walc_expected_trap_code (mut i32) (i32.const 0))
(func $print_trap_code (param $code i32)
    (call $walc_output
        (i32.add
            (i32.const ${'0'.charCodeAt(0)})
            (local.get $code)
        )
    )
)
(func $print_did_not_trap
    (call $walc_output (i32.const ${'D'.charCodeAt(0)}))
    (call $walc_output (i32.const ${'i'.charCodeAt(0)}))
    (call $walc_output (i32.const ${'d'.charCodeAt(0)}))
    (call $walc_output (i32.const ${' '.charCodeAt(0)}))
    (call $walc_output (i32.const ${'n'.charCodeAt(0)}))
    (call $walc_output (i32.const ${'o'.charCodeAt(0)}))
    (call $walc_output (i32.const ${'t'.charCodeAt(0)}))
    (call $walc_output (i32.const ${' '.charCodeAt(0)}))
    (call $walc_output (i32.const ${'t'.charCodeAt(0)}))
    (call $walc_output (i32.const ${'r'.charCodeAt(0)}))
    (call $walc_output (i32.const ${'a'.charCodeAt(0)}))
    (call $walc_output (i32.const ${'p'.charCodeAt(0)}))
)
(func $print_bad_value
    (call $walc_output (i32.const ${'B'.charCodeAt(0)}))
    (call $walc_output (i32.const ${'a'.charCodeAt(0)}))
    (call $walc_output (i32.const ${'d'.charCodeAt(0)}))
    (call $walc_output (i32.const ${' '.charCodeAt(0)}))
    (call $walc_output (i32.const ${'v'.charCodeAt(0)}))
    (call $walc_output (i32.const ${'a'.charCodeAt(0)}))
    (call $walc_output (i32.const ${'l'.charCodeAt(0)}))
    (call $walc_output (i32.const ${'u'.charCodeAt(0)}))
    (call $walc_output (i32.const ${'e'.charCodeAt(0)}))
)
(export "handle_trap" (func $walc_handle_trap))
(func $walc_handle_trap (param $trap_code i32)
    ;; used unsupported float arithmetic
    (i32.eq (local.get $trap_code) (i32.const 2))
    if
        ;; (return)

        (global.set $walc_test_id
            (i32.add (global.get $walc_test_id) (i32.const 1))
        )
        (call $walc_main)
        (return)
    else
        ;; nop
    end

    (global.get $walc_trap_is_expected)
    if
        (i32.eq
            (global.get $walc_expected_trap_code)
            (local.get $trap_code)
        )
        if
            (global.set $walc_test_id
                (i32.add (global.get $walc_test_id) (i32.const 1))
            )
            (call $walc_main)
        else
            ;; "Bad trap X, expected Y"
            (call $walc_output (i32.const ${'B'.charCodeAt(0)}))
            (call $walc_output (i32.const ${'a'.charCodeAt(0)}))
            (call $walc_output (i32.const ${'d'.charCodeAt(0)}))
            (call $walc_output (i32.const ${' '.charCodeAt(0)}))
            (call $walc_output (i32.const ${'t'.charCodeAt(0)}))
            (call $walc_output (i32.const ${'r'.charCodeAt(0)}))
            (call $walc_output (i32.const ${'a'.charCodeAt(0)}))
            (call $walc_output (i32.const ${'p'.charCodeAt(0)}))
            (call $walc_output (i32.const ${' '.charCodeAt(0)}))
            (call $print_trap_code (local.get $trap_code))
            (call $walc_output (i32.const ${', '.charCodeAt(0)}))
            (call $walc_output (i32.const ${'e'.charCodeAt(0)}))
            (call $walc_output (i32.const ${'x'.charCodeAt(0)}))
            (call $walc_output (i32.const ${'p'.charCodeAt(0)}))
            (call $walc_output (i32.const ${'e'.charCodeAt(0)}))
            (call $walc_output (i32.const ${'c'.charCodeAt(0)}))
            (call $walc_output (i32.const ${'t'.charCodeAt(0)}))
            (call $walc_output (i32.const ${'e'.charCodeAt(0)}))
            (call $walc_output (i32.const ${'d'.charCodeAt(0)}))
            (call $walc_output (i32.const ${' '.charCodeAt(0)}))
            (call $print_trap_code (global.get $walc_expected_trap_code))
        end
    else
        ;; "Unexpected trap X"
        (call $walc_output (i32.const ${'U'.charCodeAt(0)}))
        (call $walc_output (i32.const ${'n'.charCodeAt(0)}))
        (call $walc_output (i32.const ${'e'.charCodeAt(0)}))
        (call $walc_output (i32.const ${'x'.charCodeAt(0)}))
        (call $walc_output (i32.const ${'p'.charCodeAt(0)}))
        (call $walc_output (i32.const ${'e'.charCodeAt(0)}))
        (call $walc_output (i32.const ${'c'.charCodeAt(0)}))
        (call $walc_output (i32.const ${'t'.charCodeAt(0)}))
        (call $walc_output (i32.const ${'e'.charCodeAt(0)}))
        (call $walc_output (i32.const ${'d'.charCodeAt(0)}))
        (call $walc_output (i32.const ${' '.charCodeAt(0)}))
        (call $walc_output (i32.const ${'t'.charCodeAt(0)}))
        (call $walc_output (i32.const ${'r'.charCodeAt(0)}))
        (call $walc_output (i32.const ${'a'.charCodeAt(0)}))
        (call $walc_output (i32.const ${'p'.charCodeAt(0)}))
        (call $walc_output (i32.const ${' '.charCodeAt(0)}))
        (call $print_trap_code (local.get $trap_code))
    end
)
(export "main" (func $walc_main))
(func $walc_main
`

const testerCodeEnd = `
;; "OK"
(call $walc_output (i32.const ${'O'.charCodeAt(0)}))
(call $walc_output (i32.const ${'K'.charCodeAt(0)}))
)
`

function testCommand(
    id: number,
    module: TestModule,
    cmd: AssertReturnCommand | AssertTrapCommand
): string {
    const comment = `;; ${cmd.type} at line ${cmd.line}\n`

    if (cmd.action.module !== undefined) {
        return '' // We do not support multiple modules at a time
    }

    switch (cmd.type) {
        case 'assert_return':
            return comment + assertReturn(id, module, cmd)

        case 'assert_trap':
            return comment + assertTrap(id, module, cmd)
    }
}

function assertReturn(
    id: number,
    module: TestModule,
    cmd: AssertReturnCommand
): string {

    let code =
`(i32.eq (global.get $walc_test_id) (i32.const ${id}))
if
    (global.set $walc_trap_is_expected (i32.const 0))

    ${performAction(module, cmd.action)}`

    for (const expected of cmd.expected.reverse()) {
        code +=
`
    ${makeValue(expected)}
    (${expected.type}.eq)
    if
        ;; nop
    else
        (call $print_bad_value)
        (return)
    end`
    }

    code +=
`
    (global.set $walc_test_id
        (i32.add (global.get $walc_test_id) (i32.const 1))
    )
else
    ;; nop
end
`

    return code

}

function assertTrap(
    id: number,
    module: TestModule,
    cmd: AssertTrapCommand
): string {

    let expectedTrapCode = ""

    switch (cmd.text) {
        case "unreachable":
            expectedTrapCode = "0"
            break

        case "integer divide by zero":
        case "integer overflow":
            expectedTrapCode = "1"
            break

        default:
            return "" // Unsupported trap type
    }

return `(if (i32.eq (global.get $walc_test_id) (i32.const ${id}))
    (then
        (global.set $walc_trap_is_expected (i32.const 1))
        (global.set $walc_expected_trap_code
            (i32.const ${expectedTrapCode})
        )

        ${performAction(module, cmd.action)}

        (call $print_did_not_trap)
        (return)
    )
)
`

}

function performAction(module: TestModule, action: TestAction): string {
    if (action.type !== 'invoke') {
        throw new Error(`Unsupported action type: ${action.type}`)
    }

    const func = module.exports.get(action.field)

    if (func === undefined) {
        throw new Error(
            `Function "${action.field}" not found in exports of`
            + ` ${module.filename}:${module.line}`
        )
    }

    const args = (action.args ?? []).map(makeValue).map(a => ' ' + a).join('')

    return `(call ${func}${args})`
}

function makeValue(value: TestValue): string {
    if (value.type === 'f32') {
        if (value.value === 'nan:canonical') {
            return `(${value.type}.reinterpret_i32 (i32.const 0x7FC00000))`
        } else if (value.value === 'nan:arithmetic') {
            return `(${value.type}.reinterpret_i32 (i32.const 0x7FC00001))`
        }
    } else if (value.type === 'f64') {
        if (value.value === 'nan:canonical') {
            return `(${value.type}.reinterpret_i64 (i64.const 0x7FF8000000000000))`
        } else if (value.value === 'nan:arithmetic') {
            return `(${value.type}.reinterpret_i64 (i64.const 0x7FF8000000000001))`
        }
    }

    return `(${value.type}.const ${value.value})`
}


function compileTest(test: Test) {
    const module = readModule(test.moduleCommand)

    if (module === null)
        return // The test was filtered out

    let source = module.source

    source = prependCode(source, imports)

    let testerCode = testerCodeStart
    for (let i=0; i<test.commands.length; i++) {
        const cmd = test.commands[i]
        testerCode += testCommand(i, module, cmd)
    }
    testerCode += testerCodeEnd

    source = appendCode(source, testerCode)


    const outFile = `${binDir}/${path.basename(module.filename, '.wat')}.wat`
    fs.writeFileSync(outFile, source)
}

// Returns null if the file does not exist (it was filtered out)
function readModule(cmd: ModuleCommand): TestModule | null {
    const baseName = path.basename(cmd.filename, '.wasm')
    const moduleName = `${cacheDir}/${baseName}.wat`

    if (!fs.existsSync(moduleName)) {
        return null // The test was filtered out
    }

    const source = fs.readFileSync(moduleName, 'utf-8')

    if (hasAnyImports(source)) {
        return null // We do not support modules with non-WALC imports
    }

    return {
        filename: moduleName,
        line: cmd.line,
        source: source,
        exports: findExportedFunctions(source)
    }
}

function hasAnyImports(watSource: string): boolean {
    const r = /\(import/g
    return r.test(watSource)
}

function findExportedFunctions(watSource: string): Map<string, string> {
    const exports = new Map<string, string>()
    const r = /\(export "([^"]+)" \(func ([^)]+)\)\)/g
    let match

    while ((match = r.exec(watSource)) !== null) {
        exports.set(match[1], match[2])
    }

    return exports
}

function prependCode(baseWatSource: string, code: string): string {
    // The first line should be "(module" or "(module $name"
    const matches =
        /\(module[^\)\n]*/
        .exec(baseWatSource)

    if (matches === null) {
        throw new Error('Invalid .wat source: no module declaration found')
    }

    const insertIndex = matches.index + matches[0].length
    return baseWatSource.slice(0, insertIndex) + '\n'
        + code + '\n'
        + baseWatSource.slice(insertIndex)
}

function appendCode(baseWatSource: string, code: string): string {
    const lastParenIndex = baseWatSource.lastIndexOf(')')

    if (lastParenIndex === -1) {
        throw new Error('Invalid .wat source: no closing parenthesis found')
    }

    return baseWatSource.slice(0, lastParenIndex)
        + code
        + baseWatSource.slice(lastParenIndex)
}



main()
