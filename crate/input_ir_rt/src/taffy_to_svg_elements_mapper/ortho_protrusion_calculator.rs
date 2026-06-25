use disposition_ir_model::{
    entity::EntityTypes,
    node::{NodeId, NodeNestingInfos, NodeRank, NodeRanksNested},
};
use disposition_model_common::{
    edge::{MAX_GAP_FRACTION, MIN_PROTRUSION_PX, TO_PROTRUSION_MIN_PX},
    entity::EntityType,
    Map, RankDir,
};
use disposition_svg_model::{OrthoProtrusionParams, SpacerProtrusionParams, SvgNodeInfo};
use disposition_taffy_model::{taffy::TaffyTree, EdgeIdToEdgeSpacerTaffyNodes, TaffyNodeCtx};

use disposition_ir_model::node::NodeFace;

use super::svg_edge_infos_builder::{EdgeGroupPass1, EdgePass1Info};

use crate::taffy_to_svg_elements_mapper::{
    edge_model::{NodeIdAndFace, NodeIdAndFaceToContactPointOffsets},
    edge_path_builder_pass_1::SpacerCoordinates,
    SpacerCoordinatesResolver, SvgNodeInfoByNodeId,
};

use self::ortho_protrusion_geometry::OrthoProtrusionGeometry;

mod ortho_protrusion_geometry;

/// Minimum protrusion-depth separation (in pixels) between two same-side
/// endpoints whose lateral routing legs overlap along the cross axis.
///
/// Legs whose cross-axis spans overlap and whose protrusion depths differ by
/// less than this read as a single line. It is larger than `MIN_PROTRUSION_PX`
/// so the separated legs clear the "reads as one line" threshold rather than
/// merely being non-equal. Used by
/// [`OrthoProtrusionCalculator::side_jogs_separate`].
const JOG_SEPARATION_MIN_PX: f32 = 7.0;

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
    /// Cross-axis coordinate of the **next contact along the path** from this
    /// endpoint (the other end of this endpoint's lateral "jog" segment).
    ///
    /// Together with [`Self::cross_axis_coord`] this defines the cross-axis
    /// span `[min, max]` that the endpoint's lateral routing leg sweeps in
    /// this rank gap. Two legs only "read as one line" when their spans
    /// overlap, so the
    /// span lets [`OrthoProtrusionCalculator::protrusions_assign`] separate
    /// only the legs that actually overlap (interval-graph style) instead
    /// of every endpoint in the bucket. For a from-endpoint the next
    /// contact is the first spacer entry (or the to-node when there are no
    /// spacers); for a to-endpoint it is the last spacer exit (or the
    /// from-node); for a spacer entry / exit it is the adjacent spacer or
    /// node it aligns to. Cycle endpoints store
    /// their own `cross_axis_coord` (zero-width span -- they overlap nothing).
    jog_far_cross_axis: f32,
    /// The face offset (slot offset) for this endpoint.
    ///
    /// Edges further from the face midpoint (larger absolute offset)
    /// receive longer protrusions. For spacer endpoints this is
    /// `0.0` since spacers do not have face offsets.
    face_offset: f32,
    /// Pixel distance in the rank direction for this endpoint's rank
    /// gap (from the node contact point or spacer boundary to the
    /// nearest adjacent spacer or node).
    ///
    /// For node endpoints whose edge crosses spacers, this is the
    /// *post-envelope* routing channel: the distance from the node's
    /// envelope face (the outer edge of its edge-label wrapper) to the
    /// adjacent spacer. The band distributes protrusion depths within
    /// this channel; the fixed `envelope_clearance` is added on top when
    /// the protrusion is written (see [`Self::protrusion_write`]).
    rank_gap_px: f32,
    /// Fixed clearance (in pixels) added to the band-distributed
    /// protrusion before it is written, so the protrusion length spans
    /// the node's own edge-label wrapper from the inner node face.
    ///
    /// This is the depth of the endpoint node's own edge-label slot on
    /// its protruding face (the gap between the inner node face and the
    /// envelope face). It is `0.0` for spacer entries / exits (spacers
    /// have no edge labels) and for node endpoints whose edge has no
    /// spacers (their own-label clearance is supplied by the
    /// divergent-sibling adjustment instead).
    envelope_clearance: f32,
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum NodeCategory {
    Thing,
    Tag,
    Process,
    Other,
}

/// Identifies a single divergent-ancestor sibling row that an endpoint's
/// protrusion must clear.
///
/// Endpoints that share the same row key clear the **same** set of sibling
/// nodes in the same direction, so their divergent-sibling clearance is
/// (near-)identical. Without staggering they collapse to the same protrusion
/// depth and their lateral routing segments overlap. Grouping by this key lets
/// `protrusions_adjust_for_divergent_siblings` assign each a distinct depth.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct DivergentSiblingRowKey<'id> {
    /// Parent container of the divergent ancestor (`None` for the root level).
    parent_container: Option<NodeId<'id>>,
    /// Rank of the divergent ancestor within `parent_container`.
    div_ancestor_rank: NodeRank,
    /// Face the endpoint protrudes from (separates from-side and to-side rows).
    face: NodeFace,
    /// Category of the endpoint node (thing / tag / process / other).
    category: NodeCategory,
}

/// The divergent-sibling clearance for one endpoint, plus the sibling row it
/// clears.
///
/// Returned by
/// [`OrthoProtrusionCalculator::min_protrusion_divergent_sibling_extent`].
struct DivergentSiblingExtent<'id> {
    /// Minimum protrusion (from the node's inner face) needed to clear the
    /// divergent-ancestor sibling row.
    min_protrusion: f32,
    /// The sibling row this endpoint clears (used to group endpoints that must
    /// be staggered to distinct depths).
    row_key: DivergentSiblingRowKey<'id>,
}

/// Which endpoint of an edge a divergent-sibling adjustment applies to.
#[derive(Clone, Copy, Debug)]
enum AdjustEndpoint {
    From,
    To,
}

/// A single endpoint queued for divergent-sibling staggering.
///
/// Collected per [`DivergentSiblingRowKey`] in
/// `protrusions_adjust_for_divergent_siblings` so endpoints clearing the same
/// row receive distinct, staggered protrusion depths.
struct EndpointAdjustment {
    /// Index into `all_pass1_groups`.
    group_idx: usize,
    /// Index into the group's `pass1_infos`.
    edge_idx: usize,
    /// Whether the `from` or `to` protrusion field is written.
    endpoint: AdjustEndpoint,
    /// Cross-axis coordinate of the endpoint's node, used to order the stagger
    /// (deepest-first, descending -- matching `protrusions_assign`).
    cross_axis_coord: f32,
    /// The endpoint's own divergent-sibling clearance.
    min_protrusion: f32,
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
        group_is_direct: &[bool],
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
            // Direct-curvature edges (e.g. interaction edges drawn as
            // `DirectCurved`) bypass spacers and protrusions entirely -- pass 2
            // ignores their `OrthoProtrusionParams`. They must therefore neither
            // consume nor influence the shared protrusion band that sizes the
            // real orthogonal (dependency) edges, so skip the whole group.
            if group_is_direct[group_idx] {
                continue;
            }

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

                    // When the edge crosses spacers, the band channel is
                    // measured beyond the from-node's envelope; the fixed
                    // wrapper width is added back at write time so the
                    // protrusion still spans the from-node's own edge label.
                    // Without spacers, the divergent-sibling adjustment supplies
                    // the own-label clearance, so the band stays in the full
                    // inner-face gap and no clearance is added here.
                    let from_envelope_clearance = if spacer_coordinates.is_empty() {
                        0.0
                    } else {
                        Self::envelope_clearance_for(
                            &pass1_info.edge.from,
                            from_face,
                            svg_node_info_map,
                        )
                    };

                    let cross_axis_from = OrthoProtrusionGeometry::cross_axis_coord(
                        pass1_info.from_node_x,
                        pass1_info.from_node_y,
                        from_face,
                    );

                    // The from-endpoint's lateral leg reaches the first spacer
                    // (or the to-node when there are no spacers).
                    let from_jog_far = if let Some(first_spacer) = spacer_coordinates.first() {
                        OrthoProtrusionGeometry::cross_axis_coord(
                            first_spacer.entry_x,
                            first_spacer.entry_y,
                            from_face,
                        )
                    } else {
                        OrthoProtrusionGeometry::cross_axis_coord(
                            pass1_info.to_node_x,
                            pass1_info.to_node_y,
                            from_face,
                        )
                    };

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
                            jog_far_cross_axis: from_jog_far,
                            face_offset: from_offset,
                            rank_gap_px: from_rank_gap_px,
                            envelope_clearance: from_envelope_clearance,
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

                    // Symmetric to the from endpoint: the wrapper width is added
                    // back at write time for spacer-crossing edges.
                    let to_envelope_clearance = if spacer_coordinates.is_empty() {
                        0.0
                    } else {
                        Self::envelope_clearance_for(
                            &pass1_info.edge.to,
                            to_face,
                            svg_node_info_map,
                        )
                    };

                    let cross_axis_to = OrthoProtrusionGeometry::cross_axis_coord(
                        pass1_info.to_node_x,
                        pass1_info.to_node_y,
                        to_face,
                    );

                    // The to-endpoint's lateral leg reaches the last spacer
                    // (or the from-node when there are no spacers).
                    let to_jog_far = if let Some(last_spacer) = spacer_coordinates.last() {
                        OrthoProtrusionGeometry::cross_axis_coord(
                            last_spacer.exit_x,
                            last_spacer.exit_y,
                            to_face,
                        )
                    } else {
                        OrthoProtrusionGeometry::cross_axis_coord(
                            pass1_info.from_node_x,
                            pass1_info.from_node_y,
                            to_face,
                        )
                    };

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
                            jog_far_cross_axis: to_jog_far,
                            face_offset: to_offset,
                            rank_gap_px: to_rank_gap_px,
                            envelope_clearance: to_envelope_clearance,
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

                            // This entry's leg aligns to the previous spacer.
                            let entry_jog_far = OrthoProtrusionGeometry::cross_axis_coord(
                                prev_spacer.entry_x,
                                prev_spacer.entry_y,
                                from_face_for_axis,
                            );

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
                                    jog_far_cross_axis: entry_jog_far,
                                    face_offset: 0.0,
                                    rank_gap_px: gap_px,
                                    envelope_clearance: 0.0,
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

                            // This leg aligns the first spacer to the from-node.
                            let first_entry_jog_far = OrthoProtrusionGeometry::cross_axis_coord(
                                pass1_info.from_node_x,
                                pass1_info.from_node_y,
                                from_face_for_axis,
                            );

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
                                    jog_far_cross_axis: first_entry_jog_far,
                                    face_offset: 0.0,
                                    rank_gap_px: gap_px,
                                    envelope_clearance: 0.0,
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

                            // This exit leg aligns to the next spacer.
                            let exit_jog_far = OrthoProtrusionGeometry::cross_axis_coord(
                                next_spacer.entry_x,
                                next_spacer.entry_y,
                                from_face_for_axis,
                            );

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
                                    jog_far_cross_axis: exit_jog_far,
                                    face_offset: 0.0,
                                    rank_gap_px: gap_px,
                                    envelope_clearance: 0.0,
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

                            // This exit leg aligns to the to-node.
                            let last_exit_jog_far = OrthoProtrusionGeometry::cross_axis_coord(
                                pass1_info.to_node_x,
                                pass1_info.to_node_y,
                                from_face_for_axis,
                            );

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
                                    jog_far_cross_axis: last_exit_jog_far,
                                    face_offset: 0.0,
                                    rank_gap_px: gap_px,
                                    envelope_clearance: 0.0,
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
        // Endpoints clearing the SAME divergent-ancestor sibling row are
        // staggered to distinct depths so two nested-to-nested edges sharing a
        // rank gap do not collapse to the same protrusion (which would overlap
        // their lateral routing segments). See
        // `protrusions_adjust_for_divergent_siblings`.
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

        // === Step 5.5: Separate approach channels of spacer-crossing edges
        // === //
        //
        // Multiple cross-container spacer edges entering the same to-node share
        // the narrow gap between the last spacer exit and the to-node face.
        // Their `to` protrusions and last-spacer `exit` protrusions otherwise
        // floor to (near-)identical values, so their vertical approach legs
        // overlap. This step assigns each such edge a distinct leg coordinate
        // within the gap and sets both protrusions to land on it.
        Self::protrusions_separate_spacer_approach_channels(
            all_pass1_groups,
            &all_spacer_coordinates,
            svg_node_info_map,
            &mut result,
        );

        // === Step 5.6: Nest approach legs of edges entering the same to-face
        // from different rank-gap buckets === //
        //
        // A cross-container (spacer-crossing) edge's to-endpoint is keyed by the
        // LCA-level rank gap, while a plain edge into the same nested node is
        // keyed by that node's container-level rank gap. They never compete in
        // Step 3, so their approach legs are chosen independently and can cross.
        // This step nests the legs of all edges entering a shared `(to-node,
        // to-face)` when such a mix is present.
        Self::protrusions_separate_shared_to_face_channels(
            all_pass1_groups,
            to_slot_indices_all,
            face_offsets_by_node_face,
            &all_spacer_coordinates,
            svg_node_info_map,
            node_nesting_infos,
            node_ranks_nested,
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
    ///    constraint), and the total band `available = min_gap_px *
    ///    MAX_GAP_FRACTION` shared by both sides of the gap.
    /// 2. Partition entries into `Low` side and `High` side groups, sorted by
    ///    each side's spatial ordering (face offset then cross-axis
    ///    coordinate).
    /// 3. Split `available` between the two sides **proportionally to each
    ///    side's endpoint count**, reserving a per-side depth floor so the
    ///    arrow head at any `ToEndpoint` is cleared. Because the two sides grow
    ///    from opposite gap boundaries toward each other, `low_band + high_band
    ///    <= available` guarantees the deepest from-tip and deepest to-tip
    ///    never overlap, leaving `(1 - MAX_GAP_FRACTION) * gap` as the routing
    ///    channel.
    /// 4. Within each side's band, assign distinct depths deepest-first in
    ///    spatial order, so edges whose contact points are further apart
    ///    receive longer protrusions, reducing visual cross-over.
    fn protrusions_assign(
        rank_gap_entries: &mut [RankGapEntry],
        result: &mut [Vec<OrthoProtrusionParams>],
    ) {
        if rank_gap_entries.is_empty() {
            return;
        }

        // Find the tightest rank gap constraint and the total band shared by
        // both sides.
        let min_gap_px = rank_gap_entries
            .iter()
            .map(|rank_gap_entry| rank_gap_entry.rank_gap_px)
            .reduce(f32::min)
            .unwrap_or(0.0);

        let available = min_gap_px * MAX_GAP_FRACTION;

        if available < MIN_PROTRUSION_PX {
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
            b.cross_axis_coord
                .partial_cmp(&a.cross_axis_coord)
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
        // Each side of the gap gets its own band, carved out of the shared
        // `available` budget in proportion to its endpoint count. Within a
        // side, entries are assigned distinct depths deepest-first in spatial
        // order. Single-side entries take the deepest depths, crossing entries
        // the shallower ones (preserving the previous convention where
        // single-side edges protrude further than crossing edges on the same
        // side), and each side's crossing entries are ordered by that side's
        // own spatial sort so earlier entries on each side receive longer
        // protrusions.

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

        // === Per-side ordered lists (deepest-first) === //
        //
        // Low side: single_low (deepest) then crossing_low.
        // High side: crossing_high (deeper) then single_high (shallowest),
        // mirroring the previous shared-pool ordering where crossing high
        // entries occupied deeper slots than single-side high entries.
        let low_ordered: Vec<&RankGapEntry> = single_low
            .iter()
            .copied()
            .chain(crossing_low_ordered.iter().copied())
            .collect();
        let high_ordered: Vec<&RankGapEntry> = crossing_high_ordered
            .iter()
            .copied()
            .chain(single_high.iter().copied())
            .collect();

        let n_low = low_ordered.len();
        let n_high = high_ordered.len();

        // === Per-side floors === //
        //
        // A side that holds any `ToEndpoint` reserves the arrow-head clearance
        // floor (`TO_PROTRUSION_MIN_PX`) as its shallowest depth, so the
        // straight segment entering the to-node clears the arrow head before
        // the Z/S bend. From / spacer-only sides keep `MIN_PROTRUSION_PX`. A
        // backward edge reverses from/to sides, so the actual endpoint kinds
        // are inspected rather than the side name. Empty sides reserve nothing.
        let side_floor = |entries: &[&RankGapEntry]| -> f32 {
            if entries.is_empty() {
                0.0
            } else if entries
                .iter()
                .any(|e| matches!(e.endpoint_kind, RankGapEndpointKind::ToEndpoint))
            {
                TO_PROTRUSION_MIN_PX
            } else {
                MIN_PROTRUSION_PX
            }
        };
        let low_floor = side_floor(&low_ordered);
        let high_floor = side_floor(&high_ordered);

        // === Split the band proportionally to each side's count === //
        //
        // The growable slack (above the two floors) is divided in proportion to
        // the endpoint counts, so per-protrusion spacing is even regardless of
        // a from/to count imbalance. By construction `low_band + high_band <=
        // available`, so the deepest tips on the two sides never cross.
        let slack = available - low_floor - high_floor;
        if slack < 0.0 {
            // Gap is too tight to honour both sides' floors with separation;
            // fall back to minimal protrusions (to-endpoints are floored in
            // `protrusion_write`).
            for rank_gap_entry in rank_gap_entries.iter() {
                Self::protrusion_write(
                    rank_gap_entry,
                    MIN_PROTRUSION_PX.min(min_gap_px * 0.5),
                    result,
                );
            }
            return;
        }

        let n_total = (n_low + n_high) as f32;
        let low_band = low_floor + slack * (n_low as f32) / n_total;
        let high_band = high_floor + slack * (n_high as f32) / n_total;

        // `side_depth` distributes `n` distinct depths within `[floor, band]`,
        // deepest first (index 0 -> `band`, index n-1 -> `floor`). A lone entry
        // sits at the midpoint of its band so an isolated stub is not drawn at
        // the full band depth.
        let side_depth = |i: usize, n: usize, band: f32, floor: f32| -> f32 {
            if n <= 1 {
                (band + floor) * 0.5
            } else {
                band - (i as f32) * (band - floor) / ((n - 1) as f32)
            }
        };

        // Initial per-side depths from the proportional band split.
        let mut low_depths: Vec<f32> = (0..n_low)
            .map(|i| side_depth(i, n_low, low_band, low_floor))
            .collect();
        let mut high_depths: Vec<f32> = (0..n_high)
            .map(|i| side_depth(i, n_high, high_band, high_floor))
            .collect();

        // Separate the spacer-entry legs that actually overlap. The proportional
        // band split crams every endpoint of a side into one band sized by the
        // *tightest* `rank_gap_px` in the whole bucket, so a fan of cross-container
        // legs sharing one LCA gap collapses onto (near-)identical depths and
        // reads as a single line. This re-spaces only the spacer-entry legs whose
        // cross-axis spans overlap, deepening each into its own channel (so it
        // lifts toward the inter-rank gap) but never past its same-edge connecting
        // partner, so the path cannot reverse.
        Self::jogs_separate(
            &low_ordered,
            &mut low_depths,
            low_floor,
            &high_ordered,
            &mut high_depths,
            high_floor,
        );

        low_ordered
            .iter()
            .zip(low_depths)
            .for_each(|(entry, depth)| {
                Self::protrusion_write(entry, depth, result);
            });
        high_ordered
            .iter()
            .zip(high_depths)
            .for_each(|(entry, depth)| {
                Self::protrusion_write(entry, depth, result);
            });
    }

    /// Re-spaces the protrusion depths of spacer-entry legs whose lateral
    /// routing legs overlap along the cross axis and have collapsed to
    /// (near-)identical depths, so a fan of cross-container edges sharing one
    /// LCA rank gap no longer reads as a single line.
    ///
    /// `low_*` / `high_*` are the two sides of the gap (ordered deepest-first,
    /// parallel depth slices from the proportional band split, and the side
    /// floor). Each side is processed independently by
    /// [`Self::jogs_separate_side`].
    fn jogs_separate(
        low_ordered: &[&RankGapEntry],
        low_depths: &mut [f32],
        low_floor: f32,
        high_ordered: &[&RankGapEntry],
        high_depths: &mut [f32],
        high_floor: f32,
    ) {
        Self::jogs_separate_side(
            low_ordered,
            low_depths,
            low_floor,
            high_ordered,
            high_depths,
        );
        Self::jogs_separate_side(
            high_ordered,
            high_depths,
            high_floor,
            low_ordered,
            low_depths,
        );
    }

    /// Deepens the participating spacer legs of one side so their lateral legs
    /// separate.
    ///
    /// Only **spacer entry / exit** legs are moved: a spacer boundary carries
    /// the lateral leg that aligns the path from the previous contact's
    /// column onto the spacer column, which is the leg that collapses for a
    /// fan of cross-container edges. From / to endpoints are left fixed
    /// (their approach legs are separated by the dedicated Step 5.5 / 5.6
    /// passes). Each spacer leg is deepened only as far as **both** its own
    /// `rank_gap_px` channel and the room left above its same-edge
    /// connecting partner on `opp_ordered` (the from / previous-spacer leg
    /// it meets) allow, so the deepened leg can never overshoot its partner
    /// and reverse the path.
    ///
    /// Legs that do not overlap any collapsed leg keep their band-split depth
    /// (so single-leg and already-separated buckets are byte-for-byte
    /// unchanged).
    fn jogs_separate_side(
        ordered: &[&RankGapEntry],
        depths: &mut [f32],
        floor: f32,
        opp_ordered: &[&RankGapEntry],
        opp_depths: &[f32],
    ) {
        let n = ordered.len();
        if n < 2 {
            return;
        }

        // Cross-axis span of each endpoint's lateral leg.
        let span = |entry: &RankGapEntry| -> (f32, f32) {
            let a = entry.cross_axis_coord;
            let b = entry.jog_far_cross_axis;
            (a.min(b), a.max(b))
        };
        let spans_overlap = |i: usize, j: usize| -> bool {
            let (i_lo, i_hi) = span(ordered[i]);
            let (j_lo, j_hi) = span(ordered[j]);
            // Strictly-overlapping (shared interior), not merely touching.
            i_hi.min(j_hi) - i_lo.max(j_lo) > 1e-2
        };

        let movable = |entry: &RankGapEntry| -> bool {
            matches!(
                entry.endpoint_kind,
                RankGapEndpointKind::SpacerEntry { .. } | RankGapEndpointKind::SpacerExit { .. }
            )
        };

        // Only spacer legs that overlap another collapsed leg are moved; every
        // other entry keeps its band-split depth and acts as a fixed obstacle.
        let participates: Vec<bool> = (0..n)
            .map(|i| {
                movable(ordered[i])
                    && (0..n).any(|j| {
                        j != i
                            && spans_overlap(i, j)
                            && (depths[i] - depths[j]).abs() < JOG_SEPARATION_MIN_PX
                    })
            })
            .collect();

        // The deepest a leg may protrude: its own `rank_gap_px` channel, further
        // bounded so it leaves room above its fixed same-edge partner on the
        // opposite side (`partner_depth`), keeping their summed depth within the
        // shared physical gap so the connecting jog cannot reverse.
        let cap = |i: usize| -> f32 {
            let entry = ordered[i];
            let own = entry.rank_gap_px * MAX_GAP_FRACTION;
            let partner = opp_ordered.iter().position(|opp| {
                opp.pass1_group_index == entry.pass1_group_index
                    && opp.edge_index == entry.edge_index
            });
            let bound = match partner {
                Some(p) => {
                    let shared_gap = entry.rank_gap_px.min(opp_ordered[p].rank_gap_px);
                    shared_gap * MAX_GAP_FRACTION - opp_depths[p]
                }
                None => own,
            };
            own.min(bound).max(floor)
        };

        // Re-lay the participating legs in descending channel-capacity order so
        // the widest-channel legs (the cross-container legs that must reach the
        // inter-rank gap to clear the destination container's label) claim the
        // deepest depths first. Ties break by the incoming deepest-first order.
        let mut order: Vec<usize> = (0..n).filter(|&i| participates[i]).collect();
        order.sort_by(|&a, &b| {
            cap(b)
                .partial_cmp(&cap(a))
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.cmp(&b))
        });

        for placed in 0..order.len() {
            let i = order[placed];
            let cap_i = cap(i);
            let mut depth = cap_i;
            loop {
                // The deepest already-fixed / already-placed overlapping depth
                // within the separation band of the current candidate, if any.
                let conflict = (0..n)
                    .filter(|&j| {
                        j != i
                            && spans_overlap(i, j)
                            && (!participates[j] || order[..placed].contains(&j))
                    })
                    .map(|j| depths[j])
                    .filter(|&dj| (dj - depth).abs() < JOG_SEPARATION_MIN_PX)
                    .reduce(f32::min);
                match conflict {
                    Some(dj) => {
                        depth = dj - JOG_SEPARATION_MIN_PX;
                        if depth <= floor {
                            depth = floor;
                            break;
                        }
                    }
                    None => break,
                }
            }
            depths[i] = depth.clamp(floor, cap_i);
        }
    }

    /// Writes a protrusion value to the appropriate slot in `result`.
    ///
    /// For node endpoints the fixed `entry.envelope_clearance` (the node's own
    /// edge-label wrapper width) is added to the band-distributed `protrusion`,
    /// so the total protrusion spans the node's own label from the inner node
    /// face and then continues for the band depth within the post-envelope
    /// routing channel. This makes node and spacer protrusions compose
    /// additively in the same channel -- the band invariant then guarantees a
    /// node tip never overshoots the adjacent spacer tip.
    ///
    /// To-endpoint protrusions are floored to `TO_PROTRUSION_MIN_PX` (capped
    /// by the entry's own gap allowance) so the straight segment entering the
    /// to-node clears the arrow head before the Z/S bend.
    fn protrusion_write(
        entry: &RankGapEntry,
        protrusion: f32,
        result: &mut [Vec<OrthoProtrusionParams>],
    ) {
        let params = &mut result[entry.pass1_group_index][entry.edge_index];
        match entry.endpoint_kind {
            RankGapEndpointKind::FromEndpoint => {
                params.from_protrusion = protrusion + entry.envelope_clearance;
            }
            RankGapEndpointKind::ToEndpoint => {
                // The arrow-head floor applies to the full straight segment
                // entering the node, so it is measured against the whole
                // inner-face gap (`rank_gap_px + envelope_clearance`).
                let to_protrusion_min = TO_PROTRUSION_MIN_PX
                    .min((entry.rank_gap_px + entry.envelope_clearance) * MAX_GAP_FRACTION);
                params.to_protrusion =
                    (protrusion + entry.envelope_clearance).max(to_protrusion_min);
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
            .map(|info| {
                OrthoProtrusionGeometry::face_center(
                    info,
                    pass1_info.from_face.unwrap_or(NodeFace::Bottom),
                )
            })
            .unwrap_or((pass1_info.from_node_x, pass1_info.from_node_y));

        let (to_x, to_y) = to_info
            .map(|info| {
                OrthoProtrusionGeometry::face_center(
                    info,
                    pass1_info.to_face.unwrap_or(NodeFace::Top),
                )
            })
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
                OrthoProtrusionGeometry::axis_distance(
                    to_x,
                    to_y,
                    last_spacer.exit_x,
                    last_spacer.exit_y,
                    face,
                )
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
        let dist = if let Some(other_div_ancestor_id) =
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
            let (other_bx, other_by) =
                OrthoProtrusionGeometry::face_center(other_ancestor_info, other_opposite_face);
            let capped = OrthoProtrusionGeometry::axis_distance(
                this_face_x,
                this_face_y,
                other_bx,
                other_by,
                face,
            );
            full_dist.min(capped)
        } else {
            full_dist
        };

        // For spacer-crossing edges, the band distributes protrusion depths in
        // the *post-envelope* routing channel: subtract this node's own
        // edge-label wrapper width so the channel starts at the envelope face
        // (the fixed wrapper width is re-added in `protrusion_write`). Without
        // spacers the band stays in the full inner-face gap (the divergent-
        // sibling adjustment provides the own-label clearance instead).
        if spacer_coordinates.is_empty() {
            dist
        } else {
            let own_clearance = Self::envelope_clearance_for(this_node_id, face, svg_node_info_map);
            (dist - own_clearance).max(0.0)
        }
    }

    /// Returns the own edge-label wrapper width (envelope clearance) on `face`
    /// for the node identified by `node_id`, or `0.0` when its layout is not
    /// available.
    ///
    /// This is the depth of the node's own edge-label slot on that face -- the
    /// distance from the inner node face to the envelope face.
    fn envelope_clearance_for<'id>(
        node_id: &NodeId<'id>,
        face: NodeFace,
        svg_node_info_map: &SvgNodeInfoByNodeId<'_, 'id>,
    ) -> f32 {
        svg_node_info_map
            .get(node_id)
            .map(|&info| Self::own_envelope_clearance(info, face))
            .unwrap_or(0.0)
    }

    /// Finalises protrusion depths for same-rank (cycle) edges.
    ///
    /// After gap-based protrusion assignment in Step 2–3, some cycle edges may
    /// have a `to_protrusion` assigned (those whose `Top`/`Bottom` face
    /// registered in an adjacent rank gap, floored for arrow-head clearance
    /// by `protrusion_write`), while others still have zero (boundary-rank
    /// edges, or `Left`/`Right` face edges with no rank gap). The
    /// divergent-sibling adjustment (Step 5) may also have raised either
    /// protrusion.
    ///
    /// This step handles both:
    ///
    /// 1. **Registered / adjusted cycle edges** (`to_protrusion > 0` or
    ///    `from_protrusion > 0`): equalizes both endpoints at the larger depth
    ///    (with `MIN_PROTRUSION_PX` as a floor), creating a symmetric U-shaped
    ///    routing arc.
    ///
    /// 2. **Unregistered cycle edges** (both protrusions zero): groups edges by
    ///    `(from_face, rank_from)` -- all edges routing in the same direction
    ///    at the same rank. Within each group, sorts by face offset then
    ///    cross-axis coordinate (same ordering as `protrusions_assign` for
    ///    single-side entries). Assigns stacked depths from the arrow-head
    ///    clearance floor:
    ///    - N edges in group -> depths `[TO_PROTRUSION_MIN + (N-1) * MIN, ..,
    ///      TO_PROTRUSION_MIN + MIN, TO_PROTRUSION_MIN]`
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

                if params.to_protrusion > 0.0 || params.from_protrusion > 0.0 {
                    // Registered in the adjacent rank gap (to_protrusion,
                    // already floored for arrow-head clearance by
                    // `protrusion_write`) and/or raised by the
                    // divergent-sibling adjustment (Step 5): equalize from
                    // and to at the larger depth.
                    let depth = params
                        .to_protrusion
                        .max(params.from_protrusion)
                        .max(MIN_PROTRUSION_PX);
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

            for (k, entry) in group.iter().enumerate() {
                // Slot N-1 (last sorted entry) -> innermost arc at the
                // arrow-head clearance floor (`TO_PROTRUSION_MIN_PX`).
                // Slot 0 (first sorted entry) -> outermost arc, stacked
                // `MIN_PROTRUSION_PX` apart. Unregistered cycle edges
                // protrude into open space (boundary ranks or Left/Right
                // faces), so no gap cap applies.
                let depth = TO_PROTRUSION_MIN_PX + (n - 1 - k) as f32 * MIN_PROTRUSION_PX;
                let params = &mut result[entry.group_idx][entry.edge_idx];
                params.from_protrusion = depth;
                params.to_protrusion = depth;
            }

            group_start = group_end;
        }
    }

    /// Registers the cycle edge's single U-depth entry in the adjacent rank
    /// gap so protrusion depths are distributed proportionally to the
    /// available gap space.
    ///
    /// For cycle edges (`rank_from == rank_to`), both endpoints are at the same
    /// rank and use the same face. Depending on the face, the entry is
    /// registered in an adjacent gap:
    ///
    /// - `Top` face at rank R -> register in gap `(R-1, R)` on the `High` side.
    ///   Skipped if R == 0 (no gap above).
    /// - `Bottom` face at rank R -> register in gap `(R, R+1)` on the `Low`
    ///   side.
    /// - `Left` / `Right` faces -> return early;
    ///   `protrusions_assign_cycle_edges` (Step 6) handles the fallback.
    ///
    /// Only one entry is registered per edge, with the `ToEndpoint` kind so
    /// the arrow-head clearance floor in `protrusion_write` applies.
    /// `protrusions_assign_cycle_edges` (Step 6) later copies
    /// `to_protrusion` to `from_protrusion` so both endpoints protrude equally,
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
        let cross_axis_from = OrthoProtrusionGeometry::cross_axis_coord(
            pass1_info.from_node_x,
            pass1_info.from_node_y,
            from_face,
        );

        // Step 11: Register the edge's single U-depth entry in the rank gap.
        //
        // The entry is registered as a `ToEndpoint` so `protrusion_write`
        // applies the arrow-head clearance floor (`TO_PROTRUSION_MIN_PX`,
        // capped by this entry's gap allowance) -- cycle edges and self-loops
        // also carry an arrow head at their to-endpoint. The sorting keys
        // (face offset, cross-axis) are taken from the from side; both
        // endpoints share the same face and rank, so they order the U-arcs
        // identically. `protrusions_assign_cycle_edges` (Step 6) copies
        // to_protrusion to from_protrusion afterward to produce a symmetric
        // U-shaped arc.
        rank_gap_entries
            .entry(gap_key)
            .or_default()
            .push(RankGapEntry {
                pass1_group_index: group_idx,
                edge_index: edge_idx,
                endpoint_kind: RankGapEndpointKind::ToEndpoint,
                gap_side,
                cross_axis_coord: cross_axis_from,
                // Cycle edges route a U-arc in place; there is no lateral jog to
                // another column, so the span is zero-width (overlaps nothing).
                jog_far_cross_axis: cross_axis_from,
                face_offset: from_offset,
                rank_gap_px: from_rank_gap_px,
                // Cycle edges register in an open adjacent gap (no spacers),
                // so the own-label clearance is supplied elsewhere.
                envelope_clearance: 0.0,
            });
    }

    /// Adjusts protrusion values to clear sibling nodes of the Divergent
    /// ancestor of each edge endpoint, staggering endpoints that clear the
    /// **same** sibling row so they do not collapse to one depth.
    ///
    /// For each edge, the from-endpoint's protrusion must be large enough
    /// that the routing segment is in the gap between the tallest sibling
    /// node (at the same rank as the Divergent ancestor) and the to-node.
    /// Symmetric logic applies for the to-endpoint.
    ///
    /// # Why staggering is needed
    ///
    /// Two edges between nested nodes can share the same inter-rank gap and
    /// both clear the **same** divergent-ancestor sibling row (e.g. two edges
    /// from different sibling containers into a third nested container). The
    /// per-endpoint clearance is then (near-)identical for both, so a naive
    /// per-edge `max` collapses the distinct band depths assigned in Step 3
    /// onto one value -- their lateral routing segments then overlap.
    ///
    /// To avoid this, endpoints are grouped by [`DivergentSiblingRowKey`] (the
    /// sibling row they clear). Within each group the protrusions are staggered
    /// `MIN_PROTRUSION_PX` apart, deepest-first by cross-axis coordinate
    /// (matching the spatial ordering in `protrusions_assign`). A group of size
    /// one reduces to the previous `max(protrusion, min_clearance)` behaviour,
    /// so single-edge layouts are unchanged.
    ///
    /// Cycle edges are excluded from the grouping and keep the independent
    /// `max` path; they are equalised / stacked separately in Step 6
    /// (`protrusions_assign_cycle_edges`).
    ///
    /// # Future alternative (not implemented)
    ///
    /// Staggering every endpoint that clears a row is conservative: edges whose
    /// lateral (cross-axis) spans do not actually intersect could safely share
    /// a depth. A tighter packing would compute each edge's lateral span
    /// and only force differing depths for edges whose spans overlap
    /// (interval-graph coloring), at the cost of materially more complexity
    /// and harder cross-rank-direction determinism.
    fn protrusions_adjust_for_divergent_siblings<'id>(
        all_pass1_groups: &[EdgeGroupPass1<'_, 'id>],
        all_spacer_coordinates: &[Vec<Vec<SpacerCoordinates>>],
        node_nesting_infos: &NodeNestingInfos<'id>,
        node_ranks_nested: &NodeRanksNested<'id>,
        svg_node_info_map: &SvgNodeInfoByNodeId<'_, 'id>,
        entity_types: &EntityTypes<'id>,
        result: &mut [Vec<OrthoProtrusionParams>],
    ) {
        // Non-cycle endpoint adjustments, grouped by the sibling row they clear.
        // Endpoints in the same group are staggered to distinct depths.
        let mut row_groups: Map<DivergentSiblingRowKey<'id>, Vec<EndpointAdjustment>> = Map::new();

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
                // When the edge has spacers (rank-based LCA-gap spacers or
                // cross-container spacers), they already handle routing through
                // the from-node's and to-node's containers. The from / to
                // protrusion then only needs to reach the nearest spacer, not
                // exit the entire container. Applying the divergent-sibling
                // adjustment in this case would force the protrusion all the way
                // to the container's far boundary, causing the path to overshoot
                // the spacer and produce a zigzag.
                //
                // Separation of multiple spacer-crossing edges that enter the
                // same to-node is handled separately, in
                // `protrusions_separate_spacer_approach_channels` (Step 5.5).
                let edge_has_spacers = all_spacer_coordinates
                    .get(group_idx)
                    .and_then(|g| g.get(edge_idx))
                    .map(|spacers| !spacers.is_empty())
                    .unwrap_or(false);

                // === From endpoint === //
                if !edge_has_spacers
                    && let Some(from_face) = pass1_info.from_face
                    && let Some(extent) = Self::min_protrusion_divergent_sibling_extent(
                        &pass1_info.edge.from,
                        &pass1_info.edge.to,
                        from_face,
                        node_nesting_infos,
                        node_ranks_nested,
                        svg_node_info_map,
                        entity_types,
                    )
                {
                    Self::divergent_sibling_adjustment_record(
                        pass1_info,
                        group_idx,
                        edge_idx,
                        AdjustEndpoint::From,
                        from_face,
                        extent,
                        &mut row_groups,
                        result,
                    );
                }

                // === To endpoint === //
                if !edge_has_spacers
                    && let Some(to_face) = pass1_info.to_face
                    && let Some(extent) = Self::min_protrusion_divergent_sibling_extent(
                        &pass1_info.edge.to,
                        &pass1_info.edge.from,
                        to_face,
                        node_nesting_infos,
                        node_ranks_nested,
                        svg_node_info_map,
                        entity_types,
                    )
                {
                    Self::divergent_sibling_adjustment_record(
                        pass1_info,
                        group_idx,
                        edge_idx,
                        AdjustEndpoint::To,
                        to_face,
                        extent,
                        &mut row_groups,
                        result,
                    );
                }
            }
        }

        // === Apply staggered depths per sibling row === //
        Self::protrusions_adjust_stagger_row_groups(row_groups, result);
    }

    /// Records one endpoint's divergent-sibling adjustment.
    ///
    /// Cycle edges keep the independent `max(protrusion, min_clearance)` path
    /// (they are equalised in Step 6); non-cycle endpoints are queued into
    /// `row_groups` for staggering. Endpoints whose clearance is `0.0` already
    /// extend past their sibling row and need no adjustment.
    #[allow(clippy::too_many_arguments)]
    fn divergent_sibling_adjustment_record<'id>(
        pass1_info: &EdgePass1Info<'_, 'id>,
        group_idx: usize,
        edge_idx: usize,
        endpoint: AdjustEndpoint,
        face: NodeFace,
        extent: DivergentSiblingExtent<'id>,
        row_groups: &mut Map<DivergentSiblingRowKey<'id>, Vec<EndpointAdjustment>>,
        result: &mut [Vec<OrthoProtrusionParams>],
    ) {
        if extent.min_protrusion <= 0.0 {
            return;
        }

        if pass1_info.is_cycle_edge {
            // Cycle edges keep the independent max path; they are equalised
            // in Step 6 (`protrusions_assign_cycle_edges`).
            let params = &mut result[group_idx][edge_idx];
            match endpoint {
                AdjustEndpoint::From => {
                    params.from_protrusion = params.from_protrusion.max(extent.min_protrusion);
                }
                AdjustEndpoint::To => {
                    params.to_protrusion = params.to_protrusion.max(extent.min_protrusion);
                }
            }
            return;
        }

        let (node_x, node_y) = match endpoint {
            AdjustEndpoint::From => (pass1_info.from_node_x, pass1_info.from_node_y),
            AdjustEndpoint::To => (pass1_info.to_node_x, pass1_info.to_node_y),
        };
        let cross_axis_coord = OrthoProtrusionGeometry::cross_axis_coord(node_x, node_y, face);

        row_groups
            .entry(extent.row_key)
            .or_default()
            .push(EndpointAdjustment {
                group_idx,
                edge_idx,
                endpoint,
                cross_axis_coord,
                min_protrusion: extent.min_protrusion,
            });
    }

    /// Staggers the queued endpoint adjustments so endpoints clearing the same
    /// sibling row receive distinct protrusion depths.
    ///
    /// Each endpoint's base clearance is its **own** `min_protrusion`, which
    /// already encodes reaching the shared sibling-row extreme coordinate from
    /// that endpoint's own face. Endpoints are ordered deepest-first by
    /// cross-axis coordinate (descending, matching the `side_sort` tie-break in
    /// `protrusions_assign`) and spaced `MIN_PROTRUSION_PX` apart so endpoints
    /// landing at the same extreme coordinate get distinct depths. The result
    /// is `max`-ed onto the existing protrusion so a larger Step 3 / cycle
    /// value is never reduced.
    ///
    /// The per-endpoint base (rather than a group-wide `max` of clearances) is
    /// required because `min_protrusion` is a *relative* delta from each
    /// endpoint's own face to a shared absolute target. Endpoints in one row
    /// group can sit at different face coordinates (e.g. nodes nested in
    /// containers of different widths), so `max`-ing the relative deltas and
    /// applying the result to every endpoint over-shoots the endpoints that are
    /// closer to the target -- driving their protrusion tips far past the
    /// sibling row.
    fn protrusions_adjust_stagger_row_groups<'id>(
        mut row_groups: Map<DivergentSiblingRowKey<'id>, Vec<EndpointAdjustment>>,
        result: &mut [Vec<OrthoProtrusionParams>],
    ) {
        for endpoints in row_groups.values_mut() {
            // Deepest-first by cross-axis descending; break ties by edge
            // identity for determinism.
            endpoints.sort_by(|a, b| {
                b.cross_axis_coord
                    .partial_cmp(&a.cross_axis_coord)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then(a.group_idx.cmp(&b.group_idx))
                    .then(a.edge_idx.cmp(&b.edge_idx))
            });

            let n = endpoints.len();
            for (i, endpoint) in endpoints.iter().enumerate() {
                // Each endpoint clears its own sibling-row extent, then is
                // staggered apart so endpoints landing at the same extreme
                // coordinate receive distinct depths.
                let staggered = endpoint.min_protrusion + (n - 1 - i) as f32 * MIN_PROTRUSION_PX;
                let params = &mut result[endpoint.group_idx][endpoint.edge_idx];
                match endpoint.endpoint {
                    AdjustEndpoint::From => {
                        params.from_protrusion = params.from_protrusion.max(staggered);
                    }
                    AdjustEndpoint::To => {
                        params.to_protrusion = params.to_protrusion.max(staggered);
                    }
                }
            }
        }
    }

    /// Separates the approach channels of cross-container spacer edges that
    /// enter the same to-node.
    ///
    /// Multiple such edges share the narrow gap between their (co-located) last
    /// spacer exit and the to-node face. Their `to` protrusions and
    /// last-spacer `exit` protrusions otherwise floor to (near-)identical
    /// depths -- because the overloaded rank gap leaves no band to separate
    /// them -- so their vertical approach legs (which sit at the midpoint of
    /// the spacer-exit tip and the to tip) coincide and overlap.
    ///
    /// This step assigns each edge in such a group a distinct leg coordinate
    /// within the gap and sets both the `to` protrusion and the last spacer's
    /// `exit` protrusion so their tips meet on that leg (a clean straight
    /// vertical/horizontal approach with no Z/S wiggle). Edges are ordered by
    /// cross-axis so the leg ordering does not cross the spacer rows. Groups of
    /// a single edge (the common case) are left unchanged.
    fn protrusions_separate_spacer_approach_channels<'id>(
        all_pass1_groups: &[EdgeGroupPass1<'_, 'id>],
        all_spacer_coordinates: &[Vec<Vec<SpacerCoordinates>>],
        svg_node_info_map: &SvgNodeInfoByNodeId<'_, 'id>,
        result: &mut [Vec<OrthoProtrusionParams>],
    ) {
        /// One spacer edge sharing an approach channel into a to-node.
        struct ChannelEntry {
            group_idx: usize,
            edge_idx: usize,
            /// Index of the last spacer (whose `exit` protrusion is set).
            spacer_index: usize,
            /// Inner face coordinate of the to-node along the rank axis.
            to_face_coord: f32,
            /// Last spacer exit coordinate along the rank axis.
            exit_coord: f32,
            /// Minimum `to` protrusion (arrow-head clearance and own label).
            to_min: f32,
            /// Cross-axis coordinate (perpendicular to the rank axis) used to
            /// order the leg assignment.
            cross_axis_coord: f32,
        }

        /// Groups edges whose last spacer exit feeds the same to-node face at
        /// the same exit coordinate.
        #[derive(PartialEq, Eq, Hash)]
        struct ChannelKey<'id> {
            to_node_id: NodeId<'id>,
            face: NodeFace,
            /// Exit coordinate quantised to avoid float-key issues.
            exit_coord_milli: i64,
        }

        let mut channels: Map<ChannelKey<'id>, Vec<ChannelEntry>> = Map::new();

        for (group_idx, group) in all_pass1_groups.iter().enumerate() {
            for (edge_idx, pass1_info) in group.pass1_infos.iter().enumerate() {
                if pass1_info.is_cycle_edge {
                    continue;
                }
                let Some(to_face) = pass1_info.to_face else {
                    continue;
                };
                let spacers = &all_spacer_coordinates[group_idx][edge_idx];
                let Some(last_spacer) = spacers.last() else {
                    continue;
                };
                let Some(&to_info) = svg_node_info_map.get(&pass1_info.edge.to) else {
                    continue;
                };

                let to_face_coord = Self::face_coord_for_node(to_info, to_face);
                let (exit_coord, cross_axis_coord) = match to_face {
                    NodeFace::Left | NodeFace::Right => (last_spacer.exit_x, last_spacer.entry_y),
                    NodeFace::Top | NodeFace::Bottom => (last_spacer.exit_y, last_spacer.entry_x),
                };
                let to_min =
                    TO_PROTRUSION_MIN_PX.max(Self::own_envelope_clearance(to_info, to_face));

                channels
                    .entry(ChannelKey {
                        to_node_id: pass1_info.edge.to.clone(),
                        face: to_face,
                        exit_coord_milli: (exit_coord * 1000.0) as i64,
                    })
                    .or_default()
                    .push(ChannelEntry {
                        group_idx,
                        edge_idx,
                        spacer_index: spacers.len() - 1,
                        to_face_coord,
                        exit_coord,
                        to_min,
                        cross_axis_coord,
                    });
            }
        }

        for entries in channels.values_mut() {
            let n = entries.len();
            if n < 2 {
                continue;
            }

            // The gap between the spacer exit and the to-node face along the
            // rank axis. All entries share the same to-node and exit coordinate.
            let available = (entries[0].to_face_coord - entries[0].exit_coord).abs();
            let to_min = entries
                .iter()
                .map(|entry| entry.to_min)
                .fold(0.0_f32, f32::max);
            // The leg also stays at least `MIN_PROTRUSION_PX` past the spacer
            // exit so the spacer's exit stub is never zero-length.
            let span = available - to_min - MIN_PROTRUSION_PX;
            if span <= 0.0 {
                // Gap too tight to separate; leave the band-assigned values.
                continue;
            }

            // Order by cross-axis ascending: the edge nearest one side of the
            // channel turns up nearest the spacer exit, so its short spacer
            // stub does not run across another edge's leg.
            entries.sort_by(|a, b| {
                a.cross_axis_coord
                    .partial_cmp(&b.cross_axis_coord)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then(a.group_idx.cmp(&b.group_idx))
                    .then(a.edge_idx.cmp(&b.edge_idx))
            });

            for (i, entry) in entries.iter().enumerate() {
                // The smallest cross-axis edge (i = 0) turns up nearest the
                // spacer exit (largest `to` protrusion); the largest turns up
                // nearest the to-node (smallest `to` protrusion).
                let frac = (n - 1 - i) as f32 / (n - 1) as f32;
                let to_protrusion = to_min + span * frac;
                let exit_protrusion = available - to_protrusion;

                let params = &mut result[entry.group_idx][entry.edge_idx];
                params.to_protrusion = to_protrusion;
                if let Some(spacer_protrusion) =
                    params.spacer_protrusions.get_mut(entry.spacer_index)
                {
                    spacer_protrusion.exit_protrusion = exit_protrusion;
                }
            }
        }
    }

    /// Nests the approach legs of edges that enter the **same** to-node face
    /// from **different rank-gap buckets**.
    ///
    /// A cross-container (spacer-crossing) edge has its to-endpoint keyed by
    /// the LCA-level rank gap, while a plain edge into the same nested node
    /// is keyed by that node's container-level rank gap. The two never
    /// compete in `protrusions_assign`, so their approach legs (protrusion
    /// depths) are chosen independently and can cross -- e.g. in `0036`,
    /// the local edge `t_c_00 -> t_c_01` and the cross-container edge
    /// `t_a_01 -> t_c_01` both enter `t_c_01`'s `Top` face, and the
    /// cross-container edge's deeper leg sweeps across the local edge
    /// twice.
    ///
    /// This pass groups non-cycle to-endpoints by `(to-node, to-face)`. For
    /// each group that **mixes** a spacer-crossing edge with at least one
    /// other edge, it re-assigns nested approach depths within the physical
    /// band between the to-face and the nearest same-container sibling on
    /// the approach side: the edge entering nearest one side of the face
    /// gets the deepest leg, fanning toward the other side (matching the
    /// deepest-first-by-offset convention in `protrusions_assign`), so the
    /// legs no longer cross. For spacer-crossing edges the last spacer's
    /// `exit` protrusion is updated so the spacer exit and the to tip still
    /// meet on the chosen leg. Groups without this mix are left to the
    /// existing rank-gap assignment, so single-bucket layouts (and
    /// pure spacer-channel groups handled in Step 5.5) are unchanged.
    #[allow(clippy::too_many_arguments)]
    fn protrusions_separate_shared_to_face_channels<'id>(
        all_pass1_groups: &[EdgeGroupPass1<'_, 'id>],
        to_slot_indices_all: &[Vec<Option<usize>>],
        face_offsets_by_node_face: &NodeIdAndFaceToContactPointOffsets<'id>,
        all_spacer_coordinates: &[Vec<Vec<SpacerCoordinates>>],
        svg_node_info_map: &SvgNodeInfoByNodeId<'_, 'id>,
        node_nesting_infos: &NodeNestingInfos<'id>,
        node_ranks_nested: &NodeRanksNested<'id>,
        entity_types: &EntityTypes<'id>,
        result: &mut [Vec<OrthoProtrusionParams>],
    ) {
        /// One edge entering a shared to-node face.
        struct ToFaceEntry {
            group_idx: usize,
            edge_idx: usize,
            /// To-endpoint face offset; orders contacts along the face axis.
            to_offset: f32,
            /// `(last spacer index, last spacer exit coordinate)` along the
            /// rank axis, for spacer-crossing edges.
            last_spacer: Option<(usize, f32)>,
        }

        #[derive(PartialEq, Eq, Hash)]
        struct ToFaceKey<'id> {
            to_node_id: NodeId<'id>,
            face: NodeFace,
        }

        let mut groups: Map<ToFaceKey<'id>, Vec<ToFaceEntry>> = Map::new();

        for (group_idx, group) in all_pass1_groups.iter().enumerate() {
            let to_slot_indices = &to_slot_indices_all[group_idx];
            for (edge_idx, pass1_info) in group.pass1_infos.iter().enumerate() {
                if pass1_info.is_cycle_edge {
                    continue;
                }
                let Some(to_face) = pass1_info.to_face else {
                    continue;
                };

                let to_offset = Self::face_offset_resolve(
                    pass1_info,
                    to_slot_indices[edge_idx],
                    false,
                    face_offsets_by_node_face,
                );

                let spacers = &all_spacer_coordinates[group_idx][edge_idx];
                let last_spacer = spacers.last().map(|spacer| {
                    let exit_coord = match to_face {
                        NodeFace::Left | NodeFace::Right => spacer.exit_x,
                        NodeFace::Top | NodeFace::Bottom => spacer.exit_y,
                    };
                    (spacers.len() - 1, exit_coord)
                });

                groups
                    .entry(ToFaceKey {
                        to_node_id: pass1_info.edge.to.clone(),
                        face: to_face,
                    })
                    .or_default()
                    .push(ToFaceEntry {
                        group_idx,
                        edge_idx,
                        to_offset,
                        last_spacer,
                    });
            }
        }

        for (key, mut entries) in groups {
            let n = entries.len();
            if n < 2 {
                continue;
            }

            // Only act on the mixed cross-bucket case: at least one
            // spacer-crossing edge and at least one other edge. Pure single-
            // bucket fan-ins are handled by Step 3; pure spacer-channel groups
            // by Step 5.5.
            let any_spacer = entries.iter().any(|entry| entry.last_spacer.is_some());
            let any_plain = entries.iter().any(|entry| entry.last_spacer.is_none());
            if !(any_spacer && any_plain) {
                continue;
            }

            let Some(&to_info) = svg_node_info_map.get(&key.to_node_id) else {
                continue;
            };
            let to_face_coord = Self::face_coord_for_node(to_info, key.face);
            let Some(available) = Self::approach_band(
                &key.to_node_id,
                key.face,
                to_info,
                node_nesting_infos,
                node_ranks_nested,
                svg_node_info_map,
                entity_types,
            ) else {
                continue;
            };
            let to_min = TO_PROTRUSION_MIN_PX.max(Self::own_envelope_clearance(to_info, key.face));
            let band_cap = available * MAX_GAP_FRACTION;
            if band_cap <= to_min {
                // Gap too tight to nest distinct legs; leave the band-assigned
                // values.
                continue;
            }

            // Order leftmost / topmost first (smallest offset = deepest leg),
            // matching the deepest-first-by-offset convention in
            // `protrusions_assign`.
            entries.sort_by(|a, b| {
                a.to_offset
                    .partial_cmp(&b.to_offset)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then(a.group_idx.cmp(&b.group_idx))
                    .then(a.edge_idx.cmp(&b.edge_idx))
            });

            for (i, entry) in entries.iter().enumerate() {
                // Stack depths from the arrow-head clearance floor upward, so
                // the deepest leg (smallest offset) clears the others without
                // overshooting the shared band.
                let depth = (to_min + (n - 1 - i) as f32 * MIN_PROTRUSION_PX).min(band_cap);
                let params = &mut result[entry.group_idx][entry.edge_idx];

                if let Some((spacer_index, exit_coord)) = entry.last_spacer {
                    // Keep the spacer exit and the to tip meeting on the leg:
                    // their combined depth spans the spacer-exit-to-face gap.
                    let spacer_available = (to_face_coord - exit_coord).abs();
                    let depth = depth.min((spacer_available - MIN_PROTRUSION_PX).max(to_min));
                    params.to_protrusion = depth;
                    if let Some(spacer_protrusion) = params.spacer_protrusions.get_mut(spacer_index)
                    {
                        spacer_protrusion.exit_protrusion =
                            (spacer_available - depth).max(MIN_PROTRUSION_PX);
                    }
                } else {
                    params.to_protrusion = depth;
                }
            }
        }
    }

    /// Returns the pixel distance from a to-node face to the nearest
    /// same-container, same-category sibling on the approach side, or `None`
    /// when there is no such sibling.
    ///
    /// This is the physical band an incoming edge's approach leg may occupy
    /// before it would overlap a sibling node. The approach side is the
    /// outward direction of `to_face`: `+y` for `Bottom`, `-y` for `Top`, `+x`
    /// for `Right`, `-x` for `Left`. Only siblings whose near edge lies in that
    /// outward direction are considered, so the computation is robust to
    /// `RankDir`.
    fn approach_band<'id>(
        to_node_id: &NodeId<'id>,
        to_face: NodeFace,
        to_info: &SvgNodeInfo<'_>,
        node_nesting_infos: &NodeNestingInfos<'id>,
        node_ranks_nested: &NodeRanksNested<'id>,
        svg_node_info_map: &SvgNodeInfoByNodeId<'_, 'id>,
        entity_types: &EntityTypes<'id>,
    ) -> Option<f32> {
        let parent_container = node_nesting_infos
            .get(to_node_id)
            .and_then(|node_nesting_info| {
                node_nesting_info
                    .ancestor_chain
                    .len()
                    .checked_sub(2)
                    .map(|parent_index| &node_nesting_info.ancestor_chain[parent_index])
            });
        let ranks_in_scope = node_ranks_nested.ranks_for(parent_container)?;

        let category = Self::node_category(to_node_id, entity_types);
        let to_face_coord = Self::face_coord_for_node(to_info, to_face);
        let outward_positive = matches!(to_face, NodeFace::Bottom | NodeFace::Right);

        let boundary = ranks_in_scope
            .iter()
            .filter(|(sibling_id, _)| *sibling_id != to_node_id)
            .filter(|(sibling_id, _)| Self::node_category(sibling_id, entity_types) == category)
            .filter_map(|(sibling_id, _)| svg_node_info_map.get(sibling_id).copied())
            .filter_map(|info| {
                // The sibling's near edge -- the face closest to the to-node
                // along the approach axis.
                let near = match to_face {
                    NodeFace::Top => info.y + info.height_collapsed,
                    NodeFace::Bottom => info.y,
                    NodeFace::Left => info.x + info.width,
                    NodeFace::Right => info.x,
                };
                let in_outward = if outward_positive {
                    near >= to_face_coord
                } else {
                    near <= to_face_coord
                };
                in_outward.then_some(near)
            })
            .reduce(|acc, near| {
                if outward_positive {
                    acc.min(near)
                } else {
                    acc.max(near)
                }
            })?;

        Some((to_face_coord - boundary).abs())
    }

    /// Returns the depth of a node's own edge-label slot on `face` -- the
    /// distance from the node's inner face to its envelope face along the
    /// protrusion axis.
    ///
    /// An endpoint protrusion must be at least this long so the Z/S bend
    /// clears the node's own edge label (including the markdown content
    /// padding) rather than overlapping it.
    fn own_envelope_clearance(info: &SvgNodeInfo<'_>, face: NodeFace) -> f32 {
        let face_sign: f32 = match face {
            NodeFace::Bottom | NodeFace::Right => 1.0,
            NodeFace::Top | NodeFace::Left => -1.0,
        };
        (face_sign
            * (Self::face_coord_for_endpoint(info, face) - Self::face_coord_for_node(info, face)))
        .max(0.0)
    }

    /// Computes the minimum protrusion needed for `node_id`'s endpoint to
    /// clear all sibling nodes of the node's Divergent ancestor at the LCA
    /// level, along with the [`DivergentSiblingRowKey`] identifying that
    /// sibling row.
    ///
    /// The Divergent ancestor is the ancestor of `node_id` that is a direct
    /// child of the LCA of (`node_id`, `other_node_id`).
    ///
    /// Returns `None` when no adjustment applies (e.g. the node has no
    /// divergent ancestor, its rank or layout is unavailable, or it has no
    /// same-rank siblings). The returned `min_protrusion` may still be `0.0`
    /// when the endpoint already extends past its sibling row.
    ///
    /// # Parameters
    ///
    /// * `node_id_from`: the endpoint node whose protrusion is being computed.
    /// * `node_id_to`: the opposite endpoint of the edge (used to find the
    ///   LCA).
    /// * `face`: the face at which `node_id_from` protrudes.
    fn min_protrusion_divergent_sibling_extent<'id>(
        node_id_from: &NodeId<'id>,
        node_id_to: &NodeId<'id>,
        face: NodeFace,
        node_nesting_infos: &NodeNestingInfos<'id>,
        node_ranks_nested: &NodeRanksNested<'id>,
        svg_node_info_map: &SvgNodeInfoByNodeId<'_, 'id>,
        entity_types: &EntityTypes<'id>,
    ) -> Option<DivergentSiblingExtent<'id>> {
        // 1. Compute LCA depth.
        let lca_depth = Self::lca_depth(node_id_from, node_id_to, node_nesting_infos);

        // 2. Find divergent ancestor of node_id.
        let divergent_ancestor_id_from =
            Self::divergent_ancestor_id(node_id_from, lca_depth, node_nesting_infos)?;

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
        let node_ranks = node_ranks_nested.ranks_for(divergent_ancestor_parent_id_from)?;
        let &div_ancestor_rank = node_ranks.get(divergent_ancestor_id_from)?;

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
        let &node_info_from = svg_node_info_map.get(node_id_from)?;
        // The protrusion length is applied from the node's inner (geometric)
        // face, not its envelope face, so measure `node_face_coord` from node
        // bounds. The sibling extents below stay on envelope bounds (and the
        // node itself is part of that sibling set), so the protrusion still
        // clears each node's full label area -- including the markdown content
        // padding -- before the Z/S bend.
        let node_face_coord = Self::face_coord_for_node(node_info_from, face);

        // 7. Find extreme sibling coordinate in the protrusion direction.
        let sibling_extreme =
            Self::same_rank_sibling_extreme(&same_rank_siblings, face, svg_node_info_map)?;

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
        let min_protrusion = (face_sign * (sibling_extreme - node_face_coord)).max(0.0);

        // The row key groups endpoints that clear this same sibling row so they
        // can be staggered to distinct depths (see
        // `protrusions_adjust_for_divergent_siblings`).
        Some(DivergentSiblingExtent {
            min_protrusion,
            row_key: DivergentSiblingRowKey {
                parent_container: divergent_ancestor_parent_id_from.cloned(),
                div_ancestor_rank,
                face,
                category: node_category,
            },
        })
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

    /// Returns the coordinate of a node's inner (non-envelope) face along the
    /// protrusion axis -- the actual point from which the edge stub protrudes.
    ///
    /// Unlike [`face_coord_for_endpoint`](Self::face_coord_for_endpoint), this
    /// excludes the envelope's edge-label wrapper slots, because the
    /// `from_protrusion` / `to_protrusion` length is applied from the node's
    /// geometric face (see `OrthoProtrusionGeometry::face_center` and the edge
    /// path builder), not from the envelope boundary.
    ///
    /// For `Bottom` face: the bottom edge y-coordinate of the node.
    /// For `Top` face: the top edge y-coordinate of the node.
    /// For `Right` face: the right edge x-coordinate of the node.
    /// For `Left` face: the left edge x-coordinate of the node.
    fn face_coord_for_node(info: &SvgNodeInfo<'_>, face: NodeFace) -> f32 {
        match face {
            NodeFace::Bottom => info.y + info.height_collapsed,
            NodeFace::Top => info.y,
            NodeFace::Right => info.x + info.width,
            NodeFace::Left => info.x,
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
