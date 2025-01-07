/* -*- c++ -*- */
/*
 * Copyright 2024 Daniel Estevez <daniel@destevez.net>
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

#ifndef INCLUDED_QSDR_BENCHMARK_BENCHMARK_SINK_H
#define INCLUDED_QSDR_BENCHMARK_BENCHMARK_SINK_H

#include <gnuradio/qsdr_benchmark/api.h>
#include <gnuradio/sync_block.h>

namespace gr {
namespace qsdr_benchmark {

class QSDR_BENCHMARK_API benchmark_sink : virtual public gr::sync_block
{
public:
    typedef std::shared_ptr<benchmark_sink> sptr;

    static sptr make();
};

} // namespace qsdr_benchmark
} // namespace gr

#endif /* INCLUDED_QSDR_BENCHMARK_BENCHMARK_SINK_H */
