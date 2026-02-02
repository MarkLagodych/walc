#!/usr/bin/env sh

cargo install --path . --root . --profile release --target wasm32-unknown-unknown --no-track
