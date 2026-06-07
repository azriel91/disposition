use disposition_ir_model::{edge::EdgeId, node::NodeRank};
use disposition_model_common::RankDir;
use disposition_taffy_model::{taffy::TaffyTree, EdgeIdToEdgeSpacerTaffyNodes, TaffyNodeCtx};

use crate::taffy_to_svg_elements_mapper::{
    edge_path_builder_pass_1::SpacerCoordinates, EdgeSpacerCoordinatesCalculator,
};

/// Resolves the spacer coordinates an edge passes through, in visual order.
///
/// An edge may have spacer taffy nodes of three kinds:
///
/// * Rank-based spacers (`rank_to_spacer_taffy_node_id`), inserted at
///   intermediate ranks the edge crosses.
/// * Cross-container spacers (`cross_container_spacer_taffy_node_ids`),
///   inserted alongside sibling containers for edges that cross container
///   boundaries.
/// * Edge-description-container spacers
///   (`edge_desc_container_spacer_taffy_node_ids`).
///
/// Each resolved [`SpacerCoordinates`] has an entry point and an exit point
/// that slice the spacer in half, so the edge path passes straight through the
/// spacer area.
///
/// This logic is shared by the edge path builder and the ortho protrusion
/// calculator so they agree on the spacer ordering for every edge.
pub struct SpacerCoordinatesResolver;

impl SpacerCoordinatesResolver {
    /// Resolves the spacer coordinates an edge passes through, sorted into the
    /// order they appear along the edge path.
    ///
    /// When the edge only has rank-based spacers, they are ordered by
    /// [`NodeRank`]. When cross-container or edge-description spacers are also
    /// present, all spacers are merged and sorted by their absolute coordinate
    /// along the main (rank) axis.
    ///
    /// Returns an empty `Vec` when the edge has no spacer nodes.
    pub fn resolve<'id>(
        rank_dir: RankDir,
        edge_id: &EdgeId<'id>,
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        edge_spacer_taffy_nodes: &EdgeIdToEdgeSpacerTaffyNodes<'id>,
    ) -> Vec<SpacerCoordinates> {
        let Some(spacer_nodes) = edge_spacer_taffy_nodes.get(edge_id) else {
            return Vec::new();
        };

        let rank_spacers: Vec<(NodeRank, SpacerCoordinates)> = spacer_nodes
            .rank_to_spacer_taffy_node_id
            .iter()
            .filter_map(|(rank, &taffy_node_id)| {
                let spacer_coordinates = EdgeSpacerCoordinatesCalculator::calculate(
                    rank_dir,
                    taffy_tree,
                    taffy_node_id,
                )?;
                Some((*rank, spacer_coordinates))
            })
            .collect();

        let cross_container_spacers = Self::spacers_calculate(
            rank_dir,
            taffy_tree,
            &spacer_nodes.cross_container_spacer_taffy_node_ids,
        );
        let edge_desc_container_spacers = Self::spacers_calculate(
            rank_dir,
            taffy_tree,
            &spacer_nodes.edge_desc_container_spacer_taffy_node_ids,
        );

        if cross_container_spacers.is_empty() && edge_desc_container_spacers.is_empty() {
            // Fast path: only rank-based spacers -- sort by rank as before.
            return Self::rank_spacers_sort_by_rank(rank_spacers);
        }

        // Merge all kinds and sort by absolute coordinate along the main axis
        // so the spacers appear in the correct visual order along the edge
        // path.
        let all_spacers = rank_spacers
            .into_iter()
            .map(|(_rank, spacer_coordinates)| spacer_coordinates)
            .chain(cross_container_spacers)
            .chain(edge_desc_container_spacers)
            .collect();

        Self::spacers_sort_by_main_axis(rank_dir, all_spacers)
    }

    /// Calculates spacer coordinates for a slice of spacer taffy node IDs,
    /// dropping any whose layout cannot be resolved.
    fn spacers_calculate(
        rank_dir: RankDir,
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        spacer_taffy_node_ids: &[taffy::NodeId],
    ) -> Vec<SpacerCoordinates> {
        spacer_taffy_node_ids
            .iter()
            .filter_map(|&taffy_node_id| {
                EdgeSpacerCoordinatesCalculator::calculate(rank_dir, taffy_tree, taffy_node_id)
            })
            .collect()
    }

    /// Sorts rank-based spacers by [`NodeRank`] and discards the ranks.
    fn rank_spacers_sort_by_rank(
        mut rank_spacers: Vec<(NodeRank, SpacerCoordinates)>,
    ) -> Vec<SpacerCoordinates> {
        rank_spacers.sort_by_key(|(rank, _spacer_coordinates)| *rank);
        rank_spacers
            .into_iter()
            .map(|(_rank, spacer_coordinates)| spacer_coordinates)
            .collect()
    }

    /// Sorts merged spacers by their absolute coordinate along the main (rank)
    /// axis -- entry Y for vertical flow, entry X for horizontal flow.
    fn spacers_sort_by_main_axis(
        rank_dir: RankDir,
        mut all_spacers: Vec<SpacerCoordinates>,
    ) -> Vec<SpacerCoordinates> {
        let main_axis_key = |spacer_coordinates: &SpacerCoordinates| match rank_dir {
            RankDir::TopToBottom | RankDir::BottomToTop => spacer_coordinates.entry_y,
            RankDir::LeftToRight | RankDir::RightToLeft => spacer_coordinates.entry_x,
        };
        all_spacers.sort_by(|a, b| {
            main_axis_key(a)
                .partial_cmp(&main_axis_key(b))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        all_spacers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a `SpacerCoordinates` with the given entry point. Exit point is
    /// mirrored so each fixture is distinguishable.
    fn spacer(entry_x: f32, entry_y: f32) -> SpacerCoordinates {
        SpacerCoordinates {
            entry_x,
            entry_y,
            exit_x: entry_x,
            exit_y: entry_y,
        }
    }

    #[test]
    fn rank_spacers_sort_by_rank_orders_by_rank_and_drops_ranks() {
        let rank_spacers = vec![
            (NodeRank::new(2), spacer(2.0, 20.0)),
            (NodeRank::new(0), spacer(0.0, 0.0)),
            (NodeRank::new(1), spacer(1.0, 10.0)),
        ];

        let sorted = SpacerCoordinatesResolver::rank_spacers_sort_by_rank(rank_spacers);

        let entry_ys: Vec<f32> = sorted.iter().map(|spacer| spacer.entry_y).collect();
        assert_eq!(vec![0.0, 10.0, 20.0], entry_ys);
    }

    #[test]
    fn spacers_sort_by_main_axis_uses_entry_y_for_vertical_flow() {
        // Merge rank-based, cross-container, and edge-description spacers
        // out of order; vertical flow sorts by entry_y.
        let all_spacers = vec![spacer(99.0, 30.0), spacer(99.0, 10.0), spacer(99.0, 20.0)];

        let sorted =
            SpacerCoordinatesResolver::spacers_sort_by_main_axis(RankDir::TopToBottom, all_spacers);

        let entry_ys: Vec<f32> = sorted.iter().map(|spacer| spacer.entry_y).collect();
        assert_eq!(vec![10.0, 20.0, 30.0], entry_ys);
    }

    #[test]
    fn spacers_sort_by_main_axis_uses_entry_x_for_horizontal_flow() {
        let all_spacers = vec![spacer(30.0, 99.0), spacer(10.0, 99.0), spacer(20.0, 99.0)];

        let sorted =
            SpacerCoordinatesResolver::spacers_sort_by_main_axis(RankDir::LeftToRight, all_spacers);

        let entry_xs: Vec<f32> = sorted.iter().map(|spacer| spacer.entry_x).collect();
        assert_eq!(vec![10.0, 20.0, 30.0], entry_xs);
    }
}
