use anyhow::Result;
use async_channel::{Receiver as AsyncChannelReceiver, Sender as AsyncChannelSender};
use clap::{Parser, ValueEnum};
use core_affinity::CoreId;
use crossbeam_channel::{Receiver as CrossbeamReceiver, Sender as CrossbeamSender};
use flume::{Receiver as FlumeReceiver, Sender as FlumeSender};
use kanal::{
    AsyncReceiver as KanalAsyncReceiver, AsyncSender as KanalAsyncSender,
    Receiver as KanalReceiver, Sender as KanalSender,
};
use qsdr::channel::{
    mpsc::{
        futex::{Receiver as QsdrMpscFutexReceiver, Sender as QsdrMpscFutexSender},
        futures::{Receiver as QsdrMpscFuturesReceiver, Sender as QsdrMpscFuturesSender},
    },
    spsc::{
        futex::{Receiver as QsdrSpscFutexReceiver, Sender as QsdrSpscFutexSender},
        futures::{Receiver as QsdrSpscFuturesReceiver, Sender as QsdrSpscFuturesSender},
    },
};
use qsdr_benchmarks::{affinity::get_core_ids, futures::executor::block_on};
use std::{
    sync::mpsc::{self, Receiver as StdMpscReceiver, SyncSender as StdMpscSyncSender},
    thread,
    time::Instant,
};
use tokio::sync::mpsc::{Receiver as TokioReceiver, Sender as TokioSender};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Channel size
    #[arg(long, default_value_t = 64)]
    channel_size: usize,
    /// Measurement iterations
    #[arg(long, default_value_t = 10000000)]
    measurement_iters: usize,
    /// Channel type
    #[arg(long)]
    channel: ChannelType,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, ValueEnum)]
enum ChannelType {
    /// async-channel::bounded
    AsyncChannelBounded,
    /// crossbeam-channel::bounded
    CrossbeamBounded,
    /// flume::bounded
    FlumeBounded,
    /// flume::bounded (async receive)
    FlumeBoundedAsync,
    /// kanal::bounded
    KanalBounded,
    /// kanal::bounded_async
    KanalBoundedAsync,
    /// qsdr::channel::spsc::futex
    QsdrSpscFutex,
    /// qsdr::channel::mpsc::futex
    QsdrMpscFutex,
    /// qsdr::channel::spsc::futures
    QsdrSpscFutures,
    /// qsdr::channel::mpsc::futures
    QsdrMpscFutures,
    /// std::sync::mpsc
    StdMpsc,
    /// tokio::sync::mpsc
    Tokio,
}

const ITEMSIZE: usize = 4;
type Item = [u64; ITEMSIZE];
const ITEM: [u64; ITEMSIZE] = [0; ITEMSIZE];

trait Channel<T> {
    type Sender: Sender<T>;
    type Receiver: Receiver<T>;
    fn bounded(size: usize) -> (Self::Sender, Self::Receiver);
}

trait Sender<T>: Send + 'static {
    fn send(&mut self, value: T);
}

trait Receiver<T>: Send + 'static {
    fn recv(&mut self) -> T;
}

macro_rules! impl_channel {
    ($alias:ident, $sender:ident, $receiver:ident, $bounded:expr) => {
        type $alias<T> = ($sender<T>, $receiver<T>);

        impl<T: Send + 'static> Channel<T> for $alias<T> {
            type Sender = $sender<T>;
            type Receiver = $receiver<T>;
            fn bounded(size: usize) -> (Self::Sender, Self::Receiver) {
                $bounded(size)
            }
        }

        impl<T: Send + 'static> Sender<T> for $sender<T> {
            fn send(&mut self, value: T) {
                $sender::send(self, value).unwrap()
            }
        }

        impl<T: Send + 'static> Receiver<T> for $receiver<T> {
            fn recv(&mut self) -> T {
                $receiver::recv(self).unwrap()
            }
        }
    };
}

type QsdrSpscFutex<T> = (QsdrSpscFutexSender<T>, QsdrSpscFutexReceiver<T>);

impl<T: Send + 'static> Channel<T> for QsdrSpscFutex<T> {
    type Sender = QsdrSpscFutexSender<T>;
    type Receiver = QsdrSpscFutexReceiver<T>;
    fn bounded(size: usize) -> (Self::Sender, Self::Receiver) {
        qsdr::channel::spsc::futex::channel(size)
    }
}

impl<T: Send + 'static> Sender<T> for QsdrSpscFutexSender<T> {
    fn send(&mut self, value: T) {
        QsdrSpscFutexSender::send(self, value)
    }
}

impl<T: Send + 'static> Receiver<T> for QsdrSpscFutexReceiver<T> {
    fn recv(&mut self) -> T {
        QsdrSpscFutexReceiver::recv(self).unwrap()
    }
}

type QsdrMpscFutex<T> = (QsdrMpscFutexSender<T>, QsdrMpscFutexReceiver<T>);

impl<T: Send + 'static> Channel<T> for QsdrMpscFutex<T> {
    type Sender = QsdrMpscFutexSender<T>;
    type Receiver = QsdrMpscFutexReceiver<T>;
    fn bounded(size: usize) -> (Self::Sender, Self::Receiver) {
        qsdr::channel::mpsc::futex::channel(size)
    }
}

impl<T: Send + 'static> Sender<T> for QsdrMpscFutexSender<T> {
    fn send(&mut self, value: T) {
        QsdrMpscFutexSender::send(self, value)
    }
}

impl<T: Send + 'static> Receiver<T> for QsdrMpscFutexReceiver<T> {
    fn recv(&mut self) -> T {
        QsdrMpscFutexReceiver::recv(self).unwrap()
    }
}

impl_channel!(
    CrossbeamBounded,
    CrossbeamSender,
    CrossbeamReceiver,
    crossbeam_channel::bounded
);
impl_channel!(FlumeBounded, FlumeSender, FlumeReceiver, flume::bounded);
impl_channel!(KanalBounded, KanalSender, KanalReceiver, kanal::bounded);
impl_channel!(
    StdMpsc,
    StdMpscSyncSender,
    StdMpscReceiver,
    mpsc::sync_channel
);

trait AsyncChannel<T> {
    type Sender: Sender<T>;
    type Receiver: AsyncReceiver<T>;
    fn bounded(size: usize) -> (Self::Sender, Self::Receiver);
}

trait AsyncReceiver<T>: Send + 'static {
    async fn recv(&mut self) -> T;
}

macro_rules! impl_async_channel {
    ($alias:ident, $sender:ident, $receiver:ident, $bounded:expr) => {
        type $alias<T> = ($sender<T>, $receiver<T>);

        impl<T: Send + 'static> AsyncChannel<T> for $alias<T> {
            type Sender = $sender<T>;
            type Receiver = $receiver<T>;
            fn bounded(size: usize) -> (Self::Sender, Self::Receiver) {
                $bounded(size)
            }
        }

        impl<T: Send + 'static> Sender<T> for $sender<T> {
            fn send(&mut self, value: T) {
                self.try_send(value).unwrap();
            }
        }

        impl<T: Send + 'static> AsyncReceiver<T> for $receiver<T> {
            async fn recv(&mut self) -> T {
                $receiver::recv(self).await.unwrap()
            }
        }
    };
}

impl_async_channel!(
    AsyncChannelBounded,
    AsyncChannelSender,
    AsyncChannelReceiver,
    async_channel::bounded
);
impl_async_channel!(
    KanalBoundedAsync,
    KanalAsyncSender,
    KanalAsyncReceiver,
    kanal::bounded_async
);
impl_async_channel!(
    Tokio,
    TokioSender,
    TokioReceiver,
    tokio::sync::mpsc::channel
);

type FlumeBoundedAsync<T> = (FlumeSender<T>, FlumeReceiver<T>);

impl<T: Send + 'static> AsyncChannel<T> for FlumeBoundedAsync<T> {
    type Sender = FlumeSender<T>;
    type Receiver = FlumeReceiver<T>;
    fn bounded(size: usize) -> (Self::Sender, Self::Receiver) {
        flume::bounded(size)
    }
}

impl<T: Send + 'static> AsyncReceiver<T> for FlumeReceiver<T> {
    async fn recv(&mut self) -> T {
        self.recv_async().await.unwrap()
    }
}

type QsdrSpscFutures<T> = (QsdrSpscFuturesSender<T>, QsdrSpscFuturesReceiver<T>);

impl<T: Send + 'static> AsyncChannel<T> for QsdrSpscFutures<T> {
    type Sender = QsdrSpscFuturesSender<T>;
    type Receiver = QsdrSpscFuturesReceiver<T>;
    fn bounded(size: usize) -> (Self::Sender, Self::Receiver) {
        qsdr::channel::spsc::futures::channel(size)
    }
}

impl<T: Send + 'static> Sender<T> for QsdrSpscFuturesSender<T> {
    fn send(&mut self, value: T) {
        QsdrSpscFuturesSender::send(self, value)
    }
}

impl<T: Send + 'static> AsyncReceiver<T> for QsdrSpscFuturesReceiver<T> {
    async fn recv(&mut self) -> T {
        QsdrSpscFuturesReceiver::recv(self).await.unwrap()
    }
}

type QsdrMpscFutures<T> = (QsdrMpscFuturesSender<T>, QsdrMpscFuturesReceiver<T>);

impl<T: Send + 'static> AsyncChannel<T> for QsdrMpscFutures<T> {
    type Sender = QsdrMpscFuturesSender<T>;
    type Receiver = QsdrMpscFuturesReceiver<T>;
    fn bounded(size: usize) -> (Self::Sender, Self::Receiver) {
        qsdr::channel::mpsc::futures::channel(size)
    }
}

impl<T: Send + 'static> Sender<T> for QsdrMpscFuturesSender<T> {
    fn send(&mut self, value: T) {
        QsdrMpscFuturesSender::send(self, value)
    }
}

impl<T: Send + 'static> AsyncReceiver<T> for QsdrMpscFuturesReceiver<T> {
    async fn recv(&mut self) -> T {
        QsdrMpscFuturesReceiver::recv(self).await.unwrap()
    }
}

fn setup_core_ids() -> Result<(CoreId, CoreId)> {
    let core_ids = get_core_ids()?;
    anyhow::ensure!(
        core_ids.len() >= 2,
        "at least 2 CPU cores are needed (got {})",
        core_ids.len()
    );
    let core0 = core_ids[0];
    let core1 = core_ids[1];
    Ok((core0, core1))
}

fn benchmark_channel<C: Channel<Item>>(args: &Args) -> Result<()> {
    let (core0, core1) = setup_core_ids()?;
    let (mut tx0, mut rx1) = C::bounded(args.channel_size);
    let (mut tx1, mut rx0) = C::bounded(args.channel_size);

    for _ in 0..args.channel_size {
        tx1.send(ITEM);
    }

    let measurement_iters = args.measurement_iters;
    let measurement_thread = thread::spawn(move || {
        core_affinity::set_for_current(core1);
        let mut time = Instant::now();
        loop {
            for _ in 0..measurement_iters {
                let item = rx1.recv();
                tx1.send(item);
            }
            let now = Instant::now();
            let elapsed = now - time;
            let items_per_sec = measurement_iters as f64 / elapsed.as_secs_f64();
            println!("items/s = {items_per_sec:.3e}");
            time = now;
        }
    });

    thread::spawn(move || {
        core_affinity::set_for_current(core0);
        loop {
            let item = rx0.recv();
            tx0.send(item);
        }
    });

    // the thread should never return, so this blocks forever
    measurement_thread.join().unwrap();
    Ok(())
}

fn benchmark_async_channel<C: AsyncChannel<Item>>(args: &Args) -> Result<()> {
    let (core0, core1) = setup_core_ids()?;
    let (mut tx0, mut rx1) = C::bounded(args.channel_size);
    let (mut tx1, mut rx0) = C::bounded(args.channel_size);

    for _ in 0..args.channel_size {
        tx1.send(ITEM);
    }

    let measurement_iters = args.measurement_iters;
    let measurement_thread = thread::spawn(move || {
        core_affinity::set_for_current(core1);
        block_on(async move {
            let mut time = Instant::now();
            loop {
                for _ in 0..measurement_iters {
                    let item = rx1.recv().await;
                    tx1.send(item);
                }
                let now = Instant::now();
                let elapsed = now - time;
                let items_per_sec = measurement_iters as f64 / elapsed.as_secs_f64();
                println!("items/s = {items_per_sec:.3e}");
                time = now;
            }
        });
    });

    thread::spawn(move || {
        core_affinity::set_for_current(core0);
        block_on(async move {
            loop {
                let item = rx0.recv().await;
                tx0.send(item);
            }
        })
    });

    // the thread should never return, so this blocks forever
    measurement_thread.join().unwrap();
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    match args.channel {
        ChannelType::AsyncChannelBounded => {
            benchmark_async_channel::<AsyncChannelBounded<Item>>(&args)?
        }
        ChannelType::CrossbeamBounded => benchmark_channel::<CrossbeamBounded<Item>>(&args)?,
        ChannelType::FlumeBounded => benchmark_channel::<FlumeBounded<Item>>(&args)?,
        ChannelType::FlumeBoundedAsync => {
            benchmark_async_channel::<FlumeBoundedAsync<Item>>(&args)?
        }
        ChannelType::KanalBounded => benchmark_channel::<KanalBounded<Item>>(&args)?,
        ChannelType::KanalBoundedAsync => {
            benchmark_async_channel::<KanalBoundedAsync<Item>>(&args)?
        }
        ChannelType::QsdrSpscFutex => benchmark_channel::<QsdrSpscFutex<Item>>(&args)?,
        ChannelType::QsdrMpscFutex => benchmark_channel::<QsdrMpscFutex<Item>>(&args)?,
        ChannelType::QsdrSpscFutures => benchmark_async_channel::<QsdrSpscFutures<Item>>(&args)?,
        ChannelType::QsdrMpscFutures => benchmark_async_channel::<QsdrMpscFutures<Item>>(&args)?,
        ChannelType::StdMpsc => benchmark_channel::<StdMpsc<Item>>(&args)?,
        ChannelType::Tokio => benchmark_async_channel::<Tokio<Item>>(&args)?,
    }
    Ok(())
}
