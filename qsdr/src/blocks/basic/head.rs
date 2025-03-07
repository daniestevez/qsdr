use crate::prelude::*;

#[derive(Block, Debug)]
#[qsdr_crate = "crate"]
#[work(WorkInPlace)]
pub struct Head<T, Cin = Spsc, Cout = Spsc>
where
    Cin: Channel,
    Cin::Receiver<T>: Receiver<T>,
    Cout: Channel,
{
    #[port]
    input: PortIn<T, Cin>,
    #[port]
    output: PortOut<T, Cout>,
    remaining: u64,
}

impl<T, Cin, Cout> Head<T, Cin, Cout>
where
    Cin: Channel,
    Cin::Receiver<T>: Receiver<T>,
    Cout: Channel,
{
    pub fn new(count: u64) -> Self {
        Self {
            input: Default::default(),
            output: Default::default(),
            remaining: count,
        }
    }
}

impl<T, Cin, Cout> WorkInPlace<T> for Head<T, Cin, Cout>
where
    Cin: Channel,
    Cin::Receiver<T>: Receiver<T>,
    Cout: Channel,
{
    async fn work_in_place(&mut self, _: &mut T) -> Result<WorkStatus> {
        assert!(self.remaining > 0);
        self.remaining -= 1;
        if self.remaining == 0 {
            Ok(DoneWithOutput)
        } else {
            Ok(Run)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        blocks::basic::{SnapshotSink, SnapshotSource, SourceIterator},
        buffers::CacheAlignedBuffer,
        scheduler::{run, sequence3},
    };
    use futures::executor::block_on;
    use rand::prelude::*;

    #[test]
    fn head() {
        type B = CacheAlignedBuffer<u32>;
        let buffer_size = 4096;
        let num_buffers = 16;
        let num_elements_source = 10000;
        let num_elements_head = 3000;
        let mut rng = rand::rng();

        let buffers =
            std::iter::repeat_with(|| Quantum::new(B::from_fn(buffer_size, |_| rng.random())))
                .take(num_buffers)
                .collect::<Vec<_>>()
                .into_iter();
        let elements_source = std::iter::repeat_with(|| {
            std::iter::repeat_with(|| rng.random())
                .take(buffer_size)
                .collect::<Vec<_>>()
                .into()
        })
        .take(num_elements_source)
        .collect::<Vec<_>>();

        let mut fg = Flowgraph::new();
        let source = fg.add_block(SnapshotSource::<B, _>::new(SourceIterator(
            elements_source.clone().into_iter(),
        )));
        let head = fg.add_block(Head::<_, _, SpscRef>::new(
            u64::try_from(num_elements_head).unwrap(),
        ));
        let (tx, rx) = std::sync::mpsc::channel();
        let sink = fg.add_block(SnapshotSink::<B, _>::new(tx));
        let mut circ = fg.new_circuit(buffers);
        fg.connect(&mut circ, source.output(), head.input())
            .unwrap();
        fg.connect_with_return(&mut circ, head.output(), sink.input(), source.input())
            .unwrap();
        let mut fg = fg.validate().unwrap();
        let source = fg.extract_block(source).unwrap();
        let head = fg.extract_block(head).unwrap();
        let sink = fg.extract_block(sink).unwrap();

        block_on(run(sequence3(
            source.into_stream(),
            head.into_stream(),
            sink.into_stream(),
        )))
        .unwrap();

        let mut rx = rx.into_iter();
        for (n, element) in elements_source.iter().take(num_elements_head).enumerate() {
            let out = rx.next().unwrap();
            assert_eq!(*element, out, "element {n} mismatch");
        }
        assert!(rx.next().is_none());
    }
}
