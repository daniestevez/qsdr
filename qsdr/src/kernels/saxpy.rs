#[cfg(target_arch = "aarch64")]
use std::arch::aarch64;
#[cfg(all(target_arch = "x86_64", target_feature = "avx"))]
use std::arch::x86_64;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Saxpy {
    a: f32,
    b: f32,
}

impl Saxpy {
    pub fn new(a: f32, b: f32) -> Saxpy {
        Saxpy { a, b }
    }

    pub fn run_generic(&self, buf: &mut [f32]) {
        for x in buf.iter_mut() {
            *x = *x * self.a + self.b;
        }
    }

    pub fn run_generic_out_of_place(&self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), output.len());
        for (&x, y) in input.iter().zip(output.iter_mut()) {
            *y = x * self.a + self.b;
        }
    }

    #[cfg(not(any(
        target_arch = "aarch64",
        all(target_arch = "x86_64", target_feature = "avx")
    )))]
    pub fn run_best(&self, buf: &mut [f32]) {
        self.run_generic(buf);
    }

    #[cfg(not(target_arch = "aarch64"))]
    pub fn run_best_out_of_place(&self, input: &[f32], output: &mut [f32]) {
        self.run_generic_out_of_place(input, output);
    }

    #[cfg(target_arch = "aarch64")]
    pub fn run_best(&self, buf: &mut [f32]) {
        self.run_cortex_a53(buf);
    }

    #[cfg(target_arch = "aarch64")]
    pub fn run_best_out_of_place(&self, input: &[f32], output: &mut [f32]) {
        self.run_cortex_a53_out_of_place(input, output);
    }

    #[cfg(all(target_arch = "x86_64", target_feature = "avx"))]
    pub fn run_best(&self, buf: &mut [f32]) {
        self.run_znver3(buf);
    }

    #[cfg(target_arch = "aarch64")]
    pub fn run_cortex_a53(&self, buf: &mut [f32]) {
        const FLOATS_PER_ITER: usize = 32;
        assert_eq!(buf.len() % FLOATS_PER_ITER, 0);
        assert!(buf.len() >= 2 * FLOATS_PER_ITER);
        let iterations = buf.len() / FLOATS_PER_ITER - 1;

        unsafe {
            std::arch::asm!(
                "ld1.4s {{v4-v7}}, [{buff0}]",
                "fmul.4s v4, v4, {vA:v}[0]",
                "prfm PLDL1KEEP, [{buff0}, #128]",
                "fmul.4s v5, v5, {vA:v}[0]",
                "ldr {scratch0}, [{buff0}, #72]",
                "fmul.4s v6, v6, {vA:v}[0]",
                "ldr {scratch1}, [{buff0}, #88]",
                "fmul.4s v7, v7, {vA:v}[0]",
                "ldr {scratch2}, [{buff0}, #104]",
                "fadd.4s v4, v4, {vB:v}",
                "ldr {scratch3}, [{buff0}, #120]",
                "fadd.4s v5, v5, {vB:v}",
                "ldr {scratch4}, [{buff0}, #112]",
                "fadd.4s v6, v6, {vB:v}",
                "prfm PLDL1KEEP, [{buff1}, #128]",
                "ldr d0, [{buff0}, #64]",
                "ins v3.d[1], {scratch3}",
                "ldr d1, [{buff0}, #80]",
                "ins v0.d[1], {scratch0}",
                "ldr d2, [{buff0}, #96]",
                "ins v1.d[1], {scratch1}",
                "ins v3.d[0], {scratch4}",
                "ins v2.d[1], {scratch2}",
                "fadd.4s v7, v7, {vB:v}",
                "0:",
                "fmul.4s v0, v0, {vA:v}[0]",
                "ldr {scratch0}, [{buff0}, #136]",
                "fmul.4s v1, v1, {vA:v}[0]",
                "fmul.4s v2, v2, {vA:v}[0]",
                "ldr {scratch1}, [{buff0}, #152]",
                "fmul.4s v3, v3, {vA:v}[0]",
                "ldr {scratch2}, [{buff0}, #168]",
                "fadd.4s v0, v0, {vB:v}",
                "ldr {scratch3}, [{buff0}, #184]",
                "fadd.4s v1, v1, {vB:v}",
                "ldr {scratch4}, [{buff0}, #176]",
                "st1.4s {{v4-v7}}, [{buff0}]",
                "ldr d5, [{buff0}, #144]",
                "ins v7.d[1], {scratch3}",
                "ldr d6, [{buff0}, #160]",
                "ins v5.d[1], {scratch1}",
                "ldr d4, [{buff0}, #128]!",
                "ins v6.d[1], {scratch2}",
                "ins v7.d[0], {scratch4}",
                "ins v4.d[1], {scratch0}",
                "fadd.4s v2, v2, {vB:v}",
                "prfm PLDL1KEEP, [{buff1}, #192]",
                "fadd.4s v3, v3, {vB:v}",
                "prfm PLDL1KEEP, [{buff1}, #256]",
                "fmul.4s v4, v4, {vA:v}[0]",
                "ldr {scratch0}, [{buff1}, #136]",
                "fmul.4s v5, v5, {vA:v}[0]",
                "fmul.4s v6, v6, {vA:v}[0]",
                "ldr {scratch1}, [{buff1}, #152]",
                "fmul.4s v7, v7, {vA:v}[0]",
                "ldr {scratch2}, [{buff1}, #168]",
                "fadd.4s v4, v4, {vB:v}",
                "ldr {scratch3}, [{buff1}, #184]",
                "fadd.4s v5, v5, {vB:v}",
                "ldr {scratch4}, [{buff1}, #176]",
                "st1.4s {{v0-v3}}, [{buff1}]",
                "ldr d1, [{buff1}, #144]",
                "ins v3.d[1], {scratch3}",
                "ldr d2, [{buff1}, #160]",
                "ins v1.d[1], {scratch1}",
                "ldr d0, [{buff1}, #128]!",
                "ins v2.d[1], {scratch2}",
                "ins v3.d[0], {scratch4}",
                "ins v0.d[1], {scratch0}",
                "fadd.4s v6, v6, {vB:v}",
                "cmp {buff0}, {buff0_end}",
                "fadd.4s v7, v7, {vB:v}",
                "b.ne 0b",
                "fmul.4s v0, v0, {vA:v}[0]",
                "fmul.4s v1, v1, {vA:v}[0]",
                "fmul.4s v2, v2, {vA:v}[0]",
                "fmul.4s v3, v3, {vA:v}[0]",
                "st1.4s {{v4-v7}}, [{buff0}]",
                "fadd.4s v0, v0, {vB:v}",
                "fadd.4s v1, v1, {vB:v}",
                "fadd.4s v2, v2, {vB:v}",
                "fadd.4s v3, v3, {vB:v}",
                "st1.4s {{v0-v3}}, [{buff1}]",
                buff0 = inout(reg) buf.as_mut_ptr() => _,
                buff1 = inout(reg) buf.as_mut_ptr().add(FLOATS_PER_ITER / 2) => _,
                buff0_end = in(reg) buf.as_mut_ptr().add(FLOATS_PER_ITER * iterations),
                out("v0") _,
                out("v1") _,
                out("v2") _,
                out("v3") _,
                out("v4") _,
                out("v5") _,
                out("v6") _,
                out("v7") _,
                vA = in(vreg) self.a,
                vB = in(vreg) aarch64::vdupq_n_f32(self.b),
                scratch0 = out(reg) _,
                scratch1 = out(reg) _,
                scratch2 = out(reg) _,
                scratch3 = out(reg) _,
                scratch4 = out(reg) _,
                options(nostack),
            );
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn run_cortex_a53_out_of_place(&self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), output.len());
        const FLOATS_PER_ITER: usize = 32;
        assert_eq!(input.len() % FLOATS_PER_ITER, 0);
        assert!(input.len() >= 2 * FLOATS_PER_ITER);
        let iterations = input.len() / FLOATS_PER_ITER - 1;

        unsafe {
            std::arch::asm!(
                "ld1.4s {{v4-v7}}, [{buff_in0}]",
                "fmul.4s v4, v4, {vA:v}[0]",
                "prfm PLDL1KEEP, [{buff_in0}, #128]",
                "fmul.4s v5, v5, {vA:v}[0]",
                "ldr {scratch0}, [{buff_in0}, #72]",
                "fmul.4s v6, v6, {vA:v}[0]",
                "ldr {scratch1}, [{buff_in0}, #88]",
                "fmul.4s v7, v7, {vA:v}[0]",
                "ldr {scratch2}, [{buff_in0}, #104]",
                "fadd.4s v4, v4, {vB:v}",
                "ldr {scratch3}, [{buff_in0}, #120]",
                "fadd.4s v5, v5, {vB:v}",
                "ldr {scratch4}, [{buff_in0}, #112]",
                "fadd.4s v6, v6, {vB:v}",
                "prfm PLDL1KEEP, [{buff_in1}, #128]",
                "ldr d0, [{buff_in0}, #64]",
                "ins v3.d[1], {scratch3}",
                "ldr d1, [{buff_in0}, #80]",
                "ins v0.d[1], {scratch0}",
                "ldr d2, [{buff_in0}, #96]",
                "ins v1.d[1], {scratch1}",
                "ins v3.d[0], {scratch4}",
                "ins v2.d[1], {scratch2}",
                "fadd.4s v7, v7, {vB:v}",
                "0:",
                "fmul.4s v0, v0, {vA:v}[0]",
                "ldr {scratch0}, [{buff_in0}, #136]",
                "fmul.4s v1, v1, {vA:v}[0]",
                "fmul.4s v2, v2, {vA:v}[0]",
                "ldr {scratch1}, [{buff_in0}, #152]",
                "fmul.4s v3, v3, {vA:v}[0]",
                "ldr {scratch2}, [{buff_in0}, #168]",
                "fadd.4s v0, v0, {vB:v}",
                "ldr {scratch3}, [{buff_in0}, #184]",
                "fadd.4s v1, v1, {vB:v}",
                "ldr {scratch4}, [{buff_in0}, #176]",
                "st1.4s {{v4-v7}}, [{buff_out}], #64",
                "ldr d5, [{buff_in0}, #144]",
                "ins v7.d[1], {scratch3}",
                "ldr d6, [{buff_in0}, #160]",
                "ins v5.d[1], {scratch1}",
                "ldr d4, [{buff_in0}, #128]!",
                "ins v6.d[1], {scratch2}",
                "ins v7.d[0], {scratch4}",
                "ins v4.d[1], {scratch0}",
                "fadd.4s v2, v2, {vB:v}",
                "prfm PLDL1KEEP, [{buff_in1}, #192]",
                "fadd.4s v3, v3, {vB:v}",
                "prfm PLDL1KEEP, [{buff_in1}, #256]",
                "fmul.4s v4, v4, {vA:v}[0]",
                "ldr {scratch0}, [{buff_in1}, #136]",
                "fmul.4s v5, v5, {vA:v}[0]",
                "fmul.4s v6, v6, {vA:v}[0]",
                "ldr {scratch1}, [{buff_in1}, #152]",
                "fmul.4s v7, v7, {vA:v}[0]",
                "ldr {scratch2}, [{buff_in1}, #168]",
                "fadd.4s v4, v4, {vB:v}",
                "ldr {scratch3}, [{buff_in1}, #184]",
                "fadd.4s v5, v5, {vB:v}",
                "ldr {scratch4}, [{buff_in1}, #176]",
                "st1.4s {{v0-v3}}, [{buff_out}], #64",
                "ldr d1, [{buff_in1}, #144]",
                "ins v3.d[1], {scratch3}",
                "ldr d2, [{buff_in1}, #160]",
                "ins v1.d[1], {scratch1}",
                "ldr d0, [{buff_in1}, #128]!",
                "ins v2.d[1], {scratch2}",
                "ins v3.d[0], {scratch4}",
                "ins v0.d[1], {scratch0}",
                "fadd.4s v6, v6, {vB:v}",
                "cmp {buff_in0}, {buff_in0_end}",
                "fadd.4s v7, v7, {vB:v}",
                "b.ne 0b",
                "fmul.4s v0, v0, {vA:v}[0]",
                "fmul.4s v1, v1, {vA:v}[0]",
                "fmul.4s v2, v2, {vA:v}[0]",
                "fmul.4s v3, v3, {vA:v}[0]",
                "st1.4s {{v4-v7}}, [{buff_out}], #64",
                "fadd.4s v0, v0, {vB:v}",
                "fadd.4s v1, v1, {vB:v}",
                "fadd.4s v2, v2, {vB:v}",
                "fadd.4s v3, v3, {vB:v}",
                "st1.4s {{v0-v3}}, [{buff_out}]",
                buff_in0 = inout(reg) input.as_ptr() => _,
                buff_in1 = inout(reg) input.as_ptr().add(FLOATS_PER_ITER / 2) => _,
                buff_out = inout(reg) output.as_mut_ptr() => _,
                buff_in0_end = in(reg) input.as_ptr().add(FLOATS_PER_ITER * iterations),
                out("v0") _,
                out("v1") _,
                out("v2") _,
                out("v3") _,
                out("v4") _,
                out("v5") _,
                out("v6") _,
                out("v7") _,
                vA = in(vreg) self.a,
                vB = in(vreg) aarch64::vdupq_n_f32(self.b),
                scratch0 = out(reg) _,
                scratch1 = out(reg) _,
                scratch2 = out(reg) _,
                scratch3 = out(reg) _,
                scratch4 = out(reg) _,
                options(nostack),
            );
        }
    }

    #[cfg(all(target_arch = "x86_64", target_feature = "avx"))]
    pub fn run_znver3(&self, buf: &mut [f32]) {
        const FLOATS_PER_ITER: usize = 32;
        const REQUIRED_ALIGN: usize = 32;
        assert_eq!(buf.len() % FLOATS_PER_ITER, 0);
        assert!(buf.len() >= FLOATS_PER_ITER);
        assert_eq!(buf.as_ptr().align_offset(REQUIRED_ALIGN), 0);

        unsafe {
            std::arch::asm!(
                ".align 4",
                "2:",
                "vmulps {y0}, {vA}, ymmword ptr [{buff} + 4*{offset:r}]",
                "vmulps {y1}, {vA}, ymmword ptr [{buff} + 4*{offset:r} + 32]",
                "vmulps {y2}, {vA}, ymmword ptr [{buff} + 4*{offset:r} + 64]",
                "vmulps {y3}, {vA}, ymmword ptr [{buff} + 4*{offset:r} + 96]",
                "vaddps {y0}, {y0}, {vB}",
                "vaddps {y1}, {y1}, {vB}",
                "vaddps {y2}, {y2}, {vB}",
                "vaddps {y3}, {y3}, {vB}",
                "vmovaps ymmword ptr [{buff} + 4*{offset:r}], {y0}",
                "vmovaps ymmword ptr [{buff} + 4*{offset:r} + 32], {y1}",
                "vmovaps ymmword ptr [{buff} + 4*{offset:r} + 64], {y2}",
                "vmovaps ymmword ptr [{buff} + 4*{offset:r} + 96], {y3}",
                "add {offset:r}, 32",
                "cmp {offset:r}, {offset_end:r}",
                "jne 2b",
                buff = in(reg) buf.as_mut_ptr(),
                offset = inout(reg) 0 => _,
                offset_end = in(reg) buf.len(),
                vA = in(ymm_reg) x86_64::_mm256_set1_ps(self.a),
                vB = in(ymm_reg) x86_64::_mm256_set1_ps(self.b),
                y0 = out(ymm_reg) _,
                y1 = out(ymm_reg) _,
                y2 = out(ymm_reg) _,
                y3 = out(ymm_reg) _,
                options(nostack),
            );
        }
    }
}

#[cfg(test)]
mod test {
    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use crate::buffers::CacheAlignedBuffer;
    #[allow(unused_imports)]
    use rand::prelude::*;

    #[test]
    fn out_of_place() {
        let n = 1024;
        let mut rng = rand::rng();
        let mut buf: Vec<f32> = std::iter::repeat_with(|| rng.random()).take(n).collect();
        let mut out = vec![0.0; n];
        let a = rng.random();
        let b = rng.random();
        let saxpy = Saxpy::new(a, b);
        saxpy.run_generic_out_of_place(&buf, &mut out);
        saxpy.run_generic(&mut buf);
        assert_eq!(&buf, &out);
    }

    #[cfg(target_arch = "aarch64")]
    #[test]
    fn cortex_a53() {
        let n = 1024;
        let mut rng = rand::rng();
        let mut buf: Vec<f32> = std::iter::repeat_with(|| rng.random()).take(n).collect();
        let mut buf_generic = buf.clone();
        let a = rng.random();
        let b = rng.random();
        let saxpy = Saxpy::new(a, b);
        saxpy.run_generic(&mut buf_generic);
        saxpy.run_cortex_a53(&mut buf);
        assert_eq!(&buf, &buf_generic);
    }

    #[cfg(target_arch = "aarch64")]
    #[test]
    fn cortex_a53_out_of_place() {
        let n = 1024;
        let mut rng = rand::rng();
        let buf: Vec<f32> = std::iter::repeat_with(|| rng.random()).take(n).collect();
        let mut out = vec![0.0; n];
        let mut buf_generic = buf.clone();
        let a = rng.random();
        let b = rng.random();
        let saxpy = Saxpy::new(a, b);
        saxpy.run_generic(&mut buf_generic);
        saxpy.run_cortex_a53_out_of_place(&buf, &mut out);
        assert_eq!(&out, &buf_generic);
    }

    #[cfg(all(target_arch = "x86_64", target_feature = "avx"))]
    #[test]
    fn znver3() {
        let n = 1024;
        let mut rng = rand::rng();
        let mut buf_generic: Vec<f32> = std::iter::repeat_with(|| rng.random()).take(n).collect();
        let mut buf = CacheAlignedBuffer::<f32>::new(n);
        buf.clone_from_slice(&buf_generic);
        let a = rng.random();
        let b = rng.random();
        let saxpy = Saxpy::new(a, b);
        saxpy.run_generic(&mut buf_generic);
        saxpy.run_znver3(&mut buf);
        assert_eq!(&buf[..], &buf_generic);
    }
}
