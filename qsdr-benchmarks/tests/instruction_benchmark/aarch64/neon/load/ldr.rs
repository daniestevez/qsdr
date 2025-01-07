use qsdr_benchmarks::{benchmark, expected_cycles, Buffer};

// Cortex-A53
//
// ldr to a Q register works in the same way as ld1 to a single V register.

#[test]
fn ldr_q() {
    unsafe {
        let buffer = Buffer::<f32>::new(4);
        benchmark!("1x ldr q";
                   expected_cycles! {
                       "cortex-a53" => 2,
                   };
                   "ldr q0, [{buff}]",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
        );
    }
}

#[test]
fn ldr_q_fadd_dep() {
    unsafe {
        let buffer = Buffer::<f32>::new(4);
        benchmark!("1x ldr q + dependent fadd";
                   expected_cycles! {
                       "cortex-a53" => 5,
                   };
                   "ldr q0, [{buff}]",
                   "fadd.4s v0, v0, v0",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
        );
    }
}

#[test]
fn ldr_q_x2() {
    unsafe {
        let buffer = Buffer::<f32>::new(4);
        benchmark!("2x ldr q";
                   expected_cycles! {
                       "cortex-a53" => 4,
                   };
                   "ldr q0, [{buff}]",
                   "ldr q0, [{buff}]",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
                   out("v1") _,
        );
    }
}

#[test]
fn ldr_q_x2_fadd_dep_1st() {
    unsafe {
        let buffer = Buffer::<f32>::new(4);
        benchmark!("2x ldr q + dependent fadd for 1st load";
                   expected_cycles! {
                       "cortex-a53" => 5,
                   };
                   "ldr q0, [{buff}]",
                   "ldr q1, [{buff}]",
                   "fadd.4s v0, v0, v0",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
                   out("v1") _,
        );
    }
}

#[test]
fn ldr_q_x2_fadd_dep_2nd() {
    unsafe {
        let buffer = Buffer::<f32>::new(8);
        benchmark!("2x ldr q + dependent fadd for 2nd load";
                   expected_cycles! {
                       "cortex-a53" => 7,
                   };
                   "ldr q0, [{buff}]",
                   "ldr q1, [{buff}]",
                   "fadd.4s v1, v1, v1",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
                   out("v1") _,
        );
    }
}

// Cortex-A53
//
// ldp to a pair of Q registers works in the same way as ld1 to a pair of V
// registers.

#[test]
fn ldp_q() {
    unsafe {
        let buffer = Buffer::<f32>::new(8);
        benchmark!("1x ldp q";
                   expected_cycles! {
                       "cortex-a53" => 4,
                   };
                   "ldp q0, q1, [{buff}]",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
                   out("v1") _,
        );
    }
}

#[test]
fn ldp_q_fadd_dep_1st() {
    unsafe {
        let buffer = Buffer::<f32>::new(8);
        benchmark!("1x ldp q + dependent fadd for 1st reg";
                   expected_cycles! {
                       "cortex-a53" => 5,
                   };
                   "ldp q0, q1, [{buff}]",
                   "fadd.4s v0, v0, v0",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
                   out("v1") _,
        );
    }
}

#[test]
fn ldp_q_fadd_dep_2nd() {
    unsafe {
        let buffer = Buffer::<f32>::new(8);
        benchmark!("1x ldp q + dependent fadd for 2nd reg";
                   expected_cycles! {
                       "cortex-a53" => 7,
                   };
                   "ldp q0, q1, [{buff}]",
                   "fadd.4s v1, v1, v1",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
                   out("v1") _,
        );
    }
}

#[test]
fn ldp_q_x2() {
    unsafe {
        let buffer = Buffer::<f32>::new(16);
        benchmark!("2x ldp q";
                   expected_cycles! {
                       "cortex-a53" => 8,
                   };
                   "ldp q0, q1, [{buff}]",
                   "ldp q2, q3, [{buff}]",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
                   out("v1") _,
                   out("v2") _,
                   out("v3") _,
        );
    }
}

#[test]
fn ldp_q_x2_fadd_dep_1st() {
    unsafe {
        let buffer = Buffer::<f32>::new(16);
        benchmark!("2x ldp q + dependent fadd for 1st reg";
                   expected_cycles! {
                       "cortex-a53" => 9,
                   };
                   "ldp q0, q1, [{buff}]",
                   "ldp q2, q3, [{buff}]",
                   "fadd.4s v0, v0, v0",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
                   out("v1") _,
                   out("v2") _,
                   out("v3") _,
        );
    }
}

#[test]
fn ldp_q_x2_fadd_dep_2nd() {
    unsafe {
        let buffer = Buffer::<f32>::new(16);
        benchmark!("2x ldp q + dependent fadd for 2nd reg";
                   expected_cycles! {
                       "cortex-a53" => 9,
                   };
                   "ldp q0, q1, [{buff}]",
                   "ldp q2, q3, [{buff}]",
                   "fadd.4s v1, v1, v1",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
                   out("v1") _,
                   out("v2") _,
                   out("v3") _,
        );
    }
}

#[test]
fn ldp_q_x2_fadd_dep_3rd() {
    unsafe {
        let buffer = Buffer::<f32>::new(16);
        benchmark!("2x ldp q + dependent fadd for 3rd reg";
                   expected_cycles! {
                       "cortex-a53" => 9,
                   };
                   "ldp q0, q1, [{buff}]",
                   "ldp q2, q3, [{buff}]",
                   "fadd.4s v2, v2, v2",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
                   out("v1") _,
                   out("v2") _,
                   out("v3") _,
        );
    }
}

#[test]
fn ldp_q_x2_fadd_dep_4th() {
    unsafe {
        let buffer = Buffer::<f32>::new(16);
        benchmark!("2x ldp q + dependent fadd for 4th reg";
                   expected_cycles! {
                       "cortex-a53" => 11,
                   };
                   "ldp q0, q1, [{buff}]",
                   "ldp q2, q3, [{buff}]",
                   "fadd.4s v3, v3, v3",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
                   out("v1") _,
                   out("v2") _,
                   out("v3") _,
        );
    }
}

// Cortex-A53
//
// ldr to a D register has an issue latency of 1 cycle and a result latency of 3
// cycles.

#[test]
fn ldr_d() {
    unsafe {
        let buffer = Buffer::<f32>::new(2);
        benchmark!("1x ldr d";
                   expected_cycles! {
                       "cortex-a53" => 1,
                   };
                   "ldr d0, [{buff}]",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
        );
    }
}

#[test]
fn ldr_d_fadd_dep() {
    unsafe {
        let buffer = Buffer::<f32>::new(2);
        benchmark!("1x ldr d + dependent fadd";
                   expected_cycles! {
                       "cortex-a53" => 4,
                   };
                   "ldr d0, [{buff}]",
                   "fadd.4s v0, v0, v0",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
        );
    }
}

#[test]
fn ldr_d_x2() {
    unsafe {
        let buffer = Buffer::<f32>::new(4);
        benchmark!("1x ldr d";
                   expected_cycles! {
                       "cortex-a53" => 2,
                   };
                   "ldr d0, [{buff}]",
                   "ldr d0, [{buff}]",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
                   out("v1") _,
        );
    }
}

#[test]
fn ldr_d_x2_fadd_dep_1st() {
    unsafe {
        let buffer = Buffer::<f32>::new(4);
        benchmark!("2x ldr d + dependent fadd for 1st load";
                   expected_cycles! {
                       "cortex-a53" => 4,
                   };
                   "ldr d0, [{buff}]",
                   "ldr d1, [{buff}]",
                   "fadd.4s v0, v0, v0",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
                   out("v1") _,
        );
    }
}

#[test]
fn ldr_d_x2_fadd_dep_2nd() {
    unsafe {
        let buffer = Buffer::<f32>::new(4);
        benchmark!("2x ldr d + dependent fadd for 2nd load";
                   expected_cycles! {
                       "cortex-a53" => 5,
                   };
                   "ldr d0, [{buff}]",
                   "ldr d1, [{buff}]",
                   "fadd.4s v1, v1, v1",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
                   out("v1") _,
        );
    }
}

#[test]
fn ldr_d_x3_fadd_dep_1st() {
    unsafe {
        let buffer = Buffer::<f32>::new(6);
        benchmark!("3x ldr d + dependent fadd for 1st load";
                   expected_cycles! {
                       "cortex-a53" => 4,
                   };
                   "ldr d0, [{buff}]",
                   "ldr d1, [{buff}]",
                   "ldr d2, [{buff}]",
                   "fadd.4s v0, v0, v0",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
                   out("v1") _,
                   out("v2") _,
        );
    }
}

// Cortex-A53
//
// ldp to 2 registers has an issue latency of 2 cycles, the first result has a
// latency of 3 cyles, and the second result has a latency of 4 cycles.

#[test]
fn ldp_d() {
    unsafe {
        let buffer = Buffer::<f32>::new(4);
        benchmark!("1x ldp d";
                   expected_cycles! {
                       "cortex-a53" => 2,
                   };
                   "ldp d0, d1, [{buff}]",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
                   out("v1") _,
        );
    }
}

#[test]
fn ldp_d_fadd_dep_1st() {
    unsafe {
        let buffer = Buffer::<f32>::new(4);
        benchmark!("1x ldp d + dependent fadd for 1st reg";
                   expected_cycles! {
                       "cortex-a53" => 4,
                   };
                   "ldp d0, d1, [{buff}]",
                   "fadd.4s v0, v0, v0",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
                   out("v1") _,
        );
    }
}

#[test]
fn ldp_d_fadd_dep_2nd() {
    unsafe {
        let buffer = Buffer::<f32>::new(4);
        benchmark!("1x ldp d + dependent fadd for 2nd reg";
                   expected_cycles! {
                       "cortex-a53" => 4,
                   };
                   "ldp d0, d1, [{buff}]",
                   "fadd.4s v0, v0, v0",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
                   out("v1") _,
        );
    }
}

#[test]
fn ldp_d_x2() {
    unsafe {
        let buffer = Buffer::<f32>::new(8);
        benchmark!("2x ldp d";
                   expected_cycles! {
                       "cortex-a53" => 4,
                   };
                   "ldp d0, d1, [{buff}]",
                   "ldp d2, d3, [{buff}]",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
                   out("v1") _,
                   out("v2") _,
                   out("v3") _,
        );
    }
}

#[test]
fn ldp_d_x2_fadd_dep_1st() {
    unsafe {
        let buffer = Buffer::<f32>::new(8);
        benchmark!("2x ldp d + dependent fadd for 1st reg";
                   expected_cycles! {
                       "cortex-a53" => 5,
                   };
                   "ldp d0, d1, [{buff}]",
                   "ldp d2, d3, [{buff}]",
                   "fadd.4s v0, v0, v0",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
                   out("v1") _,
                   out("v2") _,
                   out("v3") _,
        );
    }
}

#[test]
fn ldp_d_x2_fadd_dep_2nd() {
    unsafe {
        let buffer = Buffer::<f32>::new(8);
        benchmark!("2x ldp d + dependent fadd for 2nd reg";
                   expected_cycles! {
                       "cortex-a53" => 5,
                   };
                   "ldp d0, d1, [{buff}]",
                   "ldp d2, d3, [{buff}]",
                   "fadd.4s v1, v1, v1",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
                   out("v1") _,
                   out("v2") _,
                   out("v3") _,
        );
    }
}

#[test]
fn ldp_d_x2_fadd_dep_3rd() {
    unsafe {
        let buffer = Buffer::<f32>::new(8);
        benchmark!("2x ldp d + dependent fadd for 3rd reg";
                   expected_cycles! {
                       "cortex-a53" => 6,
                   };
                   "ldp d0, d1, [{buff}]",
                   "ldp d2, d3, [{buff}]",
                   "fadd.4s v2, v2, v2",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
                   out("v1") _,
                   out("v2") _,
                   out("v3") _,
        );
    }
}

#[test]
fn ldp_d_x2_fadd_dep_4th() {
    unsafe {
        let buffer = Buffer::<f32>::new(8);
        benchmark!("2x ldp d + dependent fadd for 4th reg";
                   expected_cycles! {
                       "cortex-a53" => 7,
                   };
                   "ldp d0, d1, [{buff}]",
                   "ldp d2, d3, [{buff}]",
                   "fadd.4s v3, v3, v3",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
                   out("v1") _,
                   out("v2") _,
                   out("v3") _,
        );
    }
}
