/* -*- c++ -*- */
/*
 * Copyright 2024 Daniel Estevez <daniel@destevez.net>
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

#include "benchmark_sink_impl.h"
#include <gnuradio/io_signature.h>
#include <iostream>

namespace gr {
namespace qsdr_benchmark {

benchmark_sink::sptr benchmark_sink::make()
{
    return gnuradio::make_block_sptr<benchmark_sink_impl>();
}


benchmark_sink_impl::benchmark_sink_impl()
    : gr::sync_block("benchmark_sink",
                     gr::io_signature::make(1, 1, sizeof(float)),
                     gr::io_signature::make(0, 0, 0))
{
}

benchmark_sink_impl::~benchmark_sink_impl() {}

bool benchmark_sink_impl::start()
{
    d_time = ClockType::now();
    return gr::sync_block::start();
}

int benchmark_sink_impl::work(int noutput_items,
                              gr_vector_const_void_star& input_items,
                              gr_vector_void_star& output_items)
{
    d_count += noutput_items;
    if (d_count >= k_measure_every) {
        const auto now = ClockType::now();
        const double elapsed = std::chrono::duration<double>(now - d_time).count();
        const double samples_per_sec = static_cast<double>(d_count) / elapsed;
        std::cout << "samples/s = " << samples_per_sec << "\n";
        d_count = 0;
        d_time = now;
    }

    return noutput_items;
}

} /* namespace qsdr_benchmark */
} /* namespace gr */
