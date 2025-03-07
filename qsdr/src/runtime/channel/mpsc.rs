use super::Channel;
use crate::channel::mpsc::futures::{Receiver, Sender, channel};
use anyhow::Result;
use std::{fmt::Debug, future::Future};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Mpsc {}

impl Channel for Mpsc {
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
        if let Some(seed) = &dest.0 {
            // channel already exists
            source.0.replace(seed.sender.clone());
        } else {
            // create new channel
            let (tx, rx) = channel(size);
            for message in inject_messages.take(size) {
                tx.send(message);
            }
            source.0.replace(tx.clone());
            dest.0.replace(RxSeed {
                receiver: rx,
                sender: tx,
            });
        }
        Ok(())
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct ReceiverSeed<T>(pub(super) Option<RxSeed<T>>);

#[derive(Debug)]
pub(super) struct RxSeed<T> {
    pub(super) receiver: Receiver<T>,
    pub(super) sender: Sender<T>,
}

impl<T> Default for ReceiverSeed<T> {
    fn default() -> Self {
        Self(None)
    }
}

impl<T> TryFrom<ReceiverSeed<T>> for Receiver<T> {
    type Error = anyhow::Error;

    fn try_from(value: ReceiverSeed<T>) -> Result<Receiver<T>> {
        Ok(value
            .0
            .ok_or_else(|| anyhow::anyhow!("port is not connected"))?
            .receiver)
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
