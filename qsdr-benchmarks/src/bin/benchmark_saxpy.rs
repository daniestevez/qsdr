use anyhow::Result;
use clap::{Parser, Subcommand};
use qsdr::{channel::spsc, kernels::saxpy::Saxpy};
use qsdr_benchmarks::{
    affinity::{get_core_ids, pin_cpu},
    asm::get_cpu_cycles,
    futures::executor::block_on,
    Buffer,
};
use rand::prelude::*;
use std::{thread, time::Instant};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// CPU clock frequency.
    #[arg(long, default_value_t = 1.333e9)]
    clock_frequency: f64,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Single kernel on a single core.
    SingleCore(SingleCore),
    /// Multiple cores, with one kernel per core.
    MultiCore(MultiCore),
    /// Multi-core with async channels.
    MultiCoreAsync(MultiCore),
    /// Multi-core with fake message passing: each core works on its own buffer
    /// all the time.
    MultiCoreFakeMessagePassing(MultiCore),
    /// Multi-core with async channels and fake message passing.
    MultiCoreFakeMessagePassingAsync(MultiCore),
    /// Multiple kernels, pinned over multiple cores in order.
    MultiKernel(MultiKernel),
    /// Multiple kernels with async channels.
    MultiKernelAsync(MultiKernel),
    /// Test several buffer sizes in a single-core benchmark.
    ScanBufferSize(ScanBufferSize),
}

#[derive(Parser, Debug)]
struct SingleCore {
    /// Buffer size (bytes).
    #[arg(long, default_value_t = 16384)]
    buffer_size: usize,
    /// Number of buffers.
    #[arg(long, default_value_t = 1)]
    num_buffers: usize,
    /// Measurement interval (seconds).
    #[arg(long, default_value_t = 1.0)]
    measurement_interval: f64,
}

#[derive(Parser, Debug)]
struct MultiCore {
    /// Buffer size (bytes).
    #[arg(long, default_value_t = 16384)]
    buffer_size: usize,
    /// Number of buffers.
    #[arg(long, default_value_t = 5)]
    num_buffers: usize,
    /// Number of CPUs.
    #[arg(long, default_value_t = 4)]
    num_cpus: usize,
    /// Measurement interval (seconds).
    #[arg(long, default_value_t = 1.0)]
    measurement_interval: f64,
}

#[derive(Parser, Debug)]
struct MultiKernel {
    /// Buffer size (bytes).
    #[arg(long, default_value_t = 16384)]
    buffer_size: usize,
    /// Number of buffers.
    #[arg(long, default_value_t = 5)]
    num_buffers: usize,
    /// Number of kernels.
    #[arg(long, default_value_t = 4)]
    num_kernels: usize,
    /// Number of CPUs.
    #[arg(long, default_value_t = 4)]
    num_cpus: usize,
    /// Measurement interval (seconds).
    #[arg(long, default_value_t = 1.0)]
    measurement_interval: f64,
}

#[derive(Parser, Debug)]
struct ScanBufferSize {
    /// Measurement time for each buffer (seconds).
    #[arg(long, default_value_t = 10.0)]
    measurement_time: f64,
}

#[inline(always)]
fn begin_measurement() -> (Instant, u64) {
    (Instant::now(), get_cpu_cycles())
}

#[inline(always)]
fn make_measurement(time: Instant, cycles: u64, samples_per_measurement: usize) -> (Instant, u64) {
    let cycles_now = get_cpu_cycles();
    let now = Instant::now();
    let elapsed = now - time;
    let samples_per_sec = samples_per_measurement as f64 / elapsed.as_secs_f64();
    let delta_cycles = cycles_now.wrapping_sub(cycles);
    let samples_per_cycle = samples_per_measurement as f64 / delta_cycles as f64;
    let cycles_per_sec = delta_cycles as f64 / elapsed.as_secs_f64();
    println!(
        "samples/s = {samples_per_sec:.3e}, \
         samples/cycle = {samples_per_cycle:.3}, \
         cycles/s = {cycles_per_sec:.3e}"
    );
    (now, cycles_now)
}

#[inline(always)]
fn update_measurement(time: &mut Instant, cycles: &mut u64, samples_per_measurement: usize) {
    let m = make_measurement(*time, *cycles, samples_per_measurement);
    *time = m.0;
    *cycles = m.1;
}

macro_rules! check_buffer_args {
    ($args:expr) => {
        anyhow::ensure!(
            $args.buffer_size % std::mem::size_of::<f32>() == 0,
            "buffer size must be a multiple of size_of::<f32>"
        );
        anyhow::ensure!($args.num_buffers > 0, "number of buffers cannot be zero");
    };
}

fn single_core(args: &Args, args_sub: &SingleCore) -> Result<()> {
    check_buffer_args!(args_sub);
    pin_cpu()?;
    let mut rng = rand::thread_rng();
    let buf_len = args_sub.buffer_size / std::mem::size_of::<f32>();
    let saxpy = Saxpy::new(rng.gen(), rng.gen());
    let mut make_buffer = || Buffer::<f32>::from_fn(buf_len, |_| rng.gen());

    let clocks_per_sample = 2.0;
    let samples_per_iter = buf_len * args_sub.num_buffers;
    let measurement_iters = (args.clock_frequency * args_sub.measurement_interval
        / (clocks_per_sample * samples_per_iter as f64))
        .ceil() as usize;
    let samples_per_measurement = measurement_iters * samples_per_iter;

    if args_sub.num_buffers > 1 {
        let mut buffers: Vec<Buffer<f32>> = std::iter::repeat_with(make_buffer)
            .take(args_sub.num_buffers)
            .collect();
        let (mut time, mut cycles) = begin_measurement();
        loop {
            for _ in 0..measurement_iters {
                for buf in buffers.iter_mut() {
                    saxpy.run_best(&mut buf[..]);
                }
            }
            update_measurement(&mut time, &mut cycles, samples_per_measurement);
        }
    } else {
        // optimized case for only one buffer
        let mut buffer = make_buffer();
        let (mut time, mut cycles) = begin_measurement();
        loop {
            for _ in 0..measurement_iters {
                saxpy.run_best(&mut buffer[..]);
            }
            update_measurement(&mut time, &mut cycles, samples_per_measurement);
        }
    }
}

macro_rules! if_fake {
    (fake, $($tt:tt)*) => { $($tt)* };
    (not_fake, $($tt:tt)*) => {};
}

macro_rules! if_not_fake {
    (fake, $($tt:tt)*) => {};
    (not_fake, $($tt:tt)*) => { $($tt)* };
}

macro_rules! if_multi_kernel {
    (multi_kernel, $($tt:tt)*) => { $($tt)* };
    (not_multi_kernel, $($tt:tt)*) => {};
}

macro_rules! if_not_multi_kernel {
    (multi_kernel, $($tt:tt)*) => {};
    (not_multi_kernel, $($tt:tt)*) => { $($tt)* };
}

macro_rules! args_sub_ty {
    (multi_kernel) => {
        MultiKernel
    };
    (not_multi_kernel) => {
        MultiCore
    };
}

macro_rules! impl_multi {
    ($fn_ident:ident, $channel:expr, $recv:ident, $block:ident, $is_fake:ident, $is_multi_kernel:ident) => {
        fn $fn_ident(args: &Args, args_sub: &args_sub_ty!($is_multi_kernel)) -> Result<()> {
            check_buffer_args!(args_sub);
            anyhow::ensure!(args_sub.num_cpus >= 2, "number of CPUs must be at least 2");

            let core_ids = get_core_ids()?;
            anyhow::ensure!(
                core_ids.len() >= args_sub.num_cpus,
                "requested more CPUs ({}) than cores are in the system ({})",
                args_sub.num_cpus,
                core_ids.len()
            );

            if_multi_kernel!($is_multi_kernel,
                             anyhow::ensure!(
                                 args_sub.num_kernels >= args_sub.num_cpus,
                                 "requested more CPUs ({}) than kernels ({})",
                                 args_sub.num_cpus,
                                 args_sub.num_kernels
                             ));

            let mut tx = Vec::with_capacity(args_sub.num_cpus);
            let mut rx = Vec::with_capacity(args_sub.num_cpus);
            for _ in 0..args_sub.num_cpus {
                let (t, r) = $channel(args_sub.num_buffers);
                tx.push(t);
                rx.push(r);
            }
            // circularly shift rx channels to pair each thread's rx with the previous
            // thread's tx (and last thread goes to first)
            let rx_last = rx.pop().unwrap();
            rx.insert(0, rx_last);

            // create buffers and inject them into the system
            let mut rng = rand::thread_rng();
            let buf_len = args_sub.buffer_size / std::mem::size_of::<f32>();
            for _ in 0..args_sub.num_buffers {
                tx.last_mut()
                    .unwrap()
                    .send(Buffer::<f32>::from_fn(buf_len, |_| rng.gen()))
            }

            if_multi_kernel!(
                $is_multi_kernel,
                let kernels_per_core = (0..args_sub.num_cpus)
                    .map(|n| {
                        args_sub.num_kernels / args_sub.num_cpus
                            + if n < args_sub.num_kernels % args_sub.num_cpus {
                                1
                        } else {
                                0
                            }
                    })
                    .collect::<Vec<_>>(););

            let clocks_per_sample = 2.0;
            let measurement_iters = (args.clock_frequency * args_sub.measurement_interval
                / (clocks_per_sample * buf_len as f64))
                .ceil() as usize;
            let samples_per_measurement = measurement_iters * buf_len;

            macro_rules! make_saxpy {
                () => {
                    Saxpy::new(rng.gen(), rng.gen())
                };
            }

            macro_rules! make_saxpys {
                ($saxpys:ident, $idx:expr) => {
                    if_multi_kernel!(
                        $is_multi_kernel,
                        let $saxpys: Vec<Saxpy> = std::iter::repeat_with(|| make_saxpy!())
                            .take(kernels_per_core[$idx])
                            .collect(););
                    if_not_multi_kernel!(
                        $is_multi_kernel,
                        let $saxpys = make_saxpy!(););
                }
            }

            macro_rules! do_work {
                ($tx:expr, $rx:expr, $buf:ident, $saxpys:expr) => {
                    // rx_buf is not mutated when $is_fake == fake
                    #[allow(unused_mut)]
                    let mut rx_buf = $recv!($rx);
                    if_multi_kernel!(
                        $is_multi_kernel,
                        for saxpy in &$saxpys {
                            saxpy.run_best(&mut rx_buf[..]);
                        }
                    );
                    if_not_multi_kernel!(
                        $is_multi_kernel,
                        if_fake!($is_fake, $saxpys.run_best(&mut $buf[..]));
                        if_not_fake!($is_fake, $saxpys.run_best(&mut rx_buf[..]));
                    );
                    $tx.send(rx_buf);
                }
            }

            // last CPU core; performs measurement
            let last_thread = thread::spawn({
                let idx = args_sub.num_cpus - 1;
                let mut tx = tx.pop().unwrap();
                let mut rx = rx.pop().unwrap();
                let core = core_ids[idx];
                if_fake!($is_fake, let mut buf = Buffer::<f32>::from_fn(buf_len, |_| rng.gen()););
                make_saxpys!(saxpys, idx);
                move || {
                    core_affinity::set_for_current(core);
                    $block! {
                        let (mut time, mut cycles) = begin_measurement();
                        loop {
                            for _ in 0..measurement_iters {
                                do_work!(tx, rx, buf, saxpys);
                            }
                            update_measurement(&mut time, &mut cycles, samples_per_measurement);
                        }
                    }
                }
            });

            // remaining threads; do not perform measurement
            // spawn in reverse order so that all the threads are running when the first begins processing
            for idx in (0..args_sub.num_cpus - 1).rev() {
                thread::spawn({
                    let mut tx = tx.pop().unwrap();
                    let mut rx = rx.pop().unwrap();
                    let core = core_ids[idx];
                    if_fake!($is_fake, let mut buf = Buffer::<f32>::from_fn(buf_len, |_| rng.gen()););
                    make_saxpys!(saxpys, idx);
                    move || {
                        core_affinity::set_for_current(core);
                        $block! { loop { do_work!(tx, rx, buf, saxpys); } }
                    }
                });
            }

            // the last thread should never return, so this blocks forever
            last_thread.join().unwrap();
            Ok(())
        }
    };
}

macro_rules! sync_recv {
    ($rx:expr) => {
        $rx.recv().unwrap()
    };
}

macro_rules! async_recv {
    ($rx:expr) => {
        $rx.recv().await.unwrap()
    };
}

macro_rules! sync_block {
    ($($tt:tt)*) => {
        $($tt)*
    }
}

macro_rules! async_block {
    ($($tt:tt)*) => {
        block_on(async move { $($tt)* })
    }
}

impl_multi!(
    multi_core,
    spsc::futex::channel,
    sync_recv,
    sync_block,
    not_fake,
    not_multi_kernel
);
impl_multi!(
    multi_core_async,
    spsc::futures::channel,
    async_recv,
    async_block,
    not_fake,
    not_multi_kernel
);
impl_multi!(
    multi_core_fake_passing,
    spsc::futex::channel,
    sync_recv,
    sync_block,
    fake,
    not_multi_kernel
);
impl_multi!(
    multi_core_fake_passing_async,
    spsc::futures::channel,
    async_recv,
    async_block,
    fake,
    not_multi_kernel
);
impl_multi!(
    multi_kernel,
    spsc::futex::channel,
    sync_recv,
    sync_block,
    not_fake,
    multi_kernel
);
impl_multi!(
    multi_kernel_async,
    spsc::futures::channel,
    async_recv,
    async_block,
    not_fake,
    multi_kernel
);

fn scan_buffer_size(args: &Args, args_sub: &ScanBufferSize) -> Result<()> {
    pin_cpu()?;
    let mut rng = rand::thread_rng();
    let saxpy = Saxpy::new(rng.gen(), rng.gen());
    let clocks_per_sample = 2.0;

    let buffer_sizes = (8..24).map(|n| 1 << n);
    for size in buffer_sizes {
        let buf_len = size / std::mem::size_of::<f32>();
        let mut buf = Buffer::<f32>::from_fn(buf_len, |_| rng.gen());

        let measurement_buffers = (args.clock_frequency * args_sub.measurement_time
            / (clocks_per_sample * buf_len as f64))
            .ceil() as usize;
        let samples_per_measurement = measurement_buffers * buf_len;

        print!("buffer size = {size}, ");
        let (time, cycles) = begin_measurement();
        for _ in 0..measurement_buffers {
            saxpy.run_best(&mut buf[..]);
        }
        make_measurement(time, cycles, samples_per_measurement);
    }
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    match &args.command {
        Command::SingleCore(sub) => single_core(&args, sub)?,
        Command::MultiCore(sub) => multi_core(&args, sub)?,
        Command::MultiCoreAsync(sub) => multi_core_async(&args, sub)?,
        Command::MultiCoreFakeMessagePassing(sub) => multi_core_fake_passing(&args, sub)?,
        Command::MultiCoreFakeMessagePassingAsync(sub) => {
            multi_core_fake_passing_async(&args, sub)?
        }
        Command::MultiKernel(sub) => multi_kernel(&args, sub)?,
        Command::MultiKernelAsync(sub) => multi_kernel_async(&args, sub)?,
        Command::ScanBufferSize(sub) => scan_buffer_size(&args, sub)?,
    }

    Ok(())
}
