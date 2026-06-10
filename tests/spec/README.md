# Specification tests

This script runs the tests provided in the WebAssembly specification repository.

The list of used tests is in [`tests.txt`](./tests.txt).
It contains all tests relevant to WASM 1.0 and LIME1,
except for the float tests (`float*.wast`, `f32*.wast`, `f64*.wast`)
and the tests that require linking multiple WASM modules
(`export.wast`, `linking.wast`).

Each `.wast` file is compiled into several stand-alone `.wasm` files
by the `wast2wasm.ts` script in a `bin` directory.
The files are then run by the `run.ts` script.

## Prerequisites

You need the following programs installed and available in your PATH:
- [Deno](https://deno.com/)
- [Cargo](https://rust-lang.org/)
- [wasm-tools](https://github.com/bytecodealliance/wasm-tools)
  (can be installed with `cargo install wasm-tools`)

## Setup

Fetch the submodules:

```sh
git submodule update --init
```

Compile the test scripts:
```sh
./wast2wasm.ts
```
or:
```sh
deno -A ./wast2wasm.ts
```

## Run

```sh
./run.ts
```
or:
```sh
deno -A run.ts
```

## Cleanup

```sh
rm -rf ./bin/
```
