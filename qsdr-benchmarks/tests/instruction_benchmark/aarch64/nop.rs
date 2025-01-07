use qsdr_benchmarks::{benchmark, expected_cycles};

// Cortex-A53
//
// Can dual-issue two nop's

#[test]
fn nop1() {
    unsafe {
        benchmark!("1x nop";
                   Some(1);
                   "nop",
                   ;
        );
    }
}

#[test]
fn nop2() {
    unsafe {
        benchmark!("2x nop";
                   expected_cycles! {
                       "cortex-a53" => 1,
                   };
                   "nop",
                   "nop",
                   ;
        );
    }
}

#[test]
fn nop3() {
    unsafe {
        benchmark!("3x nop";
                   expected_cycles! {
                       "cortex-a53" => 2,
                   };
                   "nop",
                   "nop",
                   "nop",
                   ;
        );
    }
}

#[test]
fn nop4() {
    unsafe {
        benchmark!("4x nop";
                   expected_cycles! {
                       "cortex-a53" => 2,
                   };
                   "nop",
                   "nop",
                   "nop",
                   "nop",
                   ;
        );
    }
}
