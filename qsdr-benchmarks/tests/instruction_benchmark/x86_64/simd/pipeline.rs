#[cfg(target_feature = "avx")]
use qsdr_benchmarks::{asm::x86_64::warm_up_ymm, benchmark_x86_64, expected_cycles};

// On zen3, this sequence of 16 dependent vaddps can run at 0.5 cycles/vaddps,
// which is the reciprocal throughput of vaddps.

#[test]
#[cfg(target_feature = "avx")]
fn vaddps_dep_x16() {
    warm_up_ymm();
    unsafe {
        benchmark_x86_64!(
            "x16 dependent vaddps";
            1_000_000_000;
            expected_cycles! {
                "znver3" => 8.0,
            };
            "vxorps ymm0, ymm0, ymm0",

            "vaddps ymm0, ymm0, ymm0",
            "vaddps ymm0, ymm0, ymm0",
            "vaddps ymm0, ymm0, ymm0",
            "vaddps ymm0, ymm0, ymm0",

            "vaddps ymm0, ymm0, ymm0",
            "vaddps ymm0, ymm0, ymm0",
            "vaddps ymm0, ymm0, ymm0",
            "vaddps ymm0, ymm0, ymm0",

            "vaddps ymm0, ymm0, ymm0",
            "vaddps ymm0, ymm0, ymm0",
            "vaddps ymm0, ymm0, ymm0",
            "vaddps ymm0, ymm0, ymm0",

            "vaddps ymm0, ymm0, ymm0",
            "vaddps ymm0, ymm0, ymm0",
            "vaddps ymm0, ymm0, ymm0",
            "vaddps ymm0, ymm0, ymm0",
            ;
            out("ymm0") _,
        );
    }
}

// However, a sequence of 17 or more dependent vaddps has a performance
// penalty. This is likely caused by the floating point execution unit being
// starved of physical registers for renaming or other scheduling/retirement
// resources.
//
// Note that since vaddps has a latency of 3 instructions, assuming that 2
// vaddps per cycle are dispatched, the number of cycles that it takes to
// execute the last vaddps in the chain of n vaddps is n * 3 - n / 2
// cycles. During that time, 2 * (n * 3 - n / 2) - 1 = 5 * n - 1 other vaddps
// are dispatched.

#[test]
#[cfg(target_feature = "avx")]
fn vaddps_dep_x17() {
    warm_up_ymm();
    unsafe {
        benchmark_x86_64!(
            "x17 dependent vaddps";
            1_000_000_000;
            expected_cycles! {
                "znver3" => 10.0,
            };
            "vxorps ymm0, ymm0, ymm0",

            "vaddps ymm0, ymm0, ymm0",
            "vaddps ymm0, ymm0, ymm0",
            "vaddps ymm0, ymm0, ymm0",
            "vaddps ymm0, ymm0, ymm0",

            "vaddps ymm0, ymm0, ymm0",
            "vaddps ymm0, ymm0, ymm0",
            "vaddps ymm0, ymm0, ymm0",
            "vaddps ymm0, ymm0, ymm0",

            "vaddps ymm0, ymm0, ymm0",
            "vaddps ymm0, ymm0, ymm0",
            "vaddps ymm0, ymm0, ymm0",
            "vaddps ymm0, ymm0, ymm0",

            "vaddps ymm0, ymm0, ymm0",
            "vaddps ymm0, ymm0, ymm0",
            "vaddps ymm0, ymm0, ymm0",
            "vaddps ymm0, ymm0, ymm0",

            "vaddps ymm0, ymm0, ymm0",
            ;
            out("ymm0") _,
        );
    }
}

// On zen3, a pipeline of 4 dependent vpaddb runs at the maximum throughput of 4
// vpaddb's per clock cycle.

#[test]
#[cfg(target_feature = "avx2")]
fn vpaddb_dep_x4() {
    warm_up_ymm();
    unsafe {
        benchmark_x86_64!(
            "x4 dependent vpaddb";
            1_000_000_000;
            expected_cycles! {
                "znver3" => 1.0,
            };
            "vpxor ymm0, ymm0, ymm0",

            "vpaddb ymm0, ymm0, ymm0",
            "vpaddb ymm0, ymm0, ymm0",
            "vpaddb ymm0, ymm0, ymm0",
            "vpaddb ymm0, ymm0, ymm0",
            ;
            out("ymm0") _,
        );
    }
}

// With 5 dependent vpaddb, the performance can already drop (apparently
// depending on code alignment), and with 7 it already drops.

#[test]
#[cfg(target_feature = "avx2")]
fn vpaddb_dep_x7() {
    warm_up_ymm();
    unsafe {
        benchmark_x86_64!(
            "x7 dependent vpaddb";
            1_000_000_000;
            expected_cycles! {
                "znver3" => 2.0,
            };
            "vpxor ymm0, ymm0, ymm0",

            "vpaddb ymm0, ymm0, ymm0",
            "vpaddb ymm0, ymm0, ymm0",
            "vpaddb ymm0, ymm0, ymm0",
            "vpaddb ymm0, ymm0, ymm0",

            "vpaddb ymm0, ymm0, ymm0",
            "vpaddb ymm0, ymm0, ymm0",
            "vpaddb ymm0, ymm0, ymm0",
            ;
            out("ymm0") _,
        );
    }
}
