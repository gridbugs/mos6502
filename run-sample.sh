#!/usr/bin/env bash
set -euo pipefail
cd "$( dirname "${BASH_SOURCE[0]}" )"
cargo build --manifest-path=nes-emulator/Cargo.toml
./build-sample.sh $1 | cargo run --manifest-path=nes-emulator/Cargo.toml -- --headless-num-frames 1000
