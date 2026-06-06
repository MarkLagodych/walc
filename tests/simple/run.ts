#!/usr/bin/env -S deno --allow-read --allow-write --allow-env --allow-run

import fs from 'node:fs'
import process from 'node:process'
import { execSync } from 'node:child_process'

interface Test{
    name: string,
    expected_output: string,
}

function loadTests(): Test[] {
    const source = fs.readFileSync('tests.json', 'utf-8')
    return JSON.parse(source) as Test[]
}

function compileWatToWasm(tests: Test[], scriptDir: string, binDir: string) {
    process.chdir(scriptDir)

    const wat2wasm = 'wasm-tools parse'

    for (const test of tests) {
        const watPath = `${scriptDir}/${test.name}.wat`
        const wasmPath = `${binDir}/${test.name}.wasm`

        execSync(`${wat2wasm} ${watPath} -o ${wasmPath}`)
    }
}

function compileWasmToWalc(tests: Test[], rootDir: string, binDir: string) {
    process.chdir(rootDir)

    const walc = 'cargo run --quiet --features unbound-unreachable --'

    for (const test of tests) {
        const wasmPath = `${binDir}/${test.name}.wasm`
        const walcPath = `${binDir}/${test.name}.walc`

        execSync(`${walc} ${wasmPath} -o ${walcPath}`)
    }
}

function runTests(tests: Test[], rootDir: string, binDir: string) {
    const lambda = `${rootDir}/tools/lambda.ts`

    for (const test of tests) {
        console.log(`Running test ${test.name}...`)

        const walcPath = `${binDir}/${test.name}.walc`

        try {
            const output = execSync(`${lambda} ${walcPath}`).toString().trim()

            if (output !== test.expected_output) {
                throw new Error(
                    `expected "${test.expected_output}", got "${output}"`
                )
            }
        } catch (error) {
            console.error(`[FAIL] ${error}`)
        }
    }
}

function main() {
    const tests = loadTests()

    const scriptDir = fs.realpathSync(import.meta.dirname ?? '.')
    const rootDir = fs.realpathSync(`${scriptDir}/../..`)
    const binDir = `${scriptDir}/bin`

    if (!(fs.existsSync(binDir))) {
        fs.mkdirSync(binDir)
    }

    console.log('Compiling .wat files to .wasm...')
    compileWatToWasm(tests, scriptDir, binDir)
    console.log('Compiling .wasm files to .walc...')
    compileWasmToWalc(tests, rootDir, binDir)
    console.log('Running tests...')
    runTests(tests, rootDir, binDir)
    console.log('All tests completed')
}

main()
