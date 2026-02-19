# WebAssembly to Lambda Calculus compiler

WALC /wɑlts/ compiles stand-alone modules in
[WebAssembly 1.0](https://webassembly.org/)
(spec:
[HTML](w3.org/TR/wasm-core-1/),
[PDF](https://webassembly.github.io/multi-value/core/_download/WebAssembly.pdf))
into
[untyped lambda expressions](https://en.wikipedia.org/wiki/Lambda_calculus).

The input modules are only allowed to use custom
[WALC functions](./docs/wasm.md) for standard I/O,
see [example programs](./examples/rust) written in Rust.

The output lambda expressions are in [WALC format](./docs/format.md),
which defines how the expressions should be interpreted in order to perform I/O.
The format does not change anything about lambda calculus or how it is evaluated
inside the interpreter, it just defines what the interpreter does with
the evaluated result.
You can run some [example expressions](./examples/walc/) written
by hand with an [example interpreter](./examples/interpreter/).
