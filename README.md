# WebAssembly to Lambda Calculus compiler

WALC /wɑːlts/ compiles stand-alone [WebAssembly 3](https://webassembly.org/)
modules into pure untyped lambda expressions.

The input modules are only allowed to use custom WALC functions for standard
I/O. Advanced features like SIMD, 64-bit addresses, garbage collection, or WASM
components are unsupported, however most simple programs do not need them
anyway. See [example programs](./examples/wasm) written in Rust.

The output lambda expressions are in [WALC format](./docs/format.md)
which defines how the expressions should be interpreted in order to perform I/O.
The format does not change anything about lambda calculus or how it is evaluated
inside the interpreter, it just defines what the interpreter does with
the evaluated result.
You can run some [example programs](./examples/lambda-calculus/) written
in lambda calculus by hand with an
[example interpreter](./examples/interpreter/).
