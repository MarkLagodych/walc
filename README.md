# WebAssembly to Lambda Calculus compiler

WALC /wɑlts/ compiles stand-alone modules in [WebAssembly 1.0](https://w3.org/TR/wasm-core-1/)
into [untyped lambda expressions](https://en.wikipedia.org/wiki/Lambda_calculus).

The compiler naturally supports the [`multi-value`](https://webassembly.github.io/multi-value/core/_download/WebAssembly.pdf)
WebAssembly extension.
The input modules are only allowed to use custom [WALC functions](./docs/wasm.md)
for standard I/O, see [example programs](./examples/rust) written in Rust.

The output lambda expressions are in [WALC format](./docs/format.md),
which defines how the expressions should be interpreted in order to perform I/O.
The format does not change anything about lambda calculus or how it is evaluated
inside the interpreter, it just defines what the interpreter does with
the evaluated result.
You can run some [example expressions](./examples/walc/) written by hand
with an [example interpreter](./examples/interpreter/).

## Project state

Most of the compiler's modules are already finished and will not change much.
The only things left are arithmetic instructions.

### TODO

- [X] all control flow instructions
- [X] all variable instructions
- [X] all integer comparison instructions
- [X] `and`, `or`, `xor`
- [X] `add`, `sub`
- [ ] integer conversions (between 8/16/32/64 bits, including sign extensions)
- [ ] fix input and output by using integer conversions from/to bytes
- [ ] `load` with byte conversions and `store`
- [ ] data segment initialization
- [ ] `mul`
- [ ] `div`, `mod`
- [ ] bit shifts and rotations
- [ ] `ctz`, `clz`, `popcnt`
- [ ] floating-point numbers and arithmetic
- [ ] extensive testing

Also, a faster interpreter in Rust is planned, as well as more examples
once more WASM features become supported.
