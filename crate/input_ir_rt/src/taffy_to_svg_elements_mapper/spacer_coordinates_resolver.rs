use disposition_ir_model::{edge::EdgeId, node::NodeRank};
use disposition_model_common::RankDir;
use disposition_taffy_model::{
    taffy::TaffyTree, EdgeIdToEdgeDescriptionTaffyNodes, EdgeIdToEdgeSpacerTaffyNodes,
    EdgeSpacerCtx, EdgeSpacerTaffyNodes, TaffyNodeCtx,
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
/// passes straight through the spacer area. The description contact follows
/// suit for both cross-rank edges (whose box sits directly on the rank
/// corridor between its divergent ancestors) and same-rank/cycle edges
/// (whose box sits directly between its two divergent ancestors within
/// their shared rank) -- see [`Self::description_contact_resolve`].
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
        interaction_edge_halo_stroke_width: f32,
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
        let edge_desc_container_spacers = Self::spacers_calculate(
            rank_dir,
            taffy_tree,
            &spacer_nodes.edge_desc_container_spacer_taffy_node_ids,
        );
        // Same-rank (cycle edge) crossing spacers are resolved via the same
        // direction-aware thread-through calculation used for the owning
        // edge's own description contact
        // (`calculate_description_thread_same_rank`), not the direction-oblivious
        // generic `calculate`. A same-rank container's children stack
        // perpendicular to the edge's own travel direction, so a crossing
        // edge's approach direction relative to the container (which of its
        // two ends it reaches first) is not always the "forward" convention
        // baked into `calculate`; resolving with that convention regardless
        // can select the far end as the nominal "entry", forcing the
        // connector to cut through the container's interior to reach it
        // before the near end is threaded. See
        // [`Self::same_rank_spacers_calculate`].
        let same_rank_edge_desc_container_spacers = Self::same_rank_spacers_calculate(
            rank_dir,
            taffy_tree,
            &spacer_nodes.same_rank_edge_desc_container_spacer_taffy_node_ids,
        );
        let mut text_content_spacers = Self::spacers_calculate(
            rank_dir,
            taffy_tree,
            &spacer_nodes.text_content_spacer_taffy_node_ids,
        );
        // Text-content spacers participate in the same outermost-column snap
        // as cross-container spacers: see `spacers_snap_to_outermost_column`.
        Self::spacers_snap_to_outermost_column(
            rank_dir,
            &mut cross_container_spacers,
            &mut text_content_spacers,
        );
        let description_contact = Self::description_contact_resolve(
            rank_dir,
            edge_id,
            taffy_tree,
            edge_description_taffy_nodes,
            interaction_edge_halo_stroke_width,
        );

        if cross_container_spacers.is_empty()
            && edge_desc_container_spacers.is_empty()
            && same_rank_edge_desc_container_spacers.is_empty()
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
            .chain(same_rank_edge_desc_container_spacers)
            .chain(text_content_spacers)
            .chain(description_contact)
            .collect();

        Self::spacers_sort_by_main_axis(rank_dir, all_spacers)
    }

    /// Resolves the waypoint(s) for an edge's own `edge_description_container`
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
    /// For a **cross-rank** edge (`EdgeDescriptionTaffyNodes::is_cross_rank`),
    /// the description box sits directly on the rank corridor between the
    /// edge's divergent ancestors, so it is threaded *through* via
    /// `EdgeSpacerCoordinatesCalculator::calculate_description_thread`. For a
    /// **same-rank** (cycle edge) box, which sits directly between its two
    /// divergent ancestors *within* their shared rank, it is likewise
    /// threaded through, but on the rotated axis those siblings are laid out
    /// on, via `calculate_description_thread_same_rank`.
    ///
    /// # Example values
    ///
    /// `edge_id = "edge_dep_client_server__0"`, with that edge's
    /// `description_taffy_node_id` resolving to the post-layout rect `x=200,
    /// y=60, width=80, height=24` under `rank_dir: TopToBottom`,
    /// `sibling_index_from_cmp_to: Ordering::Less`, `is_cross_rank: true` --
    /// returns `Some(SpacerCoordinates { entry_x: 200.0, entry_y: 60.0,
    /// exit_x: 200.0, exit_y: 84.0 })`.
    pub fn description_contact_resolve<'id>(
        rank_dir: RankDir,
        edge_id: &EdgeId<'id>,
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        edge_description_taffy_nodes: &EdgeIdToEdgeDescriptionTaffyNodes<'id>,
        interaction_edge_halo_stroke_width: f32,
    ) -> Option<SpacerCoordinates> {
        let edge_description_taffy_nodes = edge_description_taffy_nodes.get(edge_id)?;
        if edge_description_taffy_nodes.is_cross_rank {
            EdgeSpacerCoordinatesCalculator::calculate_description_thread(
                rank_dir,
                taffy_tree,
                edge_description_taffy_nodes.description_taffy_node_id,
                edge_description_taffy_nodes.sibling_index_from_cmp_to,
                interaction_edge_halo_stroke_width,
            )
        } else {
            EdgeSpacerCoordinatesCalculator::calculate_description_thread_same_rank(
                rank_dir,
                taffy_tree,
                edge_description_taffy_nodes.description_taffy_node_id,
                edge_description_taffy_nodes.sibling_index_from_cmp_to,
                interaction_edge_halo_stroke_width,
            )
        }
    }

    /// Snaps an edge's cross-container spacers to a single straight column on
    /// the cross-axis (X for vertical flows, Y for horizontal flows), then
    /// pulls any **later** (deeper) text-content spacer that would otherwise
    /// sit inside that column out to meet it.
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
    /// Text-content spacers -- placed just past a described container's text
    /// band -- are resolved independently per container, so their local
    /// coordinate has no relationship to any cross-container column the edge
    /// has already committed to. When a **deeper** container's text-content
    /// spacer sits further along the path than a column already established
    /// by two or more cross-container spacers, and that column is further out
    /// than the text spacer's own position, the path bows out to the column,
    /// dips back in to reach the un-aligned text spacer, then has to jog back
    /// out again toward the to-node -- an unnecessary extra Z-bend (e.g.
    /// `t_aws_az1_subnet_tier_private`'s label spacer sitting inside the
    /// wider `t_aws_az1` cross-container column in `0062`). This function
    /// pulls that later text spacer out to the column to remove the dip.
    ///
    /// A text-content spacer that sits **earlier** than the cross-container
    /// column (i.e. it belongs to the same, outer container whose siblings
    /// produced the column, encountered before descending into it) is left
    /// alone: bowing out from the from-node straight to that spacer's own
    /// local position, then further out again to the column, is a single
    /// smooth outward bend, not a dip -- and forcing it onto the column early
    /// would move it for no routing benefit, rippling into how other edges
    /// sharing its rank gap are nested by `jogs_separate` (e.g.
    /// `edge_dep_layout_contacts` vs `edge_dep_ir_pass1` in `0044`, whose
    /// single (unsnapped) cross-container spacer and text-content spacer must
    /// stay at their own positions for their return jogs to stay separated).
    /// A text spacer that is already further out than the column (e.g. a
    /// deliberate detour further than the edge's own rank column, as
    /// `labels_offsets` in `0044`) is likewise left alone, since there is no
    /// dip to fix.
    ///
    /// All of an edge's cross-container spacers sit on the **same** side of
    /// their rows' nodes (the gap side they were appended on), so collapsing
    /// them onto the single **outermost** coordinate keeps the column clear
    /// of every row's node -- each row's node ends at or before its own
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
    /// The snap is a no-op unless `cross_container_spacers` itself holds two
    /// or more entries -- that is the condition under which cross-container
    /// spacers need aligning at all; text-content spacers only ride along
    /// once that alignment is already happening, and only when they are
    /// positioned after it.
    fn spacers_snap_to_outermost_column(
        rank_dir: RankDir,
        cross_container_spacers: &mut [SpacerCoordinates],
        text_content_spacers: &mut [SpacerCoordinates],
    ) {
        if cross_container_spacers.len() < 2 {
            return;
        }

        let vertical_flow = matches!(rank_dir, RankDir::TopToBottom | RankDir::BottomToTop);
        let reverse = matches!(rank_dir, RankDir::BottomToTop | RankDir::RightToLeft);

        let cross_axis = |spacer_coordinates: &SpacerCoordinates| {
            if vertical_flow {
                spacer_coordinates.entry_x
            } else {
                spacer_coordinates.entry_y
            }
        };
        let main_axis = |spacer_coordinates: &SpacerCoordinates| {
            if vertical_flow {
                spacer_coordinates.entry_y
            } else {
                spacer_coordinates.entry_x
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

        // The earliest point along the path at which the column is
        // established -- the first cross-container spacer encountered.
        // Reversed flows travel in decreasing main-axis order, so "earliest"
        // is the maximum there instead of the minimum.
        let Some(column_earliest_main) = cross_container_spacers
            .iter()
            .map(main_axis)
            .reduce(|a, b| if reverse { a.max(b) } else { a.min(b) })
        else {
            return;
        };

        text_content_spacers
            .iter_mut()
            .filter(|spacer_coordinates| {
                let occurs_after_column = if reverse {
                    main_axis(spacer_coordinates) < column_earliest_main
                } else {
                    main_axis(spacer_coordinates) > column_earliest_main
                };
                occurs_after_column && cross_axis(spacer_coordinates) < column
            })
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

    /// Calculates spacer coordinates for a slice of same-rank (cycle edge)
    /// crossing spacer taffy node IDs, dropping any whose layout or context
    /// cannot be resolved.
    ///
    /// Unlike [`Self::spacers_calculate`], each spacer's entry/exit are
    /// resolved via
    /// `EdgeSpacerCoordinatesCalculator::calculate_description_thread_same_rank`,
    /// using the `same_rank_sibling_index_from_cmp_to` ordering recorded on
    /// the spacer's own `EdgeSpacerCtx` (set by
    /// `EdgeSpacerBuilder::build_edge_desc_container_spacers_for_edge_same_rank`)
    /// so the thread-through direction matches this edge's own approach,
    /// rather than the container's `BetweenRanks` "forward" convention.
    /// `interaction_edge_halo_stroke_width` is `0.0`: a plain `EdgeSpacer`
    /// leaf has no halo-clearance margin to cancel (unlike the description
    /// box itself), so no pullback is needed.
    fn same_rank_spacers_calculate(
        rank_dir: RankDir,
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        spacer_taffy_node_ids: &[taffy::NodeId],
    ) -> Vec<SpacerCoordinates> {
        spacer_taffy_node_ids
            .iter()
            .filter_map(|&taffy_node_id| {
                let Some(TaffyNodeCtx::EdgeSpacer(EdgeSpacerCtx {
                    same_rank_sibling_index_from_cmp_to: Some(sibling_index_from_cmp_to),
                    ..
                })) = taffy_tree.get_node_context(taffy_node_id)
                else {
                    return None;
                };
                EdgeSpacerCoordinatesCalculator::calculate_description_thread_same_rank(
                    rank_dir,
                    taffy_tree,
                    taffy_node_id,
                    *sibling_index_from_cmp_to,
                    0.0,
                )
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
