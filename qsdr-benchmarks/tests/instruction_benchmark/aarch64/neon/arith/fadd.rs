use qsdr_benchmarks::{benchmark, expected_cycles};

// Cortex-A53
//
// fadd issue latency 1, result latency 4

#[test]
fn fadd1() {
    unsafe {
        benchmark!("1x fadd";
                   expected_cycles! {
                       "cortex-a53" => 1,
                   };
                   "fadd.4s v0, v0, v31",
                   ;
                   out("v0") _,
        );
    }
}

#[test]
fn fadd2_nodep() {
    unsafe {
        benchmark!("2x fadd (no dependencies)";
                   expected_cycles! {
                       "cortex-a53" => 2,
                   };
                   "fadd.4s v0, v0, v31",
                   "fadd.4s v1, v1, v31",
                   ;
                   out("v0") _,
                   out("v1") _,
        );
    }
}

#[test]
fn fadd3_nodep() {
    unsafe {
        benchmark!("3x fadd (no dependencies)";
                   expected_cycles! {
                       "cortex-a53" => 3,
                   };
                   "fadd.4s v0, v0, v31",
                   "fadd.4s v1, v1, v31",
                   "fadd.4s v2, v2, v31",
                   ;
                   out("v0") _,
                   out("v1") _,
                   out("v2") _,
        );
    }
}

#[test]
fn fadd4_nodep() {
    unsafe {
        benchmark!("4x fadd (no dependencies)";
                   expected_cycles! {
                       "cortex-a53" => 4,
                   };
                   "fadd.4s v0, v0, v31",
                   "fadd.4s v1, v1, v31",
                   "fadd.4s v2, v2, v31",
                   "fadd.4s v3, v3, v31",
                   ;
                   out("v0") _,
                   out("v1") _,
                   out("v2") _,
                   out("v3") _,
        );
    }
}

#[test]
fn fadd2_dep() {
    unsafe {
        benchmark!("2x fadd (with dependency)";
                   expected_cycles! {
                       "cortex-a53" => 5,
                   };
                   "fadd.4s v0, v0, v31",
                   "fadd.4s v0, v0, v31",
                   ;
                   out("v0") _,
        );
    }
}

#[test]
fn fadd3_dep() {
    unsafe {
        benchmark!("3x fadd (with dependency)";
                   expected_cycles! {
                       "cortex-a53" => 9,
                   };
                   "fadd.4s v0, v0, v31",
                   "fadd.4s v0, v0, v31",
                   "fadd.4s v0, v0, v31",
                   ;
                   out("v0") _,
        );
    }
}

#[test]
fn fadd4_dep() {
    unsafe {
        benchmark!("4x fadd (with dependency)";
                   expected_cycles! {
                       "cortex-a53" => 13,
                   };
                   "fadd.4s v0, v0, v31",
                   "fadd.4s v0, v0, v31",
                   "fadd.4s v0, v0, v31",
                   "fadd.4s v0, v0, v31",
                   ;
                   out("v0") _,
        );
    }
}

#[test]
fn fmul4_fadd4() {
    unsafe {
        benchmark!("4x fmul + 4x fadd";
                   expected_cycles! {
                       "cortex-a53" => 8,
                   };
                   "fmul.4s v0, v0, v31",
                   "fmul.4s v1, v1, v31",
                   "fmul.4s v2, v2, v31",
                   "fmul.4s v3, v3, v31",
                   "fadd.4s v0, v0, v31",
                   "fadd.4s v1, v1, v31",
                   "fadd.4s v2, v2, v31",
                   "fadd.4s v3, v3, v31",
                   ;
                   out("v0") _,
                   out("v1") _,
                   out("v2") _,
                   out("v3") _,
        );
    }
}
