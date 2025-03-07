use super::{
    block::{Block, BlockObject},
    channel::Channel,
    port::{ConnectsTo, ConnectsWithReturn, Endpoint, Port, PortId},
};
use anyhow::Result;
use std::{
    collections::{HashMap, HashSet},
    iter::ExactSizeIterator,
    sync::atomic::{AtomicUsize, Ordering},
};

#[derive(Debug)]
pub struct Flowgraph {
    id: FlowgraphId,
    next_circuit: CircuitId,
    next_node: NodeId,
    circuits: HashMap<CircuitId, CircuitData>,
}

#[derive(Debug)]
pub struct ValidatedFlowgraph {
    id: FlowgraphId,
}

#[derive(Debug)]
pub struct Circuit<Messages> {
    id: CircuitId,
    flowgraph_id: FlowgraphId,
    messages: Option<Messages>,
}

#[derive(Debug)]
struct CircuitData {
    size: usize,
    edges: Vec<Edge>,
}

#[derive(Debug)]
struct Edge {
    source: EndpointKey,
    dest: EndpointKey,
    return_endpoint: Option<EndpointKey>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct EndpointKey {
    node: NodeId,
    port: PortId,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct FlowgraphId(usize);

static FLOWGRAPH_INSTANCE_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct NodeId(usize);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct CircuitId(usize);

impl<P: Port> From<Endpoint<'_, P>> for EndpointKey {
    fn from(value: Endpoint<'_, P>) -> EndpointKey {
        EndpointKey {
            node: value.node(),
            port: value.port(),
        }
    }
}

pub trait FlowgraphNode {
    type B: Block;
    fn flowgraph_id(&self) -> FlowgraphId;
    fn node_id(&self) -> NodeId;
    fn wrap_block(flowgraph_id: FlowgraphId, node_id: NodeId, block: Self::B) -> Self;
    fn try_into_object(self, fg: &mut ValidatedFlowgraph) -> Result<BlockObject<Self::B>>;
}

impl Flowgraph {
    pub fn new() -> Flowgraph {
        let id = FLOWGRAPH_INSTANCE_COUNTER.fetch_add(1, Ordering::Relaxed);
        assert!(id < usize::MAX);
        Flowgraph {
            id: FlowgraphId(id),
            next_circuit: CircuitId(0),
            next_node: NodeId(0),
            circuits: HashMap::new(),
        }
    }

    #[must_use]
    pub fn new_circuit<Messages>(&mut self, messages: Messages) -> Circuit<Messages>
    where
        Messages: ExactSizeIterator,
    {
        assert!(self.next_circuit.0 < usize::MAX);
        let id = self.next_circuit;
        self.next_circuit = CircuitId(self.next_circuit.0 + 1);
        let circuit_present = self
            .circuits
            .insert(
                id,
                CircuitData {
                    size: messages.len(),
                    edges: Vec::new(),
                },
            )
            .is_some();
        assert!(!circuit_present);
        Circuit {
            id,
            flowgraph_id: self.id,
            messages: Some(messages),
        }
    }

    #[must_use]
    pub fn add_block<B: Block>(&mut self, block: B) -> B::Node {
        assert!(self.next_node.0 < usize::MAX);
        let node_id = self.next_node;
        self.next_node = NodeId(self.next_node.0 + 1);
        B::Node::wrap_block(self.id, node_id, block)
    }

    pub fn connect<PS, PD, M, C, T>(
        &mut self,
        circuit: &mut Circuit<M>,
        mut source: Endpoint<PS>,
        mut destination: Endpoint<PD>,
    ) -> Result<()>
    where
        C: Channel<ReturnReceiverSeed<T> = ()>,
        M: Iterator<Item = T>,
        PS: Port<ChannelType = C, Seed = C::SenderSeed<T>> + ConnectsTo<PD>,
        PD: Port<ChannelType = C, Seed = C::ReceiverSeed<T>>,
    {
        self.ensure_belong(circuit, &source, &destination)?;
        let circuit_data = self.circuits.get_mut(&circuit.id).unwrap();

        C::connect(
            circuit_data.size,
            source.seed(),
            destination.seed(),
            &mut (),
            std::iter::empty(),
        )?;

        let edge = Edge {
            source: source.into(),
            dest: destination.into(),
            return_endpoint: None,
        };
        circuit_data.edges.push(edge);

        Ok(())
    }

    pub fn connect_with_return<PS, PD, PR, M, CF, T>(
        &mut self,
        circuit: &mut Circuit<M>,
        mut source: Endpoint<PS>,
        mut destination: Endpoint<PD>,
        mut return_destination: Endpoint<PR>,
    ) -> Result<()>
    where
        CF: Channel,
        M: Iterator<Item = T>,
        PS: Port<ChannelType = CF, Seed = CF::SenderSeed<T>> + ConnectsWithReturn<PD, PR>,
        PD: Port<ChannelType = CF, Seed = CF::ReceiverSeed<T>>,
        PR: Port<Seed = CF::ReturnReceiverSeed<T>>,
    {
        self.ensure_belong(circuit, &source, &destination)?;
        anyhow::ensure!(
            return_destination.flowgraph() == self.id,
            "return_destination does not belong to this flowgraph"
        );
        let circuit_data = self.circuits.get_mut(&circuit.id).unwrap();

        macro_rules! connect {
            ($iter:expr) => {
                CF::connect(
                    circuit_data.size,
                    source.seed(),
                    destination.seed(),
                    return_destination.seed(),
                    $iter,
                )?;
            };
        }

        if let Some(messages) = circuit.messages.take() {
            connect!(messages);
        } else {
            connect!(std::iter::empty());
        }

        // TODO: add return information to the graph somehow
        let edge = Edge {
            source: source.into(),
            dest: destination.into(),
            return_endpoint: Some(return_destination.into()),
        };
        circuit_data.edges.push(edge);

        Ok(())
    }

    fn ensure_belong<PS, PD, M>(
        &self,
        circuit: &Circuit<M>,
        source: &Endpoint<PS>,
        destination: &Endpoint<PD>,
    ) -> Result<()>
    where
        PS: Port,
        PD: Port,
    {
        anyhow::ensure!(
            circuit.flowgraph_id == self.id,
            "circuit does not belong to this flowgraph"
        );
        anyhow::ensure!(
            source.flowgraph() == self.id,
            "source does not belong to this flowgraph"
        );
        anyhow::ensure!(
            destination.flowgraph() == self.id,
            "destination does not belong to this flowgraph"
        );
        Ok(())
    }

    pub fn validate(self) -> Result<ValidatedFlowgraph> {
        for (&id, circuit) in self.circuits.iter() {
            circuit.validate(id)?;
        }
        Ok(ValidatedFlowgraph { id: self.id })
    }
}

impl ValidatedFlowgraph {
    pub fn extract_block<N: FlowgraphNode>(&mut self, node: N) -> Result<BlockObject<N::B>> {
        anyhow::ensure!(
            node.flowgraph_id() == self.id,
            "node does not belong to this flowgraph"
        );
        Ok(node.try_into_object(self).unwrap())
    }
}

impl Default for Flowgraph {
    fn default() -> Flowgraph {
        Flowgraph::new()
    }
}

impl CircuitData {
    fn validate(&self, id: CircuitId) -> Result<()> {
        let return_endpoints = self
            .edges
            .iter()
            .filter_map(|edge| edge.return_endpoint)
            .collect::<HashSet<_>>();
        let num_returns = return_endpoints.len();
        anyhow::ensure!(
            num_returns == 1,
            "circuit {id:?} does not have a single return endpoint (it has {num_returns})",
        );

        // check that the circuit is a tree with the return node at the root and
        // with all the edges to a leaf having a return
        let root = *return_endpoints.iter().next().unwrap();
        let mut pending = vec![root];
        let mut visited = HashSet::new();
        while let Some(endpoint) = pending.pop() {
            anyhow::ensure!(
                visited.insert(endpoint.node),
                "circuit {id:?} contains a cycle"
            );
            for edge in self.edges_from_node(endpoint.node) {
                if self.is_leaf(edge.dest.node) {
                    anyhow::ensure!(
                        edge.return_endpoint.is_some(),
                        "edge {edge:?} in circuit {id:?} connects to a leaf but does not have a return"
                    );
                }
                pending.push(edge.dest);
            }
        }
        let all_nodes = self
            .edges
            .iter()
            .flat_map(|edge| {
                let mut nodes = vec![edge.source.node, edge.dest.node];
                if let Some(return_endpoint) = edge.return_endpoint {
                    nodes.push(return_endpoint.node);
                }
                nodes
            })
            .collect::<HashSet<_>>();
        anyhow::ensure!(
            visited == all_nodes,
            "circuit {id:?} is not a tree with the return at the root"
        );

        Ok(())
    }

    fn edges_from_node(&self, node: NodeId) -> impl Iterator<Item = &Edge> {
        self.edges
            .iter()
            .filter(move |edge| edge.source.node == node)
    }

    fn is_leaf(&self, node: NodeId) -> bool {
        self.edges_from_node(node).next().is_none()
    }
}
