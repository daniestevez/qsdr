use qsdr_benchmarks::{benchmark, expected_cycles};

// Cortex-A53
//
// fmul issue latency 1, result latency 4

#[test]
fn fmul1() {
    unsafe {
        benchmark!("1x fmul";
                   expected_cycles! {
                       "cortex-a53" => 1,
                   };
                   "fmul.4s v0, v0, v31",
                   ;
                   out("v0") _,
        );
    }
}

#[test]
fn fmul2_nodep() {
    unsafe {
        benchmark!("2x fmul (no dependencies)";
                   expected_cycles! {
                       "cortex-a53" => 2,
                   };
                   "fmul.4s v0, v0, v31",
                   "fmul.4s v1, v1, v31",
                   ;
                   out("v0") _,
                   out("v1") _,
        );
    }
}

#[test]
fn fmul3_nodep() {
    unsafe {
        benchmark!("3x fmul (no dependencies)";
                   expected_cycles! {
                       "cortex-a53" => 3,
                   };
                   "fmul.4s v0, v0, v31",
                   "fmul.4s v1, v1, v31",
                   "fmul.4s v2, v2, v31",
                   ;
                   out("v0") _,
                   out("v1") _,
                   out("v2") _,
        );
    }
}

#[test]
fn fmul4_nodep() {
    unsafe {
        benchmark!("4x fmul (no dependencies)";
                   expected_cycles! {
                       "cortex-a53" => 4,
                   };
                   "fmul.4s v0, v0, v31",
                   "fmul.4s v1, v1, v31",
                   "fmul.4s v2, v2, v31",
                   "fmul.4s v3, v3, v31",
                   ;
                   out("v0") _,
                   out("v1") _,
                   out("v2") _,
                   out("v3") _,
        );
    }
}

#[test]
fn fmul2_dep() {
    unsafe {
        benchmark!("2x fmul (with dependency)";
                   expected_cycles! {
                       "cortex-a53" => 5,
                   };
                   "fmul.4s v0, v0, v31",
                   "fmul.4s v0, v0, v31",
                   ;
                   out("v0") _,
        );
    }
}

#[test]
fn fmul3_dep() {
    unsafe {
        benchmark!("3x fmul (with dependency)";
                   expected_cycles! {
                       "cortex-a53" => 9,
                   };
                   "fmul.4s v0, v0, v31",
                   "fmul.4s v0, v0, v31",
                   "fmul.4s v0, v0, v31",
                   ;
                   out("v0") _,
        );
    }
}

#[test]
fn fmul4_dep() {
    unsafe {
        benchmark!("4x fmul (with dependency)";
                   expected_cycles! {
                       "cortex-a53" => 13,
                   };
                   "fmul.4s v0, v0, v31",
                   "fmul.4s v0, v0, v31",
                   "fmul.4s v0, v0, v31",
                   "fmul.4s v0, v0, v31",
                   ;
                   out("v0") _,
        );
    }
}
