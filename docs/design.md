# WALC design

This document describes how WebAssembly is translated into lambda calculus.

TODO

For now, the best place to learn about this is the source:
[1](../src/codegen/core/instruction.rs),
[2](../src/codegen/core/code.rs),
[3](../src/codegen/runtime/).

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
