use disposition_ir_model::{
    entity::EntityTypes,
    node::{NodeId, NodeNestingInfos, NodeRank, NodeRanksNested},
};
use disposition_model_common::{entity::EntityType, Map, RankDir};
use disposition_svg_model::{OrthoProtrusionParams, SpacerProtrusionParams, SvgNodeInfo};
use disposition_taffy_model::{taffy::TaffyTree, EdgeIdToEdgeSpacerTaffyNodes, TaffyNodeCtx};

use disposition_ir_model::node::NodeFace;

use crate::taffy_to_svg_elements_mapper::{
    edge_model::{NodeIdAndFace, NodeIdAndFaceToContactPointOffsets},
    edge_path_builder_pass_1::SpacerCoordinates,
    SpacerCoordinatesResolver, SvgNodeInfoByNodeId,
};

use self::geometry::OrthoProtrusionGeometry;

mod geometry;

use super::svg_edge_infos_builder::{EdgeGroupPass1, EdgePass1Info};

/// Maximum fraction of the rank gap that a protrusion may occupy.
///
/// # Example values
///
/// `0.6` -- each side (from and to) may use up to 60% of the gap.
const MAX_GAP_FRACTION: f32 = 0.6;

/// Minimum protrusion length in pixels.
///
/// When an edge is not perfectly straight (i.e. the from and to
/// contact points differ on the cross-axis), the protrusion is at
/// least this many pixels so the perpendicular stub is visible.
const MIN_PROTRUSION_PX: f32 = 3.0;

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
///    coordinate perpendicular to the rank direction). Earlier edges (those
///    further from the centre of the gap's cross-axis spread) get longer
///    protrusions; later edges (closer to the centre) get shorter ones. This
///    reduces visual cross-over between edge paths.
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
    /// receive longer protrusions. For spacer endpoints this is
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

/// The high-level category of a node for grouping protrusion calculations.
///
/// Protrusions and spacer computations are independent per category:
/// thing-node edges only consider other thing nodes when computing rank
/// gap boundaries and sibling extents; tag-node edges only consider tag nodes;
/// process-node edges only consider process and process step nodes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NodeCategory {
    Thing,
    Tag,
    Process,
    Other,
}

impl OrthoProtrusionCalculator {
    /// Calculates protrusion parameters for every edge in every group.
    ///
    /// Returns a `Vec` parallel to `all_pass1_groups`, where each
    /// inner `Vec` is parallel to the group's `pass1_infos`.
    ///
    /// # Parameters
    ///
    /// * `all_pass1_groups`: all edge groups with pass-1 metadata.
    /// * `face_offsets_by_node_face`: precomputed per-face offset vectors from
    ///   `face_offsets_compute`.
    /// * `svg_node_info_map`: node layout information.
    /// * `taffy_tree`: the layout tree (for spacer coordinate lookups).
    /// * `edge_spacer_taffy_nodes`: spacer node mappings per edge.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn calculate<'id>(
        rank_dir: RankDir,
        all_pass1_groups: &[EdgeGroupPass1<'_, 'id>],
        from_slot_indices_all: &[Vec<Option<usize>>],
        to_slot_indices_all: &[Vec<Option<usize>>],
        face_offsets_by_node_face: &NodeIdAndFaceToContactPointOffsets<'id>,
        svg_node_info_map: &SvgNodeInfoByNodeId<'_, 'id>,
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        edge_spacer_taffy_nodes: &EdgeIdToEdgeSpacerTaffyNodes<'id>,
        node_nesting_infos: &NodeNestingInfos<'id>,
        node_ranks_nested: &NodeRanksNested<'id>,
        entity_types: &EntityTypes<'id>,
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
                        SpacerCoordinatesResolver::resolve(
                            rank_dir,
                            &pass1_info.edge_id,
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
                let (_rank_low, _rank_high) = if rank_from <= rank_to {
                    (rank_from, rank_to)
                } else {
                    (rank_to, rank_from)
                };

                // Cycle edges: register their endpoints in the adjacent rank
                // gap so protrusion depths are distributed proportionally.
                // Left/Right face pairs fall through to the MIN_PROTRUSION_PX
                // safety net in Step 6.
                if pass1_info.is_cycle_edge {
                    Self::cycle_edge_collect_rank_gap_entries(
                        group_idx,
                        edge_idx,
                        pass1_info,
                        from_slot_indices[edge_idx],
                        face_offsets_by_node_face,
                        svg_node_info_map,
                        node_nesting_infos,
                        node_ranks_nested,
                        entity_types,
                        &mut rank_gap_entries,
                    );
                    continue;
                }

                // Same-rank non-cycle edges (adjacent siblings, tag/process
                // nodes) use normal face routing with zero protrusion and do
                // not register rank-gap entries.
                if rank_from == rank_to {
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
                        spacer_coordinates,
                        node_nesting_infos,
                    );

                    let cross_axis_from = OrthoProtrusionGeometry::cross_axis_coord(
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
                        spacer_coordinates,
                        node_nesting_infos,
                    );

                    let cross_axis_to =
                        OrthoProtrusionGeometry::cross_axis_coord(pass1_info.to_node_x, pass1_info.to_node_y, to_face);

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
                        let spacer_cross = OrthoProtrusionGeometry::cross_axis_coord(
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
                                spacer_coordinates,
                                node_nesting_infos,
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
                                spacer_coordinates,
                                node_nesting_infos,
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
        rank_gap_entries
            .values_mut()
            .for_each(|entries| Self::protrusions_assign(entries, &mut result));

        // === Step 4: Propagate node protrusions to shared spacer sides (fallback) ===
        // //
        //
        // The first spacer's entry and last spacer's exit are now
        // registered as separate rank-gap entries and assigned their
        // own protrusion depths in Step 3. This propagation only acts
        // as a safety net for edges where a face was not available
        // (e.g. `from_face` or `to_face` was `None`), which prevented
        // registration of the spacer side in Step 2.
        result
            .iter_mut()
            .flat_map(|group_params| group_params.iter_mut())
            .for_each(|params| {
                if params.spacer_protrusions.is_empty() {
                    return;
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
            });

        // === Step 5: Enforce minimum protrusions to clear divergent ancestor siblings
        // === //
        //
        // For edges where the from/to nodes are at different nesting levels,
        // the protrusion for each endpoint must be large enough to clear all
        // sibling nodes of the endpoint's Divergent ancestor at the LCA level.
        //
        // The "Divergent ancestor" of a node is the ancestor that is a direct
        // child of the LCA of the from and to nodes for this edge.
        //
        // Note: the TO endpoint adjustment is skipped when the edge has
        // cross-container spacers. In that case the spacer already handles
        // the routing inside the container, so the to_protrusion only needs
        // to reach the spacer exit (not exit the container entirely).
        Self::protrusions_adjust_for_divergent_siblings(
            all_pass1_groups,
            &all_spacer_coordinates,
            node_nesting_infos,
            node_ranks_nested,
            svg_node_info_map,
            entity_types,
            &mut result,
        );

        // === Step 6: Finalise protrusion depths for cycle edges === //
        //
        // For cycle edges registered in the adjacent rank gap (Step 2), equalise
        // from and to protrusions to produce symmetric U-shaped arcs.
        //
        // For unregistered cycle edges (boundary ranks or Left/Right faces),
        // group by routing direction and assign stacked depths
        // `(N * MIN_PROTRUSION_PX down to 1 * MIN_PROTRUSION_PX)` so that edges in
        // the same group do not overlap.
        Self::protrusions_assign_cycle_edges(
            all_pass1_groups,
            from_slot_indices_all,
            face_offsets_by_node_face,
            &mut result,
        );

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
    ///    the gap.
    /// 5. Assign protrusion depths from a pool of evenly-spaced slots. Each
    ///    side's crossing entries are sorted independently by that side's
    ///    spatial ordering (face offset then cross-axis coordinate), so that
    ///    edges sorted earlier on each side receive longer protrusions,
    ///    reducing visual cross-over near both the from-nodes and to-nodes.
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
            // Sort by signed face_offset ascending so that edges on
            // the negative side of the face (e.g. left of center for
            // Top/Bottom faces) come first, centre next, positive
            // last. This preserves the spatial ordering established
            // by `face_entries_sort_by_rank_and_coordinate`, ensuring
            // that edges whose contact points are further apart
            // receive longer protrusions and edges closer together
            // receive shorter ones, preventing visual cross-over.
            let offset_cmp = a
                .face_offset
                .partial_cmp(&b.face_offset)
                .unwrap_or(std::cmp::Ordering::Equal);
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
        // A "crossing edge" has entries on BOTH sides of the gap.
        // Crossing edges are assigned protrusion depths on each side
        // independently according to that side's spatial ordering, so
        // that edges sorted earlier on each side receive longer
        // protrusions.

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
        // `total_count` distinct protrusion slots.
        //
        // Single-side entries (only on low or only on high) do not
        // contribute to routing midpoints in this gap, so they just
        // need unique slots.
        //
        // We use `total_count` evenly-spaced slots in
        // [MIN_PROTRUSION_PX, max_protrusion]:
        //   slot[k] = MAX - k * growable / (total_count - 1)
        //
        // Slot 0 gets the longest protrusion, last slot gets the
        // shortest. Earlier edges (sorted first) receive longer
        // protrusions for visual clarity (less cross-over).
        //
        // Crossing low and crossing high entries are sorted
        // independently by each side's own spatial ordering (face
        // offset then cross-axis coordinate). Both sides assign
        // slots in forward order so that earlier entries on each
        // side receive longer protrusions. This prevents the
        // high-side protrusion ordering from being dictated by the
        // low-side sort, which would cause later edges on the high
        // side to receive incorrectly long protrusions.
        //
        // The assignment proceeds as follows:
        //
        // 1. Single-side low entries get the first slots (0, 1, ...) -- longest
        //    protrusions.
        // 2. Crossing low entries (from-endpoints) get the next slots.
        // 3. Crossing high entries (to-endpoints) get slots in forward order within
        //    their own range, sorted by high-side index. Earlier entries on the high
        //    side receive longer protrusions, matching the low-side convention.
        // 4. Single-side high entries fill the remaining slots -- shortest protrusions.

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

        // Crossing low entries follow crossing_pairs order (sorted by
        // low-side index) so that the low side respects its own spatial
        // ordering.
        let crossing_low_ordered: Vec<&RankGapEntry> = crossing_pairs
            .iter()
            .map(|&(li, _)| low_entries[li])
            .collect();

        // Crossing high entries are sorted by their high-side index
        // (i.e. the order produced by `side_sort` on the high side)
        // so that the high-side protrusion assignment respects the
        // high-side spatial ordering. Without this, the high-side
        // slots follow low-side ordering, which can cause later edges
        // (further out on the high side) to receive longer protrusions
        // than earlier edges, leading to visually crossing paths.
        let mut crossing_high_indices: Vec<usize> =
            crossing_pairs.iter().map(|&(_, hi)| hi).collect();
        crossing_high_indices.sort_unstable();
        let crossing_high_ordered: Vec<&RankGapEntry> = crossing_high_indices
            .iter()
            .map(|&hi| high_entries[hi])
            .collect();

        let protrusion_growable_space = max_protrusion - MIN_PROTRUSION_PX;
        let denominator = (total_count - 1).max(1) as f32;

        let slot_value = |slot: usize| -> f32 {
            // Slot 0 gets the longest protrusion, last slot gets the
            // shortest. Earlier edges (sorted first) receive longer
            // protrusions for visual clarity (less cross-over).
            let proportion = 1.0 - (slot as f32 / denominator);
            MIN_PROTRUSION_PX + proportion * protrusion_growable_space
        };

        // Slot assignment:
        //   [0 .. SL)                        -> single-side low
        //   [SL .. SL + NC)                  -> crossing low (from-endpoints)
        //   [total - SH - NC .. total - SH)  -> crossing high (to-endpoints, forward)
        //   [total - SH .. total)            -> single-side high
        //
        // where SL = single_low.len(), SH = single_high.len(),
        //       NC = crossing_pairs.len().

        // 1. Single-side low entries, then 2. crossing low entries
        //    (from-endpoints, in crossing_pairs order): assigned the lowest
        //    slots in sequence.
        single_low
            .iter()
            .chain(crossing_low_ordered.iter())
            .enumerate()
            .for_each(|(slot, entry)| {
                Self::protrusion_write(entry, slot_value(slot), result);
            });

        // 3. Crossing high entries (to-endpoints) -- assigned in FORWARD slot order.
        //    The entries are sorted by high-side index so that spatially earlier edges
        //    (sorted first on the high side) get longer protrusions and later edges get
        //    shorter ones, matching the low-side convention and preventing visual
        //    crossings near the to-nodes.
        let crossing_high_start = total_count - single_high.len() - crossing_high_ordered.len();
        crossing_high_ordered.iter().enumerate().for_each(|(i, entry)| {
            Self::protrusion_write(entry, slot_value(crossing_high_start + i), result);
        });

        // 4. Single-side high entries fill the top slots.
        let single_high_start = total_count - single_high.len();
        single_high.iter().enumerate().for_each(|(i, entry)| {
            Self::protrusion_write(entry, slot_value(single_high_start + i), result);
        });
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
        OrthoProtrusionGeometry::axis_distance(
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
    /// * `rank_from`: rank of the from-node.
    /// * `rank_to`: rank of the to-node.
    /// * `spacer_idx_low`: index of the earlier spacer (in forward order).
    /// * `spacer_idx_high`: index of the later spacer.
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
        face_offsets_by_node_face: &NodeIdAndFaceToContactPointOffsets<'id>,
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
    /// to-node's face center if there are no spacers), capped at the
    /// distance to the other node's divergent ancestor boundary.
    ///
    /// For the "to" endpoint, this is the distance from the to-node's
    /// face center to the last spacer exit (or the from-node's face
    /// center if there are no spacers), capped at the distance to the
    /// other node's divergent ancestor boundary.
    ///
    /// Capping at the divergent ancestor boundary prevents the
    /// slot-assigned protrusion from exceeding the actual inter-rank
    /// gap when nodes are deeply nested inside containers. Without the
    /// cap, the protrusion tip can land at or past the destination
    /// container boundary, and combined with the divergent-sibling
    /// adjustment on the opposite endpoint, both protrusion tips end
    /// up at the same coordinate. This suppresses proper Z/S bends and
    /// arc-rounded corners.
    fn rank_gap_px<'id>(
        pass1_info: &EdgePass1Info<'_, 'id>,
        face: NodeFace,
        is_from: bool,
        svg_node_info_map: &SvgNodeInfoByNodeId<'_, 'id>,
        spacer_coordinates: &[SpacerCoordinates],
        node_nesting_infos: &NodeNestingInfos<'id>,
    ) -> f32 {
        // Get node coordinates.
        let from_info = svg_node_info_map.get(&pass1_info.edge.from);
        let to_info = svg_node_info_map.get(&pass1_info.edge.to);

        let (from_x, from_y) = from_info
            .map(|info| OrthoProtrusionGeometry::face_center(info, pass1_info.from_face.unwrap_or(NodeFace::Bottom)))
            .unwrap_or((pass1_info.from_node_x, pass1_info.from_node_y));

        let (to_x, to_y) = to_info
            .map(|info| OrthoProtrusionGeometry::face_center(info, pass1_info.to_face.unwrap_or(NodeFace::Top)))
            .unwrap_or((pass1_info.to_node_x, pass1_info.to_node_y));

        let full_dist = if is_from {
            if spacer_coordinates.is_empty() {
                OrthoProtrusionGeometry::axis_distance(from_x, from_y, to_x, to_y, face)
            } else {
                let first_spacer = &spacer_coordinates[0];
                OrthoProtrusionGeometry::axis_distance(
                    from_x,
                    from_y,
                    first_spacer.entry_x,
                    first_spacer.entry_y,
                    face,
                )
            }
        } else {
            if spacer_coordinates.is_empty() {
                OrthoProtrusionGeometry::axis_distance(to_x, to_y, from_x, from_y, face)
            } else {
                let last_spacer = &spacer_coordinates[spacer_coordinates.len() - 1];
                OrthoProtrusionGeometry::axis_distance(to_x, to_y, last_spacer.exit_x, last_spacer.exit_y, face)
            }
        };

        // Cap at the boundary of the other divergent ancestor.
        //
        // When from/to nodes are deeply nested inside containers, the
        // full face-to-face distance inflates `max_protrusion` beyond
        // the actual inter-rank gap. The slot-assigned protrusion can
        // then exceed the gap distance, and combined with the
        // divergent-sibling adjustment on the opposite endpoint, both
        // protrusion tips land at the same coordinate. This suppresses
        // the proper Z/S bend and its arc-rounded corners.
        //
        // The fix: cap the distance at the boundary of the other
        // node's divergent ancestor facing this node. For a Bottom-face
        // protrusion, this is the Top face of the other divergent
        // ancestor. This limits `max_protrusion` to the inter-rank gap
        // between the two outermost containers.
        let (this_node_id, other_node_id) = if is_from {
            (&pass1_info.edge.from, &pass1_info.edge.to)
        } else {
            (&pass1_info.edge.to, &pass1_info.edge.from)
        };
        let lca_depth = Self::lca_depth(this_node_id, other_node_id, node_nesting_infos);
        if let Some(other_div_ancestor_id) =
            Self::divergent_ancestor_id(other_node_id, lca_depth, node_nesting_infos)
            && let Some(&other_ancestor_info) = svg_node_info_map.get(other_div_ancestor_id)
        {
            let (this_face_x, this_face_y) = if is_from {
                (from_x, from_y)
            } else {
                (to_x, to_y)
            };
            // The other ancestor's face pointing toward this node
            // is the opposite of the protrusion direction.
            let other_opposite_face = match face {
                NodeFace::Bottom => NodeFace::Top,
                NodeFace::Top => NodeFace::Bottom,
                NodeFace::Right => NodeFace::Left,
                NodeFace::Left => NodeFace::Right,
            };
            let (other_bx, other_by) = OrthoProtrusionGeometry::face_center(other_ancestor_info, other_opposite_face);
            let capped = OrthoProtrusionGeometry::axis_distance(this_face_x, this_face_y, other_bx, other_by, face);
            return full_dist.min(capped);
        }
        full_dist
    }

    /// Finalises protrusion depths for same-rank (cycle) edges.
    ///
    /// After gap-based protrusion assignment in Step 2–3, some cycle edges may
    /// have a `from_protrusion` assigned (those whose `Top`/`Bottom` face
    /// registered in an adjacent rank gap), while others still have zero
    /// (boundary-rank edges, or `Left`/`Right` face edges with no rank gap).
    ///
    /// This step handles both:
    ///
    /// 1. **Registered cycle edges** (`from_protrusion > 0`): copies
    ///    `from_protrusion` to `to_protrusion` so both endpoints protrude
    ///    equally, creating a symmetric U-shaped routing arc. Applies
    ///    `MIN_PROTRUSION_PX` as a floor.
    ///
    /// 2. **Unregistered cycle edges** (`from_protrusion == 0`): groups edges
    ///    by `(from_face, rank_from)` -- all edges routing in the same
    ///    direction at the same rank. Within each group, sorts by face offset
    ///    then cross-axis coordinate (same ordering as `protrusions_assign` for
    ///    single-side entries). Assigns stacked depths:
    ///    - N edges in group -> depths `[N * MIN, (N-1) * MIN, .., MIN]`
    ///    - Sets `from_protrusion = to_protrusion = depth` for each edge.
    fn protrusions_assign_cycle_edges<'id>(
        all_pass1_groups: &[EdgeGroupPass1<'_, 'id>],
        from_slot_indices_all: &[Vec<Option<usize>>],
        face_offsets_by_node_face: &NodeIdAndFaceToContactPointOffsets<'id>,
        result: &mut [Vec<OrthoProtrusionParams>],
    ) {
        struct UnregisteredEntry {
            group_idx: usize,
            edge_idx: usize,
            face_offset: f32,
            cross_axis: f32,
            from_face: NodeFace,
            rank_from: NodeRank,
        }

        let mut unregistered: Vec<UnregisteredEntry> = Vec::new();

        for (group_idx, group) in all_pass1_groups.iter().enumerate() {
            let from_slot_indices = &from_slot_indices_all[group_idx];
            for (edge_idx, pass1_info) in group.pass1_infos.iter().enumerate() {
                // Only cycle edges with a valid face.
                if !pass1_info.is_cycle_edge {
                    continue;
                }
                let Some(from_face) = pass1_info.from_face else {
                    continue;
                };

                let params = &mut result[group_idx][edge_idx];

                if params.from_protrusion > 0.0 {
                    // Registered in adjacent rank gap: equalize from and to, apply
                    // MIN floor.
                    let depth = params.from_protrusion.max(MIN_PROTRUSION_PX);
                    params.from_protrusion = depth;
                    params.to_protrusion = depth;
                } else {
                    // Unregistered: collect for group stacking.
                    let face_offset = Self::face_offset_resolve(
                        pass1_info,
                        from_slot_indices[edge_idx],
                        true,
                        face_offsets_by_node_face,
                    );
                    let cross_axis = OrthoProtrusionGeometry::cross_axis_coord(
                        pass1_info.from_node_x,
                        pass1_info.from_node_y,
                        from_face,
                    );
                    unregistered.push(UnregisteredEntry {
                        group_idx,
                        edge_idx,
                        face_offset,
                        cross_axis,
                        from_face,
                        rank_from: pass1_info.rank_from,
                    });
                }
            }
        }

        if unregistered.is_empty() {
            return;
        }

        // Sort: group by (face discriminant, rank_from), then within group by
        // (face_offset ascending, cross_axis ascending). This produces the same
        // ordering as `protrusions_assign` for single-side entries.
        unregistered.sort_by(|a, b| {
            let face_a = match a.from_face {
                NodeFace::Top => 0u8,
                NodeFace::Bottom => 1,
                NodeFace::Left => 2,
                NodeFace::Right => 3,
            };
            let face_b = match b.from_face {
                NodeFace::Top => 0u8,
                NodeFace::Bottom => 1,
                NodeFace::Left => 2,
                NodeFace::Right => 3,
            };
            let face_cmp = face_a.cmp(&face_b);
            if face_cmp != std::cmp::Ordering::Equal {
                return face_cmp;
            }
            let rank_cmp = a.rank_from.cmp(&b.rank_from);
            if rank_cmp != std::cmp::Ordering::Equal {
                return rank_cmp;
            }
            let off_cmp = a
                .face_offset
                .partial_cmp(&b.face_offset)
                .unwrap_or(std::cmp::Ordering::Equal);
            if off_cmp != std::cmp::Ordering::Equal {
                return off_cmp;
            }
            a.cross_axis
                .partial_cmp(&b.cross_axis)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Assign stacked depths within each group.
        let mut group_start = 0;
        while group_start < unregistered.len() {
            // Find the end of the current group (same face + rank).
            let group_end = {
                let key_face = unregistered[group_start].from_face;
                let key_rank = unregistered[group_start].rank_from;
                unregistered[group_start..]
                    .iter()
                    .position(|e| e.from_face != key_face || e.rank_from != key_rank)
                    .map(|rel| group_start + rel)
                    .unwrap_or(unregistered.len())
            };

            let group = &unregistered[group_start..group_end];
            let n = group.len();
            let max_prot = n as f32 * MIN_PROTRUSION_PX;

            for (k, entry) in group.iter().enumerate() {
                // Slot 0 (first sorted entry) -> longest protrusion (n * MIN).
                // Slot N-1 (last sorted entry) -> shortest protrusion (1 * MIN).
                let depth = if n == 1 {
                    MIN_PROTRUSION_PX
                } else {
                    max_prot - k as f32 * (max_prot - MIN_PROTRUSION_PX) / (n - 1) as f32
                };
                let params = &mut result[entry.group_idx][entry.edge_idx];
                params.from_protrusion = depth;
                params.to_protrusion = depth;
            }

            group_start = group_end;
        }
    }

    /// Registers the cycle edge's **from-endpoint** in the adjacent rank gap
    /// so protrusion depths are distributed proportionally to the available gap
    /// space.
    ///
    /// For cycle edges (`rank_from == rank_to`), both endpoints are at the same
    /// rank and use the same face. Depending on the face, the from-endpoint is
    /// registered in an adjacent gap:
    ///
    /// - `Top` face at rank R -> register in gap `(R-1, R)` on the `High` side.
    ///   Skipped if R == 0 (no gap above).
    /// - `Bottom` face at rank R -> register in gap `(R, R+1)` on the `Low`
    ///   side.
    /// - `Left` / `Right` faces -> return early;
    ///   `protrusions_assign_cycle_edges` (Step 6) handles the fallback.
    ///
    /// Only the from-endpoint is registered here.
    /// `protrusions_assign_cycle_edges` (Step 6) later copies
    /// `from_protrusion` to `to_protrusion` so both endpoints protrude equally,
    /// producing a symmetric U-shaped arc.
    ///
    /// This allows multiple cycle edges sharing the same gap to receive
    /// proportionally distributed protrusion depths rather than all getting the
    /// same fixed minimum.
    #[allow(clippy::too_many_arguments)]
    fn cycle_edge_collect_rank_gap_entries<'id>(
        group_idx: usize,
        edge_idx: usize,
        pass1_info: &EdgePass1Info<'_, 'id>,
        from_slot_index: Option<usize>,
        face_offsets_by_node_face: &NodeIdAndFaceToContactPointOffsets<'id>,
        svg_node_info_map: &SvgNodeInfoByNodeId<'_, 'id>,
        node_nesting_infos: &NodeNestingInfos<'id>,
        node_ranks_nested: &NodeRanksNested<'id>,
        entity_types: &EntityTypes<'id>,
        rank_gap_entries: &mut Map<RankGapKey, Vec<RankGapEntry>>,
    ) {
        // Step 1: Skip if from_face or to_face is None.
        let Some(from_face) = pass1_info.from_face else {
            return;
        };
        let Some(_to_face) = pass1_info.to_face else {
            return;
        };

        // For cycle edges, both endpoints are at the same rank.
        let rank = pass1_info.rank_from;

        // Steps 2–4: Determine gap key, gap side, and adjacent rank based on
        // face. Left/Right faces have no applicable rank gap.
        let (gap_key, gap_side, adjacent_rank) = match from_face {
            NodeFace::Top => {
                // Protrudes upward into gap (R-1, R). Skip if R == 0.
                if rank.value() == 0 {
                    return;
                }
                let adjacent_rank = NodeRank::new(rank.value() - 1);
                (
                    RankGapKey {
                        rank_low: adjacent_rank,
                        rank_high: rank,
                    },
                    GapSide::High,
                    adjacent_rank,
                )
            }
            NodeFace::Bottom => {
                // Protrudes downward into gap (R, R+1).
                let adjacent_rank = NodeRank::new(rank.value() + 1);
                (
                    RankGapKey {
                        rank_low: rank,
                        rank_high: adjacent_rank,
                    },
                    GapSide::Low,
                    adjacent_rank,
                )
            }
            NodeFace::Left | NodeFace::Right => {
                // No rank gap applicable; protrusions_assign_cycle_edges
                // handles the fallback for these faces.
                return;
            }
        };

        // Step 5: Find parent container scope from the from-node's nesting
        // info. The second-to-last element of the ancestor chain is the
        // immediate parent container (None for root-level nodes).
        let parent_container = node_nesting_infos
            .get(&pass1_info.edge.from)
            .and_then(|ni| {
                ni.ancestor_chain
                    .len()
                    .checked_sub(2)
                    .map(|i| &ni.ancestor_chain[i])
            });

        // Step 6: Look up ranks in scope for the parent container.
        let Some(ranks_in_scope) = node_ranks_nested.ranks_for(parent_container) else {
            return;
        };

        // Look up from node layout info.
        let Some(&from_info) = svg_node_info_map.get(&pass1_info.edge.from) else {
            return;
        };
        // Guard: if the to-node is missing from layout, skip registration.
        if svg_node_info_map.get(&pass1_info.edge.to).is_none() {
            return;
        }

        // Step 7: Compute the adjacent rank boundary.
        //
        // For `Top` face: adjacent rank R-1 is visually above; we want the
        // maximum bottom edge (y + height_collapsed) of all R-1 nodes.
        // For `Bottom` face: adjacent rank R+1 is visually below; we want the
        // minimum top edge (y) of all R+1 nodes.
        let from_category = Self::node_category(&pass1_info.edge.from, entity_types);
        let adjacent_boundary_opt: Option<f32> = ranks_in_scope
            .iter()
            .filter(|&(_, rank)| *rank == adjacent_rank)
            .filter(|(node_id, _)| Self::node_category(node_id, entity_types) == from_category)
            .filter_map(|(node_id, _)| svg_node_info_map.get(node_id).copied())
            .fold(None, |acc, info| {
                let coord = match from_face {
                    NodeFace::Top => info.y + info.height_collapsed,
                    NodeFace::Bottom => info.y,
                    _ => unreachable!(),
                };
                Some(match acc {
                    None => coord,
                    Some(existing) => match from_face {
                        NodeFace::Top => existing.max(coord),
                        NodeFace::Bottom => existing.min(coord),
                        _ => unreachable!(),
                    },
                })
            });

        let Some(adjacent_boundary) = adjacent_boundary_opt else {
            // No adjacent-rank nodes found; no gap exists to register in.
            return;
        };

        // Step 8: Compute from_rank_gap_px (distance from from-node face to
        // the adjacent rank boundary). Return if the gap is zero or negative
        // (nodes overlap or touch).
        let from_rank_gap_px = match from_face {
            NodeFace::Top => from_info.y - adjacent_boundary,
            NodeFace::Bottom => adjacent_boundary - (from_info.y + from_info.height_collapsed),
            _ => unreachable!(),
        };
        if from_rank_gap_px <= 0.0 {
            return;
        }

        // Step 9: Resolve from face offset.
        let from_offset =
            Self::face_offset_resolve(pass1_info, from_slot_index, true, face_offsets_by_node_face);

        // Step 10: Compute from cross-axis coordinate.
        let cross_axis_from =
            OrthoProtrusionGeometry::cross_axis_coord(pass1_info.from_node_x, pass1_info.from_node_y, from_face);

        // Step 11: Register the from-endpoint in the rank gap.
        // `protrusions_assign_cycle_edges` (Step 6) will copy from_protrusion
        // to to_protrusion afterward to produce a symmetric U-shaped arc.
        rank_gap_entries
            .entry(gap_key)
            .or_default()
            .push(RankGapEntry {
                pass1_group_index: group_idx,
                edge_index: edge_idx,
                endpoint_kind: RankGapEndpointKind::FromEndpoint,
                gap_side,
                cross_axis_coord: cross_axis_from,
                face_offset: from_offset,
                rank_gap_px: from_rank_gap_px,
            });
    }

    /// Adjusts protrusion values to clear sibling nodes of the Divergent
    /// ancestor of each edge endpoint.
    ///
    /// For each edge, the from-endpoint's protrusion must be large enough
    /// that the routing segment is in the gap between the tallest sibling
    /// node (at the same rank as the Divergent ancestor) and the to-node.
    /// Symmetric logic applies for the to-endpoint.
    fn protrusions_adjust_for_divergent_siblings<'id>(
        all_pass1_groups: &[EdgeGroupPass1<'_, 'id>],
        all_spacer_coordinates: &[Vec<Vec<SpacerCoordinates>>],
        node_nesting_infos: &NodeNestingInfos<'id>,
        node_ranks_nested: &NodeRanksNested<'id>,
        svg_node_info_map: &SvgNodeInfoByNodeId<'_, 'id>,
        entity_types: &EntityTypes<'id>,
        result: &mut [Vec<OrthoProtrusionParams>],
    ) {
        for (group_idx, group) in all_pass1_groups.iter().enumerate() {
            for (edge_idx, pass1_info) in group.pass1_infos.iter().enumerate() {
                // Same-rank non-cycle edges that share the same direct parent
                // (adjacent siblings) use nearest-face routing with zero
                // protrusion and do not need the divergent-sibling adjustment.
                //
                // Cross-container same-rank non-cycle edges (e.g. a nested node
                // connecting to an adjacent sibling container or root-level
                // node) still need the adjustment so the protrusion exits the
                // source container. These edges have endpoints at different
                // nesting depths or in different parent containers.
                if pass1_info.rank_from == pass1_info.rank_to && !pass1_info.is_cycle_edge {
                    let same_parent = {
                        let from_chain = node_nesting_infos
                            .get(&pass1_info.edge.from)
                            .map(|ni| ni.ancestor_chain.as_slice())
                            .unwrap_or(&[]);
                        let to_chain = node_nesting_infos
                            .get(&pass1_info.edge.to)
                            .map(|ni| ni.ancestor_chain.as_slice())
                            .unwrap_or(&[]);
                        // Same parent: equal depth AND identical ancestor chain
                        // up to (but not including) the node itself.
                        let from_parent = from_chain.len().saturating_sub(1);
                        let to_parent = to_chain.len().saturating_sub(1);
                        from_parent == to_parent
                            && from_chain.get(..from_parent) == to_chain.get(..to_parent)
                    };
                    if same_parent {
                        continue;
                    }
                }
                // === From endpoint === //
                if let Some(from_face) = pass1_info.from_face {
                    let min_from = Self::min_protrusion_divergent_sibling_extent(
                        &pass1_info.edge.from,
                        &pass1_info.edge.to,
                        from_face,
                        node_nesting_infos,
                        node_ranks_nested,
                        svg_node_info_map,
                        entity_types,
                    );
                    if min_from > 0.0 {
                        let params = &mut result[group_idx][edge_idx];
                        params.from_protrusion = params.from_protrusion.max(min_from);
                    }
                }

                // === To endpoint === //
                //
                // When the edge has cross-container spacers, the spacer
                // already handles routing inside the to-node's container.
                // The to_protrusion only needs to reach the spacer exit,
                // not exit the entire container. Applying the
                // divergent-sibling adjustment in this case would force
                // the protrusion all the way to the container's far
                // boundary, causing the path to overshoot the spacer
                // and produce a zigzag.
                let edge_has_spacers = all_spacer_coordinates
                    .get(group_idx)
                    .and_then(|g| g.get(edge_idx))
                    .map(|spacers| !spacers.is_empty())
                    .unwrap_or(false);

                if !edge_has_spacers && let Some(to_face) = pass1_info.to_face {
                    let min_to = Self::min_protrusion_divergent_sibling_extent(
                        &pass1_info.edge.to,
                        &pass1_info.edge.from,
                        to_face,
                        node_nesting_infos,
                        node_ranks_nested,
                        svg_node_info_map,
                        entity_types,
                    );
                    if min_to > 0.0 {
                        let params = &mut result[group_idx][edge_idx];
                        params.to_protrusion = params.to_protrusion.max(min_to);
                    }
                }
            }
        }
    }

    /// Computes the minimum protrusion needed for `node_id`'s endpoint to
    /// clear all sibling nodes of the node's Divergent ancestor at the LCA
    /// level.
    ///
    /// The Divergent ancestor is the ancestor of `node_id` that is a direct
    /// child of the LCA of (`node_id`, `other_node_id`).
    ///
    /// # Parameters
    ///
    /// * `node_id`: the endpoint node whose protrusion is being computed.
    /// * `other_node_id`: the opposite endpoint of the edge (used to find the
    ///   LCA).
    /// * `face`: the face at which `node_id` protrudes.
    fn min_protrusion_divergent_sibling_extent<'id>(
        node_id_from: &NodeId<'id>,
        node_id_to: &NodeId<'id>,
        face: NodeFace,
        node_nesting_infos: &NodeNestingInfos<'id>,
        node_ranks_nested: &NodeRanksNested<'id>,
        svg_node_info_map: &SvgNodeInfoByNodeId<'_, 'id>,
        entity_types: &EntityTypes<'id>,
    ) -> f32 {
        // 1. Compute LCA depth.
        let lca_depth = Self::lca_depth(node_id_from, node_id_to, node_nesting_infos);

        // 2. Find divergent ancestor of node_id.
        let Some(divergent_ancestor_id_from) =
            Self::divergent_ancestor_id(node_id_from, lca_depth, node_nesting_infos)
        else {
            return 0.0;
        };

        // 3. Find parent container of divergent ancestor (None = root level).
        let divergent_ancestor_parent_id_from = node_nesting_infos
            .get(divergent_ancestor_id_from)
            .and_then(|node_nesting_info| {
                node_nesting_info
                    .ancestor_chain
                    .len()
                    .checked_sub(2)
                    .map(|parent_index| &node_nesting_info.ancestor_chain[parent_index])
            });

        // 4. Get rank of divergent ancestor in its parent container.
        let Some(node_ranks) = node_ranks_nested.ranks_for(divergent_ancestor_parent_id_from)
        else {
            return 0.0;
        };
        let Some(&div_ancestor_rank) = node_ranks.get(divergent_ancestor_id_from) else {
            return 0.0;
        };

        // 5. Collect same-rank siblings of the same node category (including the
        //    divergent ancestor). Nodes from other categories (e.g. process nodes) are
        //    excluded so that thing-node edges are not routed around process nodes.
        //
        //    The divergent ancestor of `other_node_id` at the LCA level is also
        //    excluded. For forward-facing cross-container edges (adjacent divergent
        //    ancestors), the protrusion only needs to clear the source's own container
        //    boundary, not route around the target container. Excluding the target's
        //    divergent ancestor prevents over-protrusion that would make
        //    `from_protrusion_capped` zero out the source protrusion.
        //
        //    Additionally, siblings outside the range between the `from` and `to`
        //    divergent ancestors (by nesting-path index) are excluded. For adjacent
        //    divergent-ancestor pairs, the protrusion only needs to exit the source
        //    container and reach the gap to the destination -- not route around any
        //    further containers past the destination. Only siblings that lie on the
        //    same side as the `from` ancestor (up to and including the `to` ancestor
        //    position) are considered. The nesting-path index reliably reflects the
        //    structural ordering without relying on layout coordinates.
        //
        //    For cycle edges the equalization in Step 6 takes the max of both
        //    endpoints' protrusions, so excluding the target's ancestor from the from-
        //    computation is compensated by the to-computation (which excludes the
        //    from-ancestor instead), and the max covers all nodes correctly.
        let node_category = Self::node_category(node_id_from, entity_types);
        let divergent_ancestor_id_to =
            Self::divergent_ancestor_id(node_id_to, lca_depth, node_nesting_infos);
        // Look up the nesting-path index of the `from` divergent ancestor at
        // the LCA level. This bounds the sibling range to nodes structurally
        // between the two divergent ancestors.
        let divergent_ancestor_index_from: Option<usize> = node_nesting_infos
            .get(divergent_ancestor_id_from)
            .and_then(|nni| nni.nesting_path.get(lca_depth).copied());
        // Look up the nesting-path index of the `to` divergent ancestor at the
        // LCA level, if present.
        let divergent_ancestor_index_to: Option<usize> = divergent_ancestor_id_to
            .and_then(|divergent_ancestor_id_to| node_nesting_infos.get(divergent_ancestor_id_to))
            .and_then(|nni| nni.nesting_path.get(lca_depth).copied());
        let same_rank_siblings: Vec<&NodeId<'id>> = node_ranks
            .iter()
            // Only include siblings that are at the same rank as the divergent_ancestor.
            .filter(|&(_, sibling_rank)| *sibling_rank == div_ancestor_rank)
            // Only include siblings that are of the same category as the divergent_ancestor.
            .filter(|(node_id_sibling, _)| {
                Self::node_category(node_id_sibling, entity_types) == node_category
            })
            // Only include siblings that are not the `to` divergent ancestor.
            .filter(|(node_id_sibling, _)| divergent_ancestor_id_to != Some(*node_id_sibling))
            // Only include siblings whose nesting-path index does not lie
            // past the `to` divergent ancestor (i.e., keep siblings on the
            // same side as the `from` ancestor, up to and including the `to`
            // ancestor position). Siblings beyond the `to` ancestor lie past
            // the destination container and should not influence the minimum
            // protrusion.
            .filter(|(node_id_sibling, _)| {
                let (Some(index_from), Some(index_to)) =
                    (divergent_ancestor_index_from, divergent_ancestor_index_to)
                else {
                    return true;
                };
                let Some(sibling_index) = node_nesting_infos
                    .get(*node_id_sibling)
                    .and_then(|nni| nni.nesting_path.get(lca_depth).copied())
                else {
                    return true;
                };
                if index_from <= index_to {
                    sibling_index <= index_to
                } else {
                    sibling_index >= index_to
                }
            })
            .map(|(id, _)| id)
            .collect();

        // 6. Get face coordinate of node_id (the coordinate of the face in the
        //    rank/protrusion direction).
        let Some(&node_info_from) = svg_node_info_map.get(node_id_from) else {
            return 0.0;
        };
        let node_face_coord = Self::face_coord_for_endpoint(node_info_from, face);

        // 7. Find extreme sibling coordinate in the protrusion direction.
        let Some(sibling_extreme) =
            Self::same_rank_sibling_extreme(&same_rank_siblings, face, svg_node_info_map)
        else {
            return 0.0;
        };

        // 8. Compute minimum protrusion:
        //
        //     `face_sign * (sibling_extreme - node_face_coord)`
        //
        //     - For Bottom/Right faces (sign = +1): `min = sibling_extreme -
        //       node_face_coord.`
        //     - For Top/Left faces (sign = -1): `min = node_face_coord -
        //       sibling_extreme.`
        let face_sign: f32 = match face {
            NodeFace::Bottom | NodeFace::Right => 1.0,
            NodeFace::Top | NodeFace::Left => -1.0,
        };
        (face_sign * (sibling_extreme - node_face_coord)).max(0.0)
    }

    /// Returns the coordinate of a node's envelope face along the protrusion
    /// axis.
    ///
    /// Uses envelope bounds (which include edge label wrapper slots) so that
    /// protrusions clear the full label area, not just the inner node
    /// rectangle.
    ///
    /// For `Bottom` face: the bottom edge y-coordinate of the envelope.
    /// For `Top` face: the top edge y-coordinate of the envelope.
    /// For `Right` face: the right edge x-coordinate of the envelope.
    /// For `Left` face: the left edge x-coordinate of the envelope.
    fn face_coord_for_endpoint(info: &SvgNodeInfo<'_>, face: NodeFace) -> f32 {
        match face {
            NodeFace::Bottom => info.envelope_y + info.envelope_height_collapsed,
            NodeFace::Top => info.envelope_y,
            NodeFace::Right => info.envelope_x + info.envelope_width,
            NodeFace::Left => info.envelope_x,
        }
    }

    /// Returns the extreme coordinate of the same-rank sibling nodes in the
    /// protrusion direction.
    ///
    /// For `Bottom`/`Right` faces: returns the maximum far-edge coordinate
    /// (max of bottom edges or right edges across all siblings).
    ///
    /// For `Top`/`Left` faces: returns the minimum near-edge coordinate
    /// (min of top edges or left edges across all siblings).
    ///
    /// Returns `None` if no sibling has a known layout in `svg_node_info_map`.
    fn same_rank_sibling_extreme<'id>(
        sibling_ids: &[&NodeId<'id>],
        face: NodeFace,
        svg_node_info_map: &SvgNodeInfoByNodeId<'_, 'id>,
    ) -> Option<f32> {
        sibling_ids.iter().fold(None, |extreme, id| {
            let Some(&info) = svg_node_info_map.get(*id) else {
                return extreme;
            };
            let coord = Self::face_coord_for_endpoint(info, face);
            Some(match extreme {
                None => coord,
                Some(existing) => match face {
                    NodeFace::Bottom | NodeFace::Right => existing.max(coord),
                    NodeFace::Top | NodeFace::Left => existing.min(coord),
                },
            })
        })
    }

    /// Returns the depth of the Lowest Common Ancestor (LCA) of two nodes.
    ///
    /// The LCA depth is the number of common ancestors the two nodes share.
    /// A depth of 0 means the LCA is the diagram root (no shared ancestors),
    /// so the Divergent ancestors are the nodes' first ancestors (or themselves
    /// for root-level nodes).
    ///
    /// # Examples
    ///
    /// For root-level nodes `a` and `b`: returns `0` (no common ancestors).
    ///
    /// For siblings `c/child_0` and `c/child_1`: returns `1` (one common
    /// ancestor: `c`).
    fn lca_depth<'id>(
        from_id: &NodeId<'id>,
        to_id: &NodeId<'id>,
        node_nesting_infos: &NodeNestingInfos<'id>,
    ) -> usize {
        let from_chain = match node_nesting_infos.get(from_id) {
            Some(info) => info.ancestor_chain.as_slice(),
            None => return 0,
        };
        let to_chain = match node_nesting_infos.get(to_id) {
            Some(info) => info.ancestor_chain.as_slice(),
            None => return 0,
        };

        // Exclude the nodes themselves (last element of each chain).
        let from_ancestors = from_chain.len().saturating_sub(1);
        let to_ancestors = to_chain.len().saturating_sub(1);

        from_chain[..from_ancestors]
            .iter()
            .zip(to_chain[..to_ancestors].iter())
            .take_while(|(a, b)| a == b)
            .count()
    }

    /// Returns the divergent ancestor ID of `node_id` at the given LCA depth.
    ///
    /// The divergent ancestor is the ancestor of `node_id` that is a direct
    /// child of the LCA. For root-level nodes (LCA is the diagram root, depth
    /// = 0), this is the node's first ancestor (or itself if root-level).
    ///
    /// Returns `None` if the node is not found in `node_nesting_infos` or the
    /// ancestor chain does not have an element at `lca_depth`.
    fn divergent_ancestor_id<'a, 'id>(
        node_id: &'a NodeId<'id>,
        lca_depth: usize,
        node_nesting_infos: &'a NodeNestingInfos<'id>,
    ) -> Option<&'a NodeId<'id>> {
        let chain = &node_nesting_infos.get(node_id)?.ancestor_chain;
        chain.get(lca_depth)
    }

    /// Returns the [`NodeCategory`] of a node from its entity types.
    ///
    /// - `ThingDefault` nodes are [`NodeCategory::Thing`].
    /// - `TagDefault` nodes are [`NodeCategory::Tag`].
    /// - `ProcessDefault` and `ProcessStepDefault` nodes are
    ///   [`NodeCategory::Process`].
    /// - All other nodes are [`NodeCategory::Other`].
    fn node_category<'id>(node_id: &NodeId<'id>, entity_types: &EntityTypes<'id>) -> NodeCategory {
        entity_types
            .get(node_id.as_ref())
            .map_or(NodeCategory::Other, |types| {
                if types.contains(&EntityType::ThingDefault) {
                    NodeCategory::Thing
                } else if types.contains(&EntityType::TagDefault) {
                    NodeCategory::Tag
                } else if types.contains(&EntityType::ProcessDefault)
                    || types.contains(&EntityType::ProcessStepDefault)
                {
                    NodeCategory::Process
                } else {
                    NodeCategory::Other
                }
            })
    }
}
