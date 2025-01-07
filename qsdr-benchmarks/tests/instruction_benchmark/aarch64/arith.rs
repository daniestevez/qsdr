use qsdr_benchmarks::{benchmark, expected_cycles};

// Cortex-A53
//
// add issue latency 1, result latency 1, can dual-issue with another add
// without data dependencies. It can even dual-issue with another add with data
// dependencies, but there are some limitations (see below for specific
// examples).

#[test]
fn add1() {
    unsafe {
        benchmark!("1x add";
                   expected_cycles! {
                       "cortex-a53" => 1,
                   };
                   "add x0, x0, x0",
                   ;
                   out("x0") _,
        );
    }
}

#[test]
fn add2() {
    unsafe {
        benchmark!("2x add";
                   expected_cycles! {
                       "cortex-a53" => 1,
                   };
                   "add x0, x0, x0",
                   "add x1, x1, x1",
                   ;
                   out("x0") _,
                   out("x1") _,
        );
    }
}

#[test]
fn add3() {
    unsafe {
        benchmark!("3x add";
                   expected_cycles! {
                       "cortex-a53" => 2,
                   };
                   "add x0, x0, x0",
                   "add x1, x1, x1",
                   "add x2, x2, x2",
                   ;
                   out("x0") _,
                   out("x1") _,
                   out("x2") _,
        );
    }
}

#[test]
fn add4() {
    unsafe {
        benchmark!("4x add";
                   expected_cycles! {
                       "cortex-a53" => 2,
                   };
                   "add x0, x0, x0",
                   "add x1, x1, x1",
                   "add x2, x2, x2",
                   "add x3, x3, x3",
                   ;
                   out("x0") _,
                   out("x1") _,
                   out("x2") _,
                   out("x3") _,
        );
    }
}

#[test]
fn add_dep2() {
    unsafe {
        benchmark!("2x add (dependent)";
                   expected_cycles! {
                       // the two adds dual-issue
                       "cortex-a53" => 1,
                   };
                   "add x0, x0, x0",
                   "add x0, x0, x0",
                   ;
                   out("x0") _,
        );
    }
}

#[test]
fn add_dep3() {
    unsafe {
        benchmark!("3x add (dependent)";
                   expected_cycles! {
                       // two first two adds dual-issue
                       "cortex-a53" => 2,
                   };
                   "add x0, x0, x0",
                   "add x0, x0, x0",
                   "add x0, x0, x0",
                   ;
                   out("x0") _,
        );
    }
}

#[test]
fn add_dep4() {
    unsafe {
        benchmark!("4x add (dependent)";
                   expected_cycles! {
                       // the first two adds dual-issue, but the remaining don't
                       "cortex-a53" => 3,
                   };
                   "add x0, x0, x0",
                   "add x0, x0, x0",
                   "add x0, x0, x0",
                   "add x0, x0, x0",
                   ;
                   out("x0") _,
        );
    }
}

#[test]
fn add_dep5() {
    unsafe {
        benchmark!("5x add (dependent)";
                   expected_cycles! {
                       // the first two adds dual-issue, but the remaining don't
                       "cortex-a53" => 4,
                   };
                   "add x0, x0, x0",
                   "add x0, x0, x0",
                   "add x0, x0, x0",
                   "add x0, x0, x0",
                   "add x0, x0, x0",
                   ;
                   out("x0") _,
        );
    }
}

#[test]
fn add_pair_dep2() {
    unsafe {
        benchmark!("add pair 2x (with dependencies)";
                   expected_cycles! {
                       "cortex-a53" => 2,
                   };
                   "add x0, x0, x0",
                   "add x1, x1, x1",
                   "add x0, x0, x0",
                   "add x1, x1, x1",
                   ;
                   out("x0") _,
                   out("x1") _,
        );
    }
}

#[test]
fn add_pair_dep3() {
    unsafe {
        benchmark!("add pair 3x (with dependencies)";
                   expected_cycles! {
                       "cortex-a53" => 3,
                   };
                   "add x0, x0, x0",
                   "add x1, x1, x1",
                   "add x0, x0, x0",
                   "add x1, x1, x1",
                   "add x0, x0, x0",
                   "add x1, x1, x1",
                   ;
                   out("x0") _,
                   out("x1") _,
        );
    }
}
