use super::{
    Channel,
    base::Spsc,
    ref_receiver::{RefReceiver, RefReceiverSeed},
};
use crate::channel::spsc::futures::{Receiver, Sender, channel};
use anyhow::Result;
use std::fmt::Debug;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct SpscRef {}

impl Channel for SpscRef {
    type Sender<T> = Sender<T>;

    type Receiver<T> = RefReceiver<T, Spsc, Spsc>;

    type ReturnReceiver<T> = Receiver<T>;

    type SenderSeed<T> = super::spsc::SenderSeed<T>;

    type ReceiverSeed<T> = RefReceiverSeed<T, Spsc, Spsc>;

    type ReturnReceiverSeed<T> = super::spsc::ReceiverSeed<T>;

    fn connect<T, I: Iterator<Item = T>>(
        size: usize,
        source: &mut Self::SenderSeed<T>,
        dest: &mut Self::ReceiverSeed<T>,
        return_dest: &mut Self::ReturnReceiverSeed<T>,
        inject_messages: I,
    ) -> Result<()> {
        anyhow::ensure!(source.0.is_none(), "source is already connected");
        anyhow::ensure!(dest.0.is_none(), "destination is already connected");
        anyhow::ensure!(
            return_dest.0.is_none(),
            "return destination is already connected"
        );
        let (forward_tx, forward_rx) = channel(size);
        let (mut return_tx, return_rx) = channel(size);
        for message in inject_messages.take(size) {
            return_tx.send(message);
        }
        source.0.replace(forward_tx);
        dest.0.replace(RefReceiver {
            receiver: forward_rx,
            return_sender: return_tx,
        });
        return_dest.0.replace(return_rx);
        Ok(())
    }
}
