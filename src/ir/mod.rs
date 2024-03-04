//! `MokaIR` is an intermediate representation of JVM bytecode.
//! It is register based and is in SSA form, which make it easier to analyze.

pub mod control_flow;
pub mod expression;
mod generator;
mod moka_instruction;
#[cfg(feature = "petgraph")]
pub mod petgraph;

use std::collections::BTreeMap;

pub use generator::{MokaIRBrewingError, MokaIRMethodExt};
pub use moka_instruction::*;

use crate::{
    jvm::{
        code::{ExceptionTableEntry, InstructionList, ProgramCounter},
        method::MethodAccessFlags,
        references::ClassRef,
    },
    types::method_descriptor::MethodDescriptor,
};

use self::control_flow::ControlTransfer;

/// Represents a JVM method where the instructions have been converted to Moka IR.
#[derive(Debug, Clone)]
pub struct MokaIRMethod {
    /// The access flags of the method.
    pub access_flags: MethodAccessFlags,
    /// The name of the method.
    pub name: String,
    /// The descriptor of the method.
    pub descriptor: MethodDescriptor,
    /// The class that contains the method.
    pub owner: ClassRef,
    /// The body of the method.
    pub instructions: InstructionList<MokaInstruction>,
    /// The exception table of the method.
    pub exception_table: Vec<ExceptionTableEntry>,
    /// The control flow graph of the method.
    pub control_flow_graph: ControlFlowGraph<(), ControlTransfer>,
}

/// A control flow graph.
///
/// It is generic over the data associated with each node and edge.
#[derive(Debug, Clone, Default)]
pub struct ControlFlowGraph<N, E> {
    inner: BTreeMap<ProgramCounter, (N, BTreeMap<ProgramCounter, E>)>,
}

impl<N, E> ControlFlowGraph<N, E> {
    /// Returns the entry point of the control flow graph.
    #[must_use]
    pub const fn entry_point(&self) -> ProgramCounter {
        ProgramCounter::ZERO
    }

    /// Transforms the node and edge data to construt a new control flow graph.
    #[must_use]
    pub fn map<N1, E1, NMap, EMap>(self, nf: NMap, ef: EMap) -> ControlFlowGraph<N1, E1>
    where
        NMap: Fn(ProgramCounter, N) -> N1,
        EMap: Fn((ProgramCounter, ProgramCounter), E) -> E1,
    {
        let inner = self
            .inner
            .into_iter()
            .map(|(src, (node_data, edges))| {
                let data = nf(src, node_data);
                let edges = edges
                    .into_iter()
                    .map(|(dst, edge_data)| (dst, ef((src, dst), edge_data)))
                    .collect();
                (src, (data, edges))
            })
            .collect();

        ControlFlowGraph { inner }
    }

    /// Returns an iterator over the nodes
    pub fn nodes(&self) -> impl Iterator<Item = (ProgramCounter, &N)> {
        self.inner.iter().map(|(n, (d, _))| (*n, d))
    }

    /// Returns an iterator over the edges
    pub fn edges(&self) -> impl Iterator<Item = (ProgramCounter, ProgramCounter, &E)> {
        self.inner.iter().flat_map(|(src, (_, outgoing_edges))| {
            outgoing_edges.iter().map(|(dst, data)| (*src, *dst, data))
        })
    }

    /// Returns an iterator over the exits of the control flow graph.
    pub fn exits(&self) -> impl Iterator<Item = ProgramCounter> + '_ {
        self.inner
            .iter()
            .filter(|(_, (_, outgoing_edges))| outgoing_edges.is_empty())
            .map(|(n, _)| *n)
    }

    /// Returns an iterator over the edges starting at the given node.
    #[must_use]
    pub fn edges_from(
        &self,
        src: ProgramCounter,
    ) -> Option<impl Iterator<Item = (ProgramCounter, ProgramCounter, &E)>> {
        self.inner.get(&src).map(|(_, outgoing_edges)| {
            outgoing_edges
                .iter()
                .map(move |(dst, data)| (src, *dst, data))
        })
    }
}

impl<E> ControlFlowGraph<(), E> {
    /// Constructs a new control flow graph from a set of edges.
    ///
    /// # Panics
    /// Panics if there are duplicate edges.
    pub fn from_edges(
        edges: impl IntoIterator<Item = (ProgramCounter, ProgramCounter, E)>,
    ) -> Self {
        let mut inner: BTreeMap<_, (_, BTreeMap<_, _>)> = BTreeMap::new();
        edges.into_iter().for_each(|(src, dst, data)| {
            let edge_map = &mut inner.entry(src).or_default().1;
            assert!(edge_map.insert(dst, data).is_none(), "Duplicate edge");
            inner.entry(dst).or_default();
        });
        Self { inner }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_cfg() -> ControlFlowGraph<(), ()> {
        let edges = [
            (0.into(), 1.into(), ()),
            (1.into(), 2.into(), ()),
            (2.into(), 3.into(), ()),
            (3.into(), 4.into(), ()),
        ];
        ControlFlowGraph::from_edges(edges)
    }

    #[test]
    #[should_panic(expected = "Duplicate edge")]
    fn from_edges_duplicate() {
        let edges = [
            (0.into(), 1.into(), ()),
            (1.into(), 2.into(), ()),
            (2.into(), 3.into(), ()),
            (3.into(), 4.into(), ()),
            (0.into(), 1.into(), ()),
        ];
        ControlFlowGraph::from_edges(edges);
    }

    #[test]
    fn iter_nodes() {
        let cfg = build_cfg();
        let nodes = cfg.nodes().collect::<std::collections::BTreeSet<_>>();
        assert_eq!(nodes.len(), 5);
        for i in 0..=4 {
            assert!(nodes.contains(&(i.into(), &())));
        }
    }

    #[test]
    fn iter_edges() {
        let cfg = build_cfg();
        let edges = cfg.edges().collect::<std::collections::BTreeSet<_>>();
        assert_eq!(edges.len(), 4);
        for i in 0..=3 {
            assert!(edges.contains(&(i.into(), (i + 1).into(), &())));
        }
    }

    #[test]
    fn iter_exits() {
        let cfg = build_cfg();
        let exits = cfg.exits().collect::<std::collections::BTreeSet<_>>();
        assert_eq!(exits.len(), 1);
        assert!(exits.contains(&4.into()));
    }
}
