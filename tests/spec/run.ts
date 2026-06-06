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
                '--wasm-dir', binDir,
                '-o', `${binDir}/${baseName}.json`
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
                '--features', 'wasm1,lime1',
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

        const result = spawnSync(
            'wasm-tools',
            [
                'print',
                `${binDir}/${wasmFile}`,
                '-o', `${binDir}/${baseName}.wat`
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

function run() {

}

function main() {
    setup()
    run()
    console.log('All tests completed')
}

main()
