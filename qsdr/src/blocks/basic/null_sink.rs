use crate::prelude::*;

#[derive(Block, Debug)]
#[qsdr_crate = "crate"]
#[work(WorkSink)]
pub struct NullSink<T, Cin = SpscRef>
where
    Cin: Channel,
{
    #[port]
    input: PortRefIn<T, Cin>,
}

impl<T, Cin> NullSink<T, Cin>
where
    Cin: Channel,
{
    pub fn new() -> Self {
        Self {
            input: Default::default(),
        }
    }
}

impl<T, Cin> Default for NullSink<T, Cin>
where
    Cin: Channel,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, Cin> WorkSink<T> for NullSink<T, Cin>
where
    Cin: Channel,
{
    async fn work_sink(&mut self, _: &T) -> Result<BlockWorkStatus> {
        Ok(BlockWorkStatus::Run)
    }
}
