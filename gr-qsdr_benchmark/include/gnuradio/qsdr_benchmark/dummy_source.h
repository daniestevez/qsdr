/* -*- c++ -*- */
/*
 * Copyright 2024 Daniel Estevez <daniel@destevez.net>
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

#ifndef INCLUDED_QSDR_BENCHMARK_DUMMY_SOURCE_H
#define INCLUDED_QSDR_BENCHMARK_DUMMY_SOURCE_H

#include <gnuradio/qsdr_benchmark/api.h>
#include <gnuradio/sync_block.h>

namespace gr {
namespace qsdr_benchmark {

class QSDR_BENCHMARK_API dummy_source : virtual public gr::sync_block
{
public:
    typedef std::shared_ptr<dummy_source> sptr;

    static sptr make();
};

} // namespace qsdr_benchmark
} // namespace gr

#endif /* INCLUDED_QSDR_BENCHMARK_DUMMY_SOURCE_H */
