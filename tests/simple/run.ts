#!/usr/bin/env -S deno --allow-read --allow-write --allow-env --allow-run

import fs from 'node:fs'
import process from 'node:process'
import { spawnSync } from 'node:child_process'

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

    for (const test of tests) {
        const watPath = `${scriptDir}/${test.name}.wat`
        const wasmPath = `${binDir}/${test.name}.wasm`

        spawnSync('wasm-tools', ['parse', `${watPath}`, '-o', `${wasmPath}`])
    }
}

function compileWasmToWalc(tests: Test[], rootDir: string, binDir: string) {
    process.chdir(rootDir)

    for (const test of tests) {
        const wasmPath = `${binDir}/${test.name}.wasm`
        const walcPath = `${binDir}/${test.name}.walc`

        spawnSync(
            'cargo',
            [
                'run',
                '--quiet',
                '--features',
                'unbound-unreachable',
                '--',
                wasmPath,
                '-o',
                walcPath
            ]
        )
    }
}

function runTests(tests: Test[], rootDir: string, binDir: string) {
    const lambda = `${rootDir}/tools/lambda.ts`

    for (const test of tests) {
        console.log(`Running test ${test.name}...`)

        const walcPath = `${binDir}/${test.name}.walc`

        try {
            const output = spawnSync(
                'deno', ['-A', lambda, walcPath],
                { encoding: 'utf-8' }
            ).stdout

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
