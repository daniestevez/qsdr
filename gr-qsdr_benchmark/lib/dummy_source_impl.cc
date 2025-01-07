/* -*- c++ -*- */
/*
 * Copyright 2024 Daniel Estevez <daniel@destevez.net>
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

#include "dummy_source_impl.h"
#include <gnuradio/io_signature.h>

namespace gr {
namespace qsdr_benchmark {

dummy_source::sptr dummy_source::make()
{
    return gnuradio::make_block_sptr<dummy_source_impl>();
}

dummy_source_impl::dummy_source_impl()
    : gr::sync_block("dummy_source",
                     gr::io_signature::make(0, 0, 0),
                     gr::io_signature::make(1, 1, sizeof(float)))
{
}

dummy_source_impl::~dummy_source_impl() {}

int dummy_source_impl::work(int noutput_items,
                            gr_vector_const_void_star& input_items,
                            gr_vector_void_star& output_items)
{
    return noutput_items;
}

} /* namespace qsdr_benchmark */
} /* namespace gr */
