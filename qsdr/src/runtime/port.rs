use super::{
    channel::Channel,
    flowgraph::{FlowgraphId, NodeId},
    quantum::Quantum,
};
use std::{any::type_name, cell::RefMut, cmp, fmt, hash, marker::PhantomData};

macro_rules! define_port_types {
    ($($ident:ident),*) => {
        $(
            pub struct $ident<T, C> {
                _phantom: PhantomData<(T, C)>,
            }

            impl<T, C> $ident<T, C> {
                pub fn new() -> Self {
                    Default::default()
                }
            }

            impl<T, C> Copy for $ident<T, C> {}

            impl<T, C> Clone for $ident<T, C> {
                fn clone(&self) -> Self {
                    *self
                }
            }

            impl<T, C> Default for $ident<T, C> {
                fn default() -> Self {
                    Self {
                        _phantom: PhantomData,
                    }
                }
            }

            impl<T, C> cmp::PartialEq for $ident<T, C> {
                fn eq(&self, _other: &Self) -> bool {
                    true
                }
            }

            impl<T, C> cmp::Eq for $ident<T, C> {}

            impl<T, C> cmp::PartialOrd for $ident<T, C> {
                fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
                    Some(self.cmp(other))
                }
            }

            impl<T, C> cmp::Ord for $ident<T, C> {
                fn cmp(&self, _other: &Self) -> cmp::Ordering {
                    cmp::Ordering::Equal
                }
            }

            impl<T, C> hash::Hash for $ident<T, C> {
                fn hash<H>(&self, _state: &mut H)
                where
                    H: hash::Hasher,
                {
                }
            }

            impl<T, C> fmt::Debug for $ident<T, C> {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
                    f.debug_struct(&format!(
                        concat!(stringify!($ident), "<{}, {}>"),
                        type_name::<T>(),
                        type_name::<C>()
                    ))
                        .finish()
                }
            }
        )*
    }
}

define_port_types!(PortOut, PortIn, PortRefIn, PortSource);

pub type PortOutQ<B, C> = PortOut<Quantum<B>, C>;
pub type PortInQ<B, C> = PortIn<Quantum<B>, C>;
pub type PortRefInQ<B, C> = PortRefIn<Quantum<B>, C>;
pub type PortSourceQ<B, C> = PortSource<Quantum<B>, C>;

#[derive(Debug)]
pub struct Endpoint<'a, P: Port> {
    flowgraph: FlowgraphId,
    node: NodeId,
    port: PortId,
    seed: RefMut<'a, P::Seed>,
}

impl<P: Port> Endpoint<'_, P> {
    pub fn new(
        flowgraph: FlowgraphId,
        node: NodeId,
        port: PortId,
        seed: RefMut<'_, <P as Port>::Seed>,
    ) -> Endpoint<'_, P> {
        Endpoint {
            flowgraph,
            node,
            port,
            seed,
        }
    }

    pub fn flowgraph(&self) -> FlowgraphId {
        self.flowgraph
    }

    pub fn node(&self) -> NodeId {
        self.node
    }

    pub fn port(&self) -> PortId {
        self.port
    }

    pub fn seed(&mut self) -> &mut P::Seed {
        &mut self.seed
    }
}

pub trait Port: Default {
    type Channel;
    type Seed;
    type ItemType;
    type ChannelType: Channel;
}

impl<T, C: Channel> Port for PortOut<T, C> {
    type Channel = C::Sender<T>;
    type Seed = C::SenderSeed<T>;
    type ItemType = T;
    type ChannelType = C;
}

impl<T, C> Port for PortIn<T, C>
where
    C: Channel,
    C::Receiver<T>: super::channel::Receiver<T>,
{
    type Channel = C::Receiver<T>;
    type Seed = C::ReceiverSeed<T>;
    type ItemType = T;
    type ChannelType = C;
}

impl<T, C: Channel> Port for PortRefIn<T, C> {
    type Channel = C::Receiver<T>;
    type Seed = C::ReceiverSeed<T>;
    type ItemType = T;
    type ChannelType = C;
}

impl<T, C> Port for PortSource<T, C>
where
    C: Channel,
    C::Receiver<T>: super::channel::Receiver<T>,
{
    type Channel = C::Receiver<T>;
    type Seed = C::ReceiverSeed<T>;
    type ItemType = T;
    type ChannelType = C;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct PortId(u32);

impl From<u32> for PortId {
    fn from(value: u32) -> PortId {
        PortId(value)
    }
}

pub trait ConnectsTo<Dest> {}

impl<T, C: Channel> ConnectsTo<PortIn<T, C>> for PortOut<T, C> {}

pub trait ConnectsWithReturn<Dest, ReturnDest> {}

impl<T, C1: Channel, C2: Channel> ConnectsWithReturn<PortRefIn<T, C1>, PortSource<T, C2>>
    for PortOut<T, C1>
{
}
