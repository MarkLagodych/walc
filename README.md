# WebAssembly to Lambda Calculus compiler

WALC /wɑːlts/ compiles stand-alone
[WebAssembly 1.0](https://webassembly.org/) / [WASI 0.1](https://wasi.dev/)
modules into pure untyped lambda expressions.

The output lambda expressions are in [WALC format](./docs/format.md)
which defines how the expressions should be interpreted in order to perform
I/O.
The format does not change anything about lambda calculus or how it is evaluated
inside the interpreter, it just defines what the interpreter does with
the evaluated result.
You can run some [example programs](./examples/lambda-calculus/) written
in lambda calculus by hand with an
[example interpreter](./examples/interpreter/).
