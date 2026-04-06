use disposition_ir_model::node::{NodeId, NodeRank};
use disposition_model_common::Map;
use disposition_svg_model::SvgNodeInfo;
use disposition_taffy_model::{taffy::TaffyTree, EdgeSpacerTaffyNodes, TaffyNodeCtx};

use crate::taffy_to_svg_elements_mapper::{
    edge_model::{EdgeContactPointOffsets, NodeFace, NodeIdAndFace},
    edge_path_builder_pass_1::SpacerCoordinates,
    edge_path_builder_pass_2::edge_path_builder_pass_2_ortho::{
        OrthoProtrusionParams, SpacerProtrusionParams,
    },
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
    /// Which endpoint or spacer side this entry represents.
    endpoint_kind: RankGapEndpointKind,
    /// Which side of the rank gap this entry protrudes from.
    ///
    /// Entries on the `Low` side protrude from the `rank_low`
    /// boundary; entries on the `High` side protrude from the
    /// `rank_high` boundary.
    gap_side: GapSide,
    /// Cross-axis coordinate of the endpoint's node (or spacer).
    ///
    /// For `Top` / `Bottom` faces this is the node's X coordinate;
    /// for `Left` / `Right` faces this is the node's Y coordinate.
    /// For spacer endpoints, the spacer's X or Y coordinate is used.
    cross_axis_coord: f32,
    /// The face offset (slot offset) for this endpoint.
    ///
    /// Edges further from the face midpoint (larger absolute offset)
    /// receive shorter protrusions. For spacer endpoints this is
    /// `0.0` since spacers do not have face offsets.
    face_offset: f32,
    /// Pixel distance in the rank direction for this endpoint's rank
    /// gap (from the node contact point or spacer boundary to the
    /// nearest adjacent spacer or node).
    rank_gap_px: f32,
}

/// Which physical boundary of a rank gap an entry protrudes from.
///
/// When two entries from the **same** side of a gap have the same
/// protrusion depth their stubs overlap directly. When two entries
/// from **opposite** sides have protrusion depths whose difference
/// `(from_prot - to_prot)` is equal, the horizontal routing midpoint
/// (computed by `connect_waypoints` as `(y_low_prot + y_high_prot) /
/// 2`) will coincide. Both situations must be avoided.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GapSide {
    /// Protrudes from the `rank_low` boundary of the gap.
    Low,
    /// Protrudes from the `rank_high` boundary of the gap.
    High,
}

/// Identifies which endpoint or spacer side a `RankGapEntry`
/// represents.
///
/// Used by `protrusion_write` to store the computed protrusion depth
/// in the correct field of `OrthoProtrusionParams`.
#[derive(Clone, Copy, Debug)]
enum RankGapEndpointKind {
    /// The "from" node endpoint.
    FromEndpoint,
    /// The "to" node endpoint.
    ToEndpoint,
    /// The entry side of a spacer at the given index (0-based, in
    /// the same order as the `spacers` slice).
    SpacerEntry {
        /// Index into `spacer_protrusions`.
        spacer_index: usize,
    },
    /// The exit side of a spacer at the given index.
    SpacerExit {
        /// Index into `spacer_protrusions`.
        spacer_index: usize,
    },
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
        // === Step 1: Resolve spacer coordinates and initialize output === //
        //
        // Resolve spacer coordinates once per edge so that the same
        // resolved count is used for sizing `spacer_protrusions` and
        // for registering rank gap entries. This avoids a mismatch
        // between the raw taffy node count (which includes entries
        // whose layout may fail) and the actual resolved count.
        let all_spacer_coordinates: Vec<Vec<Vec<SpacerCoordinates>>> = all_pass1_groups
            .iter()
            .map(|group| {
                group
                    .pass1_infos
                    .iter()
                    .map(|pass1_info| {
                        Self::spacer_coordinates_resolve(
                            pass1_info,
                            taffy_tree,
                            edge_spacer_taffy_nodes,
                        )
                    })
                    .collect()
            })
            .collect();

        let mut result: Vec<Vec<OrthoProtrusionParams>> = all_spacer_coordinates
            .iter()
            .map(|group_spacers| {
                group_spacers
                    .iter()
                    .map(|spacer_coords| OrthoProtrusionParams {
                        from_protrusion: 0.0,
                        to_protrusion: 0.0,
                        spacer_protrusions: vec![
                            SpacerProtrusionParams::default();
                            spacer_coords.len()
                        ],
                    })
                    .collect()
            })
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

                let spacer_coordinates = &all_spacer_coordinates[group_idx][edge_idx];

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

                    // The from-node is at rank_from. In the gap key,
                    // rank_from is rank_low when going forward, or
                    // rank_high when going backward.
                    let from_gap_side = if rank_from < rank_to {
                        GapSide::Low
                    } else {
                        GapSide::High
                    };

                    rank_gap_entries
                        .entry(from_gap_key)
                        .or_default()
                        .push(RankGapEntry {
                            pass1_group_index: group_idx,
                            edge_index: edge_idx,
                            endpoint_kind: RankGapEndpointKind::FromEndpoint,
                            gap_side: from_gap_side,
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

                    // The to-node is at rank_to. In the gap key,
                    // rank_to is rank_high when going forward, or
                    // rank_low when going backward.
                    let to_gap_side = if rank_to > rank_from {
                        GapSide::High
                    } else {
                        GapSide::Low
                    };

                    rank_gap_entries
                        .entry(to_gap_key)
                        .or_default()
                        .push(RankGapEntry {
                            pass1_group_index: group_idx,
                            edge_index: edge_idx,
                            endpoint_kind: RankGapEndpointKind::ToEndpoint,
                            gap_side: to_gap_side,
                            cross_axis_coord: cross_axis_to,
                            face_offset: to_offset,
                            rank_gap_px: to_rank_gap_px,
                        });
                }

                // === Intermediate spacer endpoints === //
                //
                // Each spacer has an entry side and an exit side that
                // protrude into adjacent rank gaps. Register entries
                // so that all edges crossing the same gap get distinct
                // protrusion depths.
                //
                // For an edge from rank 0 to rank 3 with spacers at
                // ranks 1 and 2:
                //   - spacer[0] entry side is in gap (0, 1)
                //   - spacer[0] exit side is in gap (1, 2)
                //   - spacer[1] entry side is in gap (1, 2)
                //   - spacer[1] exit side is in gap (2, 3)
                //
                // The first spacer's entry and the last spacer's exit
                // share the same rank gap as the from-endpoint and
                // to-endpoint respectively, but they protrude from the
                // **opposite side** of the gap. They must be registered
                // so they compete for distinct protrusion depths with
                // other edges on the same side (e.g. from-endpoints of
                // edges originating at that spacer's rank).
                if !spacer_coordinates.is_empty() {
                    let from_face_for_axis = pass1_info.from_face.unwrap_or(NodeFace::Bottom);

                    for spacer_idx in 0..spacer_coordinates.len() {
                        let spacer = &spacer_coordinates[spacer_idx];
                        let spacer_cross = Self::cross_axis_coord(
                            spacer.entry_x,
                            spacer.entry_y,
                            from_face_for_axis,
                        );

                        // --- Entry side of this spacer --- //
                        //
                        // The entry side protrudes into the gap BEFORE
                        // this spacer.
                        if spacer_idx > 0 {
                            // Gap between previous spacer and this one.
                            // The entry side of this spacer protrudes
                            // from the high-rank boundary of the gap
                            // (forward) or low-rank boundary (backward).
                            let prev_spacer = &spacer_coordinates[spacer_idx - 1];
                            let gap_px =
                                Self::spacer_gap_px(prev_spacer, spacer, from_face_for_axis);

                            let entry_gap_key = Self::spacer_gap_key(
                                rank_from,
                                rank_to,
                                spacer_idx - 1,
                                spacer_idx,
                            );

                            let spacer_entry_side = if rank_from < rank_to {
                                GapSide::High
                            } else {
                                GapSide::Low
                            };

                            rank_gap_entries
                                .entry(entry_gap_key)
                                .or_default()
                                .push(RankGapEntry {
                                    pass1_group_index: group_idx,
                                    edge_index: edge_idx,
                                    endpoint_kind: RankGapEndpointKind::SpacerEntry {
                                        spacer_index: spacer_idx,
                                    },
                                    gap_side: spacer_entry_side,
                                    cross_axis_coord: spacer_cross,
                                    face_offset: 0.0,
                                    rank_gap_px: gap_px,
                                });
                        } else if let Some(from_face) = pass1_info.from_face {
                            // First spacer (index 0): entry shares the
                            // same gap as the from-endpoint, but
                            // protrudes from the opposite side. Register
                            // it so it competes with other edges' to-
                            // endpoints at this spacer's rank.
                            let gap_px = Self::rank_gap_px(
                                pass1_info,
                                from_face,
                                true,
                                svg_node_info_map,
                                &spacer_coordinates,
                            );

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

                            // The first spacer's entry is on the
                            // opposite side from the from-endpoint.
                            let first_spacer_entry_side = if rank_from < rank_to {
                                GapSide::High
                            } else {
                                GapSide::Low
                            };

                            rank_gap_entries
                                .entry(from_gap_key)
                                .or_default()
                                .push(RankGapEntry {
                                    pass1_group_index: group_idx,
                                    edge_index: edge_idx,
                                    endpoint_kind: RankGapEndpointKind::SpacerEntry {
                                        spacer_index: 0,
                                    },
                                    gap_side: first_spacer_entry_side,
                                    cross_axis_coord: spacer_cross,
                                    face_offset: 0.0,
                                    rank_gap_px: gap_px,
                                });
                        }

                        // --- Exit side of this spacer --- //
                        //
                        // The exit side protrudes into the gap AFTER
                        // this spacer.
                        let last_spacer_idx = spacer_coordinates.len() - 1;
                        if spacer_idx < last_spacer_idx {
                            // Gap between this spacer and the next one.
                            let next_spacer = &spacer_coordinates[spacer_idx + 1];
                            let gap_px =
                                Self::spacer_gap_px(spacer, next_spacer, from_face_for_axis);

                            let exit_gap_key = Self::spacer_gap_key(
                                rank_from,
                                rank_to,
                                spacer_idx,
                                spacer_idx + 1,
                            );

                            let spacer_exit_side = if rank_from < rank_to {
                                GapSide::Low
                            } else {
                                GapSide::High
                            };

                            rank_gap_entries
                                .entry(exit_gap_key)
                                .or_default()
                                .push(RankGapEntry {
                                    pass1_group_index: group_idx,
                                    edge_index: edge_idx,
                                    endpoint_kind: RankGapEndpointKind::SpacerExit {
                                        spacer_index: spacer_idx,
                                    },
                                    gap_side: spacer_exit_side,
                                    cross_axis_coord: spacer_cross,
                                    face_offset: 0.0,
                                    rank_gap_px: gap_px,
                                });
                        } else if let Some(to_face) = pass1_info.to_face {
                            // Last spacer: exit shares the same gap as
                            // the to-endpoint, but protrudes from the
                            // opposite side. Register it so it competes
                            // with other edges' from-endpoints at this
                            // spacer's rank.
                            let gap_px = Self::rank_gap_px(
                                pass1_info,
                                to_face,
                                false,
                                svg_node_info_map,
                                &spacer_coordinates,
                            );

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

                            // The last spacer's exit is on the
                            // opposite side from the to-endpoint.
                            let last_spacer_exit_side = if rank_from < rank_to {
                                GapSide::Low
                            } else {
                                GapSide::High
                            };

                            rank_gap_entries
                                .entry(to_gap_key)
                                .or_default()
                                .push(RankGapEntry {
                                    pass1_group_index: group_idx,
                                    edge_index: edge_idx,
                                    endpoint_kind: RankGapEndpointKind::SpacerExit {
                                        spacer_index: last_spacer_idx,
                                    },
                                    gap_side: last_spacer_exit_side,
                                    cross_axis_coord: spacer_cross,
                                    face_offset: 0.0,
                                    rank_gap_px: gap_px,
                                });
                        }
                    }
                }
            }
        }

        // === Step 3: For each rank gap, assign protrusion depths === //
        for (_gap_key, entries) in &mut rank_gap_entries {
            Self::protrusions_assign(entries, &mut result);
        }

        // === Step 4: Propagate node protrusions to shared spacer sides (fallback) ===
        // //
        //
        // The first spacer's entry and last spacer's exit are now
        // registered as separate rank-gap entries and assigned their
        // own protrusion depths in Step 3. This propagation only acts
        // as a safety net for edges where a face was not available
        // (e.g. `from_face` or `to_face` was `None`), which prevented
        // registration of the spacer side in Step 2.
        for group_params in &mut result {
            for params in group_params.iter_mut() {
                if params.spacer_protrusions.is_empty() {
                    continue;
                }

                let first = &mut params.spacer_protrusions[0];
                if first.entry_protrusion < 1e-3 {
                    first.entry_protrusion = params.from_protrusion;
                }

                let last_idx = params.spacer_protrusions.len() - 1;
                let last = &mut params.spacer_protrusions[last_idx];
                if last.exit_protrusion < 1e-3 {
                    last.exit_protrusion = params.to_protrusion;
                }
            }
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
    /// 3. Partition entries into `Low` side and `High` side groups.
    /// 4. Identify "crossing edges" -- edges that have entries on both sides of
    ///    the gap. For these edges, the orthogonal routing segment's
    ///    y-coordinate is the midpoint of the two protrusion endpoints: `mid =
    ///    (face_low + low_prot + face_high - high_prot) / 2`. The `low_prot -
    ///    high_prot` difference must be unique per crossing edge so that
    ///    routing segments do not overlap.
    /// 5. Assign crossing-edge protrusion pairs first, choosing `low_prot -
    ///    high_prot` differences from a set of distinct values.
    /// 6. Assign remaining single-side entries from the unused protrusion
    ///    slots.
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

        let total_count = rank_gap_entries.len();
        if total_count == 1 {
            let protrusion = (max_protrusion * 0.5).max(MIN_PROTRUSION_PX);
            Self::protrusion_write(&rank_gap_entries[0], protrusion, result);
            return;
        }

        // === Partition by side and sort each side === //

        let side_sort = |a: &&RankGapEntry, b: &&RankGapEntry| -> std::cmp::Ordering {
            let offset_cmp = a
                .face_offset
                .abs()
                .partial_cmp(&b.face_offset.abs())
                .unwrap_or(std::cmp::Ordering::Equal)
                .reverse();
            if offset_cmp != std::cmp::Ordering::Equal {
                return offset_cmp;
            }
            a.cross_axis_coord
                .partial_cmp(&b.cross_axis_coord)
                .unwrap_or(std::cmp::Ordering::Equal)
        };

        let (mut low_entries, mut high_entries): (Vec<&RankGapEntry>, Vec<&RankGapEntry>) =
            rank_gap_entries
                .iter()
                .partition(|entry| entry.gap_side == GapSide::Low);

        low_entries.sort_by(side_sort);
        high_entries.sort_by(side_sort);

        // === Identify crossing edges === //
        //
        // A "crossing edge" has entries on BOTH sides of the gap. Its
        // orthogonal routing segment sits at the midpoint of its two
        // protrusion endpoints:
        //
        //   mid_y = (y_low_face + low_prot + y_high_face - high_prot) / 2
        //
        // Since `y_low_face` and `y_high_face` are approximately the
        // same for all edges in the same gap, the midpoint is unique
        // when `low_prot - high_prot` is unique. We assign crossing
        // edges distinct `low_prot - high_prot` differences.

        // Edge identity key: (pass1_group_index, edge_index).
        type EdgeKey = (usize, usize);

        // Build maps from edge key to sorted index within each side.
        let mut low_edge_indices: Map<EdgeKey, usize> = Map::new();
        for (i, entry) in low_entries.iter().enumerate() {
            low_edge_indices.insert((entry.pass1_group_index, entry.edge_index), i);
        }
        let mut high_edge_indices: Map<EdgeKey, usize> = Map::new();
        for (j, entry) in high_entries.iter().enumerate() {
            high_edge_indices.insert((entry.pass1_group_index, entry.edge_index), j);
        }

        // Collect crossing edges: edges present on both sides.
        // Store (low_side_index, high_side_index) pairs.
        let mut crossing_pairs: Vec<(usize, usize)> = Vec::new();
        for (edge_key, &low_idx) in &low_edge_indices {
            if let Some(&high_idx) = high_edge_indices.get(edge_key) {
                crossing_pairs.push((low_idx, high_idx));
            }
        }

        // Sort crossing pairs for deterministic assignment.
        crossing_pairs.sort();

        // === Assign protrusion depths === //
        //
        // Strategy: distribute all entries across a shared pool of
        // `total_count` distinct protrusion slots. To ensure crossing
        // edges get unique `low_prot - high_prot` differences, we
        // assign their (low, high) slot pairs such that no two
        // crossing edges share the same slot difference.
        //
        // Single-side entries (only on low or only on high) do not
        // contribute to routing midpoints in this gap, so they just
        // need unique slots.
        //
        // We use `total_count` evenly-spaced slots in
        // [MIN_PROTRUSION_PX, max_protrusion]:
        //   slot[k] = MIN + k * growable / (total_count - 1)
        //
        // The assignment proceeds as follows:
        //
        // 1. Single-side low entries get the lowest slots (0, 1, ...).
        // 2. Crossing low entries get the next slots.
        // 3. Crossing high entries get slots in REVERSE order (from the top of the
        //    high-side allocation). This ensures that the i-th crossing edge's low slot
        //    is `single_low_count + i` and its high slot is `total_count - 1 - i`,
        //    giving a difference of `(single_low_count + i) - (total_count - 1 - i)` =
        //    `single_low_count + 2i - total_count + 1`, which is unique per `i` (since
        //    the `2i` term varies).
        // 4. Single-side high entries fill the remaining slots.

        // Separate low_entries into single-side and crossing, preserving
        // their sorted order.
        let crossing_low_set: std::collections::HashSet<usize> =
            crossing_pairs.iter().map(|&(li, _)| li).collect();
        let crossing_high_set: std::collections::HashSet<usize> =
            crossing_pairs.iter().map(|&(_, hi)| hi).collect();

        let single_low: Vec<&RankGapEntry> = low_entries
            .iter()
            .enumerate()
            .filter(|(i, _)| !crossing_low_set.contains(i))
            .map(|(_, e)| *e)
            .collect();
        let single_high: Vec<&RankGapEntry> = high_entries
            .iter()
            .enumerate()
            .filter(|(j, _)| !crossing_high_set.contains(j))
            .map(|(_, e)| *e)
            .collect();

        // Crossing entries in the order determined by crossing_pairs.
        let crossing_low_ordered: Vec<&RankGapEntry> = crossing_pairs
            .iter()
            .map(|&(li, _)| low_entries[li])
            .collect();
        let crossing_high_ordered: Vec<&RankGapEntry> = crossing_pairs
            .iter()
            .map(|&(_, hi)| high_entries[hi])
            .collect();

        let protrusion_growable_space = max_protrusion - MIN_PROTRUSION_PX;
        let denominator = (total_count - 1).max(1) as f32;

        let slot_value = |slot: usize| -> f32 {
            let proportion = slot as f32 / denominator;
            MIN_PROTRUSION_PX + proportion * protrusion_growable_space
        };

        // Slot assignment:
        //   [0 .. SL)                        -> single-side low
        //   [SL .. SL + NC)                  -> crossing low
        //   [total - SH - NC .. total - SH)  -> crossing high (reversed)
        //   [total - SH .. total)            -> single-side high
        //
        // where SL = single_low.len(), SH = single_high.len(),
        //       NC = crossing_pairs.len().

        let mut slot = 0usize;

        // 1. Single-side low entries.
        for entry in &single_low {
            Self::protrusion_write(entry, slot_value(slot), result);
            slot += 1;
        }

        // 2. Crossing low entries (in crossing_pairs order).

        for entry in &crossing_low_ordered {
            Self::protrusion_write(entry, slot_value(slot), result);
            slot += 1;
        }

        // 3. Crossing high entries -- assigned in REVERSE slot order so that crossing
        //    pair i gets: low_slot  = single_low.len() + i high_slot = total_count - 1
        //    - single_high.len() - i The difference low_slot - high_slot changes by +2
        //    per i, guaranteeing uniqueness.
        let crossing_high_top = total_count - single_high.len();
        for (i, entry) in crossing_high_ordered.iter().enumerate() {
            let high_slot = crossing_high_top - 1 - i;
            Self::protrusion_write(entry, slot_value(high_slot), result);
        }

        // 4. Single-side high entries fill the top slots.
        let single_high_start = total_count - single_high.len();
        for (i, entry) in single_high.iter().enumerate() {
            Self::protrusion_write(entry, slot_value(single_high_start + i), result);
        }
    }

    /// Writes a protrusion value to the appropriate slot in `result`.
    fn protrusion_write(
        entry: &RankGapEntry,
        protrusion: f32,
        result: &mut [Vec<OrthoProtrusionParams>],
    ) {
        let params = &mut result[entry.pass1_group_index][entry.edge_index];
        match entry.endpoint_kind {
            RankGapEndpointKind::FromEndpoint => {
                params.from_protrusion = protrusion;
            }
            RankGapEndpointKind::ToEndpoint => {
                params.to_protrusion = protrusion;
            }
            RankGapEndpointKind::SpacerEntry { spacer_index } => {
                if let Some(sp) = params.spacer_protrusions.get_mut(spacer_index) {
                    sp.entry_protrusion = protrusion;
                }
            }
            RankGapEndpointKind::SpacerExit { spacer_index } => {
                if let Some(sp) = params.spacer_protrusions.get_mut(spacer_index) {
                    sp.exit_protrusion = protrusion;
                }
            }
        }
    }

    /// Computes the pixel distance between two consecutive spacers
    /// along the rank axis.
    ///
    /// The distance is measured from the exit of `spacer_before` to
    /// the entry of `spacer_after`, using the same axis as
    /// `axis_distance`.
    fn spacer_gap_px(
        spacer_before: &SpacerCoordinates,
        spacer_after: &SpacerCoordinates,
        face: NodeFace,
    ) -> f32 {
        Self::axis_distance(
            spacer_before.exit_x,
            spacer_before.exit_y,
            spacer_after.entry_x,
            spacer_after.entry_y,
            face,
        )
    }

    /// Computes the `RankGapKey` for the gap between two consecutive
    /// spacers.
    ///
    /// The spacers are at logical positions between `rank_from` and
    /// `rank_to`. Spacer indices are 0-based. The rank for spacer `i`
    /// is interpolated between the from and to ranks.
    ///
    /// # Parameters
    ///
    /// * `rank_from` -- rank of the from-node.
    /// * `rank_to` -- rank of the to-node.
    /// * `spacer_idx_low` -- index of the earlier spacer (in forward order).
    /// * `spacer_idx_high` -- index of the later spacer.
    fn spacer_gap_key(
        rank_from: NodeRank,
        rank_to: NodeRank,
        spacer_idx_low: usize,
        spacer_idx_high: usize,
    ) -> RankGapKey {
        // Map spacer indices to intermediate ranks. Spacers occupy
        // ranks between rank_from and rank_to. For an edge from rank 0
        // to rank 3, spacers are at ranks 1 and 2:
        //   spacer_idx 0 -> rank 1
        //   spacer_idx 1 -> rank 2
        let (low_rank_val, high_rank_val) = if rank_from < rank_to {
            (
                rank_from.value() + 1 + spacer_idx_low as u32,
                rank_from.value() + 1 + spacer_idx_high as u32,
            )
        } else {
            // Reversed direction: from higher rank to lower rank.
            // spacer_idx 0 is closest to rank_from (the higher rank).
            let from_val = rank_from.value();
            (
                from_val - 1 - spacer_idx_high as u32,
                from_val - 1 - spacer_idx_low as u32,
            )
        };

        let rank_a = NodeRank::new(low_rank_val);
        let rank_b = NodeRank::new(high_rank_val);

        if rank_a <= rank_b {
            RankGapKey {
                rank_low: rank_a,
                rank_high: rank_b,
            }
        } else {
            RankGapKey {
                rank_low: rank_b,
                rank_high: rank_a,
            }
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

        let rank_spacers: Vec<(NodeRank, SpacerCoordinates)> = spacer_nodes
            .rank_to_spacer_taffy_node_id
            .iter()
            .filter_map(|(rank, &taffy_node_id)| {
                let coords = Self::spacer_absolute_coordinates(taffy_tree, taffy_node_id)?;
                Some((*rank, coords))
            })
            .collect();

        // Collect cross-container spacer coordinates.
        let cross_container_spacers: Vec<SpacerCoordinates> = spacer_nodes
            .cross_container_spacer_taffy_node_ids
            .iter()
            .filter_map(|&taffy_node_id| {
                Self::spacer_absolute_coordinates(taffy_tree, taffy_node_id)
            })
            .collect();

        if cross_container_spacers.is_empty() {
            // Fast path: only rank-based spacers -- sort by rank as before.
            let mut rank_spacers = rank_spacers;
            rank_spacers.sort_by_key(|(rank, _)| *rank);
            return rank_spacers.into_iter().map(|(_, coords)| coords).collect();
        }

        // Merge both kinds and sort by absolute y-coordinate so the
        // spacers appear in the correct visual order along the edge path.
        let mut all_spacers: Vec<SpacerCoordinates> = rank_spacers
            .into_iter()
            .map(|(_, coords)| coords)
            .chain(cross_container_spacers)
            .collect();

        all_spacers.sort_by(|a, b| {
            a.entry_y
                .partial_cmp(&b.entry_y)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        all_spacers
    }

    /// Computes absolute spacer coordinates for a single taffy node.
    ///
    /// Walks up the taffy tree to accumulate the absolute position, then
    /// returns `SpacerCoordinates` with the entry at the top midpoint
    /// and the exit at the bottom midpoint.
    fn spacer_absolute_coordinates(
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        taffy_node_id: taffy::NodeId,
    ) -> Option<SpacerCoordinates> {
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

        Some(SpacerCoordinates {
            entry_x: cx,
            entry_y: top_y,
            exit_x: cx,
            exit_y: bottom_y,
        })
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
