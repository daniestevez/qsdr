use qsdr_benchmarks::{benchmark, expected_cycles, Buffer};

#[test]
fn saxpy_rustc() {
    unsafe {
        // 32 KiB (the full L1 DCACHE size)
        let buffer = Buffer::<f32>::new(8192);
        let iterations = 512; // this is for processing 16 KiB of data

        let branch_miss_penalty = 7;
        let cycles = expected_cycles! {
            "cortex-a53" => 16 * iterations as u64 + branch_miss_penalty,
        };

        let benchmark_cycles = benchmark!(
            "saxpy kernel (rustc) ~1 flops/cycle";
            cycles;
            "0:",
            "add {p}, {buff}, {offset:x}",
            "add {offset:x}, {offset:x}, 32",
            "cmp {offset:x}, {end:x}",
            "ldp q0, q1, [{p}]",
            "fmul v0.4s, v0.4s, v2.s[0]",
            "fmul v1.4s, v1.4s, v2.s[0]",
            "fadd v0.4s, v0.4s, v3.4s",
            "fadd v1.4s, v1.4s, v3.4s",
            "stp q0, q1, [{p}]",
            "b.ne 0b",
            ;
            buff = in(reg) buffer.as_ptr(),
            offset = inout(reg) 0 => _,
            end = in(reg) iterations * 32,
            p = out(reg) _,
            out("v0") _,
            out("v1") _,
        );

        let num_floats = 8 * iterations;
        let flops = 2 * num_floats;
        let flops_per_cycle = flops as f64 / benchmark_cycles as f64;
        println!("{flops_per_cycle:.3} flops/cycle");
    }
}

#[test]
fn saxpy_q_core() {
    unsafe {
        let buffer = Buffer::<f32>::new(64);
        benchmark!("saxpy kernel core (128-bit datapath) 2 flops/cycle";
                   expected_cycles! {
                       "cortex-a53" => 32,
                   };
                   "fmul.4s v0, v0, v31",
                   "ldr x0, [{buff0}, #136]",
                   "fmul.4s v1, v1, v31",
                   "fmul.4s v2, v2, v31",
                   "ldr x1, [{buff0}, #152]",
                   "fmul.4s v3, v3, v31",
                   "ldr x2, [{buff0}, #168]",
                   "fadd.4s v0, v0, v30",
                   "ldr x3, [{buff0}, #184]",
                   "fadd.4s v1, v1, v30",
                   "ldr x4, [{buff0}, #176]",
                   "st1.4s {{v4-v7}}, [{buff0}]",
                   "ldr d5, [{buff0}, #144]",
                   "ins v7.d[1], x3",
                   "ldr d6, [{buff0}, #160]",
                   "ins v5.d[1], x1",
                   "ldr d4, [{buff0}, #128]!",
                   "ins v6.d[1], x2",
                   "ins v7.d[0], x4",
                   "ins v4.d[1], x0",
                   "fadd.4s v2, v2, v30",
                   "fadd.4s v3, v3, v30",

                   "fmul.4s v4, v4, v31",
                   "ldr x0, [{buff1}, #136]",
                   "fmul.4s v5, v5, v31",
                   // for some reason there needs to be a gap without an ldr around
                   // here or we get 2 cycles penalty
                   "fmul.4s v6, v6, v31",
                   "ldr x1, [{buff1}, #152]",
                   "fmul.4s v7, v7, v31",
                   "ldr x2, [{buff1}, #168]",
                   "fadd.4s v4, v4, v30",
                   "ldr x3, [{buff1}, #184]",
                   "fadd.4s v5, v5, v30",
                   "ldr x4, [{buff1}, #176]",
                   "st1.4s {{v0-v3}}, [{buff1}]",
                   "ldr d1, [{buff1}, #144]",
                   "ins v3.d[1], x3",
                   "ldr d2, [{buff1}, #160]",
                   "ins v1.d[1], x1",
                   "ldr d0, [{buff1}, #128]!",
                   "ins v2.d[1], x2",
                   "ins v3.d[0], x4",
                   "ins v0.d[1], x0",
                   "fadd.4s v6, v6, v30",
                   "fadd.4s v7, v7, v30",
                   ;
                   buff0 = inout(reg) buffer.as_ptr() => _,
                   buff1 = inout(reg) buffer.as_ptr().byte_add(64) => _,
                   out("v0") _,
                   out("v1") _,
                   out("v2") _,
                   out("v3") _,
                   out("v4") _,
                   out("v5") _,
                   out("v6") _,
                   out("v7") _,
                   out("x0") _,
                   out("x1") _,
                   out("x2") _,
                   out("x3") _,
                   out("x4") _,
        );
    }
}

#[test]
fn saxpy_q_full() {
    unsafe {
        // 32 KiB (the full L1 DCACHE size)
        let buffer = Buffer::<f32>::new(8192);
        let iterations = 127; // this is for processing 16 KiB of data

        let branch_miss_penalty = 7;
        let cycles = expected_cycles! {
            "cortex-a53" => 8 + 12 + 32 * iterations as u64 + 16 + branch_miss_penalty,
        };

        let benchmark_cycles = benchmark!(
            "saxpy full kernel (128-bit datapath) ~2 flops/cycle";
            cycles;
            // this takes 8 cycles
            "ld1.4s {{v4-v7}}, [{buff0}]",
            // this takes 12 cycles
            "fmul.4s v4, v4, v31",
            // dual-issue prefetch for first iteration's buff0
            "prfm PLDL1KEEP, [{buff0}, #128]",
            "fmul.4s v5, v5, v31",
            "ldr x0, [{buff0}, #72]",
            "fmul.4s v6, v6, v31",
            "ldr x1, [{buff0}, #88]",
            "fmul.4s v7, v7, v31",
            "ldr x2, [{buff0}, #104]",
            "fadd.4s v4, v4, v30",
            "ldr x3, [{buff0}, #120]",
            "fadd.4s v5, v5, v30",
            "ldr x4, [{buff0}, #112]",
            "fadd.4s v6, v6, v30",
            // dual-issue prefetch for first iteration's buff1
            "prfm PLDL1KEEP, [{buff1}, #128]",
            "ldr d0, [{buff0}, #64]",
            "ins v3.d[1], x3",
            "ldr d1, [{buff0}, #80]",
            "ins v0.d[1], x0",
            "ldr d2, [{buff0}, #96]",
            "ins v1.d[1], x1",
            "ins v3.d[0], x4",
            "ins v2.d[1], x2",
            "fadd.4s v7, v7, v30",

            // each iteration takes 32 cycles
            "0:",
            "fmul.4s v0, v0, v31",
            "ldr x0, [{buff0}, #136]",
            "fmul.4s v1, v1, v31",
            "fmul.4s v2, v2, v31",
            "ldr x1, [{buff0}, #152]",
            "fmul.4s v3, v3, v31",
            "ldr x2, [{buff0}, #168]",
            "fadd.4s v0, v0, v30",
            "ldr x3, [{buff0}, #184]",
            "fadd.4s v1, v1, v30",
            "ldr x4, [{buff0}, #176]",
            "st1.4s {{v4-v7}}, [{buff0}]",
            "ldr d5, [{buff0}, #144]",
            "ins v7.d[1], x3",
            "ldr d6, [{buff0}, #160]",
            "ins v5.d[1], x1",
            "ldr d4, [{buff0}, #128]!",
            "ins v6.d[1], x2",
            "ins v7.d[0], x4",
            "ins v4.d[1], x0",
            "fadd.4s v2, v2, v30",
            // dual-issue prefetch for next iteration's buff0
            "prfm PLDL1KEEP, [{buff1}, #192]",
            "fadd.4s v3, v3, v30",
            // dual-issue prefetch for next iterations' buff1
            "prfm PLDL1KEEP, [{buff1}, #256]",

            "fmul.4s v4, v4, v31",
            "ldr x0, [{buff1}, #136]",
            "fmul.4s v5, v5, v31",
            // for some reason there needs to be a gap without an ldr around
            // here or we get 2 cycles penalty
            "fmul.4s v6, v6, v31",
            "ldr x1, [{buff1}, #152]",
            "fmul.4s v7, v7, v31",
            "ldr x2, [{buff1}, #168]",
            "fadd.4s v4, v4, v30",
            "ldr x3, [{buff1}, #184]",
            "fadd.4s v5, v5, v30",
            "ldr x4, [{buff1}, #176]",
            "st1.4s {{v0-v3}}, [{buff1}]",
            "ldr d1, [{buff1}, #144]",
            "ins v3.d[1], x3",
            "ldr d2, [{buff1}, #160]",
            "ins v1.d[1], x1",
            "ldr d0, [{buff1}, #128]!",
            "ins v2.d[1], x2",
            "ins v3.d[0], x4",
            "ins v0.d[1], x0",
            "fadd.4s v6, v6, v30",
            "cmp {buff0}, {buff0_end}",
            "fadd.4s v7, v7, v30",
            "b.ne 0b",

            // this takes 16 cycles
            "fmul.4s v0, v0, v31",
            "fmul.4s v1, v1, v31",
            "fmul.4s v2, v2, v31",
            "fmul.4s v3, v3, v31",
            "st1.4s {{v4-v7}}, [{buff0}]",
            "fadd.4s v0, v0, v30",
            "fadd.4s v1, v1, v30",
            "fadd.4s v2, v2, v30",
            "fadd.4s v3, v3, v30",
            "st1.4s {{v0-v3}}, [{buff1}]",
            ;
            buff0 = inout(reg) buffer.as_ptr() => _,
            buff1 = inout(reg) buffer.as_ptr().byte_add(64) => _,
            buff0_end = in(reg) buffer.as_ptr().byte_add(128 * iterations),
            out("v0") _,
            out("v1") _,
            out("v2") _,
            out("v3") _,
            out("v4") _,
            out("v5") _,
            out("v6") _,
            out("v7") _,
            out("x0") _,
            out("x1") _,
            out("x2") _,
            out("x3") _,
            out("x4") _,
        );

        let num_floats = 32 * (iterations + 1);
        let flops = 2 * num_floats;
        let flops_per_cycle = flops as f64 / benchmark_cycles as f64;
        println!("{flops_per_cycle:.3} flops/cycle");
    }
}

#[test]
fn saxpy_q_full_out_of_place() {
    unsafe {
        // 32 KiB (the full L1 DCACHE size)
        let buffer = Buffer::<f32>::new(8192);
        let iterations = 63; // this is for processing 8 KiB of data

        let branch_miss_penalty = 7;
        let cycles = expected_cycles! {
            "cortex-a53" => 8 + 12 + 32 * iterations as u64 + 16 + branch_miss_penalty,
        };

        let benchmark_cycles = benchmark!(
            "saxpy full out-of-place kernel (128-bit datapath) ~2 flops/cycle";
            cycles;
            // this takes 8 cycles
            "ld1.4s {{v4-v7}}, [{buff_in0}]",
            // this takes 12 cycles
            "fmul.4s v4, v4, v31",
            // dual-issue prefetch for first iteration's buff0
            "prfm PLDL1KEEP, [{buff_in0}, #128]",
            "fmul.4s v5, v5, v31",
            "ldr x0, [{buff_in0}, #72]",
            "fmul.4s v6, v6, v31",
            "ldr x1, [{buff_in0}, #88]",
            "fmul.4s v7, v7, v31",
            "ldr x2, [{buff_in0}, #104]",
            "fadd.4s v4, v4, v30",
            "ldr x3, [{buff_in0}, #120]",
            "fadd.4s v5, v5, v30",
            "ldr x4, [{buff_in0}, #112]",
            "fadd.4s v6, v6, v30",
            // dual-issue prefetch for first iteration's buff1
            "prfm PLDL1KEEP, [{buff_in1}, #128]",
            "ldr d0, [{buff_in0}, #64]",
            "ins v3.d[1], x3",
            "ldr d1, [{buff_in0}, #80]",
            "ins v0.d[1], x0",
            "ldr d2, [{buff_in0}, #96]",
            "ins v1.d[1], x1",
            "ins v3.d[0], x4",
            "ins v2.d[1], x2",
            "fadd.4s v7, v7, v30",

            // each iteration takes 32 cycles
            "0:",
            "fmul.4s v0, v0, v31",
            "ldr x0, [{buff_in0}, #136]",
            "fmul.4s v1, v1, v31",
            "fmul.4s v2, v2, v31",
            "ldr x1, [{buff_in0}, #152]",
            "fmul.4s v3, v3, v31",
            "ldr x2, [{buff_in0}, #168]",
            "fadd.4s v0, v0, v30",
            "ldr x3, [{buff_in0}, #184]",
            "fadd.4s v1, v1, v30",
            "ldr x4, [{buff_in0}, #176]",
            "st1.4s {{v4-v7}}, [{buff_out}], #64",
            "ldr d5, [{buff_in0}, #144]",
            "ins v7.d[1], x3",
            "ldr d6, [{buff_in0}, #160]",
            "ins v5.d[1], x1",
            "ldr d4, [{buff_in0}, #128]!",
            "ins v6.d[1], x2",
            "ins v7.d[0], x4",
            "ins v4.d[1], x0",
            "fadd.4s v2, v2, v30",
            // dual-issue prefetch for next iteration's buff0
            "prfm PLDL1KEEP, [{buff_in1}, #192]",
            "fadd.4s v3, v3, v30",
            // dual-issue prefetch for next iterations' buff1
            "prfm PLDL1KEEP, [{buff_in1}, #256]",

            "fmul.4s v4, v4, v31",
            "ldr x0, [{buff_in1}, #136]",
            "fmul.4s v5, v5, v31",
            // for some reason there needs to be a gap without an ldr around
            // here or we get 2 cycles penalty
            "fmul.4s v6, v6, v31",
            "ldr x1, [{buff_in1}, #152]",
            "fmul.4s v7, v7, v31",
            "ldr x2, [{buff_in1}, #168]",
            "fadd.4s v4, v4, v30",
            "ldr x3, [{buff_in1}, #184]",
            "fadd.4s v5, v5, v30",
            "ldr x4, [{buff_in1}, #176]",
            "st1.4s {{v0-v3}}, [{buff_out}], #64",
            "ldr d1, [{buff_in1}, #144]",
            "ins v3.d[1], x3",
            "ldr d2, [{buff_in1}, #160]",
            "ins v1.d[1], x1",
            "ldr d0, [{buff_in1}, #128]!",
            "ins v2.d[1], x2",
            "ins v3.d[0], x4",
            "ins v0.d[1], x0",
            "fadd.4s v6, v6, v30",
            "cmp {buff_in0}, {buff_in0_end}",
            "fadd.4s v7, v7, v30",
            "b.ne 0b",

            // this takes 16 cycles
            "fmul.4s v0, v0, v31",
            "fmul.4s v1, v1, v31",
            "fmul.4s v2, v2, v31",
            "fmul.4s v3, v3, v31",
            "st1.4s {{v4-v7}}, [{buff_out}], #64",
            "fadd.4s v0, v0, v30",
            "fadd.4s v1, v1, v30",
            "fadd.4s v2, v2, v30",
            "fadd.4s v3, v3, v30",
            "st1.4s {{v0-v3}}, [{buff_out}]",
            ;
            buff_in0 = inout(reg) buffer.as_ptr() => _,
            buff_in1 = inout(reg) buffer.as_ptr().byte_add(64) => _,
            buff_in0_end = in(reg) buffer.as_ptr().byte_add(128 * iterations),
            // write in second half of the buffer
            buff_out = inout(reg) buffer.as_ptr().offset(4096) => _,
            out("v0") _,
            out("v1") _,
            out("v2") _,
            out("v3") _,
            out("v4") _,
            out("v5") _,
            out("v6") _,
            out("v7") _,
            out("x0") _,
            out("x1") _,
            out("x2") _,
            out("x3") _,
            out("x4") _,
        );

        let num_floats = 32 * (iterations + 1);
        let flops = 2 * num_floats;
        let flops_per_cycle = flops as f64 / benchmark_cycles as f64;
        println!("{flops_per_cycle:.3} flops/cycle");
    }
}

#[test]
fn saxpy_d_core() {
    unsafe {
        let buffer = Buffer::<f32>::new(24);
        benchmark!("saxpy kernel core (64-bit datapath) 2 flops/cycle";
                   expected_cycles! {
                       "cortex-a53" => 16,
                   };
                   "fmul.2s v0, v0, v31",
                   "str d4, [{buff_w}, #8]",
                   "fmul.2s v1, v1, v31",
                   "str d5, [{buff_w}, #16]",
                   "fmul.2s v2, v2, v31",
                   "str d6, [{buff_w}, #24]",
                   "fmul.2s v3, v3, v31",
                   "str d7, [{buff_w}, #32]!",
                   "fadd.2s v0, v0, v30",
                   "ldr d4, [{buff_r}, #8]",
                   "fadd.2s v1, v1, v30",
                   "ldr d5, [{buff_r}, #16]",
                   "fadd.2s v2, v2, v30",
                   "ldr d6, [{buff_r}, #24]",
                   "fadd.2s v3, v3, v30",
                   "ldr d7, [{buff_r}, #32]!",
                   "fmul.2s v4, v4, v31",
                   "str d0, [{buff_w}, #8]",
                   "fmul.2s v5, v5, v31",
                   "str d1, [{buff_w}, #16]",
                   "fmul.2s v6, v6, v31",
                   "str d2, [{buff_w}, #24]",
                   "fmul.2s v7, v7, v31",
                   "str d3, [{buff_w}, #32]!",
                   "fadd.2s v4, v4, v30",
                   "ldr d0, [{buff_r}, #8]",
                   "fadd.2s v5, v5, v30",
                   "ldr d1, [{buff_r}, #16]",
                   "fadd.2s v6, v6, v30",
                   "ldr d2, [{buff_r}, #24]",
                   "fadd.2s v7, v7, v30",
                   "ldr d3, [{buff_r}, #32]!",
                   ;
                   buff_w = inout(reg) buffer.as_ptr().byte_offset(-8) => _,
                   buff_r = inout(reg) buffer.as_ptr().byte_offset(32-8) => _,
                   out("v0") _,
                   out("v1") _,
                   out("v2") _,
                   out("v3") _,
                   out("v4") _,
                   out("v5") _,
                   out("v6") _,
                   out("v7") _,
        );
    }
}
