#!/bin/bash
set -euo pipefail
cd "$( dirname "${BASH_SOURCE[0]}" )"
cargo build --manifest-path=nes_samples/Cargo.toml
cargo run --manifest-path=nes_samples/Cargo.toml --example=$1
