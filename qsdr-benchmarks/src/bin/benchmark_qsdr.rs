use anyhow::Result;
use clap::{Parser, Subcommand};
use qsdr::{
    Block, BlockWorkStatus, Buffer, Channel, Flowgraph, Quantum, Receiver, Run, WorkInPlace,
    WorkSink, WorkStatus,
    blocks::basic::RefClone,
    buffers::CacheAlignedBuffer,
    channels::{Spsc, SpscRef},
    kernels,
    ports::{PortInQ, PortOut, PortOutQ, PortRefInQ, PortSource},
    scheduler::{run, sequence2, sequence3, sequence4, sequence5},
};
use qsdr_benchmarks::{
    affinity::{get_core_ids, pin_cpu},
    futures::executor::block_on,
};
use rand::prelude::*;
use std::{thread, time::Instant};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Buffer size (bytes).
    #[arg(long, default_value_t = 16384)]
    buffer_size: usize,
    /// Number of buffers.
    #[arg(long, default_value_t = 2)]
    num_buffers: usize,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Single kernel on a single core.
    SingleCore,
    /// Multiple kernels spread over multiple cores.
    MultiKernel(MultiKernel),
    /// Multiple kernels spread over multiple cores using the Tokio runtime.
    MultiKernelTokio(MultiKernel),
    /// Multiple kernels spread over multiple cores using async-executor.
    MultiKernelAsyncExecutor(MultiKernel),
    /// Multiple kernels spread over multiple cores using one thread per block
    /// (with no CPU affinities).
    MultiKernelTPB(MultiKernel),
    /// Benchmark of RefClone block.
    BenchmarkRef,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[allow(clippy::enum_variant_names)]
enum Executor {
    TokioRuntime,
    AsyncExecutor,
    ThreadPerBlock,
}

#[derive(Parser, Debug)]
struct MultiKernel {
    /// Number of kernels.
    #[arg(long, default_value_t = 4)]
    num_kernels: usize,
    /// Number of CPUs.
    #[arg(long, default_value_t = 4)]
    num_cpus: usize,
}

#[derive(Block, Debug)]
#[work(WorkInPlace)]
struct Saxpy<Bf32, Cin = Spsc, Cout = Spsc>
where
    Bf32: Buffer<Item = f32>,
    Cin: Channel,
    Cin::Receiver<Quantum<Bf32>>: Receiver<Quantum<Bf32>>,
    Cout: Channel,
{
    #[port]
    input: PortInQ<Bf32, Cin>,
    #[port]
    output: PortOutQ<Bf32, Cout>,
    saxpy_kernel: kernels::saxpy::Saxpy,
}

impl<Bf32, Cin, Cout> Saxpy<Bf32, Cin, Cout>
where
    Bf32: Buffer<Item = f32>,
    Cin: Channel,
    Cin::Receiver<Quantum<Bf32>>: Receiver<Quantum<Bf32>>,
    Cout: Channel,
{
    fn new(a: f32, b: f32) -> Self {
        Self {
            input: Default::default(),
            output: Default::default(),
            saxpy_kernel: kernels::saxpy::Saxpy::new(a, b),
        }
    }
}

impl<Bf32, Cin, Cout> WorkInPlace<Quantum<Bf32>> for Saxpy<Bf32, Cin, Cout>
where
    Bf32: Buffer<Item = f32>,
    Cin: Channel,
    Cin::Receiver<Quantum<Bf32>>: Receiver<Quantum<Bf32>>,
    Cout: Channel,
{
    async fn work_in_place(&mut self, quantum: &mut Quantum<Bf32>) -> Result<WorkStatus> {
        self.saxpy_kernel.run_best(quantum.as_mut_slice());
        Ok(Run)
    }
}

#[derive(Block, Debug)]
#[work(WorkInPlace)]
struct DummySource<T, Cin = Spsc, Cout = Spsc>
where
    Cin: Channel,
    Cin::Receiver<T>: Receiver<T>,
    Cout: Channel,
{
    #[port]
    input: PortSource<T, Cin>,
    #[port]
    output: PortOut<T, Cout>,
}

impl<T, Cin, Cout> DummySource<T, Cin, Cout>
where
    Cin: Channel,
    Cin::Receiver<T>: Receiver<T>,
    Cout: Channel,
{
    fn new() -> Self {
        Self {
            input: Default::default(),
            output: Default::default(),
        }
    }
}

impl<T, Cin, Cout> Default for DummySource<T, Cin, Cout>
where
    Cin: Channel,
    Cin::Receiver<T>: Receiver<T>,
    Cout: Channel,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, Cin, Cout> WorkInPlace<T> for DummySource<T, Cin, Cout>
where
    Cin: Channel,
    Cin::Receiver<T>: Receiver<T>,
    Cout: Channel,
{
    async fn work_in_place(&mut self, _: &mut T) -> Result<WorkStatus> {
        Ok(Run)
    }
}

#[derive(Block, Debug)]
#[work(WorkSink)]
struct BenchmarkSink<B, Cin = SpscRef>
where
    B: Buffer,
    Cin: Channel,
{
    #[port]
    input: PortRefInQ<B, Cin>,
    count: u64,
    time: Instant,
}

impl<B, Cin> BenchmarkSink<B, Cin>
where
    B: Buffer,
    Cin: Channel,
{
    const MEASURE_EVERY: u64 = 1 << 29;

    fn new() -> Self {
        Self {
            input: Default::default(),
            count: 0,
            time: Instant::now(),
        }
    }
}

impl<B, Cin> Default for BenchmarkSink<B, Cin>
where
    B: Buffer,
    Cin: Channel,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<B, Cin> WorkSink<Quantum<B>> for BenchmarkSink<B, Cin>
where
    B: Buffer,
    Cin: Channel,
{
    async fn work_sink(&mut self, quantum: &Quantum<B>) -> Result<BlockWorkStatus> {
        self.count += quantum.len() as u64;
        if self.count >= Self::MEASURE_EVERY {
            let now = Instant::now();
            let elapsed = now - self.time;
            let samples_per_sec = self.count as f64 / elapsed.as_secs_f64();
            println!("samples/s = {samples_per_sec:.3e}");
            self.count = 0;
            self.time = now;
        }
        Ok(BlockWorkStatus::Run)
    }
}

type Buff = CacheAlignedBuffer<f32>;

fn single_core(args: &Args) -> Result<()> {
    pin_cpu()?;
    let mut rng = rand::rng();
    let dummy_source = DummySource::<Quantum<Buff>>::new();
    let saxpy = Saxpy::<Buff, Spsc, SpscRef>::new(rng.random(), rng.random());
    let benchmark_sink = BenchmarkSink::new();

    let buf_len = args.buffer_size / std::mem::size_of::<<Buff as Buffer>::Item>();
    let buffers = std::iter::repeat_with(|| Quantum::new(Buff::from_fn(buf_len, |_| rng.random())))
        .take(args.num_buffers);

    let mut fg = Flowgraph::new();
    let dummy_source = fg.add_block(dummy_source);
    let saxpy = fg.add_block(saxpy);
    let benchmark_sink = fg.add_block(benchmark_sink);

    let mut circuit = fg.new_circuit(buffers);
    fg.connect(&mut circuit, dummy_source.output(), saxpy.input())?;
    fg.connect_with_return(
        &mut circuit,
        saxpy.output(),
        benchmark_sink.input(),
        dummy_source.input(),
    )?;
    let mut fg = fg.validate()?;
    let dummy_source = fg.extract_block(dummy_source)?;
    let saxpy = fg.extract_block(saxpy)?;
    let benchmark_sink = fg.extract_block(benchmark_sink)?;

    block_on(run(sequence3(
        dummy_source.into_stream(),
        saxpy.into_stream(),
        benchmark_sink.into_stream(),
    )))
}

macro_rules! sequence {
    ($stream0:expr) => {
        $stream0
    };
    ($stream0:expr, $stream1:expr) => {
        sequence2($stream0, $stream1)
    };
    ($stream0:expr, $stream1:expr, $stream2:expr) => {
        sequence3($stream0, $stream1, $stream2)
    };
    ($stream0:expr, $stream1:expr, $stream2:expr, $stream3:expr) => {
        sequence4($stream0, $stream1, $stream2, $stream3)
    };
    ($stream0:expr, $stream1:expr, $stream2:expr, $stream3:expr, $stream4:expr) => {
        sequence5($stream0, $stream1, $stream2, $stream3, $stream4)
    };
}

fn multi_kernel(args: &Args, sub: &MultiKernel) -> Result<()> {
    let core_ids = get_core_ids()?;
    anyhow::ensure!(
        core_ids.len() >= sub.num_cpus,
        "requested more CPUs ({}) than cores are in the system ({})",
        sub.num_cpus,
        core_ids.len()
    );

    macro_rules! make_conn {
        ($fg:expr, $circuit:expr, $block_first:expr, $block0:expr, $block1:expr) => {
            $fg.connect_with_return(
                &mut $circuit,
                $block0.output(),
                $block1.input(),
                $block_first.input(),
            )?;
        };
        ($fg:expr, $circuit:expr, $block_first:expr, $block0:expr, $(,)? $block1:expr, $($blocks:expr),+) => {
            $fg.connect(&mut $circuit, $block0.output(), $block1.input())?;
            make_conn!($fg, $circuit, $block_first, $block1, $($blocks),+);
        };
    }

    macro_rules! make_fg {
        ($dummy_source:ident, $benchmark_sink:ident, $last_saxpy:ident, $($saxpy:ident),*) => {
            let mut rng = rand::rng();
            let $dummy_source = DummySource::<Quantum<Buff>>::new();
            $(
                let $saxpy = Saxpy::<Buff, Spsc, Spsc>::new(rng.random(), rng.random());
            )*
            let $last_saxpy = Saxpy::<Buff, Spsc, SpscRef>::new(rng.random(), rng.random());
            let $benchmark_sink = BenchmarkSink::new();

            let buf_len = args.buffer_size / std::mem::size_of::<<Buff as Buffer>::Item>();
            let buffers = std::iter::repeat_with(|| Quantum::new(Buff::from_fn(buf_len, |_| rng.random())))
                .take(args.num_buffers);

            let mut fg = Flowgraph::new();
            let $dummy_source = fg.add_block($dummy_source);
            $(
                let $saxpy = fg.add_block($saxpy);
            )*
            let $last_saxpy = fg.add_block($last_saxpy);
            let $benchmark_sink = fg.add_block($benchmark_sink);

            let mut circuit = fg.new_circuit(buffers);
            make_conn!(fg, circuit, $dummy_source, $dummy_source, $($saxpy),*, $last_saxpy, $benchmark_sink);
            let mut fg = fg.validate()?;
            let $dummy_source = fg.extract_block($dummy_source)?;
            $(
                let $saxpy = fg.extract_block($saxpy)?;
            )*
            let $last_saxpy = fg.extract_block($last_saxpy)?;
            let $benchmark_sink = fg.extract_block($benchmark_sink)?;
        }
    }

    macro_rules! make_threads {
        ($($core:expr, $($block:expr),*);*) => {
            let mut threads = Vec::new();
            $(
                let core = $core;
                let fut = run(sequence!($($block.into_stream()),*));
                let thread = thread::spawn(move || {
                    core_affinity::set_for_current(core);
                    block_on(fut)
                });
                threads.push(thread);
            )*
                for thread in threads.into_iter() {
                    thread.join().unwrap()?;
                }
        }
    }

    match (sub.num_cpus, sub.num_kernels) {
        (1, 1) => {
            make_fg!(dummy_source, benchmark_sink, saxpy0,);
            make_threads!(core_ids[0], dummy_source, saxpy0, benchmark_sink);
        }
        (1, 2) => {
            make_fg!(dummy_source, benchmark_sink, saxpy1, saxpy0);
            make_threads!(core_ids[0], dummy_source, saxpy0, saxpy1, benchmark_sink);
        }
        (1, 3) => {
            make_fg!(dummy_source, benchmark_sink, saxpy2, saxpy0, saxpy1);
            make_threads!(
                core_ids[0],
                dummy_source,
                saxpy0,
                saxpy1,
                saxpy2,
                benchmark_sink
            );
        }
        (2, 2) => {
            make_fg!(dummy_source, benchmark_sink, saxpy1, saxpy0);
            make_threads!(core_ids[0], dummy_source, saxpy0;
                          core_ids[1], saxpy1, benchmark_sink);
        }
        (2, 3) => {
            make_fg!(dummy_source, benchmark_sink, saxpy2, saxpy0, saxpy1);
            make_threads!(core_ids[0], dummy_source, saxpy0, saxpy1;
                          core_ids[1], saxpy2, benchmark_sink);
        }
        (2, 4) => {
            make_fg!(dummy_source, benchmark_sink, saxpy3, saxpy0, saxpy1, saxpy2);
            make_threads!(core_ids[0], dummy_source, saxpy0, saxpy1;
                          core_ids[1], saxpy2, saxpy3, benchmark_sink);
        }
        (2, 5) => {
            make_fg!(
                dummy_source,
                benchmark_sink,
                saxpy4,
                saxpy0,
                saxpy1,
                saxpy2,
                saxpy3
            );
            make_threads!(core_ids[0], dummy_source, saxpy0, saxpy1, saxpy2;
                          core_ids[1], saxpy3, saxpy4, benchmark_sink);
        }
        (2, 6) => {
            make_fg!(
                dummy_source,
                benchmark_sink,
                saxpy5,
                saxpy0,
                saxpy1,
                saxpy2,
                saxpy3,
                saxpy4
            );
            make_threads!(core_ids[0], dummy_source, saxpy0, saxpy1, saxpy2;
                          core_ids[1], saxpy3, saxpy4, saxpy5, benchmark_sink);
        }
        (3, 3) => {
            make_fg!(dummy_source, benchmark_sink, saxpy2, saxpy0, saxpy1);
            make_threads!(core_ids[0], dummy_source, saxpy0;
                          core_ids[1], saxpy1;
                          core_ids[2], saxpy2, benchmark_sink);
        }
        (3, 4) => {
            make_fg!(dummy_source, benchmark_sink, saxpy3, saxpy0, saxpy1, saxpy2);
            make_threads!(core_ids[0], dummy_source, saxpy0, saxpy1;
                          core_ids[1], saxpy2;
                          core_ids[2], saxpy3, benchmark_sink);
        }
        (3, 5) => {
            make_fg!(
                dummy_source,
                benchmark_sink,
                saxpy4,
                saxpy0,
                saxpy1,
                saxpy2,
                saxpy3
            );
            make_threads!(core_ids[0], dummy_source, saxpy0, saxpy1;
                          core_ids[1], saxpy2, saxpy3;
                          core_ids[2], saxpy4, benchmark_sink);
        }
        (3, 6) => {
            make_fg!(
                dummy_source,
                benchmark_sink,
                saxpy5,
                saxpy0,
                saxpy1,
                saxpy2,
                saxpy3,
                saxpy4
            );
            make_threads!(core_ids[0], dummy_source, saxpy0, saxpy1;
                          core_ids[1], saxpy2, saxpy3;
                          core_ids[2], saxpy4, saxpy5, benchmark_sink);
        }
        (3, 7) => {
            make_fg!(
                dummy_source,
                benchmark_sink,
                saxpy6,
                saxpy0,
                saxpy1,
                saxpy2,
                saxpy3,
                saxpy4,
                saxpy5
            );
            make_threads!(core_ids[0], dummy_source, saxpy0, saxpy1, saxpy2;
                          core_ids[1], saxpy3, saxpy4;
                          core_ids[2], saxpy5, saxpy6, benchmark_sink);
        }
        (3, 8) => {
            make_fg!(
                dummy_source,
                benchmark_sink,
                saxpy7,
                saxpy0,
                saxpy1,
                saxpy2,
                saxpy3,
                saxpy4,
                saxpy5,
                saxpy6
            );
            make_threads!(core_ids[0], dummy_source, saxpy0, saxpy1, saxpy2;
                          core_ids[1], saxpy3, saxpy4, saxpy5;
                          core_ids[2], saxpy6, saxpy7, benchmark_sink);
        }
        (3, 9) => {
            make_fg!(
                dummy_source,
                benchmark_sink,
                saxpy8,
                saxpy0,
                saxpy1,
                saxpy2,
                saxpy3,
                saxpy4,
                saxpy5,
                saxpy6,
                saxpy7
            );
            make_threads!(core_ids[0], dummy_source, saxpy0, saxpy1, saxpy2;
                          core_ids[1], saxpy3, saxpy4, saxpy5;
                          core_ids[2], saxpy6, saxpy7, saxpy8, benchmark_sink);
        }
        (4, 4) => {
            make_fg!(dummy_source, benchmark_sink, saxpy3, saxpy0, saxpy1, saxpy2);
            make_threads!(core_ids[0], dummy_source, saxpy0;
                          core_ids[1], saxpy1;
                          core_ids[2], saxpy2;
                          core_ids[3], saxpy3, benchmark_sink);
        }
        (4, 5) => {
            make_fg!(
                dummy_source,
                benchmark_sink,
                saxpy4,
                saxpy0,
                saxpy1,
                saxpy2,
                saxpy3
            );
            make_threads!(core_ids[0], dummy_source, saxpy0, saxpy1;
                          core_ids[1], saxpy2;
                          core_ids[2], saxpy3;
                          core_ids[3], saxpy4, benchmark_sink);
        }
        (4, 6) => {
            make_fg!(
                dummy_source,
                benchmark_sink,
                saxpy5,
                saxpy0,
                saxpy1,
                saxpy2,
                saxpy3,
                saxpy4
            );
            make_threads!(core_ids[0], dummy_source, saxpy0, saxpy1;
                          core_ids[1], saxpy2, saxpy3;
                          core_ids[2], saxpy4;
                          core_ids[3], saxpy5, benchmark_sink);
        }
        (4, 7) => {
            make_fg!(
                dummy_source,
                benchmark_sink,
                saxpy6,
                saxpy0,
                saxpy1,
                saxpy2,
                saxpy3,
                saxpy4,
                saxpy5
            );
            make_threads!(core_ids[0], dummy_source, saxpy0, saxpy1;
                          core_ids[1], saxpy2, saxpy3;
                          core_ids[2], saxpy4, saxpy5;
                          core_ids[3], saxpy6, benchmark_sink);
        }
        (4, 8) => {
            make_fg!(
                dummy_source,
                benchmark_sink,
                saxpy7,
                saxpy0,
                saxpy1,
                saxpy2,
                saxpy3,
                saxpy4,
                saxpy5,
                saxpy6
            );
            make_threads!(core_ids[0], dummy_source, saxpy0, saxpy1;
                          core_ids[1], saxpy2, saxpy3;
                          core_ids[2], saxpy4, saxpy5;
                          core_ids[3], saxpy6, saxpy7, benchmark_sink);
        }
        (4, 9) => {
            make_fg!(
                dummy_source,
                benchmark_sink,
                saxpy8,
                saxpy0,
                saxpy1,
                saxpy2,
                saxpy3,
                saxpy4,
                saxpy5,
                saxpy6,
                saxpy7
            );
            make_threads!(core_ids[0], dummy_source, saxpy0, saxpy1, saxpy2;
                          core_ids[1], saxpy3, saxpy4;
                          core_ids[2], saxpy5, saxpy6;
                          core_ids[3], saxpy7, saxpy8, benchmark_sink);
        }
        (4, 10) => {
            make_fg!(
                dummy_source,
                benchmark_sink,
                saxpy9,
                saxpy0,
                saxpy1,
                saxpy2,
                saxpy3,
                saxpy4,
                saxpy5,
                saxpy6,
                saxpy7,
                saxpy8
            );
            make_threads!(core_ids[0], dummy_source, saxpy0, saxpy1, saxpy2;
                          core_ids[1], saxpy3, saxpy4, saxpy5;
                          core_ids[2], saxpy6, saxpy7;
                          core_ids[3], saxpy8, saxpy9, benchmark_sink);
        }
        (4, 11) => {
            make_fg!(
                dummy_source,
                benchmark_sink,
                saxpy10,
                saxpy0,
                saxpy1,
                saxpy2,
                saxpy3,
                saxpy4,
                saxpy5,
                saxpy6,
                saxpy7,
                saxpy8,
                saxpy9
            );
            make_threads!(core_ids[0], dummy_source, saxpy0, saxpy1, saxpy2;
                          core_ids[1], saxpy3, saxpy4, saxpy5;
                          core_ids[2], saxpy6, saxpy7, saxpy8;
                          core_ids[3], saxpy9, saxpy10, benchmark_sink);
        }
        (4, 12) => {
            make_fg!(
                dummy_source,
                benchmark_sink,
                saxpy11,
                saxpy0,
                saxpy1,
                saxpy2,
                saxpy3,
                saxpy4,
                saxpy5,
                saxpy6,
                saxpy7,
                saxpy8,
                saxpy9,
                saxpy10
            );
            make_threads!(core_ids[0], dummy_source, saxpy0, saxpy1, saxpy2;
                          core_ids[1], saxpy3, saxpy4, saxpy5;
                          core_ids[2], saxpy6, saxpy7, saxpy8;
                          core_ids[3], saxpy9, saxpy10, saxpy11, benchmark_sink);
        }
        _ => {
            anyhow::bail!(
                "unsupported number of CPUs {} and kernels {}",
                sub.num_cpus,
                sub.num_kernels
            );
        }
    }

    Ok(())
}

fn multi_kernel_executor(args: &Args, sub: &MultiKernel, executor: Executor) -> Result<()> {
    let core_ids = get_core_ids()?;
    anyhow::ensure!(
        core_ids.len() >= sub.num_cpus,
        "requested more CPUs ({}) than cores are in the system ({})",
        sub.num_cpus,
        core_ids.len()
    );

    let mut rng = rand::rng();
    let dummy_source = DummySource::<Quantum<Buff>>::new();
    let benchmark_sink = BenchmarkSink::new();

    let mut fg = Flowgraph::new();
    let dummy_source = fg.add_block(dummy_source);
    let mut saxpys = std::iter::repeat_with(|| {
        fg.add_block(Saxpy::<Buff, Spsc, Spsc>::new(rng.random(), rng.random()))
    })
    .take(sub.num_kernels - 1)
    .collect::<Vec<_>>();
    let last_saxpy = fg.add_block(Saxpy::<Buff, Spsc, SpscRef>::new(
        rng.random(),
        rng.random(),
    ));
    let benchmark_sink = fg.add_block(benchmark_sink);

    let buf_len = args.buffer_size / std::mem::size_of::<<Buff as Buffer>::Item>();
    let buffers = std::iter::repeat_with(|| Quantum::new(Buff::from_fn(buf_len, |_| rng.random())))
        .take(args.num_buffers);

    let mut circuit = fg.new_circuit(buffers);
    if sub.num_kernels == 1 {
        fg.connect(&mut circuit, dummy_source.output(), last_saxpy.input())?;
    } else {
        fg.connect(&mut circuit, dummy_source.output(), saxpys[0].input())?;
        for n in 0..saxpys[1..].len() {
            fg.connect(&mut circuit, saxpys[n].output(), saxpys[n + 1].input())?;
        }
        fg.connect(
            &mut circuit,
            saxpys.last_mut().unwrap().output(),
            last_saxpy.input(),
        )?;
    }
    fg.connect_with_return(
        &mut circuit,
        last_saxpy.output(),
        benchmark_sink.input(),
        dummy_source.input(),
    )?;
    let mut fg = fg.validate()?;
    let dummy_source = fg.extract_block(dummy_source)?;
    let mut saxpy_blocks = Vec::with_capacity(saxpys.len());
    for saxpy in saxpys.into_iter() {
        saxpy_blocks.push(fg.extract_block(saxpy)?);
    }
    let last_saxpy = fg.extract_block(last_saxpy)?;
    let benchmark_sink = fg.extract_block(benchmark_sink)?;

    match executor {
        Executor::TokioRuntime => {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(sub.num_cpus)
                .on_thread_start(move || {
                    static THREAD_NUM: std::sync::atomic::AtomicU32 =
                        std::sync::atomic::AtomicU32::new(0);
                    let n = usize::try_from(
                        THREAD_NUM.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
                    )
                    .unwrap();
                    core_affinity::set_for_current(core_ids[n]);
                })
                .build()?;
            let mut handles = Vec::new();
            handles.push(rt.spawn(run(dummy_source.into_stream())));
            for saxpy in saxpy_blocks.into_iter() {
                handles.push(rt.spawn(run(saxpy.into_stream())));
            }
            handles.push(rt.spawn(run(last_saxpy.into_stream())));
            handles.push(rt.spawn(run(benchmark_sink.into_stream())));
            let all_handles = futures::future::select_all(handles);
            rt.block_on(all_handles).0??;
        }
        Executor::AsyncExecutor => {
            let ex = async_executor::Executor::new();
            let mut tasks = Vec::new();
            tasks.push(ex.spawn(run(dummy_source.into_stream())));
            for saxpy in saxpy_blocks.into_iter() {
                tasks.push(ex.spawn(run(saxpy.into_stream())));
            }
            tasks.push(ex.spawn(run(last_saxpy.into_stream())));
            tasks.push(ex.spawn(run(benchmark_sink.into_stream())));
            let all_tasks = futures::future::select_all(tasks);
            thread::scope(|s| {
                for &core in core_ids.iter().take(sub.num_cpus) {
                    let ex = &ex;
                    s.spawn(move || {
                        core_affinity::set_for_current(core);
                        block_on(ex.run(std::future::pending::<()>()))
                    });
                }
                block_on(all_tasks).0
            })?;
        }
        Executor::ThreadPerBlock => {
            let mut handles = Vec::new();
            handles.push(thread::spawn(move || {
                block_on(run(dummy_source.into_stream()))
            }));
            for saxpy in saxpy_blocks.into_iter() {
                handles.push(thread::spawn(move || block_on(run(saxpy.into_stream()))));
            }
            handles.push(thread::spawn(move || {
                block_on(run(last_saxpy.into_stream()))
            }));
            handles.push(thread::spawn(move || {
                block_on(run(benchmark_sink.into_stream()))
            }));
            for h in handles.into_iter() {
                h.join().unwrap()?;
            }
        }
    }

    Ok(())
}

fn benchmark_ref(args: &Args) -> Result<()> {
    let mut rng = rand::rng();
    let dummy_source = DummySource::<Quantum<Buff>, Spsc, SpscRef>::new();
    let ref_clone = RefClone::<Buff, SpscRef, SpscRef, Spsc>::new();
    let benchmark_sink = BenchmarkSink::new();

    let buf_len = args.buffer_size / std::mem::size_of::<<Buff as Buffer>::Item>();
    let buffers0 =
        std::iter::repeat_with(|| Quantum::new(Buff::from_fn(buf_len, |_| rng.random())))
            .take(args.num_buffers)
            .collect::<Vec<_>>();
    let buffers1 =
        std::iter::repeat_with(|| Quantum::new(Buff::from_fn(buf_len, |_| rng.random())))
            .take(args.num_buffers)
            .collect::<Vec<_>>();

    let mut fg = Flowgraph::new();
    let dummy_source = fg.add_block(dummy_source);
    let ref_clone = fg.add_block(ref_clone);
    let benchmark_sink = fg.add_block(benchmark_sink);

    let mut circuit0 = fg.new_circuit(buffers0.into_iter());
    let mut circuit1 = fg.new_circuit(buffers1.into_iter());
    fg.connect_with_return(
        &mut circuit0,
        dummy_source.output(),
        ref_clone.input(),
        dummy_source.input(),
    )?;
    fg.connect_with_return(
        &mut circuit1,
        ref_clone.output(),
        benchmark_sink.input(),
        ref_clone.source(),
    )?;
    let mut fg = fg.validate()?;
    let dummy_source = fg.extract_block(dummy_source)?;
    let ref_clone = fg.extract_block(ref_clone)?;
    let benchmark_sink = fg.extract_block(benchmark_sink)?;

    block_on(run(sequence3(
        dummy_source.into_stream(),
        ref_clone.into_stream(),
        benchmark_sink.into_stream(),
    )))
}

fn main() -> Result<()> {
    let args = Args::parse();
    match &args.command {
        Command::SingleCore => single_core(&args)?,
        Command::MultiKernel(sub) => multi_kernel(&args, sub)?,
        Command::MultiKernelTokio(sub) => {
            multi_kernel_executor(&args, sub, Executor::TokioRuntime)?
        }
        Command::MultiKernelAsyncExecutor(sub) => {
            multi_kernel_executor(&args, sub, Executor::AsyncExecutor)?
        }
        Command::MultiKernelTPB(sub) => {
            multi_kernel_executor(&args, sub, Executor::ThreadPerBlock)?
        }
        Command::BenchmarkRef => benchmark_ref(&args)?,
    }
    Ok(())
}
