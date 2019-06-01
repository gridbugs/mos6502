#!/bin/bash
set -euo pipefail
cd "$( dirname "${BASH_SOURCE[0]}" )"
cargo build --manifest-path=nes_emulator/Cargo.toml
./build_sample.sh $1 | cargo run --manifest-path=nes_emulator/Cargo.toml
