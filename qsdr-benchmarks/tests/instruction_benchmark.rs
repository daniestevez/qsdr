mod instruction_benchmark {
    #[cfg(target_arch = "aarch64")]
    mod aarch64;

    #[cfg(target_arch = "x86_64")]
    mod x86_64;
}
