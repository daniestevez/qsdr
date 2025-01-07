/* -*- c++ -*- */
/*
 * Copyright 2024 Daniel Estevez <daniel@destevez.net>
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

#ifndef INCLUDED_QSDR_BENCHMARK_SAXPY_IMPL_H
#define INCLUDED_QSDR_BENCHMARK_SAXPY_IMPL_H

#include <gnuradio/qsdr_benchmark/saxpy.h>

namespace gr {
namespace qsdr_benchmark {

class saxpy_impl : public saxpy
{
private:
    const float d_a;
    const float d_b;

public:
    saxpy_impl(float a, float b);
    ~saxpy_impl() override;

    int work(int noutput_items,
             gr_vector_const_void_star& input_items,
             gr_vector_void_star& output_items) override;
};

} // namespace qsdr_benchmark
} // namespace gr

#endif /* INCLUDED_QSDR_BENCHMARK_SAXPY_IMPL_H */
