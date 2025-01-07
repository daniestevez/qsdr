use crate::prelude::*;

#[derive(Block, Debug)]
#[qsdr_crate = "crate"]
#[work(WorkWithRef)]
pub struct RefClone<
    B: Buffer,
    Cin: Channel = SpscRef,
    Cout: Channel = Spsc,
    Csource: Channel = Spsc,
> where
    B::Item: Clone,
    Csource::Receiver<Quantum<B>>: Receiver<Quantum<B>>,
{
    #[port]
    input: PortRefInQ<B, Cin>,
    #[port]
    source: PortSourceQ<B, Csource>,
    #[port]
    output: PortOutQ<B, Cout>,
}

impl<B: Buffer, Cin: Channel, Cout: Channel, Csource: Channel> RefClone<B, Cin, Cout, Csource>
where
    B::Item: Clone,
    Csource::Receiver<Quantum<B>>: Receiver<Quantum<B>>,
{
    pub fn new() -> Self {
        Self {
            input: Default::default(),
            source: Default::default(),
            output: Default::default(),
        }
    }
}

impl<B: Buffer, Cin: Channel, Cout: Channel, Csource: Channel> Default
    for RefClone<B, Cin, Cout, Csource>
where
    B::Item: Clone,
    Csource::Receiver<Quantum<B>>: Receiver<Quantum<B>>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<B: Buffer, Cin: Channel, Cout: Channel, Csource: Channel> WorkWithRef<Quantum<B>>
    for RefClone<B, Cin, Cout, Csource>
where
    B::Item: Clone,
    Csource::Receiver<Quantum<B>>: Receiver<Quantum<B>>,
{
    async fn work_with_ref(
        &mut self,
        item_in: &Quantum<B>,
        item_out: &mut Quantum<B>,
    ) -> Result<WorkStatus> {
        let slice_in = item_in.as_slice();
        let slice_out = item_out.as_mut_slice();
        assert_eq!(slice_in.len(), slice_out.len());
        slice_out.clone_from_slice(slice_in);
        Ok(Run)
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
    use rand::{prelude::*, Fill};

    #[test]
    fn ref_clone() {
        type B = CacheAlignedBuffer<u32>;
        let buffer_size = 4096;
        let num_buffers = 4;
        let num_elements = 10000;
        let mut rng = rand::thread_rng();

        let mut make_buffers = || {
            std::iter::repeat_with(|| Quantum::new(B::from_fn(buffer_size, |_| rng.gen())))
                .take(num_buffers)
                .collect::<Vec<_>>()
                .into_iter()
        };

        let buffers0 = make_buffers();
        let buffers1 = make_buffers();
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
        let ref_clone = fg.add_block(RefClone::<B, SpscRef, SpscRef>::new());
        let (tx, rx) = std::sync::mpsc::channel();
        let sink = fg.add_block(SnapshotSink::<B, _>::new(tx));
        let mut circ0 = fg.new_circuit(buffers0);
        fg.connect_with_return(
            &mut circ0,
            source.output(),
            ref_clone.input(),
            source.input(),
        )
        .unwrap();
        let mut circ1 = fg.new_circuit(buffers1);
        fg.connect_with_return(
            &mut circ1,
            ref_clone.output(),
            sink.input(),
            ref_clone.source(),
        )
        .unwrap();
        let mut fg = fg.validate().unwrap();
        let source = fg.extract_block(source).unwrap();
        let ref_clone = fg.extract_block(ref_clone).unwrap();
        let sink = fg.extract_block(sink).unwrap();

        block_on(run(sequence3(
            source.into_stream(),
            ref_clone.into_stream(),
            sink.into_stream(),
        )))
        .unwrap();

        let mut rx = rx.into_iter();
        for (n, element) in elements.iter().enumerate() {
            let out = rx.next().unwrap();
            assert_eq!(*element, out, "element {n} mismatch");
        }
        assert!(rx.next().is_none());
    }
}
