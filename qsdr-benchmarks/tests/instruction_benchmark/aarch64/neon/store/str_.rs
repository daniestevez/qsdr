use qsdr_benchmarks::{benchmark, expected_cycles, Buffer};

// Cortex-A53
//
// str to a Q register works in the same way as st1 to a single V register.

#[test]
fn str_q() {
    unsafe {
        let buffer = Buffer::<f32>::new(4);
        benchmark!("1x str q";
                   expected_cycles! {
                       "cortex-a53" => 1,
                   };
                   "str q0, [{buff}]",
                   ;
                   buff = in(reg) buffer.as_ptr(),
        );
    }
}

#[test]
fn str_q_x2() {
    unsafe {
        let buffer = Buffer::<f32>::new(8);
        benchmark!("2x str q";
                   expected_cycles! {
                       "cortex-a53" => 2,
                   };
                   "str q0, [{buff}]",
                   "str q1, [{buff}]",
                   ;
                   buff = in(reg) buffer.as_ptr(),
        );
    }
}

// Cortex-A53
//
// stp to a pair of Q registers works in the same way as st1 to a pair of V
// registers.

#[test]
fn stp_q() {
    unsafe {
        let buffer = Buffer::<f32>::new(8);
        benchmark!("1x stp q";
                   expected_cycles! {
                       "cortex-a53" => 2,
                   };
                   "stp q0, q1, [{buff}]",
                   ;
                   buff = in(reg) buffer.as_ptr(),
        );
    }
}

#[test]
fn stp_q_x2() {
    unsafe {
        let buffer = Buffer::<f32>::new(16);
        benchmark!("2x stp q";
                   expected_cycles! {
                       "cortex-a53" => 4,
                   };
                   "stp q0, q1, [{buff}]",
                   "stp q2, q3, [{buff}]",
                   ;
                   buff = in(reg) buffer.as_ptr(),
        );
    }
}

#[test]
fn stp_q_fadd() {
    unsafe {
        let buffer = Buffer::<f32>::new(8);
        benchmark!("1x stp q + fadd";
                   expected_cycles! {
                       "cortex-a53" => 3,
                   };
                   "stp q0, q1, [{buff}]",
                   "fadd.4s v2, v2, v2",
                   ;
                   buff = in(reg) buffer.as_ptr(),
                   out("v2") _,
        );
    }
}

// Cortex-A53
//
// Storing D registers works in the same way as stroing Q registers. In
// particular, two stores to D registers cannot dual-issue even though the write
// path to L1 cache is 128-bit wide.

#[test]
fn str_d() {
    unsafe {
        let buffer = Buffer::<f32>::new(2);
        benchmark!("1x str 2";
                   expected_cycles! {
                       "cortex-a53" => 1,
                   };
                   "str d0, [{buff}]",
                   ;
                   buff = in(reg) buffer.as_ptr(),
        );
    }
}

#[test]
fn str_d_x2() {
    unsafe {
        let buffer = Buffer::<f32>::new(4);
        benchmark!("2x str 2";
                   expected_cycles! {
                       "cortex-a53" => 2,
                   };
                   "str d0, [{buff}]",
                   "str d1, [{buff}]",
                   ;
                   buff = in(reg) buffer.as_ptr(),
        );
    }
}
