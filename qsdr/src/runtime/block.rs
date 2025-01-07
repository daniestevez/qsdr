use super::{flowgraph::FlowgraphNode, work::WorkStatus};
use anyhow::Result;
use futures::stream::FusedStream;
use std::{fmt::Debug, future::Future};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
pub enum BlockWorkStatus {
    #[default]
    Run,
    Done,
}

impl From<WorkStatus> for BlockWorkStatus {
    fn from(value: WorkStatus) -> BlockWorkStatus {
        match value {
            WorkStatus::Run => BlockWorkStatus::Run,
            WorkStatus::DoneWithOutput | WorkStatus::DoneWithoutOutput => BlockWorkStatus::Done,
        }
    }
}

pub trait Block: Sized {
    type Channels: Debug;
    type Seeds: TryInto<Self::Channels, Error = anyhow::Error>;
    type Node: FlowgraphNode<B = Self>;

    fn block_work(
        &mut self,
        channels: &mut Self::Channels,
    ) -> impl Future<Output = Result<BlockWorkStatus>>;
}

pub struct BlockObject<B: Block> {
    block: B,
    channels: B::Channels,
}

impl<B: Block> BlockObject<B> {
    pub fn new(block: B, channels: B::Channels) -> BlockObject<B> {
        BlockObject { block, channels }
    }

    pub fn into_stream(self) -> impl FusedStream<Item = Result<()>> {
        // Box self because it is much faster to move around a Box as the
        // closure state than a BlockObject<B>, which is typically much larger
        let self_ = Box::new(self);
        futures::stream::unfold(self_, |mut self_| async {
            match self_.block.block_work(&mut self_.channels).await {
                Ok(BlockWorkStatus::Run) => Some((Ok(()), self_)),
                Ok(BlockWorkStatus::Done) => None,
                Err(err) => Some((Err(err), self_)),
            }
        })
    }
}
