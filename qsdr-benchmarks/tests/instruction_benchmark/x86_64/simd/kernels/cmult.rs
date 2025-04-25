#[cfg(target_feature = "avx")]
use qsdr_benchmarks::{Buffer, asm::x86_64::warm_up_ymm, benchmark_x86_64, expected_cycles};

#[test]
#[cfg(all(target_feature = "avx"))]
fn cmult_avx() {
    let x = Buffer::<f32>::new(128);
    let y = Buffer::<f32>::new(128);
    let z = Buffer::<f32>::new(128);

    warm_up_ymm();
    unsafe {
        benchmark_x86_64!(
            "cmult kernel core (AVX)";
            1_000_000_000;
            expected_cycles! {
                "znver3" => 2.166, // ?
            };
            "vmovaps ymm0, ymmword ptr [{x}]",
            "vmovaps ymm1, ymmword ptr [{y}]",
            "vpermilps ymm2, ymm1, 0xb1",
            "vmovsldup ymm3, ymm0",
            "vmovshdup ymm0, ymm0",
            "vmulps ymm3, ymm2, ymm3",
            "vmulps ymm0, ymm0, ymm1",
            "vaddsubps ymm0, ymm0, ymm3",
            "vmovaps ymmword ptr [{z}], ymm0",
            ;
            x = in(reg) x.as_ptr(),
            y = in(reg) y.as_ptr(),
            z = in(reg) z.as_ptr(),
            out("ymm0") _,
            out("ymm1") _,
            out("ymm2") _,
            out("ymm3") _,
        );
    }
}

#[test]
#[cfg(all(target_feature = "avx", target_feature = "fma"))]
fn cmult_avx_fma() {
    let x = Buffer::<f32>::new(128);
    let y = Buffer::<f32>::new(128);
    let z = Buffer::<f32>::new(128);

    warm_up_ymm();
    unsafe {
        benchmark_x86_64!(
            "cmult kernel core (AVX FMA)";
            1_000_000_000;
            expected_cycles! {
                "znver3" => 2.166, // ??
            };
            "vmovaps ymm0, ymmword ptr [{x}]",
            "vmovaps ymm1, ymmword ptr [{y}]",
            "vpermilps ymm2, ymm1, 0xb1",
            "vmovsldup ymm3, ymm0",
            "vmovshdup ymm0, ymm0",
            "vmulps ymm3, ymm2, ymm3",
            "vfmaddsub132ps ymm0, ymm3, ymm1",
            "vmovaps ymmword ptr [{z}], ymm0",
            ;
            x = in(reg) x.as_ptr(),
            y = in(reg) y.as_ptr(),
            z = in(reg) z.as_ptr(),
            out("ymm0") _,
            out("ymm1") _,
            out("ymm2") _,
            out("ymm3") _,
        );
    }
}
