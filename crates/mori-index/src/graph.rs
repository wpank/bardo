//! Symbol dependency graph with PageRank scoring.

use std::collections::HashMap;

/// A directed graph of symbol dependencies.
///
/// Edges go from a symbol to the symbols it references (forward = dependencies,
/// reverse = dependents).
#[derive(Debug, Clone)]
pub struct SymbolGraph {
    /// Forward edges: symbol -> symbols it depends on.
    forward: HashMap<i64, Vec<i64>>,
    /// Reverse edges: symbol -> symbols that depend on it.
    reverse: HashMap<i64, Vec<i64>>,
    /// Total number of nodes in the graph.
    node_count: usize,
}

impl SymbolGraph {
    /// Build a graph from a list of directed edges (from, to).
    pub fn from_edges(edges: &[(i64, i64)], node_count: usize) -> Self {
        let mut forward: HashMap<i64, Vec<i64>> = HashMap::new();
        let mut reverse: HashMap<i64, Vec<i64>> = HashMap::new();

        for &(from, to) in edges {
            forward.entry(from).or_default().push(to);
            reverse.entry(to).or_default().push(from);
        }

        Self {
            forward,
            reverse,
            node_count,
        }
    }

    /// Get the symbols this symbol depends on.
    pub fn dependencies(&self, id: i64) -> &[i64] {
        self.forward.get(&id).map_or(&[], Vec::as_slice)
    }

    /// Get the symbols that depend on this symbol.
    pub fn dependents(&self, id: i64) -> &[i64] {
        self.reverse.get(&id).map_or(&[], Vec::as_slice)
    }

    /// The number of nodes in the graph.
    pub fn node_count(&self) -> usize {
        self.node_count
    }

    /// Run iterative PageRank with optional bias toward symbols in specific files.
    ///
    /// Symbols in `bias_file_ids` get a higher initial rank. `file_of_symbol` maps
    /// each symbol id to its file id. Returns symbol ids sorted by rank descending.
    pub fn pagerank(
        &self,
        bias_file_ids: &[i64],
        file_of_symbol: &HashMap<i64, i64>,
        iterations: usize,
        damping: f32,
    ) -> Vec<(i64, f32)> {
        // Collect all unique node ids
        let mut all_nodes: Vec<i64> = Vec::new();
        for &id in self.forward.keys() {
            all_nodes.push(id);
        }
        for &id in self.reverse.keys() {
            if !all_nodes.contains(&id) {
                all_nodes.push(id);
            }
        }

        if all_nodes.is_empty() {
            return Vec::new();
        }

        let n = all_nodes.len();
        let mut rank: HashMap<i64, f32> = HashMap::with_capacity(n);

        // Initialize ranks with bias
        let bias_boost = 2.0_f32;
        let mut total_init = 0.0_f32;
        for &node in &all_nodes {
            let file_id = file_of_symbol.get(&node).copied().unwrap_or(-1);
            let init = if bias_file_ids.contains(&file_id) {
                bias_boost
            } else {
                1.0
            };
            rank.insert(node, init);
            total_init += init;
        }
        // Normalize initial ranks
        if total_init > 0.0 {
            for val in rank.values_mut() {
                *val /= total_init;
            }
        }

        // Iterative PageRank
        for _ in 0..iterations {
            let mut new_rank: HashMap<i64, f32> = HashMap::with_capacity(n);
            let base = (1.0 - damping) / n as f32;

            for &node in &all_nodes {
                let mut incoming_sum = 0.0_f32;
                let inbound = self.reverse.get(&node);
                if let Some(inbound_nodes) = inbound {
                    for &src in inbound_nodes {
                        let src_rank = rank.get(&src).copied().unwrap_or(0.0);
                        let out_degree = self.forward.get(&src).map_or(1, Vec::len).max(1) as f32;
                        incoming_sum += src_rank / out_degree;
                    }
                }
                new_rank.insert(node, base + damping * incoming_sum);
            }
            rank = new_rank;
        }

        // Sort descending by rank
        let mut results: Vec<(i64, f32)> = rank.into_iter().collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_node_cycle_equal_ranks() {
        // A -> B -> C -> A (cycle)
        let edges = vec![(1, 2), (2, 3), (3, 1)];
        let graph = SymbolGraph::from_edges(&edges, 3);

        let file_map: HashMap<i64, i64> = [(1, 1), (2, 1), (3, 1)].into_iter().collect();

        let ranks = graph.pagerank(&[], &file_map, 50, 0.85);
        assert_eq!(ranks.len(), 3);

        // In a 3-node cycle, all ranks should converge to roughly equal values
        let r0 = ranks[0].1;
        let r1 = ranks[1].1;
        let r2 = ranks[2].1;
        let max_diff = (r0 - r1).abs().max((r1 - r2).abs()).max((r0 - r2).abs());
        assert!(
            max_diff < 0.01,
            "3-node cycle should have equal ranks, got diffs: {max_diff}, ranks: {r0}, {r1}, {r2}"
        );
    }

    #[test]
    fn star_topology_hub_highest() {
        // Spokes 2, 3, 4, 5 all point to hub 1
        let edges = vec![(2, 1), (3, 1), (4, 1), (5, 1)];
        let graph = SymbolGraph::from_edges(&edges, 5);

        let file_map: HashMap<i64, i64> = [(1, 1), (2, 1), (3, 1), (4, 1), (5, 1)]
            .into_iter()
            .collect();

        let ranks = graph.pagerank(&[], &file_map, 50, 0.85);
        assert!(!ranks.is_empty());

        // Hub node 1 should have the highest rank
        let hub_entry = ranks.iter().find(|(id, _)| *id == 1);
        assert!(hub_entry.is_some(), "Hub not found in results");
        let hub_rank = hub_entry.map_or(0.0, |e| e.1);

        for &(id, rank) in &ranks {
            if id != 1 {
                assert!(
                    hub_rank > rank,
                    "Hub rank {hub_rank} should exceed spoke {id} rank {rank}"
                );
            }
        }
    }

    #[test]
    fn dependencies_and_dependents() {
        let edges = vec![(1, 2), (1, 3), (2, 3)];
        let graph = SymbolGraph::from_edges(&edges, 3);

        assert_eq!(graph.dependencies(1).len(), 2);
        assert!(graph.dependencies(1).contains(&2));
        assert!(graph.dependencies(1).contains(&3));

        assert_eq!(graph.dependents(3).len(), 2);
        assert!(graph.dependents(3).contains(&1));
        assert!(graph.dependents(3).contains(&2));

        assert!(graph.dependencies(3).is_empty());
        assert!(graph.dependents(1).is_empty());
    }

    #[test]
    fn empty_graph() {
        let graph = SymbolGraph::from_edges(&[], 0);
        let ranks = graph.pagerank(&[], &HashMap::new(), 10, 0.85);
        assert!(ranks.is_empty());
    }
}
