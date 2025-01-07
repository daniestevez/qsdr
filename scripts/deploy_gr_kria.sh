#!/bin/bash

set -euox pipefail

rsync --delete -a --exclude build gr-qsdr_benchmark kria:
ssh -t kria 'bash -c "mkdir -p gr-qsdr_benchmark/build && cd gr-qsdr_benchmark/build && cmake .. && make -j$(nproc) && sudo make install"'
