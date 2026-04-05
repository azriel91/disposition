use disposition_ir_model::node::{NodeId, NodeRank};
use disposition_model_common::Map;
use disposition_svg_model::SvgNodeInfo;
use disposition_taffy_model::{taffy::TaffyTree, EdgeSpacerTaffyNodes, TaffyNodeCtx};

use crate::taffy_to_svg_elements_mapper::{
    edge_model::{EdgeContactPointOffsets, NodeFace, NodeIdAndFace},
    edge_path_builder_pass_1::SpacerCoordinates,
    edge_path_builder_pass_2::edge_path_builder_pass_2_ortho::OrthoProtrusionParams,
};

use super::svg_edge_infos_builder::{EdgeGroupPass1, EdgePass1Info};

/// Maximum fraction of the rank gap that a protrusion may occupy.
///
/// # Example values
///
/// `0.45` -- each side (from and to) may use up to 45% of the gap,
/// leaving at least 10% for the horizontal routing segment in between.
const MAX_GAP_FRACTION: f32 = 0.45;

/// Minimum protrusion length in pixels.
///
/// When an edge is not perfectly straight (i.e. the from and to
/// contact points differ on the cross-axis), the protrusion is at
/// least this many pixels so the perpendicular stub is visible.
const MIN_PROTRUSION_PX: f32 = 5.0;

/// Computes orthogonal protrusion parameters globally across all edge
/// groups.
///
/// The protrusion is a short perpendicular stub drawn when an
/// orthogonal edge exits a node face, before the main horizontal or
/// vertical routing segment begins. Protrusion lengths are assigned
/// so that edges sharing the same inter-rank gap use distinct depths,
/// preventing overlapping horizontal/vertical channels.
///
/// # Algorithm
///
/// 1. Collect every edge endpoint that exits into a rank gap (the gap between
///    the from-node's rank and the next rank, or between the to-node's rank and
///    the previous rank).
/// 2. Group these endpoints by `(rank_low, rank_high)` -- the rank gap they
///    occupy.
/// 3. Within each group, sort endpoints by their cross-axis coordinate (the
///    coordinate perpendicular to the rank direction). Edges further from the
///    centre of the gap's cross-axis spread get shorter protrusions; edges
///    closer to the centre get longer ones. This reduces the chance of crossing
///    edges sharing the same horizontal/vertical channel.
/// 4. Distribute protrusion depths evenly within the available gap space
///    (capped at `MAX_GAP_FRACTION` of the pixel distance between ranks).
///
/// The result is a parallel `Vec<OrthoProtrusionParams>` for each edge
/// group, indexed identically to `EdgeGroupPass1::pass1_infos`.
#[derive(Clone, Copy, Debug)]
pub(super) struct OrthoProtrusionCalculator;

/// Identifies a single endpoint of a single edge across all groups.
///
/// Used as an intermediate record while grouping endpoints by rank
/// gap.
#[derive(Clone, Debug)]
struct RankGapEntry {
    /// Index into `all_pass1_groups`.
    pass1_group_index: usize,
    /// Index into the group's `pass1_infos`.
    edge_index: usize,
    /// `true` for the "from" endpoint, `false` for the "to" endpoint.
    is_from_endpoint: bool,
    /// Cross-axis coordinate of the endpoint's node.
    ///
    /// For `Top` / `Bottom` faces this is the node's X coordinate;
    /// for `Left` / `Right` faces this is the node's Y coordinate.
    cross_axis_coord: f32,
    /// The face offset (slot offset) for this endpoint.
    ///
    /// Edges further from the face midpoint (larger absolute offset)
    /// receive shorter protrusions.
    face_offset: f32,
    /// Pixel distance in the rank direction for this endpoint's rank
    /// gap (from the node contact point to the nearest spacer or the
    /// other endpoint).
    rank_gap_px: f32,
}

/// Key for grouping endpoints into rank gaps.
///
/// Represents the gap between two adjacent ranks. `rank_low` is
/// always <= `rank_high`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct RankGapKey {
    rank_low: NodeRank,
    rank_high: NodeRank,
}

impl OrthoProtrusionCalculator {
    /// Calculates protrusion parameters for every edge in every group.
    ///
    /// Returns a `Vec` parallel to `all_pass1_groups`, where each
    /// inner `Vec` is parallel to the group's `pass1_infos`.
    ///
    /// # Parameters
    ///
    /// * `all_pass1_groups` -- all edge groups with pass-1 metadata.
    /// * `face_offsets_by_node_face` -- precomputed per-face offset vectors
    ///   from `face_offsets_compute`.
    /// * `svg_node_info_map` -- node layout information.
    /// * `taffy_tree` -- the layout tree (for spacer coordinate lookups).
    /// * `edge_spacer_taffy_nodes` -- spacer node mappings per edge.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn calculate<'id>(
        all_pass1_groups: &[EdgeGroupPass1<'_, 'id>],
        from_slot_indices_all: &[Vec<Option<usize>>],
        to_slot_indices_all: &[Vec<Option<usize>>],
        face_offsets_by_node_face: &Map<NodeIdAndFace<'id>, EdgeContactPointOffsets>,
        svg_node_info_map: &Map<&NodeId<'id>, &SvgNodeInfo<'id>>,
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        edge_spacer_taffy_nodes: &Map<
            disposition_ir_model::edge::EdgeId<'id>,
            EdgeSpacerTaffyNodes,
        >,
    ) -> Vec<Vec<OrthoProtrusionParams>> {
        // === Step 1: Initialize output with defaults === //
        let mut result: Vec<Vec<OrthoProtrusionParams>> = all_pass1_groups
            .iter()
            .map(|group| vec![OrthoProtrusionParams::default(); group.pass1_infos.len()])
            .collect();

        // === Step 2: Collect rank gap entries === //
        let mut rank_gap_entries: Map<RankGapKey, Vec<RankGapEntry>> = Map::new();

        for (group_idx, group) in all_pass1_groups.iter().enumerate() {
            let from_slot_indices = &from_slot_indices_all[group_idx];
            let to_slot_indices = &to_slot_indices_all[group_idx];

            for (edge_idx, pass1_info) in group.pass1_infos.iter().enumerate() {
                let rank_from = pass1_info.rank_from;
                let rank_to = pass1_info.rank_to;

                // Determine rank ordering.
                let (rank_low, rank_high) = if rank_from <= rank_to {
                    (rank_from, rank_to)
                } else {
                    (rank_to, rank_from)
                };

                // Skip self-loops and same-rank edges (no rank gap to
                // protrude into).
                if rank_low == rank_high {
                    continue;
                }

                let spacer_coordinates = Self::spacer_coordinates_resolve(
                    pass1_info,
                    taffy_tree,
                    edge_spacer_taffy_nodes,
                );

                // === From endpoint === //
                if let Some(from_face) = pass1_info.from_face {
                    let from_offset = Self::face_offset_resolve(
                        pass1_info,
                        from_slot_indices[edge_idx],
                        true,
                        face_offsets_by_node_face,
                    );

                    let from_rank_gap_px = Self::rank_gap_px(
                        pass1_info,
                        from_face,
                        true,
                        svg_node_info_map,
                        &spacer_coordinates,
                    );

                    let cross_axis_from = Self::cross_axis_coord(
                        pass1_info.from_node_x,
                        pass1_info.from_node_y,
                        from_face,
                    );

                    // The from endpoint exits into the gap between the
                    // from-node's rank and the adjacent rank toward the
                    // to-node.
                    let from_gap_key = if rank_from < rank_to {
                        RankGapKey {
                            rank_low: rank_from,
                            rank_high: NodeRank::new(rank_from.value() + 1),
                        }
                    } else {
                        RankGapKey {
                            rank_low: NodeRank::new(rank_from.value() - 1),
                            rank_high: rank_from,
                        }
                    };

                    rank_gap_entries
                        .entry(from_gap_key)
                        .or_default()
                        .push(RankGapEntry {
                            pass1_group_index: group_idx,
                            edge_index: edge_idx,
                            is_from_endpoint: true,
                            cross_axis_coord: cross_axis_from,
                            face_offset: from_offset,
                            rank_gap_px: from_rank_gap_px,
                        });
                }

                // === To endpoint === //
                if let Some(to_face) = pass1_info.to_face {
                    let to_offset = Self::face_offset_resolve(
                        pass1_info,
                        to_slot_indices[edge_idx],
                        false,
                        face_offsets_by_node_face,
                    );

                    let to_rank_gap_px = Self::rank_gap_px(
                        pass1_info,
                        to_face,
                        false,
                        svg_node_info_map,
                        &spacer_coordinates,
                    );

                    let cross_axis_to =
                        Self::cross_axis_coord(pass1_info.to_node_x, pass1_info.to_node_y, to_face);

                    // The to endpoint enters from the gap between the
                    // to-node's rank and the adjacent rank toward the
                    // from-node.
                    let to_gap_key = if rank_to > rank_from {
                        RankGapKey {
                            rank_low: NodeRank::new(rank_to.value() - 1),
                            rank_high: rank_to,
                        }
                    } else {
                        RankGapKey {
                            rank_low: rank_to,
                            rank_high: NodeRank::new(rank_to.value() + 1),
                        }
                    };

                    rank_gap_entries
                        .entry(to_gap_key)
                        .or_default()
                        .push(RankGapEntry {
                            pass1_group_index: group_idx,
                            edge_index: edge_idx,
                            is_from_endpoint: false,
                            cross_axis_coord: cross_axis_to,
                            face_offset: to_offset,
                            rank_gap_px: to_rank_gap_px,
                        });
                }
            }
        }

        // === Step 3: For each rank gap, assign protrusion depths === //
        for (_gap_key, entries) in &mut rank_gap_entries {
            Self::protrusions_assign(entries, &mut result);
        }

        result
    }

    /// Assigns protrusion depths to all endpoints in a single rank gap.
    ///
    /// # Algorithm
    ///
    /// 1. Find the minimum `rank_gap_px` across all entries (the tightest
    ///    constraint).
    /// 2. Compute the maximum allowed protrusion = `min_gap_px *
    ///    MAX_GAP_FRACTION`.
    /// 3. Sort entries by `(|face_offset| descending, cross_axis_coord
    ///    ascending)`. This places outer-face edges first (shortest protrusion)
    ///    and inner-face edges last (longest protrusion). The cross-axis
    ///    coordinate breaks ties so that edges from different nodes at the same
    ///    offset get distinct depths.
    /// 4. Assign protrusion depths evenly spaced from `MIN_PROTRUSION_PX` to
    ///    `max_protrusion`.
    fn protrusions_assign(
        rank_gap_entries: &mut [RankGapEntry],
        result: &mut [Vec<OrthoProtrusionParams>],
    ) {
        if rank_gap_entries.is_empty() {
            return;
        }

        // Find the tightest rank gap constraint.
        let min_gap_px = rank_gap_entries
            .iter()
            .map(|rank_gap_entry| rank_gap_entry.rank_gap_px)
            .reduce(f32::min)
            .unwrap_or(0.0);

        let max_protrusion = min_gap_px * MAX_GAP_FRACTION;

        if max_protrusion < MIN_PROTRUSION_PX {
            // Gap is too small for meaningful protrusions; assign
            // minimum protrusions to all.
            for rank_gap_entry in rank_gap_entries.iter() {
                Self::protrusion_write(
                    rank_gap_entry,
                    MIN_PROTRUSION_PX.min(min_gap_px * 0.5),
                    result,
                );
            }
            return;
        }

        // Sort: outer-face edges first (larger |face_offset| -> shorter
        // protrusion), then by cross-axis coordinate for tie-breaking.
        rank_gap_entries.sort_by(|rank_gap_entry_a, rank_gap_entry_b| {
            let offset_cmp = rank_gap_entry_a
                .face_offset
                .abs()
                .partial_cmp(&rank_gap_entry_b.face_offset)
                .unwrap_or(std::cmp::Ordering::Equal)
                .reverse();
            if offset_cmp != std::cmp::Ordering::Equal {
                return offset_cmp;
            }
            rank_gap_entry_a
                .cross_axis_coord
                .partial_cmp(&rank_gap_entry_b.cross_axis_coord)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let rank_gap_entry_count = rank_gap_entries.len();
        if rank_gap_entry_count == 1 {
            // Single edge: use half the available protrusion space.
            let protrusion = (max_protrusion * 0.5).max(MIN_PROTRUSION_PX);
            Self::protrusion_write(&rank_gap_entries[0], protrusion, result);
            return;
        }

        // Distribute evenly from MIN_PROTRUSION_PX to max_protrusion.
        // Index 0 (outermost face offset) gets the shortest protrusion;
        // index count-1 (innermost) gets the longest.
        let protrusion_growable_space = max_protrusion - MIN_PROTRUSION_PX;
        for (rank_gap_entry_index, rank_gap_entry) in rank_gap_entries.iter().enumerate() {
            let rank_gap_entry_proportion =
                rank_gap_entry_index as f32 / (rank_gap_entry_count - 1) as f32;
            let protrusion =
                MIN_PROTRUSION_PX + rank_gap_entry_proportion * protrusion_growable_space;
            Self::protrusion_write(rank_gap_entry, protrusion, result);
        }
    }

    /// Writes a protrusion value to the appropriate slot in `result`.
    fn protrusion_write(
        entry: &RankGapEntry,
        protrusion: f32,
        result: &mut [Vec<OrthoProtrusionParams>],
    ) {
        let params = &mut result[entry.pass1_group_index][entry.edge_index];
        if entry.is_from_endpoint {
            params.from_protrusion = protrusion;
        } else {
            params.to_protrusion = protrusion;
        }
    }

    /// Resolves the face offset for a single endpoint.
    fn face_offset_resolve<'id>(
        pass1_info: &EdgePass1Info<'_, 'id>,
        slot_index: Option<usize>,
        is_from: bool,
        face_offsets_by_node_face: &Map<NodeIdAndFace<'id>, EdgeContactPointOffsets>,
    ) -> f32 {
        let (node_id, face) = if is_from {
            (&pass1_info.edge.from, pass1_info.from_face)
        } else {
            (&pass1_info.edge.to, pass1_info.to_face)
        };

        let Some(face) = face else { return 0.0 };
        let Some(slot_index) = slot_index else {
            return 0.0;
        };

        let node_id_and_face = NodeIdAndFace {
            node_id: node_id.clone(),
            face,
        };

        face_offsets_by_node_face
            .get(&node_id_and_face)
            .and_then(|offsets| offsets.get(slot_index))
            .unwrap_or(0.0)
    }

    /// Computes the pixel distance in the rank direction for one
    /// endpoint of an edge.
    ///
    /// For the "from" endpoint, this is the distance from the
    /// from-node's face center to the first spacer entry (or the
    /// to-node's face center if there are no spacers).
    ///
    /// For the "to" endpoint, this is the distance from the to-node's
    /// face center to the last spacer exit (or the from-node's face
    /// center if there are no spacers).
    fn rank_gap_px<'id>(
        pass1_info: &EdgePass1Info<'_, 'id>,
        face: NodeFace,
        is_from: bool,
        svg_node_info_map: &Map<&NodeId<'id>, &SvgNodeInfo<'id>>,
        spacer_coordinates: &[SpacerCoordinates],
    ) -> f32 {
        // Get node coordinates.
        let from_info = svg_node_info_map.get(&pass1_info.edge.from);
        let to_info = svg_node_info_map.get(&pass1_info.edge.to);

        let (from_x, from_y) = from_info
            .map(|info| Self::face_center(info, pass1_info.from_face.unwrap_or(NodeFace::Bottom)))
            .unwrap_or((pass1_info.from_node_x, pass1_info.from_node_y));

        let (to_x, to_y) = to_info
            .map(|info| Self::face_center(info, pass1_info.to_face.unwrap_or(NodeFace::Top)))
            .unwrap_or((pass1_info.to_node_x, pass1_info.to_node_y));

        if is_from {
            if spacer_coordinates.is_empty() {
                Self::axis_distance(from_x, from_y, to_x, to_y, face)
            } else {
                let first_spacer = &spacer_coordinates[0];
                Self::axis_distance(
                    from_x,
                    from_y,
                    first_spacer.entry_x,
                    first_spacer.entry_y,
                    face,
                )
            }
        } else {
            if spacer_coordinates.is_empty() {
                Self::axis_distance(to_x, to_y, from_x, from_y, face)
            } else {
                let last_spacer = &spacer_coordinates[spacer_coordinates.len() - 1];
                Self::axis_distance(to_x, to_y, last_spacer.exit_x, last_spacer.exit_y, face)
            }
        }
    }

    /// Resolves spacer coordinates for an edge, reusing the same logic
    /// as the path builder.
    fn spacer_coordinates_resolve<'id>(
        pass1_info: &EdgePass1Info<'_, 'id>,
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        edge_spacer_taffy_nodes: &Map<
            disposition_ir_model::edge::EdgeId<'id>,
            EdgeSpacerTaffyNodes,
        >,
    ) -> Vec<SpacerCoordinates> {
        let Some(spacer_nodes) = edge_spacer_taffy_nodes.get(&pass1_info.edge_id) else {
            return Vec::new();
        };

        let mut rank_spacers: Vec<(NodeRank, SpacerCoordinates)> = spacer_nodes
            .rank_to_spacer_taffy_node_id
            .iter()
            .filter_map(|(rank, &taffy_node_id)| {
                let layout = taffy_tree.layout(taffy_node_id).ok()?;

                let mut x_acc = layout.location.x;
                let mut y_acc = layout.location.y;
                let mut current_node_id = taffy_node_id;
                while let Some(parent_taffy_node_id) = taffy_tree.parent(current_node_id) {
                    let Ok(parent_layout) = taffy_tree.layout(parent_taffy_node_id) else {
                        break;
                    };
                    x_acc += parent_layout.location.x;
                    y_acc += parent_layout.location.y;
                    current_node_id = parent_taffy_node_id;
                }

                let cx = x_acc + layout.size.width / 2.0;
                let top_y = y_acc;
                let bottom_y = y_acc + layout.size.height;

                Some((
                    *rank,
                    SpacerCoordinates {
                        entry_x: cx,
                        entry_y: top_y,
                        exit_x: cx,
                        exit_y: bottom_y,
                    },
                ))
            })
            .collect();

        rank_spacers.sort_by_key(|(rank, _)| *rank);
        rank_spacers.into_iter().map(|(_, coords)| coords).collect()
    }

    /// Returns the cross-axis coordinate of a node for a given face.
    ///
    /// For `Top` / `Bottom` faces the cross-axis is horizontal (X).
    /// For `Left` / `Right` faces the cross-axis is vertical (Y).
    fn cross_axis_coord(node_x: f32, node_y: f32, face: NodeFace) -> f32 {
        match face {
            NodeFace::Top | NodeFace::Bottom => node_x,
            NodeFace::Left | NodeFace::Right => node_y,
        }
    }

    /// Computes the absolute distance along the rank axis between two
    /// points.
    ///
    /// For `Top` / `Bottom` faces the rank axis is Y. For `Left` /
    /// `Right` faces the rank axis is X.
    fn axis_distance(ax: f32, ay: f32, bx: f32, by: f32, face: NodeFace) -> f32 {
        match face {
            NodeFace::Top | NodeFace::Bottom => (by - ay).abs(),
            NodeFace::Left | NodeFace::Right => (bx - ax).abs(),
        }
    }

    /// Returns the face center coordinates for a node.
    fn face_center(info: &SvgNodeInfo<'_>, face: NodeFace) -> (f32, f32) {
        match face {
            NodeFace::Top => (info.x + info.width / 2.0, info.y),
            NodeFace::Bottom => (info.x + info.width / 2.0, info.y + info.height_collapsed),
            NodeFace::Left => (info.x, info.y + info.height_collapsed / 2.0),
            NodeFace::Right => (info.x + info.width, info.y + info.height_collapsed / 2.0),
        }
    }
}
