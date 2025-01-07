/* -*- c++ -*- */
/*
 * Copyright 2024 Daniel Estevez <daniel@destevez.net>
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

#include "saxpy_impl.h"
#include <gnuradio/io_signature.h>
#include <cstdint>
#include <exception>

// This #ifdef works with gcc and clang, but not with other compilers
#ifdef __aarch64__
#include <arm_neon.h>
#endif

namespace gr {
namespace qsdr_benchmark {

saxpy::sptr saxpy::make(float a, float b)
{
    return gnuradio::make_block_sptr<saxpy_impl>(a, b);
}

saxpy_impl::saxpy_impl(float a, float b)
    : gr::sync_block("saxpy",
                     gr::io_signature::make(1, 1, sizeof(float)),
                     gr::io_signature::make(1, 1, sizeof(float))),
      d_a(a),
      d_b(b)
{
    // This #ifdef works with gcc and clang, but not with other compilers
#ifdef __aarch64__
    // The neon kernel requires a multiple of 32 floats and at least 64 floats.
    // Since set_min_noutput_items(64) doesn't seem to do anything, we use
    // set_output_multiple(64) instead of set_output_multiple(32) to ensure that
    // we're always given at least 64 items.
    set_output_multiple(64);
#endif
}

saxpy_impl::~saxpy_impl() {}

int saxpy_impl::work(int noutput_items,
                     gr_vector_const_void_star& input_items,
                     gr_vector_void_star& output_items)
{
    auto in = static_cast<const float*>(input_items[0]);
    auto out = static_cast<float*>(output_items[0]);

    // This #ifdef works with gcc and clang, but not with other compilers
#ifdef __aarch64__
    static constexpr int floats_per_iter = 32;
    const int iterations = noutput_items / floats_per_iter - 1;
    const float* buff_in0 = &in[0];
    const float* buff_in1 = &in[floats_per_iter / 2];
    const float* buff_in0_end = &in[floats_per_iter * iterations];
    float* buff_out = &out[0];
    uint64_t scratch0;
    uint64_t scratch1;
    uint64_t scratch2;
    uint64_t scratch3;
    uint64_t scratch4;
    // manually allocate these to v31 and v30 because gcc doesn't understand
    // that v0-v7 are overwritten and cannot be used to hold these values
    register float s31 __asm__("s31") = d_a;
    register float32x4_t v30 __asm__("v30") = vdupq_n_f32(d_b);
    __asm__ volatile("ld1 {v4.4s-v7.4s}, [%[buff_in0]]\n\t"
                     "fmul v4.4s, v4.4s, %[vA].s[0]\n\t"
                     "prfm PLDL1KEEP, [%[buff_in0], #128]\n\t"
                     "fmul v5.4s, v5.4s, %[vA].s[0]\n\t"
                     "ldr %[scratch0], [%[buff_in0], #72]\n\t"
                     "fmul v6.4s, v6.4s, %[vA].s[0]\n\t"
                     "ldr %[scratch1], [%[buff_in0], #88]\n\t"
                     "fmul v7.4s, v7.4s, %[vA].s[0]\n\t"
                     "ldr %[scratch2], [%[buff_in0], #104]\n\t"
                     "fadd v4.4s, v4.4s, %[vB].4s\n\t"
                     "ldr %[scratch3], [%[buff_in0], #120]\n\t"
                     "fadd v5.4s, v5.4s, %[vB].4s\n\t"
                     "ldr %[scratch4], [%[buff_in0], #112]\n\t"
                     "fadd v6.4s, v6.4s, %[vB].4s\n\t"
                     "prfm PLDL1KEEP, [%[buff_in1], #128]\n\t"
                     "ldr d0, [%[buff_in0], #64]\n\t"
                     "ins v3.d[1], %[scratch3]\n\t"
                     "ldr d1, [%[buff_in0], #80]\n\t"
                     "ins v0.d[1], %[scratch0]\n\t"
                     "ldr d2, [%[buff_in0], #96]\n\t"
                     "ins v1.d[1], %[scratch1]\n\t"
                     "ins v3.d[0], %[scratch4]\n\t"
                     "ins v2.d[1], %[scratch2]\n\t"
                     "fadd v7.4s, v7.4s, %[vB].4s\n\t"
                     "0:\n\t"
                     "fmul v0.4s, v0.4s, %[vA].s[0]\n\t"
                     "ldr %[scratch0], [%[buff_in0], #136]\n\t"
                     "fmul v1.4s, v1.4s, %[vA].s[0]\n\t"
                     "fmul v2.4s, v2.4s, %[vA].s[0]\n\t"
                     "ldr %[scratch1], [%[buff_in0], #152]\n\t"
                     "fmul v3.4s, v3.4s, %[vA].s[0]\n\t"
                     "ldr %[scratch2], [%[buff_in0], #168]\n\t"
                     "fadd v0.4s, v0.4s, %[vB].4s\n\t"
                     "ldr %[scratch3], [%[buff_in0], #184]\n\t"
                     "fadd v1.4s, v1.4s, %[vB].4s\n\t"
                     "ldr %[scratch4], [%[buff_in0], #176]\n\t"
                     "st1 {v4.4s-v7.4s}, [%[buff_out]], #64\n\t"
                     "ldr d5, [%[buff_in0], #144]\n\t"
                     "ins v7.d[1], %[scratch3]\n\t"
                     "ldr d6, [%[buff_in0], #160]\n\t"
                     "ins v5.d[1], %[scratch1]\n\t"
                     "ldr d4, [%[buff_in0], #128]!\n\t"
                     "ins v6.d[1], %[scratch2]\n\t"
                     "ins v7.d[0], %[scratch4]\n\t"
                     "ins v4.d[1], %[scratch0]\n\t"
                     "fadd v2.4s, v2.4s, %[vB].4s\n\t"
                     "prfm PLDL1KEEP, [%[buff_in1], #192]\n\t"
                     "fadd v3.4s, v3.4s, %[vB].4s\n\t"
                     "prfm PLDL1KEEP, [%[buff_in1], #256]\n\t"
                     "fmul v4.4s, v4.4s, %[vA].s[0]\n\t"
                     "ldr %[scratch0], [%[buff_in1], #136]\n\t"
                     "fmul v5.4s, v5.4s, %[vA].s[0]\n\t"
                     "fmul v6.4s, v6.4s, %[vA].s[0]\n\t"
                     "ldr %[scratch1], [%[buff_in1], #152]\n\t"
                     "fmul v7.4s, v7.4s, %[vA].s[0]\n\t"
                     "ldr %[scratch2], [%[buff_in1], #168]\n\t"
                     "fadd v4.4s, v4.4s, %[vB].4s\n\t"
                     "ldr %[scratch3], [%[buff_in1], #184]\n\t"
                     "fadd v5.4s, v5.4s, %[vB].4s\n\t"
                     "ldr %[scratch4], [%[buff_in1], #176]\n\t"
                     "st1 {v0.4s-v3.4s}, [%[buff_out]], #64\n\t"
                     "ldr d1, [%[buff_in1], #144]\n\t"
                     "ins v3.d[1], %[scratch3]\n\t"
                     "ldr d2, [%[buff_in1], #160]\n\t"
                     "ins v1.d[1], %[scratch1]\n\t"
                     "ldr d0, [%[buff_in1], #128]!\n\t"
                     "ins v2.d[1], %[scratch2]\n\t"
                     "ins v3.d[0], %[scratch4]\n\t"
                     "ins v0.d[1], %[scratch0]\n\t"
                     "fadd v6.4s, v6.4s, %[vB].4s\n\t"
                     "cmp %[buff_in0], %[buff_in0_end]\n\t"
                     "fadd v7.4s, v7.4s, %[vB].4s\n\t"
                     "b.ne 0b\n\t"
                     "fmul v0.4s, v0.4s, %[vA].s[0]\n\t"
                     "fmul v1.4s, v1.4s, %[vA].s[0]\n\t"
                     "fmul v2.4s, v2.4s, %[vA].s[0]\n\t"
                     "fmul v3.4s, v3.4s, %[vA].s[0]\n\t"
                     "st1 {v4.4s-v7.4s}, [%[buff_out]], #64\n\t"
                     "fadd v0.4s, v0.4s, %[vB].4s\n\t"
                     "fadd v1.4s, v1.4s, %[vB].4s\n\t"
                     "fadd v2.4s, v2.4s, %[vB].4s\n\t"
                     "fadd v3.4s, v3.4s, %[vB].4s\n\t"
                     "st1 {v0.4s-v3.4s}, [%[buff_out]]"
                     : [buff_in0] "+r"(buff_in0),
                       [buff_in1] "+r"(buff_in1),
                       [buff_in0_end] "+r"(buff_in0_end),
                       [buff_out] "+r"(buff_out),
                       [scratch0] "=r"(scratch0),
                       [scratch1] "=r"(scratch1),
                       [scratch2] "=r"(scratch2),
                       [scratch3] "=r"(scratch3),
                       [scratch4] "=r"(scratch4)
                     : [vA] "w"(s31), [vB] "w"(v30)
                     : "cc", "memory", "v0", "v1", "v2", "v3", "v4", "v5", "v6", "v7");
#else  /* __aarch64__ */
    for (int j = 0; j < noutput_items; ++j) {
        out[j] = d_a * in[j] + d_b;
    }
#endif /* __aarch64__ */

    return noutput_items;
}

} /* namespace qsdr_benchmark */
} /* namespace gr */
