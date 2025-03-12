use super::get_cpu_cycles;
use crate::{affinity::pin_cpu_num, msr::Msr};
use std::time::Instant;

#[doc(hidden)]
pub fn run_benchmark(mut f: impl FnMut(), iterations: u64, expected_aperf: Option<f64>) {
    let num_cpu = 0;
    pin_cpu_num(num_cpu).unwrap();
    let mut msr = Msr::new(num_cpu).unwrap();

    let start = Instant::now();
    let start_aperf = msr.read_aperf().unwrap();
    let start_tsc = get_cpu_cycles();

    f();

    let end_tsc = get_cpu_cycles();
    let end_aperf = msr.read_aperf().unwrap();
    let elapsed = start.elapsed();

    let tsc = end_tsc.wrapping_sub(start_tsc) as f64;
    let aperf = end_aperf.wrapping_sub(start_aperf) as f64;

    let iterations_f64 = iterations as f64;
    let tsc_per_iter = tsc / iterations_f64;
    let aperf_per_iter = aperf / iterations_f64;
    let aperf_per_tsc = aperf / tsc;
    let tsc_hz = tsc / elapsed.as_secs_f64();
    let aperf_hz = aperf / elapsed.as_secs_f64();

    println!();
    println!(
        "APERF {:.3} MHz    TSC {:.3} MHz    (x{:.2} boost)",
        aperf_hz * 1e-6,
        tsc_hz * 1e-6,
        aperf_per_tsc
    );
    println!("{aperf_per_iter:.3} APERF/iter      {tsc_per_iter:.3} TSC/iter");
    println!("benchmark runtime {:.3} ms", elapsed.as_secs_f64() * 1e3);
    println!();

    if let Some(expected_aperf) = expected_aperf {
        let tolerance = 5e-2; // 5% tolerance
        assert!(
            (aperf_per_iter - expected_aperf).abs() <= tolerance * expected_aperf,
            "APERF cycles/iter does not match expected for this CPU: {expected_aperf}"
        );
    }
}

#[macro_export]
macro_rules! benchmark_x86_64 {
    ($benchmark_name:expr; $iterations:expr; $expected_cycles:expr;
     $($instruction:expr),*,; $($extra:tt)*) => {
        use owo_colors::OwoColorize;

        let iterations: u64 = $iterations;

        let name = $benchmark_name;
        println!();
        println!("{}", name.blue());
        println!("{}", std::iter::repeat("=").take(name.len()).collect::<String>().blue());
        println!();

        println!("    2:");
        { $(println!("    {}", $instruction);)* }
        println!("    dec {{__loop_counter:r}}");
        println!("    jne 2b");
        println!();

        $crate::asm::x86_64::run_benchmark(
            || {
                std::arch::asm!(
                    ".align 4",
                    "2:",
                    $($instruction),*,
                    "dec {__loop_counter:r}",
                    "jne 2b",
                    __loop_counter = inout(reg) iterations => _,
                    $($extra)*);
            },
            iterations,
            $expected_cycles);
    }
}
