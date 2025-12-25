# WebAssembly to Lambda Calculus compiler

WALC /wɑːlts/ compiles stand-alone
[WebAssembly 2.0](https://webassembly.org/) / [WASI 0.2](https://wasi.dev/)
modules into pure untyped lambda expressions.

The output lambda expressions are in [WALC format](./docs/format.md)
which defines how the expressions should be interpreted in order to perform
I/O.
The format does not change anything about lambda calculus or how it is evaluated
inside the interpreter, it just defines what the interpreter does with
the evaluated result.
You can find an example interpreter in [examples](./examples/interpreter/).
