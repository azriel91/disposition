use disposition_ir_model::{
    edge::{Edge, EdgeGroups},
    entity::{EntityType, EntityTypes},
    node::{NodeId, NodeNestingInfos, NodeRank, NodeRanks, NodeRanksNested},
};
use disposition_model_common::{Id, Map};

/// Computes [`NodeRanksNested`] from dependency edges in an [`IrDiagram`].
///
/// Dependency edges indicate that the `to` node should be positioned after
/// the `from` node. This calculator performs hierarchy-aware rank assignment:
///
/// * Ranks are computed independently for each hierarchy level (root and each
///   container node).
/// * Dependency edges that cross container boundaries are attributed to the
///   lowest common ancestor (LCA) of the two endpoints, using the first
///   divergent sibling ancestors at that level as the effective edge.
/// * Nodes with no incoming dependency edges at their level receive rank `0`.
/// * Each node's rank is one greater than the maximum rank of its predecessors
///   at the same level.
/// * Nodes that are part of a cycle share the same rank -- the cycle is
///   contracted into a single logical node for ranking purposes.
///
/// Only **dependency** edges (not interaction edges) are considered for rank
/// computation, plus any **layout edges** passed in separately -- these are
/// invisible edges (from `thing_layout_edges`) that contribute to rank
/// exactly like dependency edges, without ever appearing in `edge_groups` or
/// being rendered.
///
/// [`IrDiagram`]: disposition_ir_model::IrDiagram
///
/// # Examples
///
/// Given edges `A -> B -> C` all at the same level, the resulting ranks are:
///
/// * `A`: 0
/// * `B`: 1
/// * `C`: 2
///
/// Given `a -> b`, `b_child_0 -> b_child_1`, `b_child_0 -> c_child`:
///
/// * Root level -- `a: 0`, `b: 1`, `c: 2` (lifted from `b_child_0 -> c_child`)
/// * `b`'s level -- `b_child_0: 0`, `b_child_1: 1`
/// * `c`'s level -- `c_child: 0` (edge is at root level, not `c`'s level)
#[derive(Clone, Copy, Debug)]
pub struct NodeRanksCalculator;

/// The dependency graph contracted to a DAG over strongly connected component
/// (SCC) ids.
///
/// Produced by [`NodeRanksCalculator::scc_dag_build`] and consumed by
/// [`NodeRanksCalculator::scc_dag_ranks_compute`].
struct SccDag {
    /// `adjacency[from_scc]` lists the distinct SCC ids that depend on
    /// `from_scc` (self-edges excluded).
    adjacency: Vec<Vec<usize>>,
    /// `in_degree[scc]` is the number of distinct predecessor SCCs.
    in_degree: Vec<usize>,
}

/// Internal state used by Tarjan's iterative strongly connected components
/// (SCC) algorithm.
struct TarjanState {
    index_counter: usize,
    stack: Vec<usize>,
    on_stack: Vec<bool>,
    index: Vec<Option<usize>>,
    lowlink: Vec<usize>,
    scc_ids: Vec<usize>,
    scc_counter: usize,
}

impl NodeRanksCalculator {
    /// Computes hierarchy-aware node ranks from dependency edges.
    ///
    /// Ranks are computed per hierarchy level using `node_nesting_infos` to
    /// group nodes and attribute cross-container edges to their LCA level.
    ///
    /// # Parameters
    ///
    /// * `edge_groups`: All edge groups in the diagram.
    /// * `entity_types`: Entity types used to distinguish dependency edges from
    ///   interaction edges.
    /// * `node_nesting_infos`: Nesting information for each node, used to build
    ///   the container-to-children map and compute LCA-level edge attribution.
    /// * `layout_edges`: Invisible layout-only edges (from
    ///   `thing_layout_edges`) that contribute to rank alongside dependency
    ///   edges, without being backed by an edge group.
    pub fn calculate<'id>(
        edge_groups: &EdgeGroups<'id>,
        entity_types: &EntityTypes<'id>,
        node_nesting_infos: &NodeNestingInfos<'id>,
        layout_edges: &[Edge<'id>],
    ) -> NodeRanksNested<'id> {
        if node_nesting_infos.is_empty() {
            return NodeRanksNested::new();
        }

        // === Build Container-to-Children Map === //
        let container_to_children = Self::container_to_children_build(node_nesting_infos);

        // === Collect Dependency Edges === //
        let dependency_edges =
            Self::dependency_edges_collect(edge_groups, entity_types, layout_edges);

        // === Lift Edges to LCA Level === //
        let lca_level_edges = Self::lca_level_edges_build(&dependency_edges, node_nesting_infos);

        // === Compute Ranks Per Level === //
        // Fold each container level into the `(root, containers)` accumulator:
        // the root level (`None`) becomes `root`, every other container is
        // inserted into the `containers` map.
        let empty_edges: Vec<(NodeId<'id>, NodeId<'id>)> = Vec::new();
        let (root, containers) = container_to_children.iter().fold(
            (NodeRanks::new(), Map::new()),
            |(mut root, mut containers), (container, children)| {
                let edges = lca_level_edges.get(container).unwrap_or(&empty_edges);
                let ranks = Self::ranks_compute(children, edges);
                match container {
                    None => root = ranks,
                    Some(container_id) => {
                        containers.insert(container_id.clone(), ranks);
                    }
                }
                (root, containers)
            },
        );

        NodeRanksNested { root, containers }
    }

    /// Builds a map from container node (or `None` for root) to its direct
    /// children.
    ///
    /// Each node's parent is the second-to-last element of its
    /// `ancestor_chain`. Top-level nodes (chain length 1) belong to the root
    /// level, keyed by `None`.
    fn container_to_children_build<'id>(
        node_nesting_infos: &NodeNestingInfos<'id>,
    ) -> Map<Option<NodeId<'id>>, Vec<NodeId<'id>>> {
        node_nesting_infos.iter().fold(
            Map::new(),
            |mut container_to_children, (node_id, nesting_info)| {
                let chain = &nesting_info.ancestor_chain;
                let parent = chain
                    .len()
                    .checked_sub(2)
                    .map(|parent_idx| chain[parent_idx].clone());
                container_to_children
                    .entry(parent)
                    .or_default()
                    .push(node_id.clone());
                container_to_children
            },
        )
    }

    /// Lifts each dependency edge to the LCA-level edge between the divergent
    /// sibling ancestors of the two endpoints.
    ///
    /// Groups resulting LCA-level edges by their LCA container (`None` for
    /// root).
    fn lca_level_edges_build<'id>(
        dependency_edges: &[(NodeId<'id>, NodeId<'id>)],
        node_nesting_infos: &NodeNestingInfos<'id>,
    ) -> Map<Option<NodeId<'id>>, Vec<(NodeId<'id>, NodeId<'id>)>> {
        dependency_edges
            .iter()
            .filter_map(|(from_id, to_id)| {
                Self::lca_level_edge_compute(from_id, to_id, node_nesting_infos)
            })
            .fold(
                Map::new(),
                |mut lca_level_edges, (lca_container, divergent_from, divergent_to)| {
                    lca_level_edges
                        .entry(lca_container)
                        .or_default()
                        .push((divergent_from, divergent_to));
                    lca_level_edges
                },
            )
    }

    /// Computes the LCA container and divergent ancestors for a single edge.
    ///
    /// Returns `None` if either endpoint is absent from `node_nesting_infos`,
    /// if one node is an ancestor of the other (edge is within a subtree), or
    /// if the two divergent ancestors are the same node (self-loop at LCA
    /// level).
    ///
    /// # Return value
    ///
    /// `Some((lca_container, divergent_from, divergent_to))` where:
    ///
    /// * `lca_container` -- `None` for root-level, `Some(id)` for a container.
    /// * `divergent_from`, `divergent_to` -- the first ancestors of `from` and
    ///   `to` that differ under the LCA, at depth `lca_depth` in their
    ///   respective `ancestor_chain`s.
    fn lca_level_edge_compute<'id>(
        from_id: &NodeId<'id>,
        to_id: &NodeId<'id>,
        node_nesting_infos: &NodeNestingInfos<'id>,
    ) -> Option<(Option<NodeId<'id>>, NodeId<'id>, NodeId<'id>)> {
        let info_from = node_nesting_infos.get(from_id)?;
        let info_to = node_nesting_infos.get(to_id)?;

        let chain_from = &info_from.ancestor_chain;
        let chain_to = &info_to.ancestor_chain;

        let lca_depth = chain_from
            .iter()
            .zip(chain_to.iter())
            .take_while(|(a, b)| a == b)
            .count();

        // If one node is an ancestor of the other, skip the edge.
        if lca_depth >= chain_from.len() || lca_depth >= chain_to.len() {
            return None;
        }

        let divergent_from = chain_from[lca_depth].clone();
        let divergent_to = chain_to[lca_depth].clone();

        // Skip self-loops at LCA level (divergent ancestors are the same node).
        if divergent_from == divergent_to {
            return None;
        }

        let lca_container = lca_depth
            .checked_sub(1)
            .map(|lca_idx| chain_from[lca_idx].clone());

        Some((lca_container, divergent_from, divergent_to))
    }

    /// Extracts dependency and layout edges that contribute to rank,
    /// filtering out interaction edges.
    ///
    /// Returns a list of `(from_id, to_id)` pairs for dependency edges (from
    /// `edge_groups`, filtered by `entity_types`) and layout edges (passed in
    /// directly -- they have no backing edge group).
    fn dependency_edges_collect<'id>(
        edge_groups: &EdgeGroups<'id>,
        entity_types: &EntityTypes<'id>,
        layout_edges: &[Edge<'id>],
    ) -> Vec<(NodeId<'id>, NodeId<'id>)> {
        let dependency_group_edges = edge_groups
            .iter()
            .filter(|(edge_group_id, _edge_group)| {
                Self::edge_group_is_dependency(edge_group_id.as_ref(), entity_types)
            })
            .flat_map(|(_edge_group_id, edge_group)| edge_group.iter())
            .map(|edge| (edge.from.clone(), edge.to.clone()));

        let layout_edge_pairs = layout_edges
            .iter()
            .map(|edge| (edge.from.clone(), edge.to.clone()));

        dependency_group_edges
            .chain(layout_edge_pairs)
            // Skip self-loops -- they don't affect rank.
            .filter(|(from, to)| from != to)
            .collect()
    }

    /// Returns whether the edge group with the given ID is a dependency edge
    /// group (as opposed to an interaction edge group).
    fn edge_group_is_dependency(edge_group_id: &Id, entity_types: &EntityTypes<'_>) -> bool {
        entity_types
            .get(edge_group_id)
            .map(|types| types.iter().any(Self::entity_type_is_dependency_edge_group))
            .unwrap_or(false)
    }

    /// Returns whether the given entity type represents a dependency edge
    /// group (as opposed to an interaction edge group).
    fn entity_type_is_dependency_edge_group(entity_type: &EntityType) -> bool {
        matches!(
            entity_type,
            EntityType::DependencyEdgeCyclicDefault
                | EntityType::DependencyEdgeSequenceDefault
                | EntityType::DependencyEdgeSymmetricDefault
        )
    }

    /// Computes ranks for all nodes using SCC-based contraction.
    ///
    /// 1. Build an adjacency list from the dependency edges.
    /// 2. Compute strongly connected components (Tarjan's algorithm).
    /// 3. Contract cycles: all nodes in the same SCC get the same SCC index.
    /// 4. Build a DAG of SCC indices and compute ranks on the DAG using
    ///    topological ordering (longest path).
    /// 5. Map SCC ranks back to individual node ranks.
    fn ranks_compute<'id>(
        all_node_ids: &[NodeId<'id>],
        dependency_edges: &[(NodeId<'id>, NodeId<'id>)],
    ) -> NodeRanks<'id> {
        if all_node_ids.is_empty() {
            return NodeRanks::new();
        }

        if dependency_edges.is_empty() {
            // No dependency edges -- all nodes get rank 0.
            return Self::node_ranks_uniform(all_node_ids, 0);
        }

        let adjacency = Self::ranks_compute_adjacency_build(all_node_ids, dependency_edges);
        let node_ranks = Self::ranks_compute_from_adjacency(&adjacency, all_node_ids.len());

        all_node_ids
            .iter()
            .zip(node_ranks)
            .map(|(node_id, rank)| (node_id.clone(), NodeRank::new(rank)))
            .collect()
    }

    /// Returns a [`NodeRanks`] assigning the same `rank` to every node.
    fn node_ranks_uniform<'id>(all_node_ids: &[NodeId<'id>], rank: u32) -> NodeRanks<'id> {
        all_node_ids
            .iter()
            .map(|node_id| (node_id.clone(), NodeRank::new(rank)))
            .collect()
    }

    /// Builds the node adjacency list from dependency edges, where each node is
    /// identified by its index into `all_node_ids`.
    ///
    /// `adjacency[from_idx]` lists the indices of nodes that depend on the node
    /// at `from_idx`. Edges whose endpoints are not in `all_node_ids` are
    /// ignored.
    fn ranks_compute_adjacency_build<'id>(
        all_node_ids: &[NodeId<'id>],
        dependency_edges: &[(NodeId<'id>, NodeId<'id>)],
    ) -> Vec<Vec<usize>> {
        let node_to_index: Map<NodeId<'id>, usize> = all_node_ids
            .iter()
            .enumerate()
            .map(|(node_idx, node_id)| (node_id.clone(), node_idx))
            .collect();

        dependency_edges.iter().fold(
            vec![Vec::new(); all_node_ids.len()],
            |mut adjacency, (from_id, to_id)| {
                if let (Some(&from_idx), Some(&to_idx)) =
                    (node_to_index.get(from_id), node_to_index.get(to_id))
                {
                    adjacency[from_idx].push(to_idx);
                }
                adjacency
            },
        )
    }

    /// Computes each node's rank from the node adjacency list.
    ///
    /// Cycles are contracted into strongly connected components (SCCs); the SCC
    /// DAG is then ranked by longest path so every node in a cycle shares the
    /// same rank. Returns `node_ranks[node_idx]` for each node index.
    fn ranks_compute_from_adjacency(adjacency: &[Vec<usize>], node_count: usize) -> Vec<u32> {
        // === Compute SCCs via Tarjan's Algorithm === //
        // `scc_ids[node_index]` is the SCC id of that node; nodes in the same
        // SCC share an id.
        let scc_ids = Self::tarjan_scc(adjacency, node_count);
        let scc_count = scc_ids.iter().copied().max().map(|m| m + 1).unwrap_or(0);

        if scc_count == 0 {
            return vec![0; node_count];
        }

        // === Rank the contracted SCC DAG (longest path), map back to nodes === //
        let scc_dag = Self::scc_dag_build(adjacency, &scc_ids, scc_count);
        let scc_ranks =
            Self::scc_dag_ranks_compute(&scc_dag.adjacency, &scc_dag.in_degree, scc_count);

        scc_ids.iter().map(|&scc_id| scc_ranks[scc_id]).collect()
    }

    /// Contracts the node adjacency list into a DAG over SCC ids.
    ///
    /// For each SCC, collects its outgoing SCC neighbours (excluding self) and
    /// the per-SCC in-degree, both used to rank the DAG topologically.
    fn scc_dag_build(adjacency: &[Vec<usize>], scc_ids: &[usize], scc_count: usize) -> SccDag {
        // Cross-indexed accumulation (`scc_adjacency[from_scc].push(to_scc)`),
        // so kept as a loop for clarity.
        let mut scc_adjacency: Vec<Vec<usize>> = vec![Vec::new(); scc_count];
        for (from_idx, to_indices) in adjacency.iter().enumerate() {
            let from_scc = scc_ids[from_idx];
            for &to_idx in to_indices {
                let to_scc = scc_ids[to_idx];
                if from_scc != to_scc {
                    scc_adjacency[from_scc].push(to_scc);
                }
            }
        }

        // Deduplicate SCC edges so each predecessor counts once.
        scc_adjacency.iter_mut().for_each(|neighbours| {
            neighbours.sort_unstable();
            neighbours.dedup();
        });

        let mut in_degree: Vec<usize> = vec![0; scc_count];
        scc_adjacency.iter().for_each(|neighbours| {
            neighbours.iter().for_each(|&to_scc| in_degree[to_scc] += 1);
        });

        SccDag {
            adjacency: scc_adjacency,
            in_degree,
        }
    }

    /// Tarjan's strongly connected components algorithm.
    ///
    /// Returns a vector where `result[node_index]` is the SCC id that node
    /// belongs to. Nodes in the same SCC share the same id.
    fn tarjan_scc(adjacency: &[Vec<usize>], node_count: usize) -> Vec<usize> {
        let mut state = TarjanState {
            index_counter: 0,
            stack: Vec::new(),
            on_stack: vec![false; node_count],
            index: vec![None; node_count],
            lowlink: vec![0; node_count],
            scc_ids: vec![0; node_count],
            scc_counter: 0,
        };

        // Iterative Tarjan to avoid stack overflow on large graphs.
        for node in 0..node_count {
            if state.index[node].is_none() {
                Self::tarjan_strongconnect_iterative(adjacency, node, &mut state);
            }
        }

        state.scc_ids
    }

    /// Iterative version of Tarjan's strongconnect to avoid deep recursion.
    ///
    /// Uses an explicit call stack to simulate the recursive DFS.
    fn tarjan_strongconnect_iterative(
        adjacency: &[Vec<usize>],
        start: usize,
        state: &mut TarjanState,
    ) {
        // Frame for our explicit stack: (node, neighbour_index).
        // `neighbour_index` tracks which neighbour we should process next.
        let mut call_stack: Vec<(usize, usize)> = Vec::new();

        // "Call" start node.
        state.index[start] = Some(state.index_counter);
        state.lowlink[start] = state.index_counter;
        state.index_counter += 1;
        state.stack.push(start);
        state.on_stack[start] = true;
        call_stack.push((start, 0));

        while let Some(&mut (v, ref mut ni)) = call_stack.last_mut() {
            if *ni < adjacency[v].len() {
                let w = adjacency[v][*ni];
                *ni += 1;

                if state.index[w].is_none() {
                    // w has not been visited; "recurse" on w.
                    state.index[w] = Some(state.index_counter);
                    state.lowlink[w] = state.index_counter;
                    state.index_counter += 1;
                    state.stack.push(w);
                    state.on_stack[w] = true;
                    call_stack.push((w, 0));
                } else if state.on_stack[w] {
                    // w is on the stack and hence in the current SCC.
                    let w_index = state.index[w].unwrap();
                    if w_index < state.lowlink[v] {
                        state.lowlink[v] = w_index;
                    }
                }
            } else {
                // All neighbours of v have been processed.
                // Check if v is a root node of an SCC.
                if state.lowlink[v] == state.index[v].unwrap() {
                    // Pop all nodes of this SCC from the stack.
                    let scc_id = state.scc_counter;
                    state.scc_counter += 1;
                    while let Some(w) = state.stack.pop() {
                        state.on_stack[w] = false;
                        state.scc_ids[w] = scc_id;
                        if w == v {
                            break;
                        }
                    }
                }

                // "Return" from v: propagate lowlink to caller.
                call_stack.pop();
                if let Some(&mut (caller, _)) = call_stack.last_mut()
                    && state.lowlink[v] < state.lowlink[caller]
                {
                    state.lowlink[caller] = state.lowlink[v];
                }
            }
        }
    }

    /// Computes ranks on the SCC DAG using a topological sort (Kahn's
    /// algorithm) combined with longest-path computation.
    ///
    /// Each SCC's rank is `max(rank(predecessor) + 1)` for all predecessors
    /// in the DAG, or `0` if it has no predecessors.
    fn scc_dag_ranks_compute(
        scc_adjacency: &[Vec<usize>],
        scc_in_degree: &[usize],
        scc_count: usize,
    ) -> Vec<u32> {
        let mut ranks: Vec<u32> = vec![0; scc_count];
        let mut in_degree = scc_in_degree.to_vec();

        // Seed the queue with all SCCs that have no incoming edges.
        let mut queue: std::collections::VecDeque<usize> = std::collections::VecDeque::new();
        in_degree
            .iter()
            .copied()
            .enumerate()
            .take(scc_count)
            .filter(|(_scc_idx, in_degree_item)| *in_degree_item == 0)
            .for_each(|(scc_idx, _in_degree_item)| queue.push_back(scc_idx));

        // Kahn's algorithm: drain the queue, relaxing successor ranks and
        // enqueuing SCCs as their in-degree reaches zero. Kept as a `while let`
        // loop -- the queue is mutated while iterating, so it is not a fold.
        while let Some(scc_idx) = queue.pop_front() {
            for &to_scc in &scc_adjacency[scc_idx] {
                let candidate_rank = ranks[scc_idx] + 1;
                if candidate_rank > ranks[to_scc] {
                    ranks[to_scc] = candidate_rank;
                }
                in_degree[to_scc] -= 1;
                if in_degree[to_scc] == 0 {
                    queue.push_back(to_scc);
                }
            }
        }

        ranks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranks_from_adjacency_linear_chain_increments() {
        // 0 -> 1 -> 2
        let adjacency = vec![vec![1], vec![2], vec![]];
        let node_ranks = NodeRanksCalculator::ranks_compute_from_adjacency(&adjacency, 3);
        assert_eq!(vec![0, 1, 2], node_ranks);
    }

    #[test]
    fn ranks_from_adjacency_cycle_shares_rank() {
        // 0 -> 1 -> 2 -> 0 (single SCC)
        let adjacency = vec![vec![1], vec![2], vec![0]];
        let node_ranks = NodeRanksCalculator::ranks_compute_from_adjacency(&adjacency, 3);
        assert_eq!(vec![0, 0, 0], node_ranks);
    }

    #[test]
    fn ranks_from_adjacency_diamond_uses_longest_path() {
        // 0 -> 1, 0 -> 2, 1 -> 3, 2 -> 3
        let adjacency = vec![vec![1, 2], vec![3], vec![3], vec![]];
        let node_ranks = NodeRanksCalculator::ranks_compute_from_adjacency(&adjacency, 4);
        assert_eq!(vec![0, 1, 1, 2], node_ranks);
    }

    #[test]
    fn ranks_from_adjacency_contracts_cycle_then_continues() {
        // 0 <-> 1 (cycle), 1 -> 2: SCC {0,1} ranks 0, node 2 rank 1.
        let adjacency = vec![vec![1], vec![0, 2], vec![]];
        let node_ranks = NodeRanksCalculator::ranks_compute_from_adjacency(&adjacency, 3);
        assert_eq!(vec![0, 0, 1], node_ranks);
    }

    #[test]
    fn ranks_from_adjacency_no_edges_all_zero() {
        let adjacency = vec![vec![], vec![], vec![]];
        let node_ranks = NodeRanksCalculator::ranks_compute_from_adjacency(&adjacency, 3);
        assert_eq!(vec![0, 0, 0], node_ranks);
    }
}
