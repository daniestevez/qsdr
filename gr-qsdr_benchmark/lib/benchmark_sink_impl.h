/* -*- c++ -*- */
/*
 * Copyright 2024 Daniel Estevez <daniel@destevez.net>
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

#ifndef INCLUDED_QSDR_BENCHMARK_BENCHMARK_SINK_IMPL_H
#define INCLUDED_QSDR_BENCHMARK_BENCHMARK_SINK_IMPL_H

#include <gnuradio/qsdr_benchmark/benchmark_sink.h>
#include <cstdint>

namespace gr {
namespace qsdr_benchmark {

class benchmark_sink_impl : public benchmark_sink
{
private:
    using ClockType = std::chrono::steady_clock;
    uint64_t d_count{ 0 };
    std::chrono::time_point<ClockType> d_time;
    static constexpr uint64_t k_measure_every = 1UL << 27;

public:
    benchmark_sink_impl();
    ~benchmark_sink_impl() override;

    bool start() override;

    int work(int noutput_items,
             gr_vector_const_void_star& input_items,
             gr_vector_void_star& output_items) override;
};

} // namespace qsdr_benchmark
} // namespace gr

#endif /* INCLUDED_QSDR_BENCHMARK_BENCHMARK_SINK_IMPL_H */
