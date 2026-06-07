# Specification tests

This script runs the tests provided in the WebAssembly specification repository.

The tester skips all tests that mention floats (`float*`, `f32*`, `f64*`)
and that are not valid WASM 1.0 / LIME1 (i.e. WASM 2.0 and newer).

Each `.wast` file is split into several stand-alone `.wasm` files that
should either succeed or fail.

The tester creates some testing files in a local `bin` directory and then uses
them for running. The testing files are only created once, so in order to
recreate them, just do `rm -rf bin`.

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

## Run

```sh
./run.ts
```

or:

```sh
deno -A run.ts
```

You can also specify the regex of the JSON files that the tester should use:

```sh
./run.ts '.*exports.json'
```

## Cleanup

```sh
rm -rf ./bin/
```
