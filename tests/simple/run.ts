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

        let result = spawnSync(
            'wasm-tools', ['parse', `${watPath}`, '-o', `${wasmPath}`]
        )

        if (result.status !== 0) {
            console.error(`Failed to compile ${watPath} to ${wasmPath}`)
            console.error(result.stderr.toString())
        }
    }
}

function compileWasmToWalc(tests: Test[], rootDir: string, binDir: string) {
    process.chdir(rootDir)

    for (const test of tests) {
        const wasmPath = `${binDir}/${test.name}.wasm`
        const walcPath = `${binDir}/${test.name}.walc`

        const result = spawnSync(
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

        if (result.status !== 0) {
            console.error(`Failed to compile ${wasmPath} to ${walcPath}`)
            console.error(result.stderr.toString())
        }
    }
}

function runTests(tests: Test[], rootDir: string, binDir: string) {
    const lambda = `${rootDir}/tools/lambda.ts`

    for (const test of tests) {
        console.log(`Running test ${test.name}...`)

        const walcPath = `${binDir}/${test.name}.walc`

        try {
            const result = spawnSync(
                'deno', ['-A', lambda, walcPath],
                { encoding: 'utf-8' }
            )

            if (result.status !== 0) {
                throw new Error(
                    `Failed to run ${walcPath}\n${result.stderr.toString()}`
                )
            }

            if (result.stdout !== test.expected_output) {
                throw new Error(
                    `expected "${test.expected_output}", got "${result.stdout}"`
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
