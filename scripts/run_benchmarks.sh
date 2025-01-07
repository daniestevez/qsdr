#!/bin/bash

set -euox pipefail

BENCH_DURATION=30

# Prepare output directory
OUT_DIR=/tmp/benchmark-results
rm -rf $OUT_DIR
mkdir $OUT_DIR

# benchmark_channel

BENCH_OUT_DIR=${OUT_DIR}/benchmark_channel
mkdir -p $BENCH_OUT_DIR
for chan in async-channel-bounded crossbeam-bounded flume-bounded flume-bounded-async kanal-bounded kanal-bounded-async qsdr-spsc-futex qsdr-mpsc-futex qsdr-spsc-futures qsdr-mpsc-futures std-mpsc tokio
do
    (timeout $BENCH_DURATION ~/benchmark_channel --channel $chan || true) | tee ${BENCH_OUT_DIR}/${chan}.txt
done

# benchmark_saxpy

BENCH_OUT_DIR=${OUT_DIR}/benchmark_saxpy
mkdir -p $BENCH_OUT_DIR

~/benchmark_saxpy scan-buffer-size | tee ${BENCH_OUT_DIR}/scan-buffer-size.txt

## single-core
BENCH_OUT_DIR=${OUT_DIR}/benchmark_saxpy/single-core
mkdir -p $BENCH_OUT_DIR

for total_size in 4096 8192 16384 32768 65536
do
    for num_buffers in 1 2 4 8 16
    do
        buffer_size=$(($total_size / $num_buffers))
        (timeout $BENCH_DURATION \
                 ~/benchmark_saxpy single-core --buffer-size $buffer_size --num-buffers $num_buffers \
                 || true) \
            | tee ${BENCH_OUT_DIR}/${buffer_size}_${num_buffers}.txt
    done
done

## multi-core
for testname in multi-core multi-core-async multi-core-fake-message-passing multi-core-fake-message-passing-async
do
    BENCH_OUT_DIR=${OUT_DIR}/benchmark_saxpy/${testname}
    mkdir -p $BENCH_OUT_DIR

    for num_cpus in {2..4}
    do
        for num_buffers in $(seq $num_cpus $(($num_cpus * 3)))
        do
            if [[ $num_cpus -eq 4 ]]; then
                buffer_sizes="1024 2048 4096 8192 16384 32768 65536"
            else
                buffer_sizes="4096 8192 16384 32768 65536 131072 262144 524288"
            fi
            for buffer_size in $buffer_sizes
            do
                (timeout $BENCH_DURATION \
                         ~/benchmark_saxpy $testname \
                         --num-cpus $num_cpus --buffer-size $buffer_size --num-buffers $num_buffers \
                     || true) \
                    | tee ${BENCH_OUT_DIR}/${num_cpus}_${buffer_size}_${num_buffers}.txt
            done
        done
    done
done

## multi-kernel
for testname in multi-kernel multi-kernel-async
do
    BENCH_OUT_DIR=${OUT_DIR}/benchmark_saxpy/${testname}
    mkdir -p $BENCH_OUT_DIR

    for num_cpus in {2..4}
    do
        for num_kernels in $(seq $num_cpus $(($num_cpus * 3)))
        do
            for num_buffers in $(seq $num_cpus $(($num_cpus + 3)))
            do
                if [[ $num_cpus -eq 4 ]]; then
                    buffer_sizes="1024 2048 4096 8192 16384 32768 65536"
                else
                    buffer_sizes="4096 8192 16384 32768 65536 131072 262144 524288"
                fi
                for buffer_size in $buffer_sizes
                do
                    (timeout $BENCH_DURATION \
                             ~/benchmark_saxpy $testname \
                             --num-cpus $num_cpus --num-kernels $num_kernels \
                             --buffer-size $buffer_size --num-buffers $num_buffers \
                         || true) \
                        | tee ${BENCH_OUT_DIR}/${num_cpus}_${num_kernels}_${buffer_size}_${num_buffers}.txt
                done
            done
        done
    done
done

# qsdr

BENCH_OUT_DIR=${OUT_DIR}/benchmark_qsdr
mkdir -p $BENCH_OUT_DIR
(timeout $BENCH_DURATION ~/benchmark_qsdr single-core || true) \
    | tee ${BENCH_OUT_DIR}/single-core.txt

for testname in multi-kernel multi-kernel-tokio multi-kernel-async-executor
do
    BENCH_OUT_DIR=${OUT_DIR}/benchmark_qsdr/${testname}
    mkdir -p $BENCH_OUT_DIR

    for num_cpus in {1..4}
    do
        for num_kernels in $(seq $num_cpus $(($num_cpus * 3)))
        do
            num_buffers=$(($num_cpus + 1))
            (timeout $BENCH_DURATION ~/benchmark_qsdr --buffer-size 16384 --num-buffers $num_buffers \
                     $testname --num-kernels $num_kernels --num-cpus $num_cpus \
                 || true) \
                | tee ${BENCH_OUT_DIR}/${num_cpus}_${num_kernels}.txt
        done
    done
done

# gr4

BENCH_OUT_DIR=${OUT_DIR}/gr4-qsdr-benchmark
mkdir -p $BENCH_OUT_DIR
(timeout $BENCH_DURATION stdbuf -o L ~/gr4-qsdr-benchmark single-core || true) \
    | tee ${BENCH_OUT_DIR}/single-core.txt

for testname in multi-kernel multi-kernel-simple
do
    BENCH_OUT_DIR=${OUT_DIR}/gr4-qsdr-benchmark/${testname}
    mkdir -p $BENCH_OUT_DIR

    for num_cpus in {1..4}
    do
        for num_kernels in $(seq $num_cpus $(($num_cpus * 3)))
        do
            (timeout $BENCH_DURATION stdbuf -o L \
                     ~/gr4-qsdr-benchmark $testname $num_kernels $num_cpus \
                 || true) \
                | tee ${BENCH_OUT_DIR}/${num_cpus}_${num_kernels}.txt
        done
    done
done

# futuresdr

BENCH_OUT_DIR=${OUT_DIR}/futuresdr-benchmark
mkdir -p $BENCH_OUT_DIR
(timeout $BENCH_DURATION ~/futuresdr-benchmark single-core || true) \
    | tee ${BENCH_OUT_DIR}/single-core.txt

for testname in multi-kernel multi-kernel-smol
do
    BENCH_OUT_DIR=${OUT_DIR}/futuresdr-benchmark/${testname}
    mkdir -p $BENCH_OUT_DIR

    for num_cpus in {1..4}
    do
        for num_kernels in $(seq $num_cpus $(($num_cpus * 3)))
        do
            (timeout $BENCH_DURATION \
                     ~/futuresdr-benchmark $testname --num-cpus $num_cpus --num-kernels $num_kernels \
                 || true) \
                | tee ${BENCH_OUT_DIR}/${num_cpus}_${num_kernels}.txt
        done
    done
done

# gr3

for testname in affinities no-affinities
do
    if [ "$testname" = "no-affinities" ]; then
        aff="--no-cpu-affinities"
    else
        aff=""
    fi
    BENCH_OUT_DIR=${OUT_DIR}/gr_qsdr_benchmark/${testname}
    mkdir -p $BENCH_OUT_DIR

    for num_cpus in {1..4}
    do
        for num_kernels in $(seq $num_cpus $(($num_cpus * 3)))
        do
            (timeout $BENCH_DURATION stdbuf -o L \
                     ~/gr-qsdr_benchmark/examples/gr_qsdr_benchmark.py \
                     $aff --num-cpus $num_cpus --num-kernels $num_kernels \
                 || true) \
                | tee ${BENCH_OUT_DIR}/${num_cpus}_${num_kernels}.txt
        done
    done
done
