use std::cmp::Ordering;

use disposition_ir_model::{
    edge::{Edge, EdgeFaceAssignment, EdgeFaceAssignments, EdgeGroups},
    entity::EntityTypes,
    node::{NodeFace, NodeId, NodeNestingInfos, NodeRank, NodeRanksNested},
};
use disposition_model_common::RankDir;

use crate::EdgeIdGenerator;

/// Computes pre-layout [`EdgeFaceAssignment`]s for every edge in the diagram.
///
/// Uses rank and sibling data from [`NodeNestingInfos`] and
/// [`NodeRanksNested`] to determine which face of each endpoint node an edge
/// exits or enters, without reading post-layout pixel coordinates.
///
/// The result is stored in [`EdgeFaceAssignments`] and used to build envelope
/// taffy nodes with the right number of edge label slots per face.
///
/// # Assignment rules (applied in priority order)
///
/// | Case                                           | from_face                  | to_face       |
/// |------------------------------------------------|----------------------------|---------------|
/// | Self-loop (`from == to`)                       | `Bottom`                   | `None`        |
/// | Contained (from is ancestor of to)             | rank-dir face              | opposite      |
/// | Contained (to is ancestor of from)             | opposite                   | rank-dir face |
/// | Cycle edge adjacent siblings (same LCA rank)   | rank-dir face              | opposite      |
/// | Cycle edge (same LCA rank)                     | clockwise by sibling index | clockwise     |
/// | Forward edge (`lca_rank_from < lca_rank_to`)   | rank-dir face              | opposite      |
/// | Reverse edge (`lca_rank_from > lca_rank_to`)   | opposite                   | rank-dir face |
///
/// **Rank-direction face** for a forward edge's `from` node:
///
/// | `RankDir`     | `from_face` | `to_face` |
/// |---------------|-------------|-----------|
/// | `LeftToRight` | `Right`     | `Left`    |
/// | `RightToLeft` | `Left`      | `Right`   |
/// | `TopToBottom` | `Bottom`    | `Top`     |
/// | `BottomToTop` | `Top`       | `Bottom`  |
///
/// **Clockwise cycle face** by sibling index at the LCA level:
///
/// | `RankDir`                     | `from_sibling < to_sibling` | `from_sibling > to_sibling` |
/// |-------------------------------|-----------------------------|-----------------------------|
/// | `LeftToRight` / `RightToLeft` | `(Right, Right)`            | `(Left, Left)`              |
/// | `TopToBottom` / `BottomToTop` | `(Top, Top)`                | `(Bottom, Bottom)`          |
#[derive(Clone, Copy, Debug)]
pub struct EdgeFaceAssigner;

/// LCA information for a pair of nodes, used by [`EdgeFaceAssigner`].
struct LcaInfo<'id> {
    /// The divergent ancestor of the `from` node at the LCA level.
    divergent_from: NodeId<'id>,
    /// The divergent ancestor of the `to` node at the LCA level.
    divergent_to: NodeId<'id>,
    /// The LCA container node, or `None` for the root level.
    lca_container: Option<NodeId<'id>>,
    /// Sibling index of `divergent_from` within the LCA container.
    sibling_index_from: usize,
    /// Sibling index of `divergent_to` within the LCA container.
    sibling_index_to: usize,
}

impl EdgeFaceAssigner {
    /// Computes an [`EdgeFaceAssignment`] for every edge in `edge_groups`.
    ///
    /// `entity_types` is accepted for API consistency with similar calculators
    /// but is not used in the current implementation (all edges are assigned a
    /// face regardless of type).
    pub fn compute<'id>(
        edge_groups: &EdgeGroups<'id>,
        _entity_types: &EntityTypes<'id>,
        node_nesting_infos: &NodeNestingInfos<'id>,
        node_ranks_nested: &NodeRanksNested<'id>,
        rank_dir: RankDir,
    ) -> EdgeFaceAssignments<'id> {
        let mut assignments = EdgeFaceAssignments::new();

        for (edge_group_id, edge_group) in edge_groups.iter() {
            for (edge_index, edge) in edge_group.iter().enumerate() {
                let edge_id = EdgeIdGenerator::generate(edge_group_id, edge_index);
                let assignment = Self::edge_face_assignment_compute(
                    edge,
                    node_nesting_infos,
                    node_ranks_nested,
                    rank_dir,
                );
                assignments.insert(edge_id, assignment);
            }
        }

        assignments
    }

    /// Computes the [`EdgeFaceAssignment`] for a single edge.
    fn edge_face_assignment_compute<'id>(
        edge: &Edge<'id>,
        node_nesting_infos: &NodeNestingInfos<'id>,
        node_ranks_nested: &NodeRanksNested<'id>,
        rank_dir: RankDir,
    ) -> EdgeFaceAssignment {
        // Case 1: Self-loop -- the from face is Bottom; no to face is needed
        // since from == to, only one label slot is used.
        if edge.is_self_loop() {
            return EdgeFaceAssignment {
                from_face: Some(NodeFace::Bottom),
                to_face: None,
            };
        }

        // Compute LCA info; returns None for contained edges.
        let Some(lca_info) = Self::lca_info_compute(&edge.from, &edge.to, node_nesting_infos)
        else {
            // Case 2: Contained edge -- assign faces based on hierarchy direction.
            return Self::contained_edge_face_assignment(
                &edge.from,
                &edge.to,
                node_nesting_infos,
                rank_dir,
            );
        };

        // Get LCA-level ranks for both divergent ancestors.
        let (rank_from, rank_to) = Self::lca_ranks_compute(&lca_info, node_ranks_nested);

        let (from_face, to_face) = if rank_from == rank_to {
            // Case 3: Cycle edge -- same LCA rank.
            Self::cycle_faces(
                lca_info.sibling_index_from,
                lca_info.sibling_index_to,
                rank_dir,
            )
        } else if rank_from < rank_to {
            // Case 4: Forward edge.
            Self::forward_faces(rank_dir)
        } else {
            // Case 5: Reverse edge -- swap the forward faces.
            let (f, t) = Self::forward_faces(rank_dir);
            (t, f)
        };

        EdgeFaceAssignment {
            from_face: Some(from_face),
            to_face: Some(to_face),
        }
    }

    /// Computes LCA information for two nodes.
    ///
    /// Returns `None` when one node is an ancestor of the other (contained
    /// edge) or when either node is absent from `node_nesting_infos`.
    fn lca_info_compute<'id>(
        from_id: &NodeId<'id>,
        to_id: &NodeId<'id>,
        node_nesting_infos: &NodeNestingInfos<'id>,
    ) -> Option<LcaInfo<'id>> {
        let info_from = node_nesting_infos.get(from_id)?;
        let info_to = node_nesting_infos.get(to_id)?;

        let chain_from = &info_from.ancestor_chain;
        let chain_to = &info_to.ancestor_chain;

        // Length of the common ancestor prefix.
        let lca_depth = chain_from
            .iter()
            .zip(chain_to.iter())
            .take_while(|(a, b)| a == b)
            .count();

        // Contained edge: one chain is a prefix of the other.
        if lca_depth >= chain_from.len() || lca_depth >= chain_to.len() {
            return None;
        }

        let divergent_from = chain_from[lca_depth].clone();
        let divergent_to = chain_to[lca_depth].clone();

        // Degenerate self-loop at LCA level (should not occur for distinct nodes).
        if divergent_from == divergent_to {
            return None;
        }

        let lca_container = lca_depth.checked_sub(1).map(|i| chain_from[i].clone());

        let sibling_index_from = info_from.nesting_path[lca_depth];
        let sibling_index_to = info_to.nesting_path[lca_depth];

        Some(LcaInfo {
            divergent_from,
            divergent_to,
            lca_container,
            sibling_index_from,
            sibling_index_to,
        })
    }

    /// Returns the LCA-level ranks for the divergent ancestors in `lca_info`.
    fn lca_ranks_compute<'id>(
        lca_info: &LcaInfo<'id>,
        node_ranks_nested: &NodeRanksNested<'id>,
    ) -> (NodeRank, NodeRank) {
        let container_ranks = node_ranks_nested.ranks_for(lca_info.lca_container.as_ref());
        let rank_from = container_ranks
            .and_then(|ranks| ranks.get(&lca_info.divergent_from).copied())
            .unwrap_or_default();
        let rank_to = container_ranks
            .and_then(|ranks| ranks.get(&lca_info.divergent_to).copied())
            .unwrap_or_default();
        (rank_from, rank_to)
    }

    /// Returns the `(from_face, to_face)` pair for a forward edge in the given
    /// `rank_dir`.
    ///
    /// For a reverse edge, swap the returned tuple.
    fn forward_faces(rank_dir: RankDir) -> (NodeFace, NodeFace) {
        match rank_dir {
            RankDir::LeftToRight => (NodeFace::Right, NodeFace::Left),
            RankDir::RightToLeft => (NodeFace::Left, NodeFace::Right),
            RankDir::TopToBottom => (NodeFace::Bottom, NodeFace::Top),
            RankDir::BottomToTop => (NodeFace::Top, NodeFace::Bottom),
        }
    }

    /// Returns the clockwise `(from_face, to_face)` pair for a cycle edge.
    ///
    /// Uses sibling indices at the LCA level as a proxy for relative position
    /// along the non-rank axis, matching the geometry used by
    /// `EdgePathBuilderPass1::cycle_edge_faces_select`. Sibling order matches
    /// visual order for every `RankDir` (reversed directions reverse the
    /// taffy insertion order to compensate for their reversed flex direction,
    /// see `TaffyContainerBuilder::rank_taffy_ids_reverse_if_direction_reversed`),
    /// so only the rank axis matters:
    ///
    /// - `LeftToRight` / `RightToLeft`: siblings are stacked vertically; a
    ///   smaller sibling index means the node is higher on screen, so the edge
    ///   exits the `Right` face (arcs down the right side) or the `Left` face
    ///   (arcs up the left side).
    /// - `TopToBottom` / `BottomToTop`: siblings sit side-by-side horizontally;
    ///   a smaller sibling index means the node is to the left, so the edge
    ///   exits the `Top` face (arcs above) or the `Bottom` face (arcs below).
    ///
    /// | `RankDir` | `from_sibling < to_sibling` | else |
    /// |---|---|---|
    /// | `LeftToRight` / `RightToLeft` | `(Right, Right)` | `(Left, Left)` |
    /// | `TopToBottom` / `BottomToTop` | `(Top, Top)` | `(Bottom, Bottom)` |
    fn cycle_faces(
        sibling_index_from: usize,
        sibling_index_to: usize,
        rank_dir: RankDir,
    ) -> (NodeFace, NodeFace) {
        let sibling_index_from_cmp_to = sibling_index_from.cmp(&sibling_index_to);
        let sibling_index_abs_diff = sibling_index_from.abs_diff(sibling_index_to);
        match (rank_dir, sibling_index_from_cmp_to, sibling_index_abs_diff) {
            // 3a: adjacent siblings don't use clockwise edges
            (RankDir::LeftToRight | RankDir::RightToLeft, Ordering::Less, 1) => {
                (NodeFace::Bottom, NodeFace::Top)
            }
            (
                RankDir::LeftToRight | RankDir::RightToLeft,
                Ordering::Equal | Ordering::Greater,
                1,
            ) => (NodeFace::Top, NodeFace::Bottom),
            (RankDir::TopToBottom | RankDir::BottomToTop, Ordering::Less, 1) => {
                (NodeFace::Right, NodeFace::Left)
            }
            (
                RankDir::TopToBottom | RankDir::BottomToTop,
                Ordering::Equal | Ordering::Greater,
                1,
            ) => (NodeFace::Left, NodeFace::Right),

            // 3b: non-adjacent siblings use clockwise edges
            (RankDir::LeftToRight | RankDir::RightToLeft, Ordering::Less, _) => {
                (NodeFace::Right, NodeFace::Right)
            }
            (
                RankDir::LeftToRight | RankDir::RightToLeft,
                Ordering::Equal | Ordering::Greater,
                _,
            ) => (NodeFace::Left, NodeFace::Left),
            (RankDir::TopToBottom | RankDir::BottomToTop, Ordering::Less, _) => {
                (NodeFace::Top, NodeFace::Top)
            }
            (
                RankDir::TopToBottom | RankDir::BottomToTop,
                Ordering::Equal | Ordering::Greater,
                _,
            ) => (NodeFace::Bottom, NodeFace::Bottom),
        }
    }

    /// Computes the [`EdgeFaceAssignment`] for a contained edge (one endpoint
    /// is an ancestor of the other).
    ///
    /// Assigns faces based on the hierarchy direction:
    /// - If `from` is an ancestor of `to` (downward edge), uses forward faces
    ///   matching the rank direction.
    /// - If `to` is an ancestor of `from` (upward edge), uses the reverse of
    ///   the forward faces.
    ///
    /// Falls back to `EdgeFaceAssignment::default()` (`None`, `None`) when
    /// either node is absent from `node_nesting_infos`.
    fn contained_edge_face_assignment<'id>(
        from_id: &NodeId<'id>,
        to_id: &NodeId<'id>,
        node_nesting_infos: &NodeNestingInfos<'id>,
        rank_dir: RankDir,
    ) -> EdgeFaceAssignment {
        let Some(info_from) = node_nesting_infos.get(from_id) else {
            return EdgeFaceAssignment::default();
        };
        let Some(info_to) = node_nesting_infos.get(to_id) else {
            return EdgeFaceAssignment::default();
        };

        let chain_from = &info_from.ancestor_chain;
        let chain_to = &info_to.ancestor_chain;

        let lca_depth = chain_from
            .iter()
            .zip(chain_to.iter())
            .take_while(|(a, b)| a == b)
            .count();

        let (from_face, to_face) = if lca_depth >= chain_from.len() {
            // `from` is an ancestor of `to` (downward edge) -- use forward faces.
            Self::forward_faces(rank_dir)
        } else {
            // `to` is an ancestor of `from` (upward edge) -- use reverse faces.
            let (f, t) = Self::forward_faces(rank_dir);
            (t, f)
        };

        EdgeFaceAssignment {
            from_face: Some(from_face),
            to_face: Some(to_face),
        }
    }
}
