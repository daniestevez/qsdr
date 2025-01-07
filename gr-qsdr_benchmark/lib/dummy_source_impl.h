/* -*- c++ -*- */
/*
 * Copyright 2024 Daniel Estevez <daniel@destevez.net>
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

#ifndef INCLUDED_QSDR_BENCHMARK_DUMMY_SOURCE_IMPL_H
#define INCLUDED_QSDR_BENCHMARK_DUMMY_SOURCE_IMPL_H

#include <gnuradio/qsdr_benchmark/dummy_source.h>

namespace gr {
namespace qsdr_benchmark {

class dummy_source_impl : public dummy_source
{
public:
    dummy_source_impl();
    ~dummy_source_impl() override;

    int work(int noutput_items,
             gr_vector_const_void_star& input_items,
             gr_vector_void_star& output_items) override;
};

} // namespace qsdr_benchmark
} // namespace gr

#endif /* INCLUDED_QSDR_BENCHMARK_DUMMY_SOURCE_IMPL_H */
