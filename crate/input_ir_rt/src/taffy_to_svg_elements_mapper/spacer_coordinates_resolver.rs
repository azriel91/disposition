use disposition_ir_model::{edge::EdgeId, node::NodeRank};
use disposition_model_common::RankDir;
use disposition_taffy_model::{
    taffy::TaffyTree, EdgeIdToEdgeDescriptionTaffyNodes, EdgeIdToEdgeSpacerTaffyNodes,
    EdgeSpacerTaffyNodes, TaffyNodeCtx,
};

use crate::taffy_to_svg_elements_mapper::{
    edge_path_builder_pass_1::SpacerCoordinates, EdgeSpacerCoordinatesCalculator,
};

/// Resolves the spacer coordinates an edge passes through, in visual order.
///
/// An edge may have spacer taffy nodes of four kinds:
///
/// * Rank-based spacers (`rank_to_spacer_taffy_node_id`), inserted at
///   intermediate ranks the edge crosses.
/// * Cross-container spacers (`cross_container_spacer_taffy_node_ids`),
///   inserted alongside sibling containers for edges that cross container
///   boundaries.
/// * Edge-description-container spacers
///   (`edge_desc_container_spacer_taffy_node_ids`).
/// * The edge's own description contact -- read directly from the
///   `edge_description_container` leaf's own resolved rect (not a spacer leaf
///   at all), via [`Self::description_contact_resolve`].
///
/// Each resolved [`SpacerCoordinates`] for the first three kinds has an entry
/// point and an exit point that slice the spacer in half, so the edge path
/// passes straight through the spacer area. The description contact is the
/// exception: entry and exit are the same point, since the description box
/// sits beside the edge's path rather than on a corridor it threads through
/// (see [`Self::description_contact_resolve`]).
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
        edge_description_taffy_nodes: &EdgeIdToEdgeDescriptionTaffyNodes<'id>,
    ) -> Vec<SpacerCoordinates> {
        // An edge whose only waypoint is its own description contact has no
        // entry in `edge_spacer_taffy_nodes` at all, so this must not early
        // return before the description contact is considered.
        let default_spacer_nodes = EdgeSpacerTaffyNodes::new();
        let spacer_nodes = edge_spacer_taffy_nodes
            .get(edge_id)
            .unwrap_or(&default_spacer_nodes);

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

        let mut cross_container_spacers = Self::spacers_calculate(
            rank_dir,
            taffy_tree,
            &spacer_nodes.cross_container_spacer_taffy_node_ids,
        );
        Self::cross_container_spacers_snap_to_column(rank_dir, &mut cross_container_spacers);
        let edge_desc_container_spacers = Self::spacers_calculate(
            rank_dir,
            taffy_tree,
            &spacer_nodes.edge_desc_container_spacer_taffy_node_ids,
        );
        // Text-content spacers are deliberately **not** snapped onto the
        // cross-container column: each is a local waypoint at its node's text
        // band so the edge only bows around that label, rather than having its
        // whole descent column pulled onto the text's far side.
        let text_content_spacers = Self::spacers_calculate(
            rank_dir,
            taffy_tree,
            &spacer_nodes.text_content_spacer_taffy_node_ids,
        );
        let description_contact = Self::description_contact_resolve(
            rank_dir,
            edge_id,
            taffy_tree,
            edge_description_taffy_nodes,
        );

        if cross_container_spacers.is_empty()
            && edge_desc_container_spacers.is_empty()
            && text_content_spacers.is_empty()
            && description_contact.is_none()
        {
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
            .chain(text_content_spacers)
            .chain(description_contact)
            .collect();

        Self::spacers_sort_by_main_axis(rank_dir, all_spacers)
    }

    /// Resolves the waypoint for an edge's own `edge_description_container`
    /// leaf, read directly from the description box's own post-layout
    /// absolute rect (not a decoupled spacer leaf).
    ///
    /// Returns `None` when the edge has no description (absent from
    /// `edge_description_taffy_nodes`) or its layout cannot be resolved.
    ///
    /// Unlike every other spacer kind, this waypoint applies
    /// **unconditionally**, regardless of `EdgeCurvature::is_direct()`: it is
    /// folded into [`Self::resolve`]'s merged list (consumed unconditionally
    /// by `Curved`/`Orthogonal` routing), and is also passed separately into
    /// `EdgePathBuilderPass2::build`'s `description_contact` parameter, which
    /// the `DirectStraight`/`DirectCurved` arms consult directly since they
    /// otherwise ignore `resolve`'s output entirely.
    ///
    /// The description box is not a corridor the path threads through, so the
    /// entry and exit point are the same: see
    /// `EdgeSpacerCoordinatesCalculator::calculate_description_contact` for
    /// how the single waypoint is chosen (a fixed side of the box, biased
    /// along the other axis by the edge's `from`/`to` divergent-ancestor
    /// sibling order).
    ///
    /// # Example values
    ///
    /// `edge_id = "edge_dep_client_server__0"`, with that edge's
    /// `description_taffy_node_id` resolving to the post-layout rect `x=200,
    /// y=60, width=80, height=24` under `rank_dir: TopToBottom` and
    /// `sibling_index_from_cmp_to: Ordering::Less` -- returns
    /// `Some(SpacerCoordinates { entry_x: 200.0, entry_y: 60.0, exit_x: 200.0,
    /// exit_y: 60.0 })`.
    pub fn description_contact_resolve<'id>(
        rank_dir: RankDir,
        edge_id: &EdgeId<'id>,
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        edge_description_taffy_nodes: &EdgeIdToEdgeDescriptionTaffyNodes<'id>,
    ) -> Option<SpacerCoordinates> {
        let edge_description_taffy_nodes = edge_description_taffy_nodes.get(edge_id)?;
        EdgeSpacerCoordinatesCalculator::calculate_description_contact(
            rank_dir,
            taffy_tree,
            edge_description_taffy_nodes.description_taffy_node_id,
            edge_description_taffy_nodes.sibling_index_from_cmp_to,
        )
    }

    /// Snaps an edge's cross-container spacers to a single straight column on
    /// the cross-axis (X for vertical flows, Y for horizontal flows).
    ///
    /// Cross-container spacers are appended to each rank row, so their
    /// cross-axis position is set by taffy from how much sibling content
    /// precedes them in that row. When an edge routes through several rows
    /// whose preceding content differs -- e.g. a deeper row drops a sibling
    /// edge's spacer that had padded this edge's spacer outward in the
    /// shallower rows -- the per-row spacers land at different cross-axis
    /// coordinates and the edge path zig-zags. That zig-zag can cross a
    /// neighbouring edge whose column is straight (e.g. `ranks_slots` vs
    /// `labels_offsets` in `0043`).
    ///
    /// All of an edge's cross-container spacers sit on the **same** side of the
    /// rows' nodes (the gap side they were appended on), so collapsing them
    /// onto the single **outermost** coordinate keeps the column clear of
    /// every row's node -- each row's node ends at or before its own
    /// spacer, which is at or inside the chosen extreme.
    ///
    /// The outermost coordinate is the **maximum** in every `RankDir`. The
    /// nodes in each content-sized rank row are packed toward the
    /// low-coordinate side (left / top) and the appended spacers toward the
    /// high-coordinate side:
    ///
    /// * Forward flows (`TopToBottom` / `LeftToRight`) use an un-reversed flex
    ///   direction, so the spacers -- appended after the nodes -- render at the
    ///   high-coordinate end.
    /// * Reversed flows (`BottomToTop` / `RightToLeft`) use a reversed flex
    ///   direction, but their rank rows are also reordered by
    ///   `TaffyContainerBuilder::rank_taffy_ids_reverse_if_direction_reversed`,
    ///   which moves the appended spacers to the row's start -- and a reversed
    ///   flex direction renders the start at the high-coordinate end. So they
    ///   too land on the max side. (An earlier version snapped reversed flows
    ///   to the minimum; that only stayed clear when every row had the same
    ///   cross-axis extent -- e.g. `RightToLeft`'s equal-height rows -- and
    ///   routed the column over the wider rows otherwise, e.g. `BottomToTop` in
    ///   `0047`.)
    ///
    /// A single spacer is already a straight column, so the snap is a no-op
    /// below two.
    fn cross_container_spacers_snap_to_column(
        rank_dir: RankDir,
        cross_container_spacers: &mut [SpacerCoordinates],
    ) {
        if cross_container_spacers.len() < 2 {
            return;
        }

        let vertical_flow = matches!(rank_dir, RankDir::TopToBottom | RankDir::BottomToTop);

        let cross_axis = |spacer_coordinates: &SpacerCoordinates| {
            if vertical_flow {
                spacer_coordinates.entry_x
            } else {
                spacer_coordinates.entry_y
            }
        };

        let Some(column) = cross_container_spacers
            .iter()
            .map(cross_axis)
            .reduce(f32::max)
        else {
            return;
        };

        cross_container_spacers
            .iter_mut()
            .for_each(|spacer_coordinates| {
                if vertical_flow {
                    spacer_coordinates.entry_x = column;
                    spacer_coordinates.exit_x = column;
                } else {
                    spacer_coordinates.entry_y = column;
                    spacer_coordinates.exit_y = column;
                }
            });
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

    /// Sorts merged spacers into the order the edge path visits them, by their
    /// absolute coordinate along the main (rank) axis -- entry Y for vertical
    /// flow, entry X for horizontal flow.
    ///
    /// The flow runs in increasing-coordinate order for `TopToBottom` /
    /// `LeftToRight`, but in *decreasing*-coordinate order for `BottomToTop` /
    /// `RightToLeft` (rank 0 sits at the high-coordinate end because the rank
    /// containers use a reversed flex direction). The sort is reversed for the
    /// latter so the spacer order matches both the visual traversal direction
    /// and the rank order used by [`Self::rank_spacers_sort_by_rank`].
    fn spacers_sort_by_main_axis(
        rank_dir: RankDir,
        mut all_spacers: Vec<SpacerCoordinates>,
    ) -> Vec<SpacerCoordinates> {
        let main_axis_key = |spacer_coordinates: &SpacerCoordinates| match rank_dir {
            RankDir::TopToBottom | RankDir::BottomToTop => spacer_coordinates.entry_y,
            RankDir::LeftToRight | RankDir::RightToLeft => spacer_coordinates.entry_x,
        };
        let reverse = matches!(rank_dir, RankDir::BottomToTop | RankDir::RightToLeft);
        all_spacers.sort_by(|a, b| {
            let ordering = main_axis_key(a)
                .partial_cmp(&main_axis_key(b))
                .unwrap_or(std::cmp::Ordering::Equal);
            if reverse {
                ordering.reverse()
            } else {
                ordering
            }
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

    #[test]
    fn spacers_sort_by_main_axis_reverses_entry_y_for_bottom_to_top_flow() {
        // Bottom-to-top flow runs in decreasing y, so spacers are ordered by
        // descending entry_y to match the visual traversal direction.
        let all_spacers = vec![spacer(99.0, 10.0), spacer(99.0, 30.0), spacer(99.0, 20.0)];

        let sorted =
            SpacerCoordinatesResolver::spacers_sort_by_main_axis(RankDir::BottomToTop, all_spacers);

        let entry_ys: Vec<f32> = sorted.iter().map(|spacer| spacer.entry_y).collect();
        assert_eq!(vec![30.0, 20.0, 10.0], entry_ys);
    }

    #[test]
    fn spacers_sort_by_main_axis_reverses_entry_x_for_right_to_left_flow() {
        // Right-to-left flow runs in decreasing x, so spacers are ordered by
        // descending entry_x to match the visual traversal direction.
        let all_spacers = vec![spacer(10.0, 99.0), spacer(30.0, 99.0), spacer(20.0, 99.0)];

        let sorted =
            SpacerCoordinatesResolver::spacers_sort_by_main_axis(RankDir::RightToLeft, all_spacers);

        let entry_xs: Vec<f32> = sorted.iter().map(|spacer| spacer.entry_x).collect();
        assert_eq!(vec![30.0, 20.0, 10.0], entry_xs);
    }
}
