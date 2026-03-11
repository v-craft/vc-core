use alloc::vec::Vec;
use fixedbitset::FixedBitSet;
use vc_utils::hash::HashMap;

use crate::schedule::{DiGraph, GraphNode};

impl<N: GraphNode> DiGraph<N> {
    pub(crate) fn transitive_reduction(&self, topo: &[N], index_map: &HashMap<N, usize>) -> Self {
        if self.node_count() == 0 {
            return Self::new();
        }

        let node_len = topo.len();
        assert_eq!(self.node_count(), node_len);

        let mut transitive_closure = DiGraph::with_capacity(node_len, node_len * 16);
        let mut reduction = DiGraph::with_capacity(node_len, node_len * 4);
        for &node in topo {
            reduction.insert_node(node);
        }

        let mut visited = FixedBitSet::with_capacity(node_len);

        for index_a in (0..node_len).rev() {
            let a = topo[index_a];

            for b in self.neighbors(a) {
                let index_b = index_map[&b];
                debug_assert!(index_a < index_b);

                if !visited[index_b] {
                    reduction.insert_edge(a, b);
                    visited.insert(index_b);
                }

                let successors: Vec<_> = transitive_closure.neighbors(b).collect();
                for c in successors {
                    let index_c = index_map[&c];
                    debug_assert!(index_b < index_c);

                    if !visited[index_c] {
                        visited.insert(index_c);
                        transitive_closure.insert_edge(a, c);
                    }
                }
            }

            visited.clear();
        }

        reduction
    }
}
