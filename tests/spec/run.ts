#!/usr/bin/env -S deno --allow-read --allow-write --allow-env --allow-run

import fs from 'node:fs'
import path from 'node:path'
import process from 'node:process'
import { spawnSync } from 'node:child_process'

const scriptDir = import.meta.dirname ?? '.'
const binDir = `${scriptDir}/bin`
const rootDir = `${scriptDir}/../..`

const wasmFiles = fs.readdirSync(binDir)
    .filter(file => file.endsWith('.wasm'))
    .map(file => `${binDir}/${file}`)

function main() {
    console.log('Compiling .wasm files to .walc...')
    compileWasmToWalc()
    console.log('Running tests...')
    runTests()
    console.log('All tests completed')
}

function compileWasmToWalc() {
    process.chdir(rootDir)

    for (const wasmFile of wasmFiles) {
        const baseName = path.basename(wasmFile, '.wasm')
        const walcFile = `${binDir}/${baseName}.walc`

        const result = spawnSync(
            'cargo',
            [
                'run',
                '--quiet',
                '--features',
                'unbound-unreachable',
                '--',
                wasmFile,
                '-o',
                walcFile
            ]
        )

        if (result.status !== 0) {
            throw new Error(
                `Failed to compile ${wasmFile} to ${walcFile}: `
                + result.stderr.toString()
            )
        }
    }
}

function runTests() {
    const lambda = `${rootDir}/tools/lambda.ts`

    for (const wasmFile of wasmFiles) {
        console.log(`Running test ${wasmFile}...`)

        const baseName = path.basename(wasmFile, '.wasm')
        const walcFile = `${binDir}/${baseName}.walc`

        try {
            const result = spawnSync(
                'deno', ['-A', lambda, walcFile],
                { encoding: 'utf-8' }
            )

            if (result.status !== 0) {
                throw new Error(
                    `Failed to run ${walcFile}\n${result.stderr.toString()}`
                )
            }

            if (result.stdout !== "OK") {
                throw new Error(
                    `expected "OK", got "${result.stdout}"`
                )
            }
        } catch (error) {
            console.error(`[FAIL] ${error}`)
        }
    }
}



main()
