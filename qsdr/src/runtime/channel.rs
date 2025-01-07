use anyhow::Result;
use std::{borrow::Borrow, fmt::Debug, future::Future};

macro_rules! impl_ref_receiver_for_receiver {
    ($ident:ident) => {
        impl<T> crate::runtime::channel::RefReceiver<T> for $ident<T> {
            type Ref<'a>
                = T
            where
                T: 'a;

            fn ref_recv(&mut self) -> impl std::future::Future<Output = Option<Self::Ref<'_>>> {
                self.recv()
            }
        }
    };
}

pub mod base;
pub mod ref_receiver;

pub mod spsc;
pub use spsc::Spsc;

pub mod spsc_ref;
pub use spsc_ref::SpscRef;

// single-producer single-consumer multiple-returners
pub mod spscmr_ref;
pub use spscmr_ref::SpscmrRef;

pub mod mpsc;
pub use mpsc::Mpsc;

pub mod spbroadcast;
pub use spbroadcast::SpBroadcast;

pub trait Channel: Debug + Default + Send + Unpin + 'static {
    type Sender<T>: Sender<T>;

    type Receiver<T>: RefReceiver<T>;

    type ReturnReceiver<T>;

    type SenderSeed<T>: TryInto<Self::Sender<T>, Error = anyhow::Error> + Default;

    type ReceiverSeed<T>: TryInto<Self::Receiver<T>, Error = anyhow::Error> + Default;

    type ReturnReceiverSeed<T>: TryInto<Self::ReturnReceiver<T>> + Default;

    fn connect<T, I: Iterator<Item = T>>(
        size: usize,
        source: &mut Self::SenderSeed<T>,
        dest: &mut Self::ReceiverSeed<T>,
        return_dest: &mut Self::ReturnReceiverSeed<T>,
        inject_messages: I,
    ) -> Result<()>;
}

pub trait Sender<T> {
    fn send(&mut self, value: T);
}

pub trait RefReceiver<T> {
    type Ref<'a>: Borrow<T>
    where
        Self: 'a;

    fn ref_recv(&mut self) -> impl Future<Output = Option<Self::Ref<'_>>>;
}

pub trait Receiver<T> {
    fn recv(&mut self) -> impl Future<Output = Option<T>>;
}
