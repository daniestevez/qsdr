use super::{Receiver, Sender, base::BaseChannel};
use anyhow::Result;
use std::{borrow::Borrow, convert::AsRef, mem::ManuallyDrop};

pub struct RefReceiver<T, Rx: BaseChannel, Tx: BaseChannel> {
    pub(super) receiver: Rx::Receiver<T>,
    pub(super) return_sender: Tx::Sender<T>,
}

pub struct RefReceiverSeed<T, Rx: BaseChannel, Tx: BaseChannel>(
    pub(super) Option<RefReceiver<T, Rx, Tx>>,
);

impl<T, Rx: BaseChannel, Tx: BaseChannel> Default for RefReceiverSeed<T, Rx, Tx> {
    fn default() -> Self {
        Self(None)
    }
}

impl<T, Rx: BaseChannel, Tx: BaseChannel> TryFrom<RefReceiverSeed<T, Rx, Tx>>
    for RefReceiver<T, Rx, Tx>
{
    type Error = anyhow::Error;

    fn try_from(value: RefReceiverSeed<T, Rx, Tx>) -> Result<RefReceiver<T, Rx, Tx>> {
        value
            .0
            .ok_or_else(|| anyhow::anyhow!("port is not connected"))
    }
}

pub struct RefEnvelope<'a, T, Tx: BaseChannel> {
    value: ManuallyDrop<T>,
    return_sender: &'a mut Tx::Sender<T>,
}

impl<T, Tx: BaseChannel> Borrow<T> for RefEnvelope<'_, T, Tx> {
    fn borrow(&self) -> &T {
        &self.value
    }
}

impl<T, Tx: BaseChannel> AsRef<T> for RefEnvelope<'_, T, Tx> {
    fn as_ref(&self) -> &T {
        &self.value
    }
}

impl<T, Tx: BaseChannel> Drop for RefEnvelope<'_, T, Tx> {
    fn drop(&mut self) {
        // SAFETY: self.value will never be used again after the value is
        // taken. self.return_sender is not full if the flowgraph connections
        // have been done properly
        unsafe { self.return_sender.send(ManuallyDrop::take(&mut self.value)) }
    }
}

impl<T, Rx: BaseChannel, Tx: BaseChannel> super::RefReceiver<T> for RefReceiver<T, Rx, Tx> {
    type Ref<'a>
        = RefEnvelope<'a, T, Tx>
    where
        T: 'a;

    async fn ref_recv(&mut self) -> Option<Self::Ref<'_>> {
        let item = self.receiver.recv().await?;
        Some(RefEnvelope {
            value: ManuallyDrop::new(item),
            return_sender: &mut self.return_sender,
        })
    }
}
