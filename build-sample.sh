#!/usr/bin/env bash
set -euo pipefail
cd "$( dirname "${BASH_SOURCE[0]}" )"
cargo build --manifest-path=nes-samples/Cargo.toml
cargo run --manifest-path=nes-samples/Cargo.toml --example=$1
