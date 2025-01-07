use crate::{prelude::*, QuantumSnapshot};
use std::future::Future;

pub trait Sink<T> {
    fn send(&mut self, snapshot: QuantumSnapshot<T>) -> impl Future<Output = Result<()>>;
}

#[derive(Block, Debug)]
#[qsdr_crate = "crate"]
#[work(WorkSink)]
pub struct SnapshotSink<B, S, Cin = SpscRef>
where
    B: Buffer,
    B::Item: Clone,
    S: Sink<B::Item>,
    Cin: Channel,
{
    #[port]
    input: PortRefInQ<B, Cin>,
    sink: S,
}

impl<B, S, Cin> SnapshotSink<B, S, Cin>
where
    B: Buffer,
    B::Item: Clone,
    S: Sink<B::Item>,
    Cin: Channel,
{
    pub fn new(sink: S) -> Self {
        Self {
            input: Default::default(),
            sink,
        }
    }
}

impl<B, S, Cin> WorkSink<Quantum<B>> for SnapshotSink<B, S, Cin>
where
    B: Buffer,
    B::Item: Clone,
    S: Sink<B::Item>,
    Cin: Channel,
{
    async fn work_sink(&mut self, quantum: &Quantum<B>) -> Result<BlockWorkStatus> {
        self.sink.send(quantum.snapshot()).await?;
        Ok(BlockWorkStatus::Run)
    }
}

impl<T> Sink<T> for std::sync::mpsc::Sender<QuantumSnapshot<T>> {
    async fn send(&mut self, snapshot: QuantumSnapshot<T>) -> Result<()> {
        std::sync::mpsc::Sender::send(self, snapshot)
            .map_err(|e| anyhow::anyhow!("send error: {e:?}"))
    }
}
