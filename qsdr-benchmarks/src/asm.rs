use std::collections::HashMap;

const BENCHMARK_ITERATIONS: usize = 10000;

#[doc(hidden)]
pub fn run_benchmark(f: impl FnMut() -> u64) -> HashMap<u64, usize> {
    let results: Vec<_> = std::iter::repeat_with(f)
        .take(BENCHMARK_ITERATIONS)
        .collect();
    let mut histogram = HashMap::new();
    for &r in &results {
        histogram
            .entry(r)
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }
    histogram
}

#[doc(hidden)]
#[cfg(target_arch = "aarch64")]
#[macro_export]
macro_rules! time_asm {
    ($($instruction:expr),*,; $($extra:tt)*) => {
        {
            let start: u64;
            let end: u64;
            // The following 32 nop's are used to flush the pipeline after
            // branching back to the beginning of this block in a loop, which
            // can affect the timing of some load instructions (for instance
            // ld1.4s {v0-v3} shows an issue latency of 10 cycles if these nop's
            // are not included).
            std::arch::asm!(
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "mrs {__time_asm_start}, pmccntr_el0",
                $($instruction),*,
                "mrs {__time_asm_end}, pmccntr_el0",
                __time_asm_start = out(reg) start,
                __time_asm_end = out(reg) end,
                $($extra)*);
            end - start - 1
        }
    }
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
#[inline(always)]
pub fn get_cpu_cycles() -> u64 {
    0
}

// Note that this counts reference clock cycles, so the count is not affected by
// frequency scaling (and it should be).
#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub fn get_cpu_cycles() -> u64 {
    let rax: u64;
    let rdx: u64;
    unsafe {
        std::arch::asm!("rdtsc", out("rax") rax, out("rdx") rdx);
    }
    (rdx << 32) | rax
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub fn get_cpu_cycles() -> u64 {
    let ret;
    unsafe {
        std::arch::asm!("mrs {0}, pmccntr_el0", out(reg) ret);
    }
    ret
}

#[macro_export]
macro_rules! benchmark {
    ($benchmark_name:expr; $expected_cycles:expr;
     $($instruction:expr),*,; $($extra:tt)*) => {
        {
            use owo_colors::OwoColorize;

            let hist = $crate::asm::run_benchmark(
                || $crate::time_asm!($($instruction),*, ; $($extra)*)
            );

            let name = $benchmark_name;
            println!();
            println!("{}", name.blue());
            println!("{}", std::iter::repeat("=").take(name.len()).collect::<String>().blue());
            println!();

            { $(println!("    {}", $instruction);)* }
            println!();

            let (&mode_cycles, &mode_count) = hist.iter().max_by_key(|(_, &v)| v).unwrap();

            println!("{}", "cycles | runs".blue());
            println!("{}", "-------------".blue());
            let mut cycles: Vec<u64> = hist.keys().copied().collect();
            cycles.sort_unstable();
            for cyc in &cycles {
                let color = if *cyc == mode_cycles {
                    owo_colors::AnsiColors::Green
                } else {
                    owo_colors::AnsiColors::Default
                };
                println!("{:6} {} {:4}",
                         cyc.color(color),
                         "|".blue(),
                         hist[cyc].color(color));
            }
            println!();

            let mode_threshold = 922; // ~90% of 1024
            #[cfg(test)]
            assert!(mode_count >= mode_threshold,
                    "mode of test cycles not obtained in enough runs; benchmark results unreliable");

            if let Some(expected_cycles) = $expected_cycles {
                assert_eq!(mode_cycles, expected_cycles,
                           "mode of test cycles does not match expected for this CPU: {expected_cycles}");
            }

            mode_cycles
        }
    }
}

#[macro_export]
macro_rules! expected_cycles {
    ($($cpu:expr => $cycles:expr),*$(,)?) => {
        {
            if let Ok(cpu_env) = std::env::var("CPU") {
                match &cpu_env[..] {
                    $($cpu => Some($cycles)),*,
                    _ => None,
                }
            } else {
                None
            }
        }
    };
}
