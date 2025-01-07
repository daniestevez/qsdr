use qsdr_benchmarks::{benchmark, expected_cycles};

// Cortex-A53
//
// ins has an issue latency of 1 cycle and a result latency of 2 cycles.
// Moreover, it is possible to dual-issue two ins instructions to different
// registers.

#[test]
fn ins() {
    unsafe {
        benchmark!("1x ins";
                   expected_cycles! {
                       "cortex-a53" => 1,
                   };
                   "ins v0.d[1], x0",
                   ;
                   out("v0") _,
        );
    }
}

#[test]
fn ins_fadd_dep() {
    unsafe {
        benchmark!("1x ins + dependent fadd";
                   expected_cycles! {
                       "cortex-a53" => 3,
                   };
                   "ins v0.d[1], x0",
                   "fadd.4s v0, v0, v0",
                   ;
                   out("v0") _,
        );
    }
}

#[test]
fn ins_2x() {
    unsafe {
        benchmark!("2x ins";
                   expected_cycles! {
                       "cortex-a53" => 1,
                   };
                   "ins v0.d[1], x0",
                   "ins v1.d[1], x1",
                   ;
                   out("v0") _,
                   out("v1") _,
        );
    }
}

#[test]
fn ins_2x_same_dest() {
    unsafe {
        benchmark!("2x ins same destination register";
                   expected_cycles! {
                       "cortex-a53" => 2,
                   };
                   "ins v0.d[0], x0",
                   "ins v0.d[1], x1",
                   ;
                   out("v0") _,
        );
    }
}

#[test]
fn ins_4x_fadd_dep() {
    unsafe {
        benchmark!("4x ins + dependent fadd 1st reg";
                   expected_cycles! {
                       "cortex-a53" => 3,
                   };
                   "ins v0.d[1], x0",
                   "ins v1.d[1], x1",
                   "ins v2.d[1], x2",
                   "ins v3.d[1], x3",
                   "fadd.4s v0, v0, v0",
                   ;
                   out("v0") _,
                   out("v1") _,
                   out("v2") _,
                   out("v3") _,
        );
    }
}
