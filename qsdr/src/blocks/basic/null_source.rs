use crate::prelude::*;

#[derive(Block, Debug)]
#[qsdr_crate = "crate"]
#[work(WorkInPlace)]
pub struct NullSource<T, Cin = Spsc, Cout = Spsc>
where
    Cin: Channel,
    Cin::Receiver<T>: Receiver<T>,
    Cout: Channel,
{
    #[port]
    input: PortSource<T, Cin>,
    #[port]
    output: PortOut<T, Cout>,
}

impl<T, Cin, Cout> NullSource<T, Cin, Cout>
where
    Cin: Channel,
    Cin::Receiver<T>: Receiver<T>,
    Cout: Channel,
{
    pub fn new() -> Self {
        Self {
            input: Default::default(),
            output: Default::default(),
        }
    }
}

impl<T, Cin, Cout> Default for NullSource<T, Cin, Cout>
where
    Cin: Channel,
    Cin::Receiver<T>: Receiver<T>,
    Cout: Channel,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, Cin, Cout> WorkInPlace<T> for NullSource<T, Cin, Cout>
where
    Cin: Channel,
    Cin::Receiver<T>: Receiver<T>,
    Cout: Channel,
{
    async fn work_in_place(&mut self, _: &mut T) -> Result<WorkStatus> {
        Ok(Run)
    }
}
