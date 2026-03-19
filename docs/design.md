# WALC design

The best way to learn how WebAssembly is translated to lambda calculus is the
source, all essential algorithms are documented and all output code is generated
with human-readable Rust code.

Lambda expressions are most often generated directly, without any intermediate
representation. Some data or control flow structures require unintuitive
shenanigans, e.g. `let VAR1 = VAL1 in let VAR2 = VAL2 in BODY` requires
generating the code in reverse: first the `let VAR2 = VAL2 in BODY` statement
and then `let VAR1 = VAL1 in ...`. In those cases,
[builder](https://refactoring.guru/design-patterns/builder/rust/example)
objects are used.

It is important to keep the resulting code as small as possible because after
all, no debugging is available other than just reading the code yourself.
This is exactly why square brackets where chosen instead of lambdas or other
characters -- it is just easier to go through the code and manually insert
spaces and line breaks where needed.

WALC extracts all numbers and all operation definitions into global variables,
so that only several definitions are used thoughout the whole file.
That is achieved by using generator objects -- objects that accumulate all
the required definitions and then, given a `let..in` expression builder,
add definitions to it. This also helps insure that the definitions have the
correct order. Numbers are ordered by values because in such a way it is easier
to see what constants the program uses and to check if all constants are
generated correctly. Maths and simple WASM instructions are generated in their
dependency order.

You can see how instructions work [here](../src/codegen/core/instruction.rs),
how they are joined into instruction chains [here](../src/codegen/core/code.rs),
and how all the algorithms are implemented [here](../src/codegen/runtime/).

## Supported extensions

This compiler supports `WASM 1.0` (the WWW standard released in 2019) with
[`LIME1`](https://github.com/WebAssembly/tool-conventions/blob/main/Lime.md)
extensions, here's some notes about them:
* `multi-value`: support for multiple return values in blocks and functions.
    This was really easy to implement because WASM VM is a just stack machine.
    Not supporting this extension would rather be an unreasonable limitation.
* `sign-extension-ops`: support for `iNN.extendMM_s` sign-extension
    instructions. These operations were implemented in the WASM VM anyway
    because they are used by the `iNN.loadMM_s` instructions, which are in the
    core spec. This extension is simply handy and also allows for better
    testing.
* `bulk-memory-opt`: a subset of `bulk-memory-operations` that just defines the
    `memory.init` and `memory.fill` instructions.
    These instructions might be interesting for benchmarking as they are
    the fastest way to initialize/copy memory chunks.
* `extended-const`: support for `add`, `sub` and `mul` in initializers for
    globals and data segment offsets. The expressions are evaluated completely
    at compile time, so no additional code is generated for them.
* `call-indirect-overlong`:
    This just enables the wasmparser's support for a slightly different encoding
    of the `call_indirect` instruction that was introduced in the
    `reference-types` extension and later in WASM 2.0.
* `nontrapping-float-to-int-conversions`:
    Not supported. Floating-point arithmetic is not supported at all.
