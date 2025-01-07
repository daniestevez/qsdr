mod basic {
    use qsdr_benchmarks::benchmark;

    #[test]
    fn empty() {
        unsafe {
            benchmark!("empty asm section";
                       Some(0);
                       "",
                       ;
            );
        }
    }
}

mod arith;
mod branch;
mod neon {
    mod arith {
        mod fadd;
        mod fmul;
    }
    mod dual_issue;
    mod gpr;
    mod kernels {
        mod saxpy;
    }
    mod load {
        mod ld1;
        mod ldr;
    }
    mod store {
        mod st1;
        mod str_;
    }
}
mod nop;
