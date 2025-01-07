#!/usr/bin/env python3
# -*- coding: utf-8 -*-
#
# Copyright 2024 Daniel Estevez <daniel@destevez.net>
#
# SPDX-License-Identifier: MIT OR Apache-2.0
#

from gnuradio import gr, qsdr_benchmark

import argparse
import random
import signal
import sys


class fg(gr.top_block):
    def __init__(self, args):
        gr.top_block.__init__(self, "gr-qsdr_benchmark benchmark",
                              catch_exceptions=True)

        def set_affinity(block, cpu):
            if args.no_cpu_affinities:
                aff = range(args.num_cpus)
            else:
                aff = [cpu]
            block.set_processor_affinity(aff)

        self.blocks = []

        source = qsdr_benchmark.dummy_source()
        set_affinity(source, 0)
        self.blocks.append(source)

        kernels_per_core = [
            args.num_kernels // args.num_cpus
            + (1 if n < args.num_kernels % args.num_cpus else 0)
            for n in range(args.num_cpus)
        ]

        core = 0
        kernels_in_core = 0
        for _ in range(args.num_kernels):
            if kernels_in_core == kernels_per_core[core]:
                core += 1
                kernels_in_core = 0

            saxpy = qsdr_benchmark.saxpy(
                random.random(), random.random())
            set_affinity(saxpy, core)
            self.blocks.append(saxpy)
            kernels_in_core += 1

        sink = qsdr_benchmark.benchmark_sink()
        set_affinity(sink, args.num_cpus - 1)
        self.blocks.append(sink)

        self.connect(*self.blocks)


def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        '--num-kernels', type=int, default=4,
        help='Number of Saxpy kernels [default=%(default)r]')
    parser.add_argument(
        '--num-cpus', type=int, default=4,
        help='Number of CPU cores [default=%(default)r]')
    parser.add_argument(
        '--no-cpu-affinities', action='store_true',
        help='Use CPU affinities just to limit the number of CPUs')
    return parser.parse_args()


def main():
    args = parse_args()
    tb = fg(args)

    def sig_handler(sig=None, frame=None):
        tb.stop()
        tb.wait()
        sys.exit(0)

    signal.signal(signal.SIGINT, sig_handler)
    signal.signal(signal.SIGTERM, sig_handler)
    tb.run()


if __name__ == '__main__':
    main()
