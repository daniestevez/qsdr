#!/bin/bash

set -euox pipefail

scp scripts/run_benchmarks.sh kria:/tmp/
ssh -t kria '/tmp/run_benchmarks.sh'
rsync --delete -av kria:/tmp/benchmark-results .
