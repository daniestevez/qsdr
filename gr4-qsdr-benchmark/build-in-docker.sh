#!/bin/bash

dir=$( dirname -- "$( readlink -f -- "$0"; )"; )
docker run -t --rm --mount type=bind,source=${dir},destination=/qsdr \
       ghcr.io/daniestevez/gnuradio4-aarch64-build:latest bash -c \
       "mkdir -p /qsdr/build && cd /qsdr/build && cmake -DCMAKE_TOOLCHAIN_FILE=/toolchain.cmake .. && make -j4"
