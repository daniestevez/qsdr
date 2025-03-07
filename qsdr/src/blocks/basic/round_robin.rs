use crate::{prelude::*, runtime::channel::Sender};

#[derive(Block, Debug)]
#[qsdr_crate = "crate"]
#[work(WorkCustom)]
pub struct RoundRobin<T, Cin = Spsc, Cout = Spsc>
where
    Cin: Channel,
    Cin::Receiver<T>: Receiver<T>,
    Cout: Channel,
{
    #[port]
    input: PortIn<T, Cin>,
    #[port]
    output0: PortOut<T, Cout>,
    #[port]
    output1: PortOut<T, Cout>,
    current_output: OutputPort,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Hash)]
enum OutputPort {
    #[default]
    Output0,
    Output1,
}

impl OutputPort {
    fn next(self) -> OutputPort {
        match self {
            OutputPort::Output0 => OutputPort::Output1,
            OutputPort::Output1 => OutputPort::Output0,
        }
    }
}

impl<T, Cin, Cout> RoundRobin<T, Cin, Cout>
where
    Cin: Channel,
    Cin::Receiver<T>: Receiver<T>,
    Cout: Channel,
{
    pub fn new() -> Self {
        Self {
            input: Default::default(),
            output0: Default::default(),
            output1: Default::default(),
            current_output: Default::default(),
        }
    }
}

impl<T, Cin, Cout> Default for RoundRobin<T, Cin, Cout>
where
    Cin: Channel,
    Cin::Receiver<T>: Receiver<T>,
    Cout: Channel,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, Cin, Cout> WorkCustom for RoundRobin<T, Cin, Cout>
where
    Cin: Channel,
    Cin::Receiver<T>: Receiver<T>,
    Cout: Channel,
{
    async fn work_custom(&mut self, channels: &mut Self::Channels) -> Result<BlockWorkStatus> {
        let Some(item) = channels.input.recv().await else {
            return Ok(BlockWorkStatus::Done);
        };
        match self.current_output {
            OutputPort::Output0 => channels.output0.send(item),
            OutputPort::Output1 => channels.output1.send(item),
        }
        self.current_output = self.current_output.next();
        Ok(BlockWorkStatus::Run)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        blocks::basic::{SnapshotSink, SnapshotSource, SourceIterator},
        buffers::CacheAlignedBuffer,
        scheduler::{run, sequence4},
    };
    use futures::executor::block_on;
    use rand::prelude::*;

    #[test]
    fn round_robin() {
        type B = CacheAlignedBuffer<u32>;
        let buffer_size = 4096;
        let num_buffers = 4;
        let num_elements = 10000;
        let mut rng = rand::rng();

        let buffers =
            std::iter::repeat_with(|| Quantum::new(B::from_fn(buffer_size, |_| rng.random())))
                .take(num_buffers)
                .collect::<Vec<_>>()
                .into_iter();
        let elements = std::iter::repeat_with(|| {
            std::iter::repeat_with(|| rng.random())
                .take(buffer_size)
                .collect::<Vec<_>>()
                .into()
        })
        .take(num_elements)
        .collect::<Vec<_>>();

        let mut fg = Flowgraph::new();
        let source = fg.add_block(SnapshotSource::<B, _, Mpsc>::new(SourceIterator(
            elements.clone().into_iter(),
        )));
        let round_robin = fg.add_block(RoundRobin::<Quantum<B>, Spsc, SpscmrRef>::new());
        let (tx0, rx0) = std::sync::mpsc::channel();
        let sink0 = fg.add_block(SnapshotSink::<B, _, SpscmrRef>::new(tx0));
        let (tx1, rx1) = std::sync::mpsc::channel();
        let sink1 = fg.add_block(SnapshotSink::<B, _, SpscmrRef>::new(tx1));
        let mut circ = fg.new_circuit(buffers);
        fg.connect(&mut circ, source.output(), round_robin.input())
            .unwrap();
        fg.connect_with_return(
            &mut circ,
            round_robin.output0(),
            sink0.input(),
            source.input(),
        )
        .unwrap();
        fg.connect_with_return(
            &mut circ,
            round_robin.output1(),
            sink1.input(),
            source.input(),
        )
        .unwrap();
        let mut fg = fg.validate().unwrap();
        let source = fg.extract_block(source).unwrap();
        let round_robin = fg.extract_block(round_robin).unwrap();
        let sink0 = fg.extract_block(sink0).unwrap();
        let sink1 = fg.extract_block(sink1).unwrap();

        block_on(run(sequence4(
            source.into_stream(),
            round_robin.into_stream(),
            sink0.into_stream(),
            sink1.into_stream(),
        )))
        .unwrap();

        let mut rx = rx0.into_iter();
        for (n, element) in elements.iter().enumerate().step_by(2) {
            let out = rx.next().unwrap();
            assert_eq!(*element, out, "element {n} mismatch");
        }
        assert!(rx.next().is_none());

        let mut rx = rx1.into_iter();
        for (n, element) in elements.iter().enumerate().skip(1).step_by(2) {
            let out = rx.next().unwrap();
            assert_eq!(*element, out, "element {n} mismatch");
        }
        assert!(rx.next().is_none());
    }
}
