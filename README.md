# qsdr

qsdr is an experimental work-in-progress high-performance graph-based SDR
runtime. The main difference between qsdr and other SDR runtimes such as
[GNU Radio 3.10](https://github.com/gnuradio/gnuradio),
[GNU Radio 4.0](https://github.com/fair-acc/gnuradio4/tree/main),
and [FutureSDR](https://github.com/FutureSDR/FutureSDR)
is that in qsdr data travels in packets called quanta through closed circuits in
the flowgraph, instead of traveling as a continuous stream of samples from
sources to sinks. Much of the development of qsdr is focused on high performance
in embedded aarch64 systems, such as the AMD RFSoC/MPSoC quad-core Cortex-A53
CPU complex, which is present in several AMD FPGA SoCs. Another focus of the
project experiments is benchmarking and comparisons with other SDR runtimes.

## Benchmark report

The report of the benchmarks contained in this repository is built with CI and
published to Github pages. It can be accessed at
[https://daniestevez.github.io/qsdr/](https://daniestevez.github.io/qsdr/)

## Repository structure

This repository is organized as follows:

- [benchmark-report](benchmark-report) is a
  [Sphinx](https://www.sphinx-doc.org/) project that generates an HTML report of
  the benchmarks by plotting figures of the data in `benchmark-results` using
  Matplotlib.
  
- [benchmark-results](benchmark-results) contains the raw results of all
  the benchmarks. It is generated by running
  [scripts/run_benchmarks_kria.sh](scripts/run_benchmarks_kria.sh).

- [futuresdr-benchmark](futuresdr-benchmark) is a Rust crate that contains
  a benchmark of [FutureSDR](https://github.com/FutureSDR/FutureSDR).

- [gr-qsdr_benchmark](gr-qsdr_benchmark) is a GNU Radio 3.10 out-of-tree module
  that contains a benchmark of [GNU Radio 3.10](https://github.com/gnuradio/gnuradio).

- [gr4-qsdr-benchmark](gr4-qsdr-benchmark) contains a benchmark of
  [GNU Radio 4.0](https://github.com/fair-acc/gnuradio4/tree/main).
  
- [qsdr-benchmarks](qsdr-benchmarks) is a Rust crate that contains
  benchmarks related to qsdr. It includes assembler instructions timing
  benchmarks as well as benchmarks of Rust channels, computational kernels and
  qsdr flowgraphs.

- [qsdr-macros](qsdr-macros) is a Rust crate that contains macros used in
  the `qsdr` crate.

- [qsdr](qsdr) is a Rust crate that contains the qsdr runtime.

- [scripts](scripts) contains miscellaneous scripts to help in development, such
  as building deploying and running the code on a
  [Kria KV260](https://www.amd.com/en/products/system-on-modules/kria/k26/kv260-vision-starter-kit.html) board.

## Building

All the Rust code is organized as a [Cargo
workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html), so it
can be built with `cargo build` from the repository root. Cross-compilation can
be done with [cross](https://github.com/cross-rs/cross). Running
```
cross build --release
```
will build all the Rust code for the Cortex-A53 (there are some compiler flags
specific to the Cortex-A53 in [.cargo/config.toml](.cargo/config.toml), and the
default `cross` target is set to `aarch64-unknown-linux-gnu`).

The GNU Radio 3.10 out-of-tree module [gr-qsdr_benchmark](gr-qsdr_benchmark) can
be built as any other out-of-tree module by using CMake. It is possible to build
it directly in an embedded system that has GNU Radio 3.10 and a toolchain
installed (for instance, in the
[Ubuntu image used on AMD development boards](https://ubuntu.com/download/amd)).

The GNU Radio 4.0 benchmark [gr4-qsdr-benchmark](gr4-qsdr-benchmark) is a CMake
project, but cross-compilation for aarch64 can be tricky due to compiler
requirements of GNU Radio 4.0. A Docker image
[gnuradio4-aarch64-build](https://github.com/daniestevez/gnuradio4-aarch64-build)
is provided to simplify this process. The script
[gr4-qsdr-benchmark/build-in-docker.sh](gr4-qsdr-benchmark/build-in-docker.sh)
can be used to build with this Docker image.
