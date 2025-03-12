mod basic {
    use qsdr_benchmarks::{benchmark_x86_64, expected_cycles};

    #[test]
    fn empty() {
        unsafe {
            benchmark_x86_64!("empty asm section";
                              1_000_000_000;
                              expected_cycles! {
                                  "znver3" => 1.0,
                              };
                              "",
                              ;
            );
        }
    }
}

mod simd {
    mod kernels {
        mod saxpy;
    }
}
