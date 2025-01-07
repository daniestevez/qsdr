use qsdr_benchmarks::{benchmark, expected_cycles, Buffer};

// Cortex-A53
//
// ldr to a D register does not dual issue with NEON arithmetic operations such
// as fadd.

#[test]
fn ldr_d_fadd() {
    unsafe {
        let buffer = Buffer::<f32>::new(2);
        benchmark!("ldr d + fadd";
                   expected_cycles! {
                       "cortex-a53" => 2,
                   };
                   "ldr d0, [{buff}]",
                   "fadd.4s v1, v1, v1",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
                   out("v1") _,
        );
    }
}

#[test]
fn fadd_ldr_d() {
    unsafe {
        let buffer = Buffer::<f32>::new(2);
        benchmark!("fadd + ldr d";
                   expected_cycles! {
                       "cortex-a53" => 2,
                   };
                   "fadd.4s v1, v1, v1",
                   "ldr d0, [{buff}]",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
                   out("v1") _,
        );
    }
}

// Cortex-A53
//
// ldr to a GPR X register does dual issue with a NEON arithmetic operation
// such as fadd if it is after it, but not if it is before it.

#[test]
fn ldr_x_fadd() {
    unsafe {
        let buffer = Buffer::<u64>::new(1);
        benchmark!("ldr x + fadd";
                   expected_cycles! {
                       "cortex-a53" => 2,
                   };
                   "ldr x0, [{buff}]",
                   "fadd.4s v0, v0, v0",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
                   out("x0") _,
        );
    }
}

#[test]
fn fadd_ldr_x() {
    unsafe {
        let buffer = Buffer::<u64>::new(1);
        benchmark!("fadd + ldr x";
                   expected_cycles! {
                       "cortex-a53" => 1,
                   };
                   "fadd.4s v0, v0, v0",
                   "ldr x0, [{buff}]",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
                   out("v1") _,
        );
    }
}

// Cortex-A53
//
// ldr to a D register and ins to a different register can dual-issue regardless
// of the order.

#[test]
fn ldr_d_ins() {
    unsafe {
        let buffer = Buffer::<f32>::new(2);
        benchmark!("ldr d + ins";
                   expected_cycles! {
                       "cortex-a53" => 1,
                   };
                   "ldr d0, [{buff}]",
                   "ins v1.d[1], x0",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
                   out("v1") _,
        );
    }
}

#[test]
fn ins_ldr_d() {
    unsafe {
        let buffer = Buffer::<f32>::new(2);
        benchmark!("ins + ldr d";
                   expected_cycles! {
                       "cortex-a53" => 1,
                   };
                   "ins v1.d[1], x0",
                   "ldr d0, [{buff}]",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
                   out("v1") _,
        );
    }
}

#[test]
fn ldr_d_ins_same_reg() {
    unsafe {
        let buffer = Buffer::<f32>::new(2);
        benchmark!("ldr d + ins (different halves of same register)";
                   expected_cycles! {
                       "cortex-a53" => 2,
                   };
                   "ldr d0, [{buff}]",
                   "ins v0.d[1], x0",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
        );
    }
}

#[test]
fn ins_ldr_d_same_reg() {
    unsafe {
        let buffer = Buffer::<f32>::new(2);
        benchmark!("ins + ldr d (different halves of same register)";
                   expected_cycles! {
                       "cortex-a53" => 2,
                   };
                   "ldr d0, [{buff}]",
                   "ins v0.d[1], x0",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
        );
    }
}

// Cortex-A53
//
// ld1 to a D register and ins to a different register cannot dual issue

#[test]
fn ld1_d_ins() {
    unsafe {
        let buffer = Buffer::<f32>::new(2);
        benchmark!("ld1 d + ins";
                   expected_cycles! {
                       "cortex-a53" => 2,
                   };
                   "ld1 {{v0.d}}[0], [{buff}]",
                   "ins v1.d[1], x0",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
                   out("v1") _,
        );
    }
}

#[test]
fn ins_ld1_d() {
    unsafe {
        let buffer = Buffer::<f32>::new(2);
        benchmark!("ins + ld1 d";
                   expected_cycles! {
                       "cortex-a53" => 2,
                   };
                   "ins v1.d[1], x0",
                   "ld1 {{v0.d}}[0], [{buff}]",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
                   out("v1") _,
        );
    }
}

// Cortex-A53
//
// str of a Q register cannot dual issue with an arithmetic NEON instruction
// such as fadd.

#[test]
fn fadd_str_q() {
    unsafe {
        let buffer = Buffer::<f32>::new(4);
        benchmark!("fadd + str q";
                   expected_cycles! {
                       "cortex-a53" => 2,
                   };
                   "fadd.4s v0, v0, v0",
                   "str q1, [{buff}]",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
        );
    }
}

#[test]
fn str_q_fadd() {
    unsafe {
        let buffer = Buffer::<f32>::new(4);
        benchmark!("str q + fadd";
                   expected_cycles! {
                       "cortex-a53" => 2,
                   };
                   "str q1, [{buff}]",
                   "fadd.4s v0, v0, v0",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
        );
    }
}

// Cortex-A53
//
// ldr to a d register and str to a q register cannot dual-issue.

#[test]
fn ldr_d_str_q() {
    unsafe {
        let buffer = Buffer::<f32>::new(8);
        benchmark!("ldr d + str q";
                   expected_cycles! {
                       "cortex-a53" => 2,
                   };
                   "ldr d0, [{buff}]",
                   "str q1, [{buff}]",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
        );
    }
}

#[test]
fn str_q_ldr_d() {
    unsafe {
        let buffer = Buffer::<f32>::new(8);
        benchmark!("str q + ldr d";
                   expected_cycles! {
                       "cortex-a53" => 2,
                   };
                   "str q1, [{buff}]",
                   "ldr d0, [{buff}, #16]",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
        );
    }
}

// Cortex-A53
//
// ins and str to a q register cannot dual-issue

#[test]
fn ins_str_q() {
    unsafe {
        let buffer = Buffer::<f32>::new(4);
        benchmark!("ins + str q";
                   expected_cycles! {
                       "cortex-a53" => 2,
                   };
                   "ins v0.d[1], x0",
                   "str q1, [{buff}]",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
        );
    }
}

#[test]
fn str_q_ins() {
    unsafe {
        let buffer = Buffer::<f32>::new(4);
        benchmark!("str q + ins";
                   expected_cycles! {
                       "cortex-a53" => 2,
                   };
                   "str q1, [{buff}]",
                   "ins v0.d[1], x0",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
        );
    }
}

// Cortex-A53
//
// ldr and str of d registers cannot dual-issue.

#[test]
fn ldr_d_str_d() {
    unsafe {
        let buffer = Buffer::<f32>::new(32);
        benchmark!("ldr d + str d";
                   expected_cycles! {
                       "cortex-a53" => 2,
                   };
                   "ldr d0, [{buff}]",
                   "str d1, [{buff}, #64]",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
        );
    }
}

#[test]
fn str_d_ldr_d() {
    unsafe {
        let buffer = Buffer::<f32>::new(32);
        benchmark!("ldr d + str d";
                   expected_cycles! {
                       "cortex-a53" => 2,
                   };
                   "str d1, [{buff}, #64]",
                   "ldr d0, [{buff}]",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
        );
    }
}

// Cortex-A53
//
// str of a GPR X register and a NEON arithmetic instruction such as fadd can
// dual-issue, but only if the str comes after the fadd.

#[test]
fn fadd_str_x() {
    unsafe {
        let buffer = Buffer::<u64>::new(1);
        benchmark!("fadd + str x";
                   expected_cycles! {
                       "cortex-a53" => 1,
                   };
                   "fadd.4s v0, v0, v0",
                   "str x0, [{buff}]",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
        );
    }
}

#[test]
fn str_x_fadd() {
    unsafe {
        let buffer = Buffer::<u64>::new(1);
        benchmark!("str x + fadd";
                   expected_cycles! {
                       "cortex-a53" => 2,
                   };
                   "str x0, [{buff}]",
                   "fadd.4s v0, v0, v0",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
        );
    }
}

// Cortex-A53
//
// mov from a V register to a GPR X register and a NEON arithmetic instruction
// such as fadd cannot dual-issue.

#[test]
fn fadd_mov_d_x() {
    unsafe {
        benchmark!("fadd + str x";
                   expected_cycles! {
                       "cortex-a53" => 2,
                   };
                   "fadd.4s v0, v0, v0",
                   "mov x0, v1.d[0]",
                   ;
                   out("v0") _,
                   out("x0") _,
        );
    }
}

#[test]
fn mov_d_x_fadd() {
    unsafe {
        benchmark!("str x + fadd";
                   expected_cycles! {
                       "cortex-a53" => 2,
                   };
                   "mov x0, v1.d[0]",
                   "fadd.4s v0, v0, v0",
                   ;
                   out("v0") _,
                   out("x0") _,
        );
    }
}

// Cortex-A53
//
// ldr to a D register and a NEON arithmetic instruction such as fadd using a
// different D register can dual-issue.

#[test]
fn fadd_d_ldr_d_x() {
    unsafe {
        let buffer = Buffer::<f32>::new(2);
        benchmark!("fadd d + ldr d";
                   expected_cycles! {
                       "cortex-a53" => 1,
                   };
                   "fadd.2s v0, v0, v0",
                   "ldr d1, [{buff}]",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
                   out("v1") _,
        );
    }
}

#[test]
fn ldr_d_fadd_d_x() {
    unsafe {
        let buffer = Buffer::<f32>::new(2);
        benchmark!("ldr d + fadd";
                   expected_cycles! {
                       "cortex-a53" => 1,
                   };
                   "ldr d1, [{buff}]",
                   "fadd.2s v0, v0, v0",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v0") _,
                   out("v1") _,
        );
    }
}
