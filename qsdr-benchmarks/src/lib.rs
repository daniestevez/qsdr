pub mod affinity {
    use anyhow::Result;

    pub fn get_core_ids() -> Result<Vec<core_affinity::CoreId>> {
        let core_ids = core_affinity::get_core_ids()
            .ok_or_else(|| anyhow::anyhow!("could not get CPU cores for affinity"))?;
        anyhow::ensure!(
            !core_ids.is_empty(),
            "did not get any CPU cores for affinity"
        );
        Ok(core_ids)
    }

    // pin to a single CPU to prevent seeing jumps in get_cpu_cycles
    // (pmccntr_el0 under aarch64) if we get migrated to another CPU
    pub fn pin_cpu() -> Result<()> {
        core_affinity::set_for_current(get_core_ids()?[0]);
        Ok(())
    }
}

pub mod asm;
mod buffer;
pub use buffer::Buffer;
pub mod futures;