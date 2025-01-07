use super::{Receiver, Sender};

pub trait BaseChannel: 'static {
    type Sender<T>: Sender<T>;

    type Receiver<T>: Receiver<T>;
}

macro_rules! impl_base_channel {
    ($ident:ident) => {
        impl BaseChannel for $ident {
            type Sender<T> = Sender<T>;
            type Receiver<T> = Receiver<T>;
        }
    };
}

pub use spsc::Spsc;
mod spsc {
    use super::BaseChannel;
    use crate::channel::spsc::futures::{Receiver, Sender};

    #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
    pub struct Spsc {}

    impl_base_channel!(Spsc);
}

pub use mpsc::Mpsc;
mod mpsc {
    use super::BaseChannel;
    use crate::channel::mpsc::futures::{Receiver, Sender};

    #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
    pub struct Mpsc {}

    impl_base_channel!(Mpsc);
}
