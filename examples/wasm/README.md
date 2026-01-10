# WASM examples in Rust

Build:

```sh
rustup target add wasm32-unknown-unknown
cargo install --path . --root . --profile release --target wasm32-unknown-unknown --no-track
```
