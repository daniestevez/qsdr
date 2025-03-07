use clap::{Parser, Subcommand};
use futuresdr::{
    macros::connect,
    runtime::{
        Flowgraph, Result, Runtime,
        scheduler::{CpuPinScheduler, SmolScheduler},
    },
};
use futuresdr_benchmark::blocks::{BenchmarkSink, DummySource, Saxpy};
use rand::prelude::*;
use std::collections::HashMap;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Single kernel on a single core.
    SingleCore,
    /// Multiple kernels, pinned over multiple cores in order.
    MultiKernel(MultiKernel),
    /// Multiple kernels, using default smol scheduler.
    MultiKernelSmol(MultiKernel),
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

fn single_core() -> Result<()> {
    let mut fg = Flowgraph::new();
    let source = DummySource::<f32>::new();
    let mut rng = rand::rng();
    let saxpy = Saxpy::new(rng.random(), rng.random());
    let sink = BenchmarkSink::<f32>::new();
    connect!(fg, source > saxpy > sink);
    Runtime::with_scheduler(SmolScheduler::new(1, true)).run(fg)?;
    Ok(())
}

fn multi_kernel(args: &MultiKernel, cpu_pin: bool) -> Result<()> {
    let kernels_per_core = (0..args.num_cpus)
        .map(|n| {
            args.num_kernels / args.num_cpus
                + if n < args.num_kernels % args.num_cpus {
                    1
                } else {
                    0
                }
        })
        .collect::<Vec<_>>();

    let mut fg = Flowgraph::new();
    let mut cpu_pins = HashMap::new();
    let source = fg.add_block(DummySource::<f32>::new())?;
    cpu_pins.insert(source, 0);

    let mut rng = rand::rng();
    macro_rules! make_saxpy {
        () => {
            fg.add_block(Saxpy::new(rng.random(), rng.random()))?
        };
    }

    let first_saxpy = make_saxpy!();
    fg.connect_stream(source, "out", first_saxpy, "in")?;
    cpu_pins.insert(first_saxpy, 0);

    let mut core = 0;
    let mut kernels_in_core = 1;
    let mut previous_saxpy = first_saxpy;
    for _ in 1..args.num_kernels {
        if kernels_in_core == kernels_per_core[core] {
            core += 1;
            kernels_in_core = 0;
        }
        let saxpy = make_saxpy!();
        fg.connect_stream(previous_saxpy, "out", saxpy, "in")?;
        cpu_pins.insert(saxpy, core);
        previous_saxpy = saxpy;
        kernels_in_core += 1;
    }

    let sink = fg.add_block(BenchmarkSink::<f32>::new())?;
    fg.connect_stream(previous_saxpy, "out", sink, "in")?;
    cpu_pins.insert(sink, args.num_cpus - 1);

    if cpu_pin {
        Runtime::with_scheduler(CpuPinScheduler::new(cpu_pins)).run(fg)?;
    } else {
        Runtime::with_scheduler(SmolScheduler::new(args.num_cpus, true)).run(fg)?;
    }
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    match &args.command {
        Command::SingleCore => single_core()?,
        Command::MultiKernel(args) => multi_kernel(args, true)?,
        Command::MultiKernelSmol(args) => multi_kernel(args, false)?,
    }
    Ok(())
}
