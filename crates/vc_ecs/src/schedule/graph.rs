use alloc::vec::Vec;
use core::fmt::Debug;
use core::hash::Hash;

use thiserror::Error;
use vc_utils::hash::HashMap;
use vc_utils::index::IndexMap;
use vc_utils::{hash::HashSet, vec::SmallVec};

use Direction::{Incoming, Outgoing};

// -----------------------------------------------------------------------------
// Graph

/// Edge direction.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
#[repr(u8)]
pub enum Direction {
    /// An `Outgoing` edge is an outward edge *from* the current node.
    Outgoing = 0,
    /// An `Incoming` edge is an inbound edge *to* the current node.
    Incoming = 1,
}

impl Direction {
    /// Return the opposite `Direction`.
    #[inline]
    pub const fn opposite(self) -> Self {
        match self {
            Self::Outgoing => Self::Incoming,
            Self::Incoming => Self::Outgoing,
        }
    }
}

/// Types that can be used as node identifiers in a [`DiGraph`]/[`UnGraph`].
pub trait GraphNode: Copy + Eq + Hash + Ord + Debug {
    /// The type that packs and unpacks this [`GraphNode`] with a [`Direction`].
    /// This is used to save space in the graph's adjacency list.
    type Link: Copy + Debug + From<(Self, Direction)> + Into<(Self, Direction)>;
    /// The type that packs and unpacks this [`GraphNode`] with another
    /// [`GraphNode`]. This is used to save space in the graph's edge list.
    type Edge: Copy + Eq + Hash + Debug + From<(Self, Self)> + Into<(Self, Self)>;

    /// Name of the kind of this node id.
    ///
    /// For structs, this should return a human-readable name of the struct.
    /// For enums, this should return a human-readable name of the enum variant.
    fn name(&self) -> &'static str;
}

/// `Graph<DIRECTED>` is a graph datastructure using an associative array
/// of its node weights of some [`GraphNodeId`].
///
/// It uses a combined adjacency list and sparse adjacency matrix
/// representation, using **O(|N| + |E|)** space, and allows testing for edge
/// existence in constant time.
///
/// `Graph` is parameterized over:
///
/// - Constant generic bool `DIRECTED` determines whether the graph edges are directed or
///   undirected.
/// - The `GraphNodeId` type `N`, which is used as the node weight.
/// - The `BuildHasher` `S`.
///
/// You can use the type aliases `UnGraph` and `DiGraph` for convenience.
///
/// `Graph` does not allow parallel edges, but self loops are allowed.
#[derive(Clone)]
pub struct Graph<const DIRECTED: bool, N: GraphNode> {
    nodes: IndexMap<N, Vec<N::Link>>,
    edges: HashSet<N::Edge>,
}

pub type DiGraph<N> = Graph<true, N>;
pub type UnGraph<N> = Graph<false, N>;

impl<const DIRECTED: bool, N: GraphNode> Debug for Graph<DIRECTED, N> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.nodes.fmt(f)
    }
}

impl<const DIRECTED: bool, N: GraphNode> Default for Graph<DIRECTED, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const DIRECTED: bool, N: GraphNode> Graph<DIRECTED, N> {
    pub const fn new() -> Self {
        Self {
            nodes: IndexMap::new(),
            edges: HashSet::new(),
        }
    }

    pub fn with_capacity(nodes: usize, edges: usize) -> Self {
        Self {
            nodes: IndexMap::with_capacity(nodes),
            edges: HashSet::with_capacity(edges),
        }
    }

    #[inline]
    fn edge_key(a: N, b: N) -> N::Edge {
        let (a, b) = if DIRECTED || a <= b { (a, b) } else { (b, a) };

        N::Edge::from((a, b))
    }

    fn remove_link(&mut self, x: N, y: N, dir: Direction) -> bool {
        if let Some(links) = self.nodes.get_mut(&x) {
            let index = links
                .iter()
                .copied()
                .map(N::Link::into)
                .position(|link| link == (y, dir));
            if let Some(index) = index {
                links.swap_remove(index);
                return true;
            }
        };
        false
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn reserve_nodes(&mut self, additional: usize) {
        self.nodes.reserve(additional);
    }

    pub fn reserve_edges(&mut self, additional: usize) {
        self.edges.reserve(additional);
    }

    pub fn contains_node(&self, n: N) -> bool {
        self.nodes.contains_key(&n)
    }

    pub fn contains_edge(&self, a: N, b: N) -> bool {
        self.edges.contains(&Self::edge_key(a, b))
    }

    pub fn insert_node(&mut self, n: N) {
        self.nodes.entry(n).or_default();
    }

    pub fn remove_node(&mut self, n: N) {
        let Some(links) = self.nodes.swap_remove(&n) else {
            return;
        };

        let links = links.into_iter().map(N::Link::into);

        links.into_iter().for_each(|(to, dir)| {
            let (edge, rdir) = if dir == Outgoing {
                (Self::edge_key(n, to), Incoming)
            } else {
                (Self::edge_key(to, n), Outgoing)
            };

            self.remove_link(to, n, rdir);
            self.edges.remove(&edge);
        })
    }

    pub fn insert_edge(&mut self, a: N, b: N) {
        if self.edges.insert(Self::edge_key(a, b)) {
            // insert in the adjacency list if it's a new edge
            self.nodes
                .entry(a)
                .or_insert_with(|| Vec::with_capacity(1))
                .push(N::Link::from((b, Outgoing)));
            if a != b {
                // self loops don't have the Incoming entry
                self.nodes
                    .entry(b)
                    .or_insert_with(|| Vec::with_capacity(1))
                    .push(N::Link::from((a, Incoming)));
            }
        }
    }

    pub fn remove_edge(&mut self, a: N, b: N) -> bool {
        let exist1 = self.remove_link(a, b, Outgoing);
        let exist2 = if a != b {
            self.remove_link(b, a, Incoming)
        } else {
            vc_utils::cold_path();
            exist1
        };
        let weight = self.edges.remove(&Self::edge_key(a, b));
        debug_assert!(exist1 == exist2 && exist1 == weight);
        weight
    }

    pub fn nodes(&self) -> impl DoubleEndedIterator<Item = N> + ExactSizeIterator<Item = N> + '_ {
        self.nodes.keys().copied()
    }

    pub fn all_edges(&self) -> impl ExactSizeIterator<Item = (N, N)> + '_ {
        self.edges.iter().copied().map(N::Edge::into)
    }

    pub fn edges(&self, a: N) -> impl DoubleEndedIterator<Item = (N, N)> + '_ {
        let iter = match self.nodes.get(&a) {
            Some(neigh) => neigh.iter(),
            None => [].iter(),
        };

        iter.copied()
            .map(N::Link::into)
            .filter_map(move |(b, dir)| (!DIRECTED || dir == Outgoing).then_some((a, b)))
    }

    pub fn neighbors(&self, a: N) -> impl DoubleEndedIterator<Item = N> + '_ {
        let iter = match self.nodes.get(&a) {
            Some(neigh) => neigh.iter(),
            None => [].iter(),
        };

        iter.copied()
            .map(N::Link::into)
            .filter_map(|(n, dir)| (!DIRECTED || dir == Outgoing).then_some(n))
    }

    pub fn edges_directed(
        &self,
        a: N,
        dir: Direction,
    ) -> impl DoubleEndedIterator<Item = (N, N)> + '_ {
        let iter = match self.nodes.get(&a) {
            Some(neigh) => neigh.iter(),
            None => [].iter(),
        };

        iter.copied()
            .map(N::Link::into)
            .filter_map(move |(b, d)| (!DIRECTED || d == dir || b == a).then_some((a, b)))
    }

    pub fn neighbors_directed(
        &self,
        a: N,
        dir: Direction,
    ) -> impl DoubleEndedIterator<Item = N> + '_ {
        let iter = match self.nodes.get(&a) {
            Some(neigh) => neigh.iter(),
            None => [].iter(),
        };

        iter.copied()
            .map(N::Link::into)
            .filter_map(move |(n, d)| (!DIRECTED || d == dir || n == a).then_some(n))
    }

    pub fn try_convert<T>(self) -> Result<Graph<DIRECTED, T>, N::Error>
    where
        N: TryInto<T>,
        T: GraphNode,
    {
        // Converts the node key and every adjacency list entry from `N` to `T`.
        fn try_convert_node<N: GraphNode + TryInto<T>, T: GraphNode>(
            (key, links): (N, Vec<N::Link>),
        ) -> Result<(T, Vec<T::Link>), N::Error> {
            let key = key.try_into()?;
            let links = links
                .into_iter()
                .map(|link| {
                    let (id, dir) = link.into();
                    Ok(T::Link::from((id.try_into()?, dir)))
                })
                .collect::<Result<_, N::Error>>()?;
            Ok((key, links))
        }
        // Unpacks the edge pair, converts the nodes from `N` to `T`, and repacks them.
        fn try_convert_edge<N: GraphNode + TryInto<T>, T: GraphNode>(
            edge: N::Edge,
        ) -> Result<T::Edge, N::Error> {
            let (a, b) = edge.into();
            Ok(T::Edge::from((a.try_into()?, b.try_into()?)))
        }

        let nodes = self
            .nodes
            .into_iter()
            .map(try_convert_node::<N, T>)
            .collect::<Result<_, N::Error>>()?;
        let edges = self
            .edges
            .into_iter()
            .map(try_convert_edge::<N, T>)
            .collect::<Result<_, N::Error>>()?;
        Ok(Graph { nodes, edges })
    }

    pub(super) fn to_index(&self, ix: N) -> usize {
        self.nodes.get_index_of(&ix).unwrap()
    }
}

// -----------------------------------------------------------------------------
// DiGraph

#[derive(Error, Debug)]
pub enum DiGraphToposortError<N: GraphNode> {
    /// A self-loop was detected.
    #[error("self-loop detected at node `{0:?}`")]
    Loop(N),
    /// Cycles were detected.
    #[error("cycles detected: {0:?}")]
    Cycle(Vec<Vec<N>>),
}

impl<N: GraphNode> DiGraph<N> {
    /// Iterate over all *Strongly Connected Components* in this graph.
    fn iter_sccs(&self) -> impl Iterator<Item = SmallVec<N, 4>> + '_ {
        super::tarjan_scc::new_tarjan_scc(self)
    }

    pub fn toposort(&self, mut scratch: Vec<N>) -> Result<Vec<N>, DiGraphToposortError<N>> {
        // Check explicitly for self-edges.
        // `iter_sccs` won't report them as cycles because they still form components of one node.
        if let Some((node, _)) = self.all_edges().find(|(x, y)| x == y) {
            vc_utils::cold_path();
            return Err(DiGraphToposortError::Loop(node));
        }

        // Tarjan's SCC algorithm returns elements in *reverse* topological order.
        scratch.clear();
        scratch.reserve_exact(self.node_count().saturating_sub(scratch.capacity()));
        let mut top_sorted_nodes = scratch;
        let mut sccs_with_cycles = Vec::new();

        for scc in self.iter_sccs() {
            let slice = scc.as_slice();
            // A strongly-connected component is a group of nodes who can all reach each other
            // through one or more paths. If an SCC contains more than one node, there must be
            // at least one cycle within them.
            top_sorted_nodes.extend_from_slice(slice);
            // Do not use `SmallVec::len`, it may make an additional judgment.
            if slice.len() > 1 {
                vc_utils::cold_path();
                sccs_with_cycles.push(scc);
            }
        }

        if sccs_with_cycles.is_empty() {
            // reverse to get topological order
            top_sorted_nodes.reverse();
            Ok(top_sorted_nodes)
        } else {
            vc_utils::cold_path();
            let mut cycles = Vec::new();
            for scc in &sccs_with_cycles {
                cycles.append(&mut self.simple_cycles_in_component(scc));
            }

            Err(DiGraphToposortError::Cycle(cycles))
        }
    }

    /// Returns the simple cycles in a strongly-connected component of a directed graph.
    ///
    /// The algorithm implemented comes from
    /// ["Finding all the elementary circuits of a directed graph"][1] by D. B. Johnson.
    ///
    /// [1]: https://doi.org/10.1137/0204007
    pub fn simple_cycles_in_component(&self, scc: &[N]) -> Vec<Vec<N>> {
        let mut cycles = Vec::new();

        let mut sccs = Vec::new();
        sccs.push(SmallVec::from_slice(scc));

        while let Some(mut scc) = sccs.pop() {
            // only look at nodes and edges in this strongly-connected component
            let mut subgraph = DiGraph::<N>::with_capacity(scc.len(), 0);
            for &node in &scc {
                subgraph.insert_node(node);
            }

            for &node in &scc {
                for successor in self.neighbors(node) {
                    if subgraph.contains_node(successor) {
                        subgraph.insert_edge(node, successor);
                    }
                }
            }

            // path of nodes that may form a cycle
            let mut path: Vec<N> = Vec::with_capacity(subgraph.node_count());
            // we mark nodes as "blocked" to avoid finding permutations of the same cycles
            let mut blocked: HashSet<N> = HashSet::with_capacity(subgraph.node_count());
            // connects nodes along path segments that can't be part of a cycle (given current root)
            // those nodes can be unblocked at the same time
            let mut unblock_together: HashMap<N, HashSet<N>> =
                HashMap::with_capacity(subgraph.node_count());
            // stack for unblocking nodes
            let mut unblock_stack: Vec<N> = Vec::with_capacity(subgraph.node_count());
            // nodes can be involved in multiple cycles
            let mut maybe_in_more_cycles: HashSet<N> =
                HashSet::with_capacity(subgraph.node_count());
            // stack for DFS
            let mut stack = Vec::with_capacity(subgraph.node_count());

            // we're going to look for all cycles that begin and end at this node
            let root = scc.pop().unwrap();
            // start a path at the root
            path.clear();
            path.push(root);
            // mark this node as blocked
            blocked.insert(root);

            // DFS
            stack.clear();
            stack.push((root, subgraph.neighbors(root)));
            while !stack.is_empty() {
                let &mut (ref node, ref mut successors) = stack.last_mut().unwrap();
                if let Some(next) = successors.next() {
                    if next == root {
                        // found a cycle
                        maybe_in_more_cycles.extend(path.iter());
                        cycles.push(path.clone());
                    } else if !blocked.contains(&next) {
                        // first time seeing `next` on this path
                        maybe_in_more_cycles.remove(&next);
                        path.push(next);
                        blocked.insert(next);
                        stack.push((next, subgraph.neighbors(next)));
                        continue;
                    } else {
                        // not first time seeing `next` on this path
                    }
                }

                if successors.peekable().peek().is_none() {
                    if maybe_in_more_cycles.contains(node) {
                        unblock_stack.push(*node);
                        // unblock this node's ancestors
                        while let Some(n) = unblock_stack.pop() {
                            if blocked.remove(&n) {
                                let unblock_predecessors = unblock_together.entry(n).or_default();
                                unblock_stack.extend(unblock_predecessors.iter());
                                unblock_predecessors.clear();
                            }
                        }
                    } else {
                        // if its descendants can be unblocked later, this node will be too
                        for successor in subgraph.neighbors(*node) {
                            unblock_together.entry(successor).or_default().insert(*node);
                        }
                    }

                    // remove node from path and DFS stack
                    path.pop();
                    stack.pop();
                }
            }

            drop(stack);

            // remove node from subgraph
            subgraph.remove_node(root);

            // divide remainder into smaller SCCs
            sccs.extend(subgraph.iter_sccs().filter(|scc| scc.len() > 1));
        }

        cycles
    }
}
