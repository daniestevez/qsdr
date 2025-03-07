use super::{
    Channel,
    base::{Mpsc, Spsc},
    ref_receiver::{RefReceiver, RefReceiverSeed},
};
use crate::channel::{
    mpsc::futures::{Receiver as MpscReceiver, channel as mpsc_channel},
    spsc::futures::{Sender, channel},
};
use anyhow::Result;
use std::fmt::Debug;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct SpscmrRef {}

impl Channel for SpscmrRef {
    type Sender<T> = Sender<T>;

    type Receiver<T> = RefReceiver<T, Spsc, Mpsc>;

    type ReturnReceiver<T> = MpscReceiver<T>;

    type SenderSeed<T> = super::spsc::SenderSeed<T>;

    type ReceiverSeed<T> = RefReceiverSeed<T, Spsc, Mpsc>;

    type ReturnReceiverSeed<T> = super::mpsc::ReceiverSeed<T>;

    fn connect<T, I: Iterator<Item = T>>(
        size: usize,
        source: &mut Self::SenderSeed<T>,
        dest: &mut Self::ReceiverSeed<T>,
        return_dest: &mut Self::ReturnReceiverSeed<T>,
        inject_messages: I,
    ) -> Result<()> {
        anyhow::ensure!(source.0.is_none(), "source is already connected");
        anyhow::ensure!(dest.0.is_none(), "destination is already connected");
        let return_tx = if let Some(return_seed) = &return_dest.0 {
            // return channel already exists
            return_seed.sender.clone()
        } else {
            // return channel does not exist yet
            let (return_tx, return_rx) = mpsc_channel(size);
            for message in inject_messages.take(size) {
                return_tx.send(message);
            }
            return_dest.0.replace(super::mpsc::RxSeed {
                receiver: return_rx,
                sender: return_tx.clone(),
            });
            return_tx
        };
        let (forward_tx, forward_rx) = channel(size);
        source.0.replace(forward_tx);
        dest.0.replace(RefReceiver {
            receiver: forward_rx,
            return_sender: return_tx,
        });
        Ok(())
    }
}
