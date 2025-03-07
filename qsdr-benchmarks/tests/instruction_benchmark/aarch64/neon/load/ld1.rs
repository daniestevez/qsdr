use qsdr_benchmarks::{Buffer, benchmark, expected_cycles};

// Cortex-A53
//
// Generally loading V registers with ld1 has an issue latency of 2 cycles
// per register and a result latency of 4 cycles per register, regardless of
// whether the loads are divided into multiple ld1 instructions loading
// fewer registers or a single ld1 instruction loading more registers.
//
// This makes sense because the L1 cache has a 64-bit read path
// https://developer.arm.com/documentation/ddi0500/e/level-1-memory-system/about-the-l1-memory-system?
//
// When loading multiple registers in a single instruction, the results are
// available as if they were loaded in the given order (the 1st register
// after 4 cycles, the 2nd after 6, the 3rd after 8, etc).

#[test]
fn ld1_1() {
    unsafe {
        let buffer = Buffer::<f32>::new(4);
        benchmark!("1x ld1{1}";
                   expected_cycles! {
                       "cortex-a53" => 2,
                   };
                   "ld1.4s {{v0}}, [{buff}]",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
        );
    }
}

#[test]
fn ld1_1_fadd_dep() {
    unsafe {
        let buffer = Buffer::<f32>::new(4);
        benchmark!("1x ld1{1} + dependent fadd";
                   expected_cycles! {
                       "cortex-a53" => 5,
                   };
                   "ld1.4s {{v0}}, [{buff}]",
                   "fadd.4s v0, v0, v0",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
        );
    }
}

#[test]
fn ld1_1_x2() {
    unsafe {
        let buffer = Buffer::<f32>::new(8);
        benchmark!("2x ld1{1}";
                   expected_cycles! {
                       "cortex-a53" => 4,
                   };
                   "ld1.4s {{v0}}, [{buff}]",
                   "ld1.4s {{v1}}, [{buff}]",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
                   out("v1") _,
        );
    }
}

#[test]
fn ld1_1_x2_fadd_dep_1st() {
    unsafe {
        let buffer = Buffer::<f32>::new(8);
        benchmark!("2x ld1{1} + dependent fadd for 1st load";
                   expected_cycles! {
                       "cortex-a53" => 5,
                   };
                   "ld1.4s {{v0}}, [{buff}]",
                   "ld1.4s {{v1}}, [{buff}]",
                   "fadd.4s v0, v0, v0",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
                   out("v1") _,
        );
    }
}

#[test]
fn ld1_1_x2_fadd_dep_2nd() {
    unsafe {
        let buffer = Buffer::<f32>::new(8);
        benchmark!("2x ld1{1} + dependent fadd for 2nd load";
                   expected_cycles! {
                       "cortex-a53" => 7,
                   };
                   "ld1.4s {{v0}}, [{buff}]",
                   "ld1.4s {{v1}}, [{buff}]",
                   "fadd.4s v1, v1, v1",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
                   out("v1") _,
        );
    }
}

#[test]
fn ld1_2() {
    unsafe {
        let buffer = Buffer::<f32>::new(8);
        benchmark!("1x ld1{2}";
                   expected_cycles! {
                       "cortex-a53" => 4,
                   };
                   "ld1.4s {{v0-v1}}, [{buff}]",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
                   out("v1") _,
        );
    }
}

#[test]
fn ld1_2_fadd_dep_1st() {
    unsafe {
        let buffer = Buffer::<f32>::new(8);
        benchmark!("1x ld1{2} + dependent fadd for 1st reg";
                   expected_cycles! {
                       "cortex-a53" => 5,
                   };
                   "ld1.4s {{v0-v1}}, [{buff}]",
                   "fadd.4s v0, v0, v0",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
                   out("v1") _,
        );
    }
}

#[test]
fn ld1_2_fadd_dep_2nd() {
    unsafe {
        let buffer = Buffer::<f32>::new(8);
        benchmark!("1x ld1{2} + dependent fadd for 2nd reg";
                   expected_cycles! {
                       "cortex-a53" => 7,
                   };
                   "ld1.4s {{v0-v1}}, [{buff}]",
                   "fadd.4s v1, v1, v1",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
                   out("v1") _,
        );
    }
}

#[test]
fn ld1_2_x2() {
    unsafe {
        let buffer = Buffer::<f32>::new(16);
        benchmark!("2x ld1{2}";
                   expected_cycles! {
                       "cortex-a53" => 8,
                   };
                   "ld1.4s {{v0-v1}}, [{buff}]",
                   "ld1.4s {{v2-v3}}, [{buff}]",
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
fn ld1_2_x2_fadd_dep_1st() {
    unsafe {
        let buffer = Buffer::<f32>::new(16);
        benchmark!("2x ld1{2} + dependent fadd for 1st reg";
                   expected_cycles! {
                       "cortex-a53" => 9,
                   };
                   "ld1.4s {{v0-v1}}, [{buff}]",
                   "ld1.4s {{v2-v3}}, [{buff}]",
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
fn ld1_2_x2_fadd_dep_2nd() {
    unsafe {
        let buffer = Buffer::<f32>::new(16);
        benchmark!("2x ld1{2} + dependent fadd for 2nd reg";
                   expected_cycles! {
                       "cortex-a53" => 9,
                   };
                   "ld1.4s {{v0-v1}}, [{buff}]",
                   "ld1.4s {{v2-v3}}, [{buff}]",
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
fn ld1_2_x2_fadd_dep_3rd() {
    unsafe {
        let buffer = Buffer::<f32>::new(16);
        benchmark!("2x ld1{2} + dependent fadd for 3rd reg";
                   expected_cycles! {
                       "cortex-a53" => 9,
                   };
                   "ld1.4s {{v0-v1}}, [{buff}]",
                   "ld1.4s {{v2-v3}}, [{buff}]",
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
fn ld1_2_x2_fadd_dep_4th() {
    unsafe {
        let buffer = Buffer::<f32>::new(16);
        benchmark!("2x ld1{2} + dependent fadd for 4th reg";
                   expected_cycles! {
                       "cortex-a53" => 11,
                   };
                   "ld1.4s {{v0-v1}}, [{buff}]",
                   "ld1.4s {{v2-v3}}, [{buff}]",
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

#[test]
fn ld1_4() {
    unsafe {
        let buffer = Buffer::<f32>::new(16);
        benchmark!("1x ld1{4}";
                   expected_cycles! {
                       "cortex-a53" => 8,
                   };
                   "ld1.4s {{v0-v3}}, [{buff}]",
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
fn ld1_4_fadd_dep_1st() {
    unsafe {
        let buffer = Buffer::<f32>::new(16);
        benchmark!("1x ld1{4} + dependent fadd for 1st reg";
                   expected_cycles! {
                       "cortex-a53" => 9,
                   };
                   "ld1.4s {{v0-v3}}, [{buff}]",
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
fn ld1_4_fadd_dep_2nd() {
    unsafe {
        let buffer = Buffer::<f32>::new(16);
        benchmark!("1x ld1{4} + dependent fadd for 2nd reg";
                   expected_cycles! {
                       "cortex-a53" => 9,
                   };
                   "ld1.4s {{v0-v3}}, [{buff}]",
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
fn ld1_4_fadd_dep_3rd() {
    unsafe {
        let buffer = Buffer::<f32>::new(16);
        benchmark!("1x ld1{4} + dependent fadd for 3rd reg";
                   expected_cycles! {
                       "cortex-a53" => 9,
                   };
                   "ld1.4s {{v0-v3}}, [{buff}]",
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
fn ld1_4_fadd_dep_4th() {
    unsafe {
        let buffer = Buffer::<f32>::new(16);
        benchmark!("1x ld1{4} + dependent fadd for 4th reg";
                   expected_cycles! {
                       "cortex-a53" => 11,
                   };
                   "ld1.4s {{v0-v3}}, [{buff}]",
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
