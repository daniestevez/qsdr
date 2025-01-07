use qsdr_benchmarks::{benchmark, expected_cycles};

// Cortex-A53
//
// An unconditional branch has no penalty, issues in one cycle and can dual
// issue. A cmp followed by a conditional branch can dual-issue together in the
// same cycle, taking a single cycle to issue both instructions.
//
// A typical loop with a conditional branch to go to the beginning takes 7 extra
// cycles of penalty, probably because of the penalty due to the branch
// predictor failing once (when the branch is not taken). However, when the loop
// does 8 iterations or less and is executed multiple times (by an outer loop,
// for instance), the branch predictor is able to learn the whole pattern and
// there is no misprediction penalty.

#[test]
fn b_no_dual_issue() {
    unsafe {
        benchmark!("b (without dual-issue)";
                   expected_cycles! {
                       "cortex-a53" => 3,
                   };
                   "b 0f",
                   "fadd.4s v0, v0, v0",
                   "fadd.4s v1, v2, v2",
                   "0:",
                   "fadd.4s v3, v3, v3",
                   "fadd.4s v4, v4, v4",
                   ;
                   out("v0") _,
                   out("v1") _,
                   out("v2") _,
                   out("v3") _,
        );
    }
}

#[test]
fn b_dual_issue() {
    unsafe {
        benchmark!("b (with dual-issue)";
                   expected_cycles! {
                       "cortex-a53" => 3,
                   };
                   "fadd.4s v0, v0, v0",
                   "b 0f",
                   "fadd.4s v1, v1, v1",
                   "fadd.4s v2, v2, v2",
                   "0:",
                   "fadd.4s v3, v3, v3",
                   "fadd.4s v4, v4, v4",
                   ;
                   out("v0") _,
                   out("v1") _,
                   out("v2") _,
                   out("v3") _,
                   out("v4") _,
        );
    }
}

#[test]
fn cbz_not_taken() {
    unsafe {
        benchmark!("cbz (branch not taken)";
                   expected_cycles! {
                       "cortex-a53" => 5,
                   };
                   "cbz x0, 0f",
                   "fadd.4s v0, v0, v0",
                   "fadd.4s v1, v1, v1",
                   "0:",
                   "fadd.4s v2, v2, v2",
                   "fadd.4s v3, v3, v3",
                   ;
                   in("x0") 1,
                   out("v0") _,
                   out("v1") _,
                   out("v2") _,
                   out("v3") _,
        );
    }
}

#[test]
fn cbz_taken() {
    unsafe {
        benchmark!("cbz (branch taken)";
                   expected_cycles! {
                       "cortex-a53" => 3,
                   };
                   "cbz x0, 0f",
                   "fadd.4s v0, v0, v0",
                   "fadd.4s v1, v1, v1",
                   "0:",
                   "fadd.4s v2, v2, v2",
                   "fadd.4s v3, v3, v3",
                   ;
                   in("x0") 0,
                   out("v0") _,
                   out("v1") _,
                   out("v2") _,
                   out("v3") _,
        );
    }
}

#[test]
fn cmp_beq_not_taken() {
    unsafe {
        benchmark!("cmp + b.eq (branch not taken)";
                   expected_cycles! {
                       "cortex-a53" => 6,
                   };
                   "fadd.4s v0, v0, v0",
                   "cmp x0, 42",
                   "fadd.4s v1, v1, v1",
                   "b.eq 0f",
                   "fadd.4s v2, v2, v2",
                   "fadd.4s v3, v3, v3",
                   "0:",
                   "fadd.4s v4, v4, v4",
                   "fadd.4s v5, v5, v5",
                   ;
                   in("x0") 40,
                   out("v0") _,
                   out("v1") _,
                   out("v2") _,
                   out("v3") _,
                   out("v4") _,
                   out("v5") _,
        );
    }
}

#[test]
fn cmp_beq_ntaken() {
    unsafe {
        benchmark!("cmp + b.eq (branch taken)";
                   expected_cycles! {
                       "cortex-a53" => 4,
                   };
                   "fadd.4s v0, v0, v0",
                   "cmp x0, 42",
                   "fadd.4s v1, v1, v1",
                   "b.eq 0f",
                   "fadd.4s v2, v2, v2",
                   "fadd.4s v3, v3, v3",
                   "0:",
                   "fadd.4s v4, v4, v4",
                   "fadd.4s v5, v5, v5",
                   ;
                   in("x0") 42,
                   out("v0") _,
                   out("v1") _,
                   out("v2") _,
                   out("v3") _,
                   out("v4") _,
                   out("v5") _,
        );
    }
}

#[test]
fn cmp_beq_not_taken_dual_issue() {
    unsafe {
        benchmark!("cmp + b.eq (branch not taken, cmp & b.eq dual-issued)";
                   expected_cycles! {
                       "cortex-a53" => 5,
                   };
                   // cmp and b.eq are dual-issued
                   "cmp x0, 42",
                   "b.eq 0f",
                   "fadd.4s v0, v0, v0",
                   "fadd.4s v1, v1, v1",
                   "0:",
                   "fadd.4s v2, v2, v2",
                   "fadd.4s v3, v3, v3",
                   ;
                   in("x0") 40,
                   out("v0") _,
                   out("v1") _,
                   out("v2") _,
                   out("v3") _,
        );
    }
}

#[test]
fn cmp_beq_taken_dual_issue() {
    unsafe {
        benchmark!("cmp + b.eq (branch not taken, cmp & b.eq dual-issued)";
                   expected_cycles! {
                       "cortex-a53" => 3,
                   };
                   "cmp x0, 42",
                   "b.eq 0f",
                   "fadd.4s v0, v0, v0",
                   "fadd.4s v1, v1, v1",
                   "0:",
                   "fadd.4s v2, v2, v2",
                   "fadd.4s v3, v3, v3",
                   ;
                   in("x0") 42,
                   out("v0") _,
                   out("v1") _,
                   out("v2") _,
                   out("v3") _,
        );
    }
}

fn basic_loop(iterations: u64, penalty: u64) {
    let loop_cycles = 5;
    unsafe {
        benchmark!(format!("basic loop ({iterations} iterations)");
                   expected_cycles! {
                       "cortex-a53" => loop_cycles * iterations + penalty,
                   };
                   "0:",
                   "fadd.4s v0, v0, v0",

                   "fadd.4s v1, v1, v1",

                   "fadd.4s v2, v2, v2",

                   "fadd.4s v3, v3, v3",
                   "add x0, x0, 1",

                   "cmp x0, x1",
                   "b.ne 0b",
                   ;
                   inout("x0") 0 => _,
                   in("x1") iterations,
                   out("v0") _,
                   out("v1") _,
                   out("v2") _,
                   out("v3") _,
        );
    }
}

#[test]
fn basic_loop_small_iters() {
    for iter in 1..=8 {
        basic_loop(iter, 0);
    }
}

#[test]
fn basic_loop_medium_iters() {
    for iter in 9..=32 {
        basic_loop(iter, 7);
    }
}

#[test]
fn basic_loop_large_iters() {
    for iter_log2 in 6..=11 {
        basic_loop(1 << iter_log2, 7);
    }
}
