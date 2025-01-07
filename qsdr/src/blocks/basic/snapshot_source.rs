use crate::{prelude::*, QuantumSnapshot};
use std::future::Future;

pub trait Source<T> {
    fn recv(&mut self) -> impl Future<Output = Result<Option<QuantumSnapshot<T>>>>;
}

#[derive(Block, Debug)]
#[qsdr_crate = "crate"]
#[work(WorkInPlace)]
pub struct SnapshotSource<B, S, Cin = Spsc, Cout = Spsc>
where
    B: Buffer,
    B::Item: Clone,
    S: Source<B::Item>,
    Cin: Channel,
    Cin::Receiver<Quantum<B>>: Receiver<Quantum<B>>,
    Cout: Channel,
{
    #[port]
    input: PortSourceQ<B, Cin>,
    #[port]
    output: PortOutQ<B, Cout>,
    source: S,
}

impl<B, S, Cin, Cout> SnapshotSource<B, S, Cin, Cout>
where
    B: Buffer,
    B::Item: Clone,
    S: Source<B::Item>,
    Cin: Channel,
    Cin::Receiver<Quantum<B>>: Receiver<Quantum<B>>,
    Cout: Channel,
{
    pub fn new(source: S) -> Self {
        Self {
            input: Default::default(),
            output: Default::default(),
            source,
        }
    }
}

impl<B, S, Cin, Cout> WorkInPlace<Quantum<B>> for SnapshotSource<B, S, Cin, Cout>
where
    B: Buffer,
    B::Item: Clone,
    S: Source<B::Item>,
    Cin: Channel,
    Cin::Receiver<Quantum<B>>: Receiver<Quantum<B>>,
    Cout: Channel,
{
    async fn work_in_place(&mut self, quantum: &mut Quantum<B>) -> Result<WorkStatus> {
        let Some(snapshot) = self.source.recv().await? else {
            return Ok(DoneWithoutOutput);
        };
        assert_eq!(quantum.as_slice().len(), snapshot.as_slice().len());
        quantum.as_mut_slice().clone_from_slice(snapshot.as_slice());
        Ok(Run)
    }
}

#[derive(Debug)]
pub struct SourceIterator<I>(pub I);

impl<I, T> Source<T> for SourceIterator<I>
where
    I: Iterator<Item = QuantumSnapshot<T>>,
{
    async fn recv(&mut self) -> Result<Option<QuantumSnapshot<T>>> {
        Ok(self.0.next())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        blocks::basic::SnapshotSink,
        buffers::CacheAlignedBuffer,
        scheduler::{run, sequence2},
    };
    use futures::executor::block_on;
    use rand::{prelude::*, Fill};

    #[test]
    fn source_iterator() {
        type B = CacheAlignedBuffer<u32>;
        let buffer_size = 4096;
        let num_buffers = 4;
        let num_elements = 10000;
        let mut rng = rand::thread_rng();

        let buffers =
            std::iter::repeat_with(|| Quantum::new(B::from_fn(buffer_size, |_| rng.gen())))
                .take(num_buffers)
                .collect::<Vec<_>>()
                .into_iter();
        let elements = std::iter::repeat_with(|| {
            let mut v = vec![0; buffer_size];
            v.try_fill(&mut rng).unwrap();
            v.into()
        })
        .take(num_elements)
        .collect::<Vec<_>>();

        let mut fg = Flowgraph::new();
        let source = fg.add_block(SnapshotSource::<B, _, Spsc, SpscRef>::new(SourceIterator(
            elements.clone().into_iter(),
        )));
        let (tx, rx) = std::sync::mpsc::channel();
        let sink = fg.add_block(SnapshotSink::<B, _>::new(tx));
        let mut circ = fg.new_circuit(buffers);
        fg.connect_with_return(&mut circ, source.output(), sink.input(), source.input())
            .unwrap();
        let mut fg = fg.validate().unwrap();
        let source = fg.extract_block(source).unwrap();
        let sink = fg.extract_block(sink).unwrap();

        block_on(run(sequence2(source.into_stream(), sink.into_stream()))).unwrap();

        let mut rx = rx.into_iter();
        for (n, element) in elements.iter().enumerate() {
            let out = rx.next().unwrap();
            assert_eq!(*element, out, "element {n} mismatch");
        }
        assert!(rx.next().is_none());
    }
}
