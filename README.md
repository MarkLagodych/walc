# WebAssembly to Lambda Calculus compiler

WALC /wɑlts/ compiles stand-alone modules in [WebAssembly 1.0](https://w3.org/TR/wasm-core-1/)
([pdf](https://webassembly.github.io/spec/versions/core/WebAssembly-1.0.pdf))
into [untyped lambda expressions](https://en.wikipedia.org/wiki/Lambda_calculus).
It also supports [LIME1](https://github.com/WebAssembly/tool-conventions/blob/main/Lime.md)
WebAssembly extensions.

The input modules are only allowed to use custom [WALC functions](./docs/wasm.md)
for standard I/O, see [example programs](./examples/rust/README.md)
written in Rust.

The output lambda expressions are in [WALC format](./docs/format.md),
which defines how the expressions should be interpreted in order to perform I/O.
The format does not change anything about lambda calculus or how it is evaluated
inside the interpreter, it just defines what the interpreter does with
the evaluated result.
You can run some [example expressions](./examples/walc/) written by hand
with an [example interpreter](./examples/interpreter/).

## Project state

The compiler is almost done, the only things left are some arithmetic
instructions.

**Basic instructions:**

- [X] all control flow instructions
- [X] all variable instructions
- [X] all memory instructions

**LIME1 extensions:**

- [X] `multi-value`
- [X] `bulk-memory-opt`
- [X] `sign-extension-ops`
- [X] `extended-const`
- [X] `call-indirect-overlong`
- [ ] `nontrapping-float-to-int-conversions`

**Arithmetic:**

- [X] all bit operations
- [X] all comparison instructions
- [X] `add`, `sub`
- [X] `mul`
- [ ] `div`, `rem`
- [ ] floating-point arithmetic?

**Other:**

- [ ] extensive testing
- [ ] faster interpreter in Rust
- [ ] more WASM examples
- [ ] compile Doom to lambda calculus?

