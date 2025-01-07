# qsdr-benchmarks

## Instruction timing benchmark

Prerequisites: https://github.com/jerinjacobk/armv8_pmu_cycle_counter_el0

Building:

```
cross build --release --tests
```

Running:

```
CPU="cortex-a53" FORCE_COLOR=1 ./instruction_benchmark-<...> --nocapture --test-threads=1 --color=always
```

# cargo asm

```
RUSTFLAGS="-C linker=aarch64-linux-gnu-gcc -C lto" cargo asm --target aarch64-unknown-linux-gnu -p qsdr-benchmarks --bin benchmark_qsdr --target-cpu cortex-a53 --color --rust benchmark_qsdr::main | less -R
```
