#!/bin/bash
set -euxo pipefail
ROM=$1
NUM_FRAMES=$2
EXPECTED_HASH=$3
ACTUAL_HASH=$(cargo run --manifest-path=nes-emulator/Cargo.toml -- --rom-file $ROM --headless-num-frames $NUM_FRAMES)
test "$ACTUAL_HASH" == "$EXPECTED_HASH"
