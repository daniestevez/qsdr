use qsdr_benchmarks::{benchmark, expected_cycles, Buffer};

// Cortex-A53
//
// st1 has an issue latency equal to the number of V registers that are
// stored. There is no difference between splitting the store in multiple st1
// instructions addressing less registers (besides the number of instructions).

#[test]
fn st1_1() {
    unsafe {
        let buffer = Buffer::<f32>::new(4);
        benchmark!("1x st1{1}";
                   expected_cycles! {
                       "cortex-a53" => 1,
                   };
                   "st1.4s {{v0}}, [{buff}]",
                   ;
                   buff = in(reg) buffer.as_ptr(),
        );
    }
}

#[test]
fn st1_1_x2() {
    unsafe {
        let buffer = Buffer::<f32>::new(8);
        benchmark!("2x st1{1}";
                   expected_cycles! {
                       "cortex-a53" => 2,
                   };
                   "st1.4s {{v0}}, [{buff}]",
                   "st1.4s {{v1}}, [{buff}]",
                   ;
                   buff = in(reg) buffer.as_ptr(),
        );
    }
}

#[test]
fn st1_1_x3() {
    unsafe {
        let buffer = Buffer::<f32>::new(12);
        benchmark!("4x st1{1}";
                   expected_cycles! {
                       "cortex-a53" => 3,
                   };
                   "st1.4s {{v0}}, [{buff}]",
                   "st1.4s {{v1}}, [{buff}]",
                   "st1.4s {{v2}}, [{buff}]",
                   ;
                   buff = in(reg) buffer.as_ptr(),
        );
    }
}

#[test]
fn st1_1_x4() {
    unsafe {
        let buffer = Buffer::<f32>::new(16);
        benchmark!("4x st1{1}";
                   expected_cycles! {
                       "cortex-a53" => 4,
                   };
                   "st1.4s {{v0}}, [{buff}]",
                   "st1.4s {{v1}}, [{buff}]",
                   "st1.4s {{v2}}, [{buff}]",
                   "st1.4s {{v3}}, [{buff}]",
                   ;
                   buff = in(reg) buffer.as_ptr(),
        );
    }
}

#[test]
fn st1_2() {
    unsafe {
        let buffer = Buffer::<f32>::new(8);
        benchmark!("1x st1{2}";
                   expected_cycles! {
                       "cortex-a53" => 2,
                   };
                   "st1.4s {{v0-v1}}, [{buff}]",
                   ;
                   buff = in(reg) buffer.as_ptr(),
        );
    }
}

#[test]
fn st1_2_x2() {
    unsafe {
        let buffer = Buffer::<f32>::new(16);
        benchmark!("2x st1{2}";
                   expected_cycles! {
                       "cortex-a53" => 4,
                   };
                   "st1.4s {{v0-v1}}, [{buff}]",
                   "st1.4s {{v2-v3}}, [{buff}]",
                   ;
                   buff = in(reg) buffer.as_ptr(),
        );
    }
}

#[test]
fn st1_4() {
    unsafe {
        let buffer = Buffer::<f32>::new(16);
        benchmark!("1x st1{4}";
                   expected_cycles! {
                       "cortex-a53" => 4,
                   };
                   "st1.4s {{v0-v3}}, [{buff}]",
                   ;
                   buff = in(reg) buffer.as_ptr(),
        );
    }
}

#[test]
fn st1_8() {
    unsafe {
        let buffer = Buffer::<f32>::new(32);
        benchmark!("2x st1{4}";
                   expected_cycles! {
                       "cortex-a53" => 8,
                   };
                   "st1.4s {{v0-v3}}, [{buff}]",
                   "st1.4s {{v4-v7}}, [{buff}]",
                   ;
                   buff = in(reg) buffer.as_ptr(),
        );
    }
}
