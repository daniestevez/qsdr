pub mod blocks;
pub mod channel;
pub mod kernels {
    pub mod saxpy;
}
mod runtime;
pub use runtime::{
    block::{Block, BlockObject, BlockWorkStatus},
    buffer::Buffer,
    channel::{Channel, Receiver, RefReceiver, Sender},
    flowgraph::{Flowgraph, ValidatedFlowgraph},
    quantum::{Quantum, QuantumSnapshot},
    work::{
        WorkCustom, WorkInPlace, WorkSink, WorkStatus,
        WorkStatus::{DoneWithOutput, DoneWithoutOutput, Run},
        WorkWithRef,
    },
};
pub mod buffers {
    pub use crate::runtime::buffer::CacheAlignedBuffer;
}
pub mod ports {
    pub use crate::runtime::port::{
        Endpoint, PortIn, PortInQ, PortOut, PortOutQ, PortRefIn, PortRefInQ, PortSource,
        PortSourceQ,
    };
}
pub mod channels {
    pub use crate::runtime::channel::{Mpsc, SpBroadcast, Spsc, SpscRef, SpscmrRef};
}

pub mod scheduler {
    pub use crate::runtime::scheduler::{
        Sequence2, Sequence3, Sequence4, Sequence5, Sequence6, Sequence7, Sequence8, run,
        sequence2, sequence3, sequence4, sequence5, sequence6, sequence7, sequence8,
    };
}

pub mod prelude {
    pub use crate::{
        Block, BlockWorkStatus, Buffer, Channel, DoneWithOutput, DoneWithoutOutput, Flowgraph,
        Quantum, Receiver, Run, WorkCustom, WorkInPlace, WorkSink, WorkStatus, WorkWithRef,
        channels::{Mpsc, SpBroadcast, Spsc, SpscRef, SpscmrRef},
        ports::{
            PortIn, PortInQ, PortOut, PortOutQ, PortRefIn, PortRefInQ, PortSource, PortSourceQ,
        },
    };
    pub use anyhow::Result;
}

pub use qsdr_macros::Block;

// used by qsdr-macros
#[doc(hidden)]
pub mod __private {
    pub use crate::runtime::{
        flowgraph::{FlowgraphId, FlowgraphNode, NodeId},
        port::{Port, PortId},
    };

    pub use pin_project_lite;
}
