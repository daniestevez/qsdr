#!/bin/bash

set -euox pipefail

gr4-qsdr-benchmark/build-in-docker.sh
scp gr4-qsdr-benchmark/build/gr4-qsdr-benchmark  kria:
