# Rust examples

Here are some example programs written in Rust that target WebAssembly and
WALC environment spcifically.

## Build

```sh
cargo install \
    --quiet \
    --path . \
    --root . \
    --no-track \
    --profile release \
    --target wasm32-unknown-unknown
```

This will install `.wasm` binaries into a `bin` directory inside the current
directory.

In case the `wasm32-unknown-unknown` target ever evolves beyond WASM 1.0 and
LIME1, use the `wasm32v1-none` target instead.
