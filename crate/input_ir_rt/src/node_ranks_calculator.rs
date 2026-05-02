use std::collections::HashMap;

use disposition_ir_model::{
    edge::EdgeGroups,
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
/// computation.
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
    pub fn calculate<'id>(
        edge_groups: &EdgeGroups<'id>,
        entity_types: &EntityTypes<'id>,
        node_nesting_infos: &NodeNestingInfos<'id>,
    ) -> NodeRanksNested<'id> {
        if node_nesting_infos.is_empty() {
            return NodeRanksNested::new();
        }

        // === Build Container-to-Children Map === //
        let container_to_children = Self::container_to_children_build(node_nesting_infos);

        // === Collect Dependency Edges === //
        let dependency_edges = Self::dependency_edges_collect(edge_groups, entity_types);

        // === Lift Edges to LCA Level === //
        let lca_level_edges = Self::lca_level_edges_build(&dependency_edges, node_nesting_infos);

        // === Compute Ranks Per Level === //
        let mut root = NodeRanks::new();
        let mut containers: Map<NodeId<'id>, NodeRanks<'id>> = Map::new();

        let empty_edges: Vec<(NodeId<'id>, NodeId<'id>)> = Vec::new();
        for (container, children) in &container_to_children {
            let edges = lca_level_edges.get(container).unwrap_or(&empty_edges);
            let ranks = Self::ranks_compute(children, edges);
            match container {
                None => root = ranks,
                Some(container_id) => {
                    containers.insert(container_id.clone(), ranks);
                }
            }
        }

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
    ) -> HashMap<Option<NodeId<'id>>, Vec<NodeId<'id>>> {
        let mut container_to_children: HashMap<Option<NodeId<'id>>, Vec<NodeId<'id>>> =
            HashMap::new();
        for (node_id, nesting_info) in node_nesting_infos.iter() {
            let chain = &nesting_info.ancestor_chain;
            let parent = chain
                .len()
                .checked_sub(2)
                .map(|parent_idx| chain[parent_idx].clone());
            container_to_children
                .entry(parent)
                .or_default()
                .push(node_id.clone());
        }
        container_to_children
    }

    /// Lifts each dependency edge to the LCA-level edge between the divergent
    /// sibling ancestors of the two endpoints.
    ///
    /// Groups resulting LCA-level edges by their LCA container (`None` for
    /// root).
    fn lca_level_edges_build<'id>(
        dependency_edges: &[(NodeId<'id>, NodeId<'id>)],
        node_nesting_infos: &NodeNestingInfos<'id>,
    ) -> HashMap<Option<NodeId<'id>>, Vec<(NodeId<'id>, NodeId<'id>)>> {
        let mut lca_level_edges: HashMap<Option<NodeId<'id>>, Vec<(NodeId<'id>, NodeId<'id>)>> =
            HashMap::new();
        for (from_id, to_id) in dependency_edges {
            if let Some((lca_container, divergent_from, divergent_to)) =
                Self::lca_level_edge_compute(from_id, to_id, node_nesting_infos)
            {
                lca_level_edges
                    .entry(lca_container)
                    .or_default()
                    .push((divergent_from, divergent_to));
            }
        }
        lca_level_edges
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

    /// Extracts dependency edges from edge groups, filtering out interaction
    /// edges.
    ///
    /// Returns a list of `(from_id, to_id)` pairs for dependency edges only.
    fn dependency_edges_collect<'id>(
        edge_groups: &EdgeGroups<'id>,
        entity_types: &EntityTypes<'id>,
    ) -> Vec<(NodeId<'id>, NodeId<'id>)> {
        let mut dependency_edges = Vec::new();

        for (edge_group_id, edge_group) in edge_groups.iter() {
            let edge_group_id: &Id = edge_group_id.as_ref();

            // Check if this edge group is a dependency type.
            let is_dependency = entity_types
                .get(edge_group_id)
                .map(|types| types.iter().any(Self::entity_type_is_dependency_edge_group))
                .unwrap_or(false);

            if !is_dependency {
                continue;
            }

            for edge in edge_group.iter() {
                // Skip self-loops -- they don't affect rank.
                if edge.from == edge.to {
                    continue;
                }

                dependency_edges.push((edge.from.clone(), edge.to.clone()));
            }
        }

        dependency_edges
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
            return all_node_ids
                .iter()
                .map(|node_id| (node_id.clone(), NodeRank::new(0)))
                .collect();
        }

        // === Assign Each Node a Numeric Index === //
        let mut node_to_index: Map<NodeId<'id>, usize> = Map::new();
        for (i, node_id) in all_node_ids.iter().enumerate() {
            node_to_index.insert(node_id.clone(), i);
        }
        let node_count = all_node_ids.len();

        // === Build Adjacency List === //
        let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); node_count];
        for (from_id, to_id) in dependency_edges {
            if let (Some(&from_idx), Some(&to_idx)) =
                (node_to_index.get(from_id), node_to_index.get(to_id))
            {
                adjacency[from_idx].push(to_idx);
            }
        }

        // === Compute SCCs via Tarjan's Algorithm === //
        let scc_ids = Self::tarjan_scc(&adjacency, node_count);

        // scc_ids[node_index] = scc component id
        // Nodes in the same SCC share the same scc_ids value.

        let scc_count = scc_ids.iter().copied().max().map(|m| m + 1).unwrap_or(0);

        if scc_count == 0 {
            return all_node_ids
                .iter()
                .map(|node_id| (node_id.clone(), NodeRank::new(0)))
                .collect();
        }

        // === Build SCC DAG === //
        // For each SCC, collect its outgoing SCC neighbours (excluding self).
        let mut scc_adjacency: Vec<Vec<usize>> = vec![Vec::new(); scc_count];
        let mut scc_in_degree: Vec<usize> = vec![0; scc_count];

        for from_idx in 0..node_count {
            let from_scc = scc_ids[from_idx];
            for &to_idx in &adjacency[from_idx] {
                let to_scc = scc_ids[to_idx];
                if from_scc != to_scc {
                    scc_adjacency[from_scc].push(to_scc);
                }
            }
        }

        // Deduplicate SCC edges and compute in-degrees.
        for neighbours in &mut scc_adjacency {
            neighbours.sort_unstable();
            neighbours.dedup();
        }
        // Recompute in-degrees after dedup.
        scc_in_degree.fill(0);
        scc_adjacency.iter().take(scc_count).for_each(|scc_idx| {
            scc_idx.iter().copied().for_each(|to_scc| {
                scc_in_degree[to_scc] += 1;
            });
        });

        // === Compute Ranks on SCC DAG (Longest Path / Topological Order) === //
        let scc_ranks = Self::scc_dag_ranks_compute(&scc_adjacency, &scc_in_degree, scc_count);

        // === Map SCC Ranks Back to Node Ranks === //
        all_node_ids
            .iter()
            .enumerate()
            .map(|(node_idx, node_id)| {
                let scc_id = scc_ids[node_idx];
                let rank = scc_ranks[scc_id];
                (node_id.clone(), NodeRank::new(rank))
            })
            .collect()
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
