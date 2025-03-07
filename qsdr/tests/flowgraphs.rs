use futures::executor::block_on;
use qsdr::{
    blocks::basic::{RefClone, SnapshotSink, SnapshotSource, SourceIterator},
    buffers::CacheAlignedBuffer,
    prelude::*,
    scheduler::{run, sequence2, sequence4},
};
use rand::prelude::*;

#[test]
fn spbroadcast() {
    type B = CacheAlignedBuffer<u32>;
    let buffer_size = 4096;
    let num_buffers = 4;
    let num_elements = 10000;
    let mut rng = rand::rng();

    let elements = std::iter::repeat_with(|| {
        std::iter::repeat_with(|| rng.random()).take(buffer_size).collect::<Vec<_>>().into()
    })
    .take(num_elements)
    .collect::<Vec<_>>();

    let mut make_buffers = || {
        std::iter::repeat_with(|| Quantum::new(B::from_fn(buffer_size, |_| rng.random())))
            .take(num_buffers)
            .collect::<Vec<_>>()
            .into_iter()
    };

    let mut fg = Flowgraph::new();
    let source = fg.add_block(SnapshotSource::<B, _, Mpsc, SpBroadcast>::new(
        SourceIterator(elements.clone().into_iter()),
    ));
    let mut circ0 = fg.new_circuit(make_buffers());

    macro_rules! make_branch {
        () => {{
            let ref_clone = fg.add_block(RefClone::<B, SpBroadcast, SpscRef>::new());
            fg.connect_with_return(
                &mut circ0,
                source.output(),
                ref_clone.input(),
                source.input(),
            )
            .unwrap();
            let (tx, rx) = std::sync::mpsc::channel();
            let sink = fg.add_block(SnapshotSink::<B, _>::new(tx));
            let mut circ = fg.new_circuit(make_buffers());
            fg.connect_with_return(
                &mut circ,
                ref_clone.output(),
                sink.input(),
                ref_clone.source(),
            )
            .unwrap();
            (ref_clone, sink, rx)
        }};
    }

    let (ref_clone0, sink0, rx0) = make_branch!();
    let (ref_clone1, sink1, rx1) = make_branch!();
    let (ref_clone2, sink2, rx2) = make_branch!();

    let mut fg = fg.validate().unwrap();

    macro_rules! extract_branch {
        ($ref_clone:expr, $sink:expr) => {{
            let ref_clone = fg.extract_block($ref_clone).unwrap();
            let sink = fg.extract_block($sink).unwrap();
            sequence2(ref_clone.into_stream(), sink.into_stream())
        }};
    }

    let source = fg.extract_block(source).unwrap();
    let branch0 = extract_branch!(ref_clone0, sink0);
    let branch1 = extract_branch!(ref_clone1, sink1);
    let branch2 = extract_branch!(ref_clone2, sink2);

    block_on(run(sequence4(
        source.into_stream(),
        branch0,
        branch1,
        branch2,
    )))
    .unwrap();

    macro_rules! check_elements {
        ($rx:expr) => {
            let mut rx = $rx.into_iter();
            for (n, element) in elements.iter().enumerate() {
                let out = rx.next().unwrap();
                assert_eq!(*element, out, "element {n} mismatch");
            }
            assert!(rx.next().is_none());
        };
        ($($rx:expr),*) => {
            $(check_elements!($rx));*
        };
    }

    check_elements!(rx0, rx1, rx2);
}
