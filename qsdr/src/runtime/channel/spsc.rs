use super::Channel;
use crate::channel::spsc::futures::{channel, Receiver, Sender};
use anyhow::Result;
use std::{fmt::Debug, future::Future};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Spsc {}

impl Channel for Spsc {
    type Sender<T> = Sender<T>;

    type Receiver<T> = Receiver<T>;

    type ReturnReceiver<T> = ();

    type SenderSeed<T> = SenderSeed<T>;

    type ReceiverSeed<T> = ReceiverSeed<T>;

    type ReturnReceiverSeed<T> = ();

    fn connect<T, I: Iterator<Item = T>>(
        size: usize,
        source: &mut Self::SenderSeed<T>,
        dest: &mut Self::ReceiverSeed<T>,
        _return_dest: &mut Self::ReturnReceiverSeed<T>,
        inject_messages: I,
    ) -> Result<()> {
        anyhow::ensure!(source.0.is_none(), "source is already connected");
        anyhow::ensure!(dest.0.is_none(), "destination is already connected");
        let (mut tx, rx) = channel(size);
        for message in inject_messages.take(size) {
            tx.send(message);
        }
        source.0.replace(tx);
        dest.0.replace(rx);
        Ok(())
    }
}

pub struct SenderSeed<T>(pub(super) Option<Sender<T>>);

impl<T> Default for SenderSeed<T> {
    fn default() -> Self {
        Self(None)
    }
}

impl<T> TryFrom<SenderSeed<T>> for Sender<T> {
    type Error = anyhow::Error;

    fn try_from(value: SenderSeed<T>) -> Result<Sender<T>> {
        value
            .0
            .ok_or_else(|| anyhow::anyhow!("port is not connected"))
    }
}

pub struct ReceiverSeed<T>(pub(super) Option<Receiver<T>>);

impl<T> Default for ReceiverSeed<T> {
    fn default() -> Self {
        Self(None)
    }
}

impl<T> TryFrom<ReceiverSeed<T>> for Receiver<T> {
    type Error = anyhow::Error;

    fn try_from(value: ReceiverSeed<T>) -> Result<Receiver<T>> {
        value
            .0
            .ok_or_else(|| anyhow::anyhow!("port is not connected"))
    }
}

impl<T> super::Sender<T> for Sender<T> {
    fn send(&mut self, value: T) {
        Sender::send(self, value)
    }
}

impl<T> super::Receiver<T> for Receiver<T> {
    fn recv(&mut self) -> impl Future<Output = Option<T>> {
        Receiver::recv(self)
    }
}

impl_ref_receiver_for_receiver!(Receiver);
