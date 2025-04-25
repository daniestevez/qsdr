#[cfg(target_feature = "avx")]
use qsdr_benchmarks::{Buffer, asm::x86_64::warm_up_ymm, benchmark_x86_64, expected_cycles};

// In znver3, this kernel shows good performance in this micro-benchmark, but in
// more realistic scenarios it underperforms somewhat (for some unknown
// reason). The saxpy_avx_full_x4 kernel performs better in practice.
#[test]
#[cfg(target_feature = "avx")]
fn saxpy_avx_full_x2() {
    // 32 KiB (the full L1 DCACHE size in znver3)
    let buffer = Buffer::<f32>::new(8192);

    // This is for processing 4 KiB of data
    let iterations = 64;

    warm_up_ymm();
    unsafe {
        benchmark_x86_64!(
            "saxpy kernel (AVX x2)";
            25_000_000;
            expected_cycles! {
                "znver3" => 2.0 * iterations as f64,
            };
            "xor {offset}, {offset}",
            ".align 4",
            "3:",
            "vmulps ymm2, ymm0, ymmword ptr [{buff} + 4*{offset:r}]",
            "vmulps ymm3, ymm0, ymmword ptr [{buff} + 4*{offset:r} + 32]",
            "vaddps ymm2, ymm2, ymm1",
            "vaddps ymm3, ymm3, ymm1",
            "vmovaps ymmword ptr [{buff} + 4*{offset:r}], ymm2",
            "vmovaps ymmword ptr [{buff} + 4*{offset:r} + 32], ymm3",
            "add {offset}, 16",
            "cmp {offset}, {offset_end:r}",
            "jne 3b",
            ;
            buff = in(reg) buffer.as_ptr(),
            offset = out(reg) _,
            offset_end = in(reg) 16 * iterations,
            in("ymm0") 0.0,
            in("ymm1") 0.0,
            out("ymm2") _,
            out("ymm3") _,
        );
    }
}

#[test]
#[cfg(target_feature = "avx")]
fn saxpy_avx_full_x4() {
    // 32 KiB (the full L1 DCACHE size in znver3)
    let buffer = Buffer::<f32>::new(8192);

    // This is for processing 4 KiB of data
    let iterations = 32;

    warm_up_ymm();
    unsafe {
        benchmark_x86_64!(
            "saxpy kernel (AVX x4)";
            25_000_000;
            expected_cycles! {
                "znver3" => 4.0 * iterations as f64,
            };
            "xor {offset}, {offset}",
            ".align 4",
            "3:",
            "vmulps ymm2, ymm0, ymmword ptr [{buff} + 4*{offset:r}]",
            "vmulps ymm3, ymm0, ymmword ptr [{buff} + 4*{offset:r} + 32]",
            "vmulps ymm4, ymm0, ymmword ptr [{buff} + 4*{offset:r} + 64]",
            "vmulps ymm5, ymm0, ymmword ptr [{buff} + 4*{offset:r} + 96]",
            "vaddps ymm2, ymm2, ymm1",
            "vaddps ymm3, ymm3, ymm1",
            "vaddps ymm4, ymm4, ymm1",
            "vaddps ymm5, ymm5, ymm1",
            "vmovaps ymmword ptr [{buff} + 4*{offset:r}], ymm2",
            "vmovaps ymmword ptr [{buff} + 4*{offset:r} + 32], ymm3",
            "vmovaps ymmword ptr [{buff} + 4*{offset:r} + 64], ymm4",
            "vmovaps ymmword ptr [{buff} + 4*{offset:r} + 96], ymm5",
            "add {offset}, 32",
            "cmp {offset}, {offset_end:r}",
            "jne 3b",
            ;
            buff = in(reg) buffer.as_ptr(),
            offset = out(reg) _,
            offset_end = in(reg) 32 * iterations,
            in("ymm0") 0.0,
            in("ymm1") 0.0,
            out("ymm2") _,
            out("ymm3") _,
            out("ymm4") _,
            out("ymm5") _,
        );
    }
}

#[test]
#[cfg(all(target_feature = "avx", target_feature = "fma"))]
fn saxpy_avx_fma_full_x1() {
    // 32 KiB (the full L1 DCACHE size in znver3)
    let buffer = Buffer::<f32>::new(8192);

    // This is for processing 16 KiB of data
    let iterations = 512;

    // This overhead is for 512 iterations. Less iterations have more overhead.
    let overhead = 0.05;

    warm_up_ymm();
    unsafe {
        benchmark_x86_64!(
            "saxpy kernel (AVX FMA x1)";
            10_000_000;
            expected_cycles! {
                "znver3" => (1.0 + overhead) * iterations as f64,
            };
            "mov {buff}, {buff_start}",
            "3:",
            "vmovaps ymm2, ymmword ptr [{buff}]",
            "vfmadd132ps ymm2, ymm1, ymm0",
            "vmovaps ymmword ptr [{buff}], ymm2",
            "add {buff}, 32",
            "cmp {buff}, {buff_end}",
            "jne 3b",
            ;
            buff_start = in(reg) buffer.as_ptr(),
            buff = out(reg) _,
            buff_end = in(reg) buffer.as_ptr().byte_offset(32 * iterations),
            in("ymm0") 0.0,
            in("ymm1") 0.0,
            out("ymm2") _,
        );
    }
}

#[test]
#[cfg(target_feature = "avx")]
fn saxpy_avx_core() {
    let buffer = Buffer::<f32>::new(128);
    let buffer_out = Buffer::<f32>::new(128);

    warm_up_ymm();
    unsafe {
        benchmark_x86_64!(
            "saxpy kernel core (AVX)";
            1_000_000_000;
            expected_cycles! {
                "znver3" => 2.0,
            };
            "vmulps ymm2, ymm0, ymmword ptr [{buff} + 4*{offset:r}]",
            "vmulps ymm3, ymm0, ymmword ptr [{buff} + 4*{offset:r} + 32]",
            "vaddps ymm2, ymm2, ymm1",
            "vaddps ymm3, ymm3, ymm1",
            "vmovaps ymmword ptr [{buff_out} + 4*{offset:r}], ymm2",
            "vmovaps ymmword ptr [{buff_out} + 4*{offset:r} + 32], ymm3",
            ;
            buff = in(reg) buffer.as_ptr(),
            buff_out = in(reg) buffer_out.as_ptr(),
            offset = in(reg) 0,
            in("ymm0") 0.0,
            in("ymm1") 0.0,
            out("ymm2") _,
            out("ymm3") _,
        );
    }
}

#[test]
#[cfg(all(target_feature = "avx", target_feature = "fma"))]
fn saxpy_avx_fma_x1() {
    let buffer = Buffer::<f32>::new(128);
    let buffer_out = Buffer::<f32>::new(128);

    // This kernel takes 1 cycles per iteration on znver3, which is what is
    // expected (the bottleneck is one store per cycle).

    warm_up_ymm();
    unsafe {
        benchmark_x86_64!(
            "saxpy kernel core (AVX FMA x1)";
            2_000_000_000;
            expected_cycles! {
                "znver3" => 1.0,
            };
            "vmovaps ymm2, ymmword ptr [{buff}]",
            "vfmadd132ps ymm2, ymm1, ymm0",
            "vmovaps ymmword ptr [{buff_out}], ymm2",
            ;
            buff = in(reg) buffer.as_ptr(),
            buff_out = in(reg) buffer_out.as_ptr(),
            in("ymm0") 0.0,
            in("ymm1") 0.0,
            out("ymm2") _,
        );
    }
}

#[test]
#[cfg(all(target_feature = "avx", target_feature = "fma"))]
fn saxpy_avx_fma_x2() {
    let buffer = Buffer::<f32>::new(128);
    let buffer_out = Buffer::<f32>::new(128);

    // On znver3, performance of this kernel is highly sensitive to alignment,
    // presumably because of OC utilization quirks.

    warm_up_ymm();
    unsafe {
        benchmark_x86_64!(
            "saxpy kernel core (AVX FMA x2)";
            1_000_000_000;
            None;
            "vmovaps ymm2, ymmword ptr [{buff} + 4*{offset:r}]",
            "vmovaps ymm3, ymmword ptr [{buff} + 4*{offset:r} + 32]",
            "vfmadd132ps ymm2, ymm1, ymm0",
            "vfmadd132ps ymm3, ymm1, ymm0",
            "vmovaps ymmword ptr [{buff_out} + 4*{offset:r}], ymm2",
            "vmovaps ymmword ptr [{buff_out} + 4*{offset:r} + 32], ymm3",
            ;
            buff = in(reg) buffer.as_ptr(),
            buff_out = in(reg) buffer_out.as_ptr(),
            offset = in(reg) 0,
            in("ymm0") 0.0,
            in("ymm1") 0.0,
            out("ymm2") _,
            out("ymm3") _,
        );
    }
}

#[test]
#[cfg(all(target_feature = "avx", target_feature = "fma"))]
fn saxpy_avx_fma_x4() {
    let buffer = Buffer::<f32>::new(128);
    let buffer_out = Buffer::<f32>::new(128);

    // On znver3, performance of this kernel is highly sensitive to alignment,
    // presumably because of OC utilization quirks.

    warm_up_ymm();
    unsafe {
        benchmark_x86_64!(
            "saxpy kernel core (AVX FMA x4)";
            1_000_000_000;
            None;
            "vmovaps ymm2, ymmword ptr [{buff} + 4*{offset:r}]",
            "vmovaps ymm3, ymmword ptr [{buff} + 4*{offset:r} + 32]",
            "vmovaps ymm4, ymmword ptr [{buff} + 4*{offset:r} + 64]",
            "vmovaps ymm5, ymmword ptr [{buff} + 4*{offset:r} + 96]",
            "vfmadd132ps ymm2, ymm1, ymm0",
            "vfmadd132ps ymm3, ymm1, ymm0",
            "vfmadd132ps ymm4, ymm1, ymm0",
            "vfmadd132ps ymm5, ymm1, ymm0",
            "vmovaps ymmword ptr [{buff_out} + 4*{offset:r}], ymm2",
            "vmovaps ymmword ptr [{buff_out} + 4*{offset:r} + 32], ymm3",
            "vmovaps ymmword ptr [{buff_out} + 4*{offset:r} + 64], ymm4",
            "vmovaps ymmword ptr [{buff_out} + 4*{offset:r} + 96], ymm5",
            ;
            buff = in(reg) buffer.as_ptr(),
            buff_out = in(reg) buffer_out.as_ptr(),
            offset = in(reg) 0,
            in("ymm0") 0.0,
            in("ymm1") 0.0,
            out("ymm2") _,
            out("ymm3") _,
            out("ymm4") _,
            out("ymm5") _,
        );
    }
}
