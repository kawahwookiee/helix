#!/usr/bin/env bash
cargo install --profile opt --config 'build.rustflags="-C target-cpu=native"' --path helix-term --locked
cp target/release/hx ~/.cargo/bin/
