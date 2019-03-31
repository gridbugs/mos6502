#!/bin/bash
set -euo pipefail
cd "$( dirname "${BASH_SOURCE[0]}" )"
cargo run --manifest-path=nes-samples/Cargo.toml --example=$1
