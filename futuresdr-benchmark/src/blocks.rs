use futuresdr::{
    macros::async_trait,
    runtime::{
        Block, BlockMeta, BlockMetaBuilder, Kernel, MessageIo, MessageIoBuilder, Result, StreamIo,
        StreamIoBuilder, WorkIo,
    },
};
use qsdr::kernels::saxpy::Saxpy as SaxpyKernel;
use std::{marker::PhantomData, time::Instant};

pub struct DummySource<T: Send + 'static> {
    _phantom: PhantomData<T>,
}

impl<T: Send + 'static> DummySource<T> {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> Block {
        Block::new(
            BlockMetaBuilder::new("DummySource").build(),
            StreamIoBuilder::new().add_output::<T>("out").build(),
            MessageIoBuilder::new().build(),
            DummySource::<T> {
                _phantom: PhantomData,
            },
        )
    }
}

#[async_trait]
impl<T: Send + 'static> Kernel for DummySource<T> {
    async fn work(
        &mut self,
        _io: &mut WorkIo,
        sio: &mut StreamIo,
        _mio: &mut MessageIo<Self>,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        let output = sio.output(0);
        let len = output.slice_unchecked::<T>().len();
        output.produce(len);
        Ok(())
    }
}

pub struct BenchmarkSink<T: Send + 'static> {
    count: u64,
    time: Instant,
    _phantom: PhantomData<T>,
}

impl<T: Send + 'static> BenchmarkSink<T> {
    const MEASURE_EVERY: u64 = 1 << 29;

    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> Block {
        Block::new(
            BlockMetaBuilder::new("BenchmarkSink").build(),
            StreamIoBuilder::new().add_input::<T>("in").build(),
            MessageIoBuilder::new().build(),
            BenchmarkSink::<T> {
                count: 0,
                time: Instant::now(),
                _phantom: PhantomData,
            },
        )
    }
}

#[async_trait]
impl<T: Send + 'static> Kernel for BenchmarkSink<T> {
    async fn work(
        &mut self,
        _io: &mut WorkIo,
        sio: &mut StreamIo,
        _mio: &mut MessageIo<Self>,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        let input = sio.input(0);
        let len = input.slice_unchecked::<T>().len();

        if len > 0 {
            self.count += len as u64;
            if self.count >= Self::MEASURE_EVERY {
                let now = Instant::now();
                let elapsed = now - self.time;
                let samples_per_sec = self.count as f64 / elapsed.as_secs_f64();
                println!("samples/s = {samples_per_sec:.3e}");
                self.count = 0;
                self.time = now;
            }

            input.consume(len);
        }

        Ok(())
    }
}

pub struct Saxpy {
    kernel: SaxpyKernel,
}

impl Saxpy {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(a: f32, b: f32) -> Block {
        Block::new(
            BlockMetaBuilder::new("Saxpy").build(),
            StreamIoBuilder::new()
                .add_input::<f32>("in")
                .add_output::<f32>("out")
                .build(),
            MessageIoBuilder::new().build(),
            Saxpy {
                kernel: SaxpyKernel::new(a, b),
            },
        )
    }
}

#[async_trait]
impl Kernel for Saxpy {
    async fn work(
        &mut self,
        _io: &mut WorkIo,
        sio: &mut StreamIo,
        _mio: &mut MessageIo<Self>,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        let in_slice = sio.input(0).slice_unchecked::<f32>();
        let out_slice = sio.output(0).slice_unchecked::<f32>();

        let len = in_slice.len().min(out_slice.len());
        // enforce constraints for cortex_a53 saxpy kernel: at least 64 f32's
        // and multiple of 32 f32's
        let len = (len / 32) * 32;
        if len >= 64 {
            self.kernel
                .run_best_out_of_place(&in_slice[..len], &mut out_slice[..len]);
            sio.input(0).consume(len);
            sio.output(0).produce(len);
        }

        Ok(())
    }
}
