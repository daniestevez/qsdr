#!/bin/bash

set -euox pipefail

cross build --release --bins --tests
scp target/aarch64-unknown-linux-gnu/release/deps/instruction_benchmark-* \
    target/aarch64-unknown-linux-gnu/release/benchmark_* \
    target/aarch64-unknown-linux-gnu/release/futuresdr-benchmark \
    kria:
