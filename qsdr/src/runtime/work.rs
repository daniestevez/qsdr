use super::block::{Block, BlockWorkStatus};
use anyhow::Result;
use std::future::Future;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
pub enum WorkStatus {
    #[default]
    Run,
    DoneWithOutput,
    DoneWithoutOutput,
}

impl WorkStatus {
    pub fn produces_output(&self) -> bool {
        match self {
            WorkStatus::Run | WorkStatus::DoneWithOutput => true,
            WorkStatus::DoneWithoutOutput => false,
        }
    }
}

pub trait WorkInPlace<T> {
    fn work_in_place(&mut self, item: &mut T) -> impl Future<Output = Result<WorkStatus>>;
}

pub trait WorkSink<T> {
    // this returns BlockWorkStatus rather than WorkStatus because a WorkSink
    // does not produce an output
    fn work_sink(&mut self, item: &T) -> impl Future<Output = Result<BlockWorkStatus>>;
}

pub trait WorkWithRef<T> {
    fn work_with_ref(
        &mut self,
        item_in: &T,
        item_out: &mut T,
    ) -> impl Future<Output = Result<WorkStatus>>;
}

pub trait WorkCustom: Block {
    fn work_custom(
        &mut self,
        channels: &mut Self::Channels,
    ) -> impl Future<Output = Result<BlockWorkStatus>>;
}
