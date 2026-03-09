use core::fmt::Debug;
use core::hash::Hash;
use core::ops::{Deref, DerefMut};

use alloc::vec::Vec;
use fixedbitset::FixedBitSet;
use thiserror::Error;
use vc_utils::hash::{HashMap, HashSet};
use vc_utils::index::IndexSet;

use crate::schedule::{DiGraphToposortError, UnGraph};

use super::Direction::{Incoming, Outgoing};
use super::{DiGraph, GraphNode, flatten_index, unflatten_index};

// -----------------------------------------------------------------------------
// Dag

/// A directed acyclic graph structure.
#[derive(Clone)]
pub struct Dag<N: GraphNode> {
    /// The underlying directed graph.
    graph: DiGraph<N>,
    /// A cached topological ordering of the graph. This is recomputed when the
    /// graph is modified, and is not valid when `dirty` is true.
    toposort: Vec<N>,
    /// Whether the graph has been modified since the last topological sort.
    dirty: bool,
}

impl<N: GraphNode> Default for Dag<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<N: GraphNode> Dag<N> {
    /// Creates a new directed acyclic graph.
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            toposort: Vec::new(),
            dirty: false,
        }
    }
    /// Read-only access to the underlying directed graph.
    #[must_use]
    pub fn graph(&self) -> &DiGraph<N> {
        &self.graph
    }

    /// Mutable access to the underlying directed graph. Marks the graph as dirty.
    #[must_use = "This function marks the graph as dirty, so it should be used."]
    pub fn graph_mut(&mut self) -> &mut DiGraph<N> {
        self.dirty = true;
        &mut self.graph
    }

    /// Returns whether the graph is dirty (i.e., has been modified since the
    /// last topological sort).
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Returns whether the graph is topologically sorted (i.e., not dirty).
    #[must_use]
    pub fn is_toposorted(&self) -> bool {
        !self.dirty
    }

    pub fn ensure_toposorted(&mut self) -> Result<(), DiGraphToposortError<N>> {
        if self.dirty {
            // recompute the toposort, reusing the existing allocation
            self.toposort = self.graph.toposort(core::mem::take(&mut self.toposort))?;
            self.dirty = false;
        }
        Ok(())
    }

    /// Returns the cached toposort if the graph is not dirty, otherwise returns
    /// `None`.
    #[must_use = "This method only returns a cached value and does not compute anything."]
    pub fn get_toposort(&self) -> Option<&[N]> {
        if self.dirty {
            None
        } else {
            Some(&self.toposort)
        }
    }

    pub fn toposort(&mut self) -> Result<&[N], DiGraphToposortError<N>> {
        self.ensure_toposorted()?;
        Ok(&self.toposort)
    }

    pub fn toposort_and_graph(&mut self) -> Result<(&[N], &DiGraph<N>), DiGraphToposortError<N>> {
        self.ensure_toposorted()?;
        Ok((&self.toposort, &self.graph))
    }

    pub fn try_convert<T>(self) -> Result<Dag<T>, N::Error>
    where
        N: TryInto<T>,
        T: GraphNode,
    {
        Ok(Dag {
            graph: self.graph.try_convert()?,
            toposort: Vec::new(),
            dirty: true,
        })
    }
}

impl<N: GraphNode> Deref for Dag<N> {
    type Target = DiGraph<N>;

    fn deref(&self) -> &Self::Target {
        self.graph()
    }
}

impl<N: GraphNode> DerefMut for Dag<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.graph_mut()
    }
}

impl<N: GraphNode> Debug for Dag<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.dirty {
            f.debug_struct("Dag")
                .field("graph", &self.graph)
                .field("dirty", &self.dirty)
                .finish()
        } else {
            f.debug_struct("Dag")
                .field("graph", &self.graph)
                .field("toposort", &self.toposort)
                .finish()
        }
    }
}

// -----------------------------------------------------------------------------
// DagGroups

/// A mapping of keys to groups of values in a [`Dag`].
pub struct DagGroups<K, V>(HashMap<K, IndexSet<V>>);

impl<K, V> Deref for DagGroups<K, V> {
    type Target = HashMap<K, IndexSet<V>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K, V> DerefMut for DagGroups<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<K, V> Default for DagGroups<K, V> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<K: Debug, V: Debug> Debug for DagGroups<K, V> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("DagGroups").field(&self.0).finish()
    }
}

impl<K: Eq + Hash, V: Clone + Eq + Hash> DagGroups<K, V> {
    pub fn new<N>(graph: &DiGraph<N>, toposort: &[N]) -> Self
    where
        N: GraphNode + TryInto<K, Error = V>,
    {
        Self::with_capacity(0, graph, toposort)
    }

    pub fn with_capacity<N>(capacity: usize, graph: &DiGraph<N>, toposort: &[N]) -> Self
    where
        N: GraphNode + TryInto<K, Error = V>,
    {
        let mut groups: HashMap<K, IndexSet<V>> = HashMap::with_capacity(capacity);

        // Iterate in reverse topological order (bottom-up) so we hit children before parents.
        for &id in toposort.iter().rev() {
            let Ok(key) = id.try_into() else {
                continue;
            };

            let mut children = IndexSet::new();

            for node in graph.neighbors_directed(id, Outgoing) {
                match node.try_into() {
                    Ok(key) => {
                        // If the child is a key, this key inherits all of its children.
                        let key_children = groups.get(&key).unwrap();
                        children.extend(key_children.iter().cloned());
                    }
                    Err(value) => {
                        // If the child is a value, add it directly.
                        children.insert(value);
                    }
                }
            }

            groups.insert(key, children);
        }

        Self(groups)
    }
}

impl<K: GraphNode, V: GraphNode> DagGroups<K, V> {
    pub fn flatten<N>(
        &self,
        dag: Dag<N>,
        mut collapse_group: impl FnMut(K, &IndexSet<V>, &Dag<N>, &mut Vec<(N, N)>),
    ) -> Dag<V>
    where
        N: GraphNode + TryInto<V, Error = K> + From<K> + From<V>,
    {
        let mut flattening = dag;
        let mut temp = Vec::new();

        for (&key, values) in self.iter() {
            // Call the user-provided function to handle collapsing the group.
            collapse_group(key, values, &flattening, &mut temp);

            if values.is_empty() {
                // Replace connections to the key node with connections between its neighbors.
                for a in flattening.neighbors_directed(N::from(key), Incoming) {
                    for b in flattening.neighbors_directed(N::from(key), Outgoing) {
                        temp.push((a, b));
                    }
                }
            } else {
                // Redirect edges to/from the key node to connect to its value nodes.
                for a in flattening.neighbors_directed(N::from(key), Incoming) {
                    for &value in values {
                        temp.push((a, N::from(value)));
                    }
                }
                for b in flattening.neighbors_directed(N::from(key), Outgoing) {
                    for &value in values {
                        temp.push((N::from(value), b));
                    }
                }
            }

            // Remove the key node from the graph.
            flattening.remove_node(N::from(key));
            // Add all previously collected edges.
            flattening.reserve_edges(temp.len());
            for (a, b) in temp.drain(..) {
                flattening.insert_edge(a, b);
            }
        }

        // By this point, we should have removed all keys from the graph,
        // so this conversion should never fail.
        flattening
            .try_convert::<V>()
            .unwrap_or_else(|n| unreachable!("Flattened graph has a leftover key {n:?}"))
    }

    pub fn flatten_undirected<N>(&self, graph: &UnGraph<N>) -> UnGraph<V>
    where
        N: GraphNode + TryInto<V, Error = K>,
    {
        let mut flattened = UnGraph::new();

        for (lhs, rhs) in graph.all_edges() {
            match (lhs.try_into(), rhs.try_into()) {
                (Ok(lhs), Ok(rhs)) => {
                    // Normal edge between two value nodes
                    flattened.insert_edge(lhs, rhs);
                }
                (Err(lhs_key), Ok(rhs)) => {
                    // Edge from a key node to a value node, expand to all values in the key's group
                    let Some(lhs_group) = self.get(&lhs_key) else {
                        continue;
                    };
                    flattened.reserve_edges(lhs_group.len());
                    for &lhs in lhs_group {
                        flattened.insert_edge(lhs, rhs);
                    }
                }
                (Ok(lhs), Err(rhs_key)) => {
                    // Edge from a value node to a key node, expand to all values in the key's group
                    let Some(rhs_group) = self.get(&rhs_key) else {
                        continue;
                    };
                    flattened.reserve_edges(rhs_group.len());
                    for &rhs in rhs_group {
                        flattened.insert_edge(lhs, rhs);
                    }
                }
                (Err(lhs_key), Err(rhs_key)) => {
                    // Edge between two key nodes, expand to all combinations of their value nodes
                    let Some(lhs_group) = self.get(&lhs_key) else {
                        continue;
                    };
                    let Some(rhs_group) = self.get(&rhs_key) else {
                        continue;
                    };
                    flattened.reserve_edges(lhs_group.len() * rhs_group.len());
                    for &lhs in lhs_group {
                        for &rhs in rhs_group {
                            flattened.insert_edge(lhs, rhs);
                        }
                    }
                }
            }
        }

        flattened
    }
}

// -----------------------------------------------------------------------------
// DagAnalysis

/// Stores the results of a call to [`Dag::analyze`].
pub struct DagAnalysis<N: GraphNode> {
    /// Boolean reachability matrix for the graph.
    reachable: FixedBitSet,
    /// Pairs of nodes that have a path connecting them.
    connected: HashSet<(N, N)>,
    /// Pairs of nodes that don't have a path connecting them.
    disconnected: Vec<(N, N)>,
    /// Edges that are redundant because a longer path exists.
    transitive_edges: Vec<(N, N)>,
    /// Variant of the graph with no transitive edges.
    transitive_reduction: DiGraph<N>,
    /// Variant of the graph with all possible transitive edges.
    transitive_closure: DiGraph<N>,
}

impl<N: GraphNode> Default for DagAnalysis<N> {
    fn default() -> Self {
        Self {
            reachable: Default::default(),
            connected: Default::default(),
            disconnected: Default::default(),
            transitive_edges: Default::default(),
            transitive_reduction: Default::default(),
            transitive_closure: Default::default(),
        }
    }
}

impl<N: GraphNode> Debug for DagAnalysis<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DagAnalysis")
            .field("reachable", &self.reachable)
            .field("connected", &self.connected)
            .field("disconnected", &self.disconnected)
            .field("transitive_edges", &self.transitive_edges)
            .field("transitive_reduction", &self.transitive_reduction)
            .field("transitive_closure", &self.transitive_closure)
            .finish()
    }
}

impl<N: GraphNode> DagAnalysis<N> {
    /// Processes a DAG and computes its:
    /// - transitive reduction (along with the set of removed edges)
    /// - transitive closure
    /// - reachability matrix (as a bitset)
    /// - pairs of nodes connected by a path
    /// - pairs of nodes not connected by a path
    ///
    /// The algorithm implemented comes from
    /// ["On the calculation of transitive reduction-closure of orders"][1] by Habib, Morvan and Rampon.
    ///
    /// [1]: https://doi.org/10.1016/0012-365X(93)90164-O
    pub fn new(graph: &DiGraph<N>, topological_order: &[N]) -> Self {
        if graph.node_count() == 0 {
            return DagAnalysis::default();
        }
        let n = graph.node_count();

        // build a copy of the graph where the nodes and edges appear in topsorted order
        let mut map = HashMap::with_capacity(n);
        let mut topsorted =
            DiGraph::<N>::with_capacity(topological_order.len(), graph.edge_count());

        // iterate nodes in topological order
        for (i, &node) in topological_order.iter().enumerate() {
            map.insert(node, i);
            topsorted.insert_node(node);
            // insert nodes as successors to their predecessors
            for pred in graph.neighbors_directed(node, Incoming) {
                topsorted.insert_edge(pred, node);
            }
        }

        let mut reachable = FixedBitSet::with_capacity(n * n);
        let mut connected = HashSet::default();
        let mut disconnected = Vec::default();
        let mut transitive_edges = Vec::default();
        let mut transitive_reduction = DiGraph::with_capacity(topsorted.node_count(), 0);
        let mut transitive_closure = DiGraph::with_capacity(topsorted.node_count(), 0);

        let mut visited = FixedBitSet::with_capacity(n);

        // iterate nodes in topological order
        for node in topsorted.nodes() {
            transitive_reduction.insert_node(node);
            transitive_closure.insert_node(node);
        }

        // iterate nodes in reverse topological order
        for a in topsorted.nodes().rev() {
            let index_a = *map.get(&a).unwrap();
            // iterate their successors in topological order
            for b in topsorted.neighbors_directed(a, Outgoing) {
                let index_b = *map.get(&b).unwrap();
                debug_assert!(index_a < index_b);
                if !visited[index_b] {
                    // edge <a, b> is not redundant
                    transitive_reduction.insert_edge(a, b);
                    transitive_closure.insert_edge(a, b);
                    reachable.insert(flatten_index(index_a, index_b, n));

                    let successors = transitive_closure
                        .neighbors_directed(b, Outgoing)
                        .collect::<Vec<_>>();
                    for c in successors {
                        let index_c = *map.get(&c).unwrap();
                        debug_assert!(index_b < index_c);
                        if !visited[index_c] {
                            visited.insert(index_c);
                            transitive_closure.insert_edge(a, c);
                            reachable.insert(flatten_index(index_a, index_c, n));
                        }
                    }
                } else {
                    // edge <a, b> is redundant
                    transitive_edges.push((a, b));
                }
            }

            visited.clear();
        }

        // partition pairs of nodes into "connected by path" and "not connected by path"
        for i in 0..(n - 1) {
            // reachable is upper triangular because the nodes were topsorted
            for index in flatten_index(i, i + 1, n)..=flatten_index(i, n - 1, n) {
                let (a, b) = unflatten_index(index, n);
                let pair = (topological_order[a], topological_order[b]);
                if reachable[index] {
                    connected.insert(pair);
                } else {
                    disconnected.push(pair);
                }
            }
        }

        // fill diagonal (nodes reach themselves)
        // for i in 0..n {
        //     reachable.set(index(i, i, n), true);
        // }

        DagAnalysis {
            reachable,
            connected,
            disconnected,
            transitive_edges,
            transitive_reduction,
            transitive_closure,
        }
    }

    /// Returns the reachability matrix.
    pub fn reachable(&self) -> &FixedBitSet {
        &self.reachable
    }

    /// Returns the set of node pairs that are connected by a path.
    pub fn connected(&self) -> &HashSet<(N, N)> {
        &self.connected
    }

    /// Returns the list of node pairs that are not connected by a path.
    pub fn disconnected(&self) -> &[(N, N)] {
        &self.disconnected
    }

    /// Returns the list of redundant edges because a longer path exists.
    pub fn transitive_edges(&self) -> &[(N, N)] {
        &self.transitive_edges
    }

    /// Returns the transitive reduction of the graph.
    pub fn transitive_reduction(&self) -> &DiGraph<N> {
        &self.transitive_reduction
    }

    /// Returns the transitive closure of the graph.
    pub fn transitive_closure(&self) -> &DiGraph<N> {
        &self.transitive_closure
    }

    /// Checks if the graph has any redundant (transitive) edges.
    ///
    /// # Errors
    ///
    /// If there are redundant edges, returns a [`DagRedundancyError`]
    /// containing the list of redundant edges.
    pub fn check_for_redundant_edges(&self) -> Result<(), DagRedundancyError<N>> {
        if self.transitive_edges.is_empty() {
            Ok(())
        } else {
            Err(DagRedundancyError(self.transitive_edges.clone()))
        }
    }

    /// Checks if there are any pairs of nodes that have a path in both this
    /// graph and another graph.
    ///
    /// # Errors
    ///
    /// Returns [`DagCrossDependencyError`] if any node pair is connected in
    /// both graphs.
    pub fn check_for_cross_dependencies(
        &self,
        other: &Self,
    ) -> Result<(), DagCrossDependencyError<N>> {
        for &(a, b) in &self.connected {
            if other.connected.contains(&(a, b)) || other.connected.contains(&(b, a)) {
                return Err(DagCrossDependencyError(a, b));
            }
        }

        Ok(())
    }

    /// Checks if any connected node pairs that are both keys have overlapping
    /// groups.
    ///
    /// # Errors
    ///
    /// If there are overlapping groups, returns a [`DagOverlappingGroupError`]
    /// containing the first pair of keys that have overlapping groups.
    pub fn check_for_overlapping_groups<K, V>(
        &self,
        groups: &DagGroups<K, V>,
    ) -> Result<(), DagOverlappingGroupError<K>>
    where
        N: TryInto<K>,
        K: Eq + Hash,
        V: Eq + Hash,
    {
        for &(a, b) in &self.connected {
            let (Ok(a_key), Ok(b_key)) = (a.try_into(), b.try_into()) else {
                continue;
            };
            let a_group = groups.get(&a_key).unwrap();
            let b_group = groups.get(&b_key).unwrap();
            if !a_group.is_disjoint(b_group) {
                return Err(DagOverlappingGroupError(a_key, b_key));
            }
        }
        Ok(())
    }
}

/// Error indicating that the graph has redundant edges.
#[derive(Error, Debug)]
#[error("DAG has redundant edges: {0:?}")]
pub struct DagRedundancyError<N: GraphNode>(pub Vec<(N, N)>);

/// Error indicating that two graphs both have a dependency between the same nodes.
#[derive(Error, Debug)]
#[error("DAG has a cross-dependency between nodes {0:?} and {1:?}")]
pub struct DagCrossDependencyError<N>(pub N, pub N);

/// Error indicating that the graph has overlapping groups between two keys.
#[derive(Error, Debug)]
#[error("DAG has overlapping groups between keys {0:?} and {1:?}")]
pub struct DagOverlappingGroupError<K>(pub K, pub K);
