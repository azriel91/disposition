use disposition_ir_model::{
    edge::EdgeGroups,
    entity::{EntityType, EntityTypes},
    node::{NodeHierarchy, NodeId, NodeRank, NodeRanks},
};
use disposition_model_common::{Id, Map};

/// Computes [`NodeRanks`] from dependency edges in an [`IrDiagram`].
///
/// Dependency edges indicate that the `to` node should be positioned after
/// the `from` node. This calculator performs a rank assignment so that:
///
/// * Nodes with no incoming dependency edges receive rank `0`.
/// * Each node's rank is one greater than the maximum rank of its predecessors.
/// * Nodes that are part of a cycle (strongly connected component) share the
///   same rank -- the cycle is contracted into a single logical node for
///   ranking purposes.
///
/// Only **dependency** edges (not interaction edges) are considered for rank
/// computation.
///
/// [`IrDiagram`]: disposition_ir_model::IrDiagram
///
/// # Examples
///
/// Given edges `A -> B -> C`, the resulting ranks would be:
///
/// * `A`: 0
/// * `B`: 1
/// * `C`: 2
///
/// Given edges `A -> B`, `B -> A`, `B -> C`, the resulting ranks would be:
///
/// * `A`: 0 (part of cycle with B)
/// * `B`: 0 (part of cycle with A)
/// * `C`: 1
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
    /// Computes node ranks from the given edge groups and entity types.
    ///
    /// Only edges whose edge group has a dependency entity type are used for
    /// rank computation. All nodes present in `node_hierarchy` are included
    /// in the output, defaulting to rank `0` if they have no incoming
    /// dependency edges.
    ///
    /// # Parameters
    ///
    /// * `node_hierarchy`: The full node hierarchy -- used to discover all node
    ///   IDs that should receive a rank.
    /// * `edge_groups`: All edge groups in the diagram.
    /// * `entity_types`: Entity types used to distinguish dependency edges from
    ///   interaction edges.
    pub fn calculate<'id>(
        node_hierarchy: &NodeHierarchy<'id>,
        edge_groups: &EdgeGroups<'id>,
        entity_types: &EntityTypes<'id>,
    ) -> NodeRanks<'id> {
        // === Collect All Node IDs === //
        let mut all_node_ids: Vec<NodeId<'id>> = Vec::new();
        Self::node_ids_collect(node_hierarchy, &mut all_node_ids);

        if all_node_ids.is_empty() {
            return NodeRanks::new();
        }

        // === Collect Dependency Edges === //
        let dependency_edges = Self::dependency_edges_collect(edge_groups, entity_types);

        if dependency_edges.is_empty() {
            // No dependency edges -- all nodes get rank 0.
            return all_node_ids
                .into_iter()
                .map(|node_id| (node_id, NodeRank::new(0)))
                .collect();
        }

        // === Compute Ranks via SCC Contraction === //
        Self::ranks_compute(&all_node_ids, &dependency_edges)
    }

    /// Recursively collects all node IDs from a `NodeHierarchy`.
    fn node_ids_collect<'id>(node_hierarchy: &NodeHierarchy<'id>, node_ids: &mut Vec<NodeId<'id>>) {
        for (node_id, child_hierarchy) in node_hierarchy.iter() {
            node_ids.push(node_id.clone());
            Self::node_ids_collect(child_hierarchy, node_ids);
        }
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
                .map(|types| {
                    types
                        .iter()
                        .any(|entity_type| Self::entity_type_is_dependency_edge_group(entity_type))
                })
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
        for scc_idx in 0..scc_count {
            for &to_scc in &scc_adjacency[scc_idx] {
                scc_in_degree[to_scc] += 1;
            }
        }

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
                if let Some(&mut (caller, _)) = call_stack.last_mut() {
                    if state.lowlink[v] < state.lowlink[caller] {
                        state.lowlink[caller] = state.lowlink[v];
                    }
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
        for scc_idx in 0..scc_count {
            if in_degree[scc_idx] == 0 {
                queue.push_back(scc_idx);
            }
        }

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
