#!/usr/bin/env -S deno --allow-read --allow-write --allow-env --allow-run

import fs from 'node:fs'
import path from 'node:path'
import process from 'node:process'
import { spawnSync } from 'node:child_process'

const scriptDir = fs.realpathSync(import.meta.dirname ?? '.')
const rootDir = fs.realpathSync(`${scriptDir}/../..`)
const binDir = `${scriptDir}/bin`
const wastRootDir = `${scriptDir}/spec/test`


function discoverWastFiles(): string[] {
    const corePath = `${wastRootDir}/core`
    const bulkMemoryPath = `${wastRootDir}/core/bulk-memory`

    // All core tests that don't mention floats
    const coreTests = fs.readdirSync(corePath)
        .filter(file => file.endsWith('.wast'))
        .filter(file => !/^f32|f64|float/.test(file))
        .map(file => `${corePath}/${file}`)

    // memory.fill/copy tests
    const bulkMemoryTests = fs.readdirSync(bulkMemoryPath)
        .filter(file => file.endsWith('.wast'))
        .filter(file => /^memory/.test(file))
        .map(file => `${bulkMemoryPath}/${file}`)

    return [...coreTests, ...bulkMemoryTests]
}

function compileWastFiles(wastFiles: string[]) {
    console.log('Compiling .wast files to .wasm/.json...')

    for (const wastFile of wastFiles) {
        const baseName = path.basename(wastFile, '.wast')

        console.log(`Processing ${wastFile}...`)

        spawnSync(
            'wasm-tools',
            [
                'json-from-wast',
                wastFile,
                '--wasm-dir',
                binDir,
                '-o',
                `${binDir}/${baseName}.json`
            ]
        )
    }
}

function removeMalformedWatFiles() {
    console.log('Removing malformedness tests...')

    const malformedWatFiles = fs.readdirSync(binDir)
        .filter(file => file.endsWith('.wat'))

    for (const watFile of malformedWatFiles) {
        fs.rmSync(`${binDir}/${watFile}`)
    }
}

function filterWasm1Tests() {
    console.log('Filtering WASM1/LIME1 tests...')

    const wasmFiles = fs.readdirSync(binDir)
        .filter(file => file.endsWith('.wasm'))

    for (const wasmFile of wasmFiles) {
        const result = spawnSync(
            'wasm-tools',
            [
                'validate',
                '--features',
                'wasm1,lime1',
                `${binDir}/${wasmFile}`
            ]
        )

        if (result.status === 0) {
            continue
        }

        fs.rmSync(`${binDir}/${wasmFile}`)
    }
}

function convertWasmToWat() {
    console.log('Converting .wasm files to .wat...')

    const wasmFiles = fs.readdirSync(binDir)
        .filter(file => file.endsWith('.wasm'))

    for (const wasmFile of wasmFiles) {
        const baseName = path.basename(wasmFile, '.wasm')

        // We need --name-unnamed because we will be inserting additional
        // imports manually and so we need all function references
        // to be stable and not change when we add imports
        const result = spawnSync(
            'wasm-tools',
            [
                'print',
                `${binDir}/${wasmFile}`,
                '-o',
                `${binDir}/${baseName}.wat`,
                '--name-unnamed'
            ]
        )

        if (result.status !== 0) {
            throw new Error(`Failed to convert ${wasmFile} to .wat`)
        }

        fs.rmSync(`${binDir}/${wasmFile}`)
    }
}

function createTestingFiles() {
    const wastFiles = discoverWastFiles()
    compileWastFiles(wastFiles)
    removeMalformedWatFiles()
    filterWasm1Tests()
    convertWasmToWat()
}

function setup() {
    if (!fs.existsSync(binDir)) {
        console.log('Creating "bin/"...')
        fs.mkdirSync(binDir)
        createTestingFiles()
    } else {
        console.log('"bin/" already exists, skipping test case generation')
    }
}


interface TestSet {
    source_filename: string,
    commands: TestCommand[]
}

interface TestCommand {
    type: string,
}

interface AssertReturnCommand extends TestCommand {
    type: "assert_return",
    action: TestAction,
    expected: TestValue[]
}

interface AssertTrapCommand extends TestCommand {
    type: "assert_trap",
    action: TestAction,
    expected: undefined
}

interface ModuleCommand extends TestCommand {
    type: "module",
    filename: string,
}

interface TestAction {
    type: "invoke",
    field: string,
    args: TestValue[]
}

interface TestValue {
    type: "i32" | "i64" | "f32" | "f64",
    value: string
}


// Returns the STDOUT or throws an error
function convertWat2Wasm(watSource: string): Buffer {
    const result = spawnSync(
        'wasm-tools',
        ['parse'],
        { input: watSource }
    )

    if (result.status !== 0) {
        throw new Error(`Failed to convert .wat to .wasm: ${result.stderr}`)
    }

    return result.stdout
}

// Returns the STDOUT or throws an error
function convertWasm2Walc(wasmSource: Buffer): Buffer {
    const result = spawnSync(
        'cargo',
        [
            'run',
            '--quiet',
            '--features',
            'unbound-unreachable',
            '--',
            '-' // Read the source from STDIN
        ],
        { input: wasmSource }
    )

    if (result.status !== 0) {
        throw new Error(`Failed to convert .wasm to .walc: ${result.stderr}`)
    }

    return result.stdout
}

// Returns the STDOUT or throws an error
function runWalc(walcSource: Buffer): Buffer {
    // '-' makes the interpreter read the source from STDIN
    const runResult = spawnSync(
        'deno', ['-A', `${rootDir}/tools/lambda.ts`, '-'],
        { input: walcSource }
    )

    if (runResult.status !== 0) {
        throw new Error(`Failed to run .walc: ${runResult.stderr}`)
    }

    return runResult.stdout
}

// Returns the STDOUT or throws an error
function runWat(watSource: string): string {
    const wasmSource = convertWat2Wasm(watSource)
    const walcSource = convertWasm2Walc(wasmSource)
    return runWalc(walcSource).toString()
}

// Tests if the program prints the expected output.
// Prints a message on error.
function testPrints(
    module: TestBaseModule, mainBody: string, expected: string
) {
    try {
        const watSource = appendCode(module.source, mainFunc(mainBody))

        const output = runWat(watSource)

        if (output !== expected) {
            throw new Error(`Expected "${expected}", but got "${output}"`)
        }
    } catch (error) {
        if (!(error instanceof Error)) {
            throw error
        }

        console.error('')
        console.error(`[FAIL] ${module.filename}: ${error.message}`)
        console.error('The test source is: \n' + mainBody)
        console.error('--------------------------------')
    }
}

interface Export {
    // Exported name
    name: string,
    // Function identifier (a number or an identifier like "$foo")
    func: string
}

interface TestBaseModule {
    filename: string,
    source: string,
    exports: Map<string, string>
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
    const firstParenIndex = baseWatSource.indexOf('(module')

    if (firstParenIndex === -1) {
        throw new Error('Invalid .wat source: no module declaration found')
    }

    const insertIndex = firstParenIndex + '(module'.length
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

const walcImports = '(import "walc" "output" (func $walc-print (param i32)))'

function mainFunc(body: string): string {
    return '(export "main" (func $walcMain))'
        + `(func $walcMain ${body})`
}

function pushValue(value: TestValue): string {
    return `(${value.type}.const ${value.value})`
}

function performAction(module: TestBaseModule, action: TestAction): string {
    if (action.type !== 'invoke') {
        throw new Error(`Unsupported action type: ${action.type}`)
    }

    const func = module.exports.get(action.field)

    if (func === undefined) {
        throw new Error(`Function "${action.field}" not found in exports`)
    }

    const args = action.args.map(pushValue).map(a => ' ' + a).join('')

    return `(call ${func}${args})\n`
}

function testExpected(cmd: AssertReturnCommand): string {
    // The code prints 'N' on failure and 'Y' on success

    const N = 'N'.charCodeAt(0)
    const Y = 'Y'.charCodeAt(0)

    let code = ''
    for (const expected of cmd.expected) {
        const val = pushValue(expected)
        code += `(if (${expected.type}.eq ${val})\n`
            + '  (then nop)\n'
            + `  (else (call $walc-print (i32.const ${N})))) ;; print N\n`
    }

    code += `(call $walc-print (i32.const ${Y})) ;; print Y\n`

    return code
}

function testReturn(module: TestBaseModule, cmd: AssertReturnCommand) {
    const body = performAction(module, cmd.action) + testExpected(cmd)
    testPrints(module, body, 'Y')
}

function failInTheEnd() {
    // The code prints nothing on successful trap and 'N' on failure

    const N = 'N'.charCodeAt(0)
    return `(call $walc-print (i32.const ${N})) ;; print N\n`
}

function testTrap(module: TestBaseModule, cmd: AssertTrapCommand) {
    const body = performAction(module, cmd.action) + failInTheEnd()
    testPrints(module, body, '')
}

function readModule(cmd: ModuleCommand): TestBaseModule {
    const baseName = path.basename(cmd.filename, '.wasm')
    const moduleName = `${binDir}/${baseName}.wat`
    let source = fs.readFileSync(moduleName, 'utf-8')

    const exports = findExportedFunctions(source)

    source = prependCode(source, walcImports)

    return {
        filename: cmd.filename,
        source: source,
        exports: exports
    }
}

function run() {
    // Needed to run Cargo to run WALC
    process.chdir(rootDir)

    const testSetFiles = fs.readdirSync(binDir)
        .filter(file => file.endsWith('.json'))
        .map(file => `${binDir}/${file}`)

    for (const testSetFile of testSetFiles) {
        console.log(`Running tests from ${testSetFile}...`)

        const testSetSource = fs.readFileSync(testSetFile, 'utf-8')
        const testSet = JSON.parse(testSetSource) as TestSet

        let module = null as TestBaseModule | null

        for (const command of testSet.commands) {
            switch (command.type) {
                case 'module': {
                    const cmd = command as ModuleCommand
                    module = readModule(cmd)
                }
                break

                case 'assert_trap': {
                    const cmd = command as AssertTrapCommand

                    if (module === null) {
                        throw new Error(`bad test set: ${testSetFile}`)
                    }

                    testTrap(module, cmd)
                }
                break

                case 'assert_return': {
                    const cmd = command as AssertReturnCommand

                    if (module === null) {
                        throw new Error(`bad test set: ${testSetFile}`)
                    }

                    testReturn(module, cmd)
                }
                break
            }
        }
    }
}

function main() {
    setup()
    run()
    console.log('All tests completed')
}

main()
