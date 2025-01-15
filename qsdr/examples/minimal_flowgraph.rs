use futures::executor::block_on;
use qsdr::{
    blocks::basic::{Head, NullSink, NullSource},
    buffers::CacheAlignedBuffer,
    prelude::*,
    scheduler::{run, sequence3},
};

fn main() -> Result<()> {
    type B = CacheAlignedBuffer<f32>;
    let buffer_size = 4096;
    let num_buffers = 4;

    let head_elements = 100_000_000;

    let buffers = std::iter::repeat_with(|| Quantum::new(B::new(buffer_size))).take(num_buffers);

    let mut fg = Flowgraph::new();

    let source = fg.add_block(NullSource::<Quantum<B>>::new());
    let head = fg.add_block(Head::<Quantum<B>, Spsc, SpscRef>::new(head_elements));
    let sink = fg.add_block(NullSink::<Quantum<B>>::new());

    let mut circ = fg.new_circuit(buffers);
    fg.connect(&mut circ, source.output(), head.input())?;
    fg.connect_with_return(&mut circ, head.output(), sink.input(), source.input())?;

    let mut fg = fg.validate()?;

    let source = fg.extract_block(source)?;
    let head = fg.extract_block(head)?;
    let sink = fg.extract_block(sink)?;

    println!("running flowgraph...");
    block_on(run(sequence3(
        source.into_stream(),
        head.into_stream(),
        sink.into_stream(),
    )))?;
    println!("flowgraph finished");

    Ok(())
}
