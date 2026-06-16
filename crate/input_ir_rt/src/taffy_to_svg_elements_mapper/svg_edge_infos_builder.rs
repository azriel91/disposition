use disposition_input_ir_model::EdgeAnimationActive;
use disposition_input_model::DiagramFocus;
use disposition_ir_model::{
    edge::{Edge, EdgeFaceAssignments, EdgeGroup, EdgeId},
    entity::EntityTypes,
    node::{NodeFace, NodeId, NodeNestingInfos, NodeRank, NodeRanksNested},
    IrDiagram,
};
use disposition_model_common::{
    edge::EdgeCurvature, entity::EntityType, theme::Css, Id, Map, RankDir,
};
use disposition_svg_model::{OrthoProtrusionParams, SvgEdgeInfo};
use disposition_taffy_model::{
    taffy::TaffyTree, EdgeIdToEdgeLabelTaffyNodeIds, EdgeIdToEdgeSpacerTaffyNodes, TaffyNodeCtx,
};
use kurbo::Shape;

use disposition_ir_model::entity::EntityTailwindClasses;
use disposition_model_common::edge::EdgeGroupId;

use crate::{
    input_to_ir_diagram_mapper::tailwind_focus_mode::TailwindFocusMode,
    taffy_to_svg_elements_mapper::{
        edge_face_contact_tracker::EdgeFaceContactTracker,
        edge_model::{
            EdgeAnimationParams, EdgeContactPointOffsets, EdgePathInfo, EdgeType, NodeIdAndFace,
            NodeIdAndFaceToContactPointOffsets, PathBounds, PathMidpoint,
        },
        edge_path_builder_pass_1::EdgeFaceOffset,
        ortho_protrusion_calculator::OrthoProtrusionCalculator,
        ArrowHeadBuilder, EdgeAnimationCalculator, EdgePathBuilderPass1, EdgePathBuilderPass2,
        EdgePathLocusCalculator, SpacerCoordinatesResolver, StringCharReplacer,
        SvgNodeInfoByNodeId,
    },
    AbsoluteCoordinates, EdgeFaceAssigner, EdgeIdGenerator, TaffyNodeAbsoluteCoordinatesCalculator,
};

/// Builds [`SvgEdgeInfo`]s for all edges in the diagram from edge groups and
/// node layout information.
#[derive(Clone, Copy, Debug)]
pub(super) struct SvgEdgeInfosBuilder;

impl SvgEdgeInfosBuilder {
    /// Builds [`SvgEdgeInfo`] for all edges in the diagram.
    ///
    /// This iterates over all edge groups and their edges, computing the
    /// curved path for each edge based on the relative positions of the
    /// source and target nodes.
    ///
    /// A global two-pass algorithm spreads edges that share the same
    /// node face so their contact points are evenly distributed around
    /// the face midpoint:
    ///
    /// 1. **Pass 1** -- iterate every edge group, build paths with zero
    ///    offsets, register face contacts, and store rank distances and target
    ///    node coordinates.
    /// 2. Sort contacts per face globally by rank distance and target
    ///    coordinate, then compute offsets.
    /// 3. **Pass 2** -- rebuild every path using the calculated offsets, then
    ///    emit `SvgEdgeInfo`s and animation CSS.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn build<'id>(
        ir_diagram: &IrDiagram<'id>,
        svg_node_info_map: &SvgNodeInfoByNodeId<'_, 'id>,
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        edge_spacer_taffy_nodes: &EdgeIdToEdgeSpacerTaffyNodes<'id>,
        edge_label_taffy_nodes: &EdgeIdToEdgeLabelTaffyNodeIds<'id>,
        tailwind_classes: &mut EntityTailwindClasses<'id>,
        css: &mut Css,
        edge_animation_active: EdgeAnimationActive,
        focus_mode: TailwindFocusMode<'_, 'id>,
    ) -> Vec<SvgEdgeInfo<'id>> {
        let IrDiagram {
            edge_groups,
            entity_types,
            edge_face_assignments,
            process_step_entities,
            render_options,
            ..
        } = ir_diagram;
        let edge_curvature = render_options.edge_curvature;
        let rank_dir = render_options.rank_dir;

        // Build a reverse map: entity (edge group) ID -> list of process step NodeIds.
        // This allows efficient lookup of which process steps reference a given edge
        // group when `OnProcessStepFocus` is selected.
        let entity_to_process_steps: Map<&Id<'id>, Vec<&NodeId<'id>>> = process_step_entities
            .iter()
            .fold(Map::new(), |mut acc, (process_step_node_id, entity_ids)| {
                entity_ids.iter().for_each(|entity_id| {
                    acc.entry(entity_id).or_default().push(process_step_node_id);
                });
                acc
            });

        // The keyframe percentages of an edge's animation should be proportional to the
        // length of the edge within the total length of all edges in its edge group.
        //
        // So we need to precompute the total length of all edges in each edge group,
        // and pass that in when computing the keyframe percentages.
        //
        // If we only computed the edge's keyframe percentages based on its index within
        // the edge group, the line would be animated faster because it would travel a
        // larger distance in the same amount of time, whereas it is easier to
        // understand if we got each edge to be animated at the same speed (which means
        // increasing the amount of time for that edge's animation).
        //
        // Algorithm:
        //
        // 1. Compute the total length of all edges in each edge group using the edge's
        //    `bounding_box` as an approximation, then sum them.
        // 2. The `total_animation_time` should be a constant
        //    `seconds_per_distance_units * total_length`.
        // 3. The `start_pct` ("request start") will be `preceding_edge_lengths_sum /
        //    total_length`.
        // 4. The `end_pct` ("request end") will be `(preceding_edge_lengths_sum +
        //    current_edge_length) / total_length`.
        // 5. The `duration` for each edge's animation will be the `total_animation_time
        //    * (edge_length / total_length)`.

        /// 0.3 seconds per 100 pixels
        const SECONDS_PER_PIXEL: f64 = 0.3 / 100.0;

        // === Global Pass 1: collect metadata and register face contacts === //

        let mut face_contact_tracker = EdgeFaceContactTracker::new();
        let mut all_pass1_groups: Vec<EdgeGroupPass1<'_, 'id>> = Vec::new();

        for (edge_group_id, edge_group) in edge_groups.iter() {
            let edge_group_pass1 = Self::build_edge_pass1_infos(
                rank_dir,
                edge_group_id,
                edge_group,
                entity_types,
                edge_face_assignments,
                svg_node_info_map,
                &ir_diagram.node_ranks_nested,
                &ir_diagram.node_nesting_infos,
                &mut face_contact_tracker,
            );
            all_pass1_groups.push(edge_group_pass1);
        }

        // === Global sort and offset computation === //

        let face_offsets_by_node_face = Self::face_offsets_compute(
            &mut all_pass1_groups,
            svg_node_info_map,
            &mut face_contact_tracker,
            edge_label_taffy_nodes,
            taffy_tree,
        );

        // === Global orthogonal protrusion computation === //
        //
        // Collect slot indices from all groups so the protrusion
        // computer can resolve face offsets.
        let from_slot_indices_all: Vec<Vec<Option<usize>>> = all_pass1_groups
            .iter()
            .map(|g| g.from_slot_indices.clone())
            .collect();
        let to_slot_indices_all: Vec<Vec<Option<usize>>> = all_pass1_groups
            .iter()
            .map(|g| g.to_slot_indices.clone())
            .collect();

        let ortho_protrusions_all = OrthoProtrusionCalculator::calculate(
            rank_dir,
            &all_pass1_groups,
            &from_slot_indices_all,
            &to_slot_indices_all,
            &face_offsets_by_node_face,
            svg_node_info_map,
            taffy_tree,
            edge_spacer_taffy_nodes,
            &ir_diagram.node_nesting_infos,
            &ir_diagram.node_ranks_nested,
            entity_types,
        );

        // === Global Pass 2: rebuild paths with offsets, emit SvgEdgeInfos === //

        let mut svg_edge_infos = Vec::new();

        for (group_index, edge_group_pass1) in all_pass1_groups.into_iter().enumerate() {
            let EdgeGroupPass1 {
                edge_group_id,
                edge_animation_params,
                pass1_infos,
                from_slot_indices,
                to_slot_indices,
            } = edge_group_pass1;

            let visible_segments_length = edge_animation_params.visible_segments_length;
            let ortho_protrusions = &ortho_protrusions_all[group_index];

            let edge_path_infos = Self::build_edge_path_infos_with_offsets(
                edge_curvature,
                rank_dir,
                &pass1_infos,
                &from_slot_indices,
                &to_slot_indices,
                &face_offsets_by_node_face,
                svg_node_info_map,
                taffy_tree,
                edge_spacer_taffy_nodes,
                visible_segments_length,
                ortho_protrusions,
            );

            // Total `travel` distance animated by the whole group: each edge
            // animates its `stroke-dashoffset` across `visible_segments_length +
            // trailing_gap` pixels, where `trailing_gap = max(path_length,
            // visible_segments_length)`. Summing these keeps every edge at a
            // constant pixel speed and makes the windows tile the cycle exactly.
            let edge_group_travel_total = edge_path_infos
                .iter()
                .map(|edge_path_info| {
                    let trailing_gap = edge_path_info.path_length.max(visible_segments_length);
                    visible_segments_length + trailing_gap
                })
                .sum::<f64>();
            // The end-of-cycle pause expressed in the same pixel-distance units
            // as `travel`, so the keyframe windows leave exactly
            // `pause_duration_secs` of dead time at the end of the cycle.
            let edge_group_pause_distance =
                edge_animation_params.pause_duration_secs / SECONDS_PER_PIXEL;
            let edge_group_cycle_distance = edge_group_travel_total + edge_group_pause_distance;
            let edge_group_animation_duration_total_s = SECONDS_PER_PIXEL * edge_group_travel_total
                + edge_animation_params.pause_duration_secs;

            // Look up the process steps associated with this edge group (by its
            // inner Id) once per group so every edge in the group can reuse the
            // result.
            let edge_group_id_inner: &Id<'id> = edge_group_id.as_ref();
            let associated_process_steps: &[&NodeId<'id>] = entity_to_process_steps
                .get(edge_group_id_inner)
                .map(Vec::as_slice)
                .unwrap_or(&[]);

            edge_path_infos.into_iter().for_each(|edge_path_info| {
                // Compute animation for interaction edges.
                let is_interaction_edge = entity_types
                    .get(AsRef::<Id<'_>>::as_ref(&edge_path_info.edge_id))
                    .map(|edge_entity_types| {
                        edge_entity_types
                            .iter()
                            .any(EntityType::is_interaction_edge)
                    })
                    .unwrap_or(false);

                if is_interaction_edge {
                    let css_animation_append_params = CssAnimationAppendParams {
                        tailwind_classes,
                        css,
                        edge_animation_params,
                        edge_group_cycle_distance,
                        edge_group_animation_duration_total_s,
                        edge_path_info: &edge_path_info,
                        edge_animation_active,
                        focus_mode,
                        associated_process_steps,
                    };
                    Self::css_animation_append(css_animation_append_params);
                }

                let EdgePathInfo {
                    edge_id,
                    edge,
                    edge_type: _,
                    path,
                    path_length: _,
                    preceding_travel: _,
                    ortho_protrusion_params,
                } = edge_path_info;

                let path_d = path.to_svg();

                // Compute arrowhead path.
                let (arrow_head_path, locus_path) = if is_interaction_edge {
                    // Origin-centred V-shape; CSS offset-path handles
                    // positioning and rotation.
                    let arrow_head_path = ArrowHeadBuilder::build_origin_arrow_head();
                    // Positioned V-shape at the `to` node end of the edge.
                    let arrow_head_path_at_to_node =
                        ArrowHeadBuilder::build_static_arrow_head(&path);
                    let locus_path =
                        EdgePathLocusCalculator::calculate(&path, &arrow_head_path_at_to_node);

                    (arrow_head_path, locus_path)
                } else {
                    // Positioned V-shape at the `to` node end of the edge.
                    let arrow_head_path = ArrowHeadBuilder::build_static_arrow_head(&path);
                    let locus_path = EdgePathLocusCalculator::calculate(&path, &arrow_head_path);

                    (arrow_head_path, locus_path)
                };
                let arrow_head_path_d = arrow_head_path.to_svg();
                let locus_path_d = locus_path.to_svg();

                let tooltip = ir_diagram
                    .entity_tooltips
                    .get(edge_id.as_ref())
                    .cloned()
                    .unwrap_or_default();

                svg_edge_infos.push(SvgEdgeInfo::new(
                    edge_id,
                    edge_group_id.clone(),
                    edge.from.clone(),
                    edge.to.clone(),
                    path_d,
                    arrow_head_path_d,
                    locus_path_d,
                    tooltip,
                    ortho_protrusion_params,
                ));
            });
        }

        svg_edge_infos
    }

    /// **Pass 1** for a single edge group: determines edge types, builds
    /// zero-offset paths, registers face contacts, and stores the rank
    /// distance and target node coordinates needed for the global
    /// rank-and-coordinate sorting phase.
    ///
    /// The returned `EdgeGroupPass1` contains everything needed for
    /// pass 2 to rebuild the paths with offsets.
    #[allow(clippy::too_many_arguments)]
    fn build_edge_pass1_infos<'edge, 'id>(
        rank_dir: RankDir,
        edge_group_id: &'edge EdgeGroupId<'id>,
        edge_group: &'edge EdgeGroup<'id>,
        entity_types: &'edge EntityTypes<'id>,
        edge_face_assignments: &EdgeFaceAssignments<'id>,
        svg_node_info_map: &'edge SvgNodeInfoByNodeId<'_, 'id>,
        node_ranks_nested: &NodeRanksNested<'id>,
        node_nesting_infos: &NodeNestingInfos<'id>,
        face_contact_tracker: &mut EdgeFaceContactTracker<'id>,
    ) -> EdgeGroupPass1<'edge, 'id> {
        let edge_animation_params = EdgeAnimationParams::default();
        let mut pass1_infos: Vec<EdgePass1Info<'edge, 'id>> = Vec::new();

        for (edge_index, edge) in edge_group.iter().enumerate() {
            let Some(from_info) = svg_node_info_map.get(&edge.from) else {
                // TODO: warn user that they probably got a Node ID wrong.
                continue;
            };
            let Some(to_info) = svg_node_info_map.get(&edge.to) else {
                // TODO: warn user that they probably got a Node ID wrong.
                continue;
            };

            let edge_id = EdgeIdGenerator::generate(edge_group_id, edge_index);
            let edge_type = Self::edge_type_determine(&edge_id, entity_types);

            // Compute rank distance before face selection so that same-rank
            // (cycle) edges can use clockwise face routing. Adjacent siblings
            // with the same direct parent, adjacent divergent-ancestor siblings
            // (cross-container edges where the divergent ancestors at the LCA
            // level are adjacent), and edges involving tag, process, or
            // process step nodes always use normal face selection regardless of
            // rank.
            //
            // Use LCA-level ranks so that cross-container edges (e.g. a top-level
            // node connecting to a nested node) are not incorrectly classified as
            // cycle edges. A simple local-context comparison can give false
            // positives: both endpoints might have rank 0 in their respective
            // parent containers while sitting at visually different positions in
            // the diagram (because their containers have different root-level
            // ranks).
            let (rank_from, rank_to) = Self::nodes_lca_ranks_compute(
                &edge.from,
                &edge.to,
                node_ranks_nested,
                node_nesting_infos,
            )
            .unwrap_or_else(|| {
                let rank_from = node_ranks_nested
                    .node_rank_for(&edge.from, node_nesting_infos)
                    .unwrap_or_default();
                let rank_to = node_ranks_nested
                    .node_rank_for(&edge.to, node_nesting_infos)
                    .unwrap_or_default();
                (rank_from, rank_to)
            });
            let rank_distance = rank_to.value().abs_diff(rank_from.value());
            let is_same_rank = rank_from == rank_to;
            // Self-loops are intentionally classified as cycle edges: both
            // checks below return `false` for `from == to`, so the protrusion
            // calculator distributes their U-shape depth alongside other
            // same-face cycle edges.
            let is_cycle_edge = is_same_rank
                && !Self::nodes_adjacent_siblings_are(&edge.from, &edge.to, node_nesting_infos)
                && !Self::nodes_divergent_ancestors_adjacent_siblings_are(
                    &edge.from,
                    &edge.to,
                    node_nesting_infos,
                );

            // Build the path with zero offsets to determine natural coordinates.
            let path = EdgePathBuilderPass1::build(rank_dir, from_info, to_info, edge_type);

            // Step 4.4 (Option B): Use pre-layout face assignments from
            // `IrDiagram::edge_face_assignments` to drive path routing.
            //
            // `EdgeFaceAssigner` computes faces from rank and sibling data
            // before layout (matching `cycle_edge_faces_select` for cycle
            // edges, and `select_edge_faces` for all other cases).  Using
            // the same source for both envelope-slot construction and path
            // routing ensures the label slot always sits on the face the
            // path exits.
            //
            // Contained edges (detected via pixel positions, consistent with
            // pass-2 path building) still bypass face-based contact points.
            //
            // Fallback to post-layout `faces_select` when no pre-layout
            // assignment exists (should not occur for a well-formed diagram).
            let (from_face, to_face) = if edge.is_self_loop() {
                // Self-loop: both contacts sit on the same rank-direction
                // face. The IR assignment stores only the from face (one
                // label slot); pass 1 duplicates it so the offset and
                // protrusion machinery treats the loop as two contacts on
                // the same face. Self-loops are classified as cycle edges
                // (`is_cycle_edge` above), so the cycle-edge protrusion
                // assignment gives both contacts the same depth.
                //
                // This branch must run before the contained check, which
                // would otherwise match (a node geometrically contains
                // itself) and clear both faces.
                let face = edge_face_assignments
                    .get(&edge_id)
                    .and_then(|assignment| assignment.from_face)
                    .unwrap_or_else(|| EdgeFaceAssigner::forward_faces(rank_dir).0);
                (Some(face), Some(face))
            } else if EdgePathBuilderPass1::is_node_contained_in(from_info, to_info) {
                // Contained edges bypass face-based contact points.
                (None, None)
            } else if let Some(assignment) = edge_face_assignments.get(&edge_id) {
                (assignment.from_face, assignment.to_face)
            } else {
                // Fallback for edges absent from pre-layout assignments.
                let faces =
                    EdgePathBuilderPass1::faces_select(rank_dir, from_info, to_info, is_cycle_edge);
                match faces {
                    Some((ff, tf)) => (Some(ff), Some(tf)),
                    None => (None, None),
                }
            };

            // Register contacts.
            if let Some(from_face) = from_face {
                face_contact_tracker.contact_register(edge.from.clone(), from_face);
            }
            if let Some(to_face) = to_face {
                face_contact_tracker.contact_register(edge.to.clone(), to_face);
            }

            // Compute path midpoint and bounds for curvature-center sorting.
            let path_midpoint = Self::path_midpoint_compute(&path);
            let path_bounds = Self::path_bounds_compute(&path);

            // Store to-node coordinates for tie-breaking during sorting.
            let to_node_x = to_info.x;
            let to_node_y = to_info.y;
            let from_node_x = from_info.x;
            let from_node_y = from_info.y;

            pass1_infos.push(EdgePass1Info {
                edge,
                edge_id,
                edge_type,
                from_face,
                to_face,
                path_midpoint,
                path_bounds,
                rank_distance,
                to_node_x,
                to_node_y,
                rank_from,
                rank_to,
                from_node_x,
                from_node_y,
                is_cycle_edge,
            });
        }

        // Slot assignment is deferred to the global phase; here we just
        // prepare per-face entry lists that the global phase will merge.

        let from_slot_indices: Vec<Option<usize>> = vec![None; pass1_infos.len()];
        let to_slot_indices: Vec<Option<usize>> = vec![None; pass1_infos.len()];

        // These will be filled globally by `face_offsets_compute`.

        EdgeGroupPass1 {
            edge_group_id,
            edge_animation_params,
            pass1_infos,
            from_slot_indices,
            to_slot_indices,
        }
    }

    /// Computes per-face offset vectors across **all** edge groups using
    /// rank-distance and target-coordinate sorting.
    ///
    /// For each (node, face) the algorithm:
    ///
    /// 1. Gathers contact entries from every edge touching that face.
    /// 2. Sorts entries by rank distance ascending, then by target node
    ///    coordinate along the face axis (x for Top/Bottom, y for Left/Right)
    ///    as a tie-breaker.
    /// 3. Assigns offset slots so that short-range edges get slots nearest to
    ///    the face midpoint, and longer-range edges fan outward.
    ///
    /// This prevents edges from crossing each other: short-range edges
    /// stay tight against the node and long-range edges arc around them.
    ///
    /// When an edge has a description label on a face, the contact point
    /// offset is derived from the label leaf's absolute SVG position so the
    /// edge path exits through the centre of the rendered label rather than
    /// cutting through it.  Edges without a label (zero-size leaf) fall back
    /// to the slot-based formula.
    fn face_offsets_compute<'edge, 'id>(
        all_pass1_groups: &mut Vec<EdgeGroupPass1<'edge, 'id>>,
        svg_node_info_map: &SvgNodeInfoByNodeId<'_, 'id>,
        face_contact_tracker: &mut EdgeFaceContactTracker<'id>,
        edge_label_taffy_nodes: &EdgeIdToEdgeLabelTaffyNodeIds<'id>,
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
    ) -> NodeIdAndFaceToContactPointOffsets<'id> {
        // Collect face contact entries per (node, face) across all groups.
        let mut face_contact_entries_by_node_face: Map<NodeIdAndFace<'id>, Vec<FaceContactEntry>> =
            Map::new();

        all_pass1_groups
            .iter()
            .enumerate()
            .for_each(|(pass1_group_index, edge_group_pass1)| {
                edge_group_pass1.pass1_infos.iter().enumerate().for_each(
                    |(edge_index, pass1_info)| {
                        if let Some(from_face) = pass1_info.from_face {
                            let node_id_and_face = NodeIdAndFace {
                                node_id: pass1_info.edge.from.clone(),
                                face: from_face,
                            };
                            face_contact_entries_by_node_face
                                .entry(node_id_and_face)
                                .or_default()
                                .push(FaceContactEntry {
                                    path_midpoint: pass1_info.path_midpoint,
                                    path_bounds: pass1_info.path_bounds,
                                    rank_distance: pass1_info.rank_distance,
                                    to_node_x: pass1_info.to_node_x,
                                    to_node_y: pass1_info.to_node_y,
                                    pass1_group_index,
                                    edge_index,
                                    is_from_endpoint: true,
                                });
                        }
                        if let Some(to_face) = pass1_info.to_face {
                            let node_id_and_face = NodeIdAndFace {
                                node_id: pass1_info.edge.to.clone(),
                                face: to_face,
                            };
                            face_contact_entries_by_node_face
                                .entry(node_id_and_face)
                                .or_default()
                                .push(FaceContactEntry {
                                    path_midpoint: pass1_info.path_midpoint,
                                    path_bounds: pass1_info.path_bounds,
                                    rank_distance: pass1_info.rank_distance,
                                    to_node_x: pass1_info.to_node_x,
                                    to_node_y: pass1_info.to_node_y,
                                    pass1_group_index,
                                    edge_index,
                                    is_from_endpoint: false,
                                });
                        }
                    },
                );
            });

        // Sort each face's entries by rank distance and target coordinate,
        // then assign slot indices.
        face_contact_entries_by_node_face.iter_mut().for_each(
            |(node_id_and_face, face_contact_entries)| {
                Self::face_entries_sort_by_rank_and_coordinate(
                    node_id_and_face.face,
                    face_contact_entries,
                );

                face_contact_entries.iter().enumerate().for_each(
                    |(slot_index, face_contact_entry)| {
                        if face_contact_entry.is_from_endpoint {
                            all_pass1_groups[face_contact_entry.pass1_group_index]
                                .from_slot_indices[face_contact_entry.edge_index] =
                                Some(slot_index);
                        } else {
                            all_pass1_groups[face_contact_entry.pass1_group_index]
                                .to_slot_indices[face_contact_entry.edge_index] = Some(slot_index);
                        }
                    },
                );
            },
        );

        // Reset tracker indices so `offset_calculate` hands out slots in
        // the order we request them.
        face_contact_tracker.indices_reset();

        // Pre-compute per-face ordered offset values so we can index by
        // slot rather than relying on call order.
        let mut face_offsets_by_node_face: NodeIdAndFaceToContactPointOffsets<'id> = Map::new();

        for (node_id_and_face, face_contact_entries) in &face_contact_entries_by_node_face {
            let contact_count = face_contact_entries.len();
            let face_length = Self::face_length_for_node(
                &node_id_and_face.node_id,
                node_id_and_face.face,
                svg_node_info_map,
            );
            // Compute slot-based fallback offsets for all contacts first.
            //
            // No direction-based negation is needed: sibling nodes are
            // inserted in reversed order for reversed rank directions (see
            // `TaffyContainerBuilder::rank_taffy_ids_reverse_if_direction_reversed`),
            // so visual order matches declaration order for all directions.
            let slot_based_offsets: Vec<f32> = (0..contact_count)
                .map(|_| {
                    face_contact_tracker.offset_calculate(
                        &node_id_and_face.node_id,
                        node_id_and_face.face,
                        face_length,
                    )
                })
                .collect();

            // Substitute label-based offsets where the edge has a
            // non-zero description label on this face.
            let mut offsets: Vec<f32> = face_contact_entries
                .iter()
                .zip(slot_based_offsets)
                .map(|(entry, slot_offset)| {
                    let edge_id = &all_pass1_groups[entry.pass1_group_index].pass1_infos
                        [entry.edge_index]
                        .edge_id;
                    Self::label_face_offset_compute(
                        node_id_and_face.face,
                        edge_id,
                        entry.is_from_endpoint,
                        edge_label_taffy_nodes,
                        taffy_tree,
                        svg_node_info_map,
                        &node_id_and_face.node_id,
                    )
                    .unwrap_or(slot_offset)
                })
                .collect();

            // Self-loop from/to contacts may come from different sources
            // (label-aligned from vs slot-based to); enforce the face contact
            // gap between them so the from segment clears the arrow head at
            // the to contact.
            Self::face_offsets_self_loop_separation_enforce(
                face_contact_entries,
                face_length,
                &mut offsets,
            );

            face_offsets_by_node_face.insert(
                node_id_and_face.clone(),
                EdgeContactPointOffsets::new(offsets),
            );
        }

        face_offsets_by_node_face
    }

    /// Enforces a minimum separation between the from and to contact offsets
    /// of self-loop edges on a single (node, face).
    ///
    /// The from contact of a self-loop is label-aligned (the edge label leaf
    /// always has a non-zero padded size, e.g. 4 px from
    /// `EDGE_LABEL_PADDING_PX`), while the to contact has no label leaf
    /// (`to_face` is `None` in the IR assignment) and falls back to the
    /// slot-based offset. The two values come from unrelated coordinate
    /// systems, so without this adjustment they can land arbitrarily close
    /// together -- e.g. 3 px apart -- placing the from segment inside the
    /// arrow head drawn at the to contact.
    ///
    /// When the separation is below the face's contact gap (see
    /// `EdgeFaceContactTracker::gap_calculate`), the to offset is moved to
    /// `from_offset + gap` or `from_offset - gap`, preferring the candidate
    /// that stays within the face and is furthest from the other contacts on
    /// the face.
    fn face_offsets_self_loop_separation_enforce(
        face_contact_entries: &[FaceContactEntry],
        face_length: f32,
        offsets: &mut [f32],
    ) {
        let contact_gap =
            EdgeFaceContactTracker::gap_calculate(face_contact_entries.len(), face_length);
        let half_face_length = face_length / 2.0;

        for from_index in 0..face_contact_entries.len() {
            let from_entry = &face_contact_entries[from_index];
            if !from_entry.is_from_endpoint {
                continue;
            }
            // Both endpoints of an edge appear on the same (node, face) only
            // for self-loops.
            let Some(to_index) = face_contact_entries.iter().position(|entry| {
                !entry.is_from_endpoint
                    && entry.pass1_group_index == from_entry.pass1_group_index
                    && entry.edge_index == from_entry.edge_index
            }) else {
                continue;
            };

            let from_offset = offsets[from_index];
            let to_offset = offsets[to_index];
            if (to_offset - from_offset).abs() >= contact_gap {
                continue;
            }

            // Pick the side of the from contact that stays within the face
            // and has the most clearance from the other contacts.
            let candidate_score = |candidate: f32| -> (bool, f32) {
                let within_face = candidate.abs() <= half_face_length;
                let other_contact_clearance = offsets
                    .iter()
                    .enumerate()
                    .filter(|(offset_index, _)| {
                        *offset_index != from_index && *offset_index != to_index
                    })
                    .map(|(_, other_offset)| (candidate - other_offset).abs())
                    .fold(f32::INFINITY, f32::min);
                (within_face, other_contact_clearance)
            };

            let candidate_after = from_offset + contact_gap;
            let candidate_before = from_offset - contact_gap;
            offsets[to_index] =
                if candidate_score(candidate_after) >= candidate_score(candidate_before) {
                    candidate_after
                } else {
                    candidate_before
                };
        }
    }

    /// Sorts the entries for a single (node, face) by rank distance
    /// and target node coordinate, so that edges connecting to the same
    /// face are ordered to reduce crossovers.
    ///
    /// # Algorithm
    ///
    /// 1. **Primary key** -- `rank_distance` ascending. Edges spanning fewer
    ///    ranks get slots closer to the face midpoint so their shorter paths
    ///    stay on the inside.
    /// 2. **Secondary key** (tie-breaker when rank distances are equal):
    ///    * For Top / Bottom faces: the `to` node's `x` coordinate ascending
    ///      (left-to-right).
    ///    * For Left / Right faces: the `to` node's `y` coordinate ascending
    ///      (top-to-bottom).
    ///
    /// This keeps short-range edges tight against the node and fans
    /// long-range edges outward, while co-ranked edges follow the
    /// spatial order of their targets along the face axis.
    fn face_entries_sort_by_rank_and_coordinate(
        face: NodeFace,
        face_contact_entries: &mut [FaceContactEntry],
    ) {
        if face_contact_entries.len() <= 1 {
            return;
        }

        face_contact_entries.sort_by(|entry_a, entry_b| {
            // Primary: smaller rank distance first.
            let rank_cmp = entry_a.rank_distance.cmp(&entry_b.rank_distance);
            if rank_cmp != std::cmp::Ordering::Equal {
                return rank_cmp;
            }

            // Secondary: coordinate along the face axis.

            match face {
                NodeFace::Top | NodeFace::Bottom => entry_a
                    .to_node_x
                    .partial_cmp(&entry_b.to_node_x)
                    .unwrap_or(std::cmp::Ordering::Equal),
                NodeFace::Left | NodeFace::Right => entry_a
                    .to_node_y
                    .partial_cmp(&entry_b.to_node_y)
                    .unwrap_or(std::cmp::Ordering::Equal),
            }
        });
    }

    /// **Pass 2** for a single edge group: rebuilds every path using the
    /// globally computed face offsets.
    #[allow(clippy::too_many_arguments)]
    fn build_edge_path_infos_with_offsets<'edge, 'id>(
        edge_curvature: EdgeCurvature,
        rank_dir: RankDir,
        pass1_infos: &[EdgePass1Info<'edge, 'id>],
        from_slot_indices: &[Option<usize>],
        to_slot_indices: &[Option<usize>],
        face_offsets_by_node_face: &NodeIdAndFaceToContactPointOffsets<'id>,
        svg_node_info_map: &SvgNodeInfoByNodeId<'_, 'id>,
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        edge_spacer_taffy_nodes: &EdgeIdToEdgeSpacerTaffyNodes<'id>,
        visible_segments_length: f64,
        ortho_protrusions: &[OrthoProtrusionParams],
    ) -> Vec<EdgePathInfo<'edge, 'id>> {
        let mut edge_path_infos = pass1_infos
            .iter()
            .enumerate()
            .map(|(pass1_info_index, pass1_info)| {
                let from_info = svg_node_info_map
                    .get(&pass1_info.edge.from)
                    .expect("from node validated in pass 1");
                let to_info = svg_node_info_map
                    .get(&pass1_info.edge.to)
                    .expect("to node validated in pass 1");

                let from_offset = pass1_info
                    .from_face
                    .and_then(|from_face| {
                        let slot_index = from_slot_indices[pass1_info_index]?;
                        let node_id_and_face = NodeIdAndFace {
                            node_id: pass1_info.edge.from.clone(),
                            face: from_face,
                        };
                        let contact_point_offsets =
                            face_offsets_by_node_face.get(&node_id_and_face)?;
                        contact_point_offsets.get(slot_index)
                    })
                    .unwrap_or(0.0);

                let to_offset = pass1_info
                    .to_face
                    .and_then(|to_face| {
                        let slot_index = to_slot_indices[pass1_info_index]?;
                        let node_id_and_face = NodeIdAndFace {
                            node_id: pass1_info.edge.to.clone(),
                            face: to_face,
                        };
                        let contact_point_offsets =
                            face_offsets_by_node_face.get(&node_id_and_face)?;
                        contact_point_offsets.get(slot_index)
                    })
                    .unwrap_or(0.0);

                let face_offset = EdgeFaceOffset {
                    from_offset,
                    to_offset,
                };

                // Compute spacer coordinates from spacer taffy nodes if
                // this edge has any intermediate-rank spacers.
                let spacer_coordinates = SpacerCoordinatesResolver::resolve(
                    rank_dir,
                    &pass1_info.edge_id,
                    taffy_tree,
                    edge_spacer_taffy_nodes,
                );

                let ortho_protrusion_default = OrthoProtrusionParams::default();
                let ortho_protrusion = ortho_protrusions
                    .get(pass1_info_index)
                    .unwrap_or(&ortho_protrusion_default);

                let path = EdgePathBuilderPass2::build(
                    edge_curvature,
                    rank_dir,
                    from_info,
                    to_info,
                    pass1_info.edge_type,
                    face_offset,
                    &spacer_coordinates,
                    ortho_protrusion,
                    // Pass the faces computed in pass 1 (which uses cycle-aware
                    // face selection) so that pass 2 uses the same faces.
                    pass1_info.from_face.zip(pass1_info.to_face),
                );
                let path_length = {
                    let accuracy = 1.0;
                    path.perimeter(accuracy)
                };

                EdgePathInfo {
                    edge_id: pass1_info.edge_id.clone(),
                    edge: pass1_info.edge,
                    edge_type: pass1_info.edge_type,
                    path,
                    path_length,
                    // Filled in the cumulative scan below, once every edge's
                    // `path_length` (and therefore `travel`) is known.
                    preceding_travel: 0.0,
                    ortho_protrusion_params: ortho_protrusion.clone(),
                }
            })
            .collect::<Vec<EdgePathInfo>>();

        // Fill `preceding_travel` as the running sum of each preceding edge's
        // `travel` distance. An edge's `travel` is the `stroke-dashoffset` span
        // it animates across: `visible_segments_length + trailing_gap`, where
        // `trailing_gap = max(path_length, visible_segments_length)`. Sizing the
        // keyframe windows by `travel` (rather than the constant
        // `visible_segments_length`) keeps every edge moving at the same pixel
        // speed.
        let mut preceding_travel = 0.0;
        edge_path_infos.iter_mut().for_each(|edge_path_info| {
            edge_path_info.preceding_travel = preceding_travel;
            let trailing_gap = edge_path_info.path_length.max(visible_segments_length);
            preceding_travel += visible_segments_length + trailing_gap;
        });

        edge_path_infos
    }

    /// Determines the `EdgeType` for an edge based on the entity types
    /// associated with its edge ID.
    fn edge_type_determine(edge_id: &EdgeId<'_>, entity_types: &EntityTypes<'_>) -> EdgeType {
        entity_types
            .get(&**edge_id)
            .map(|entity_types_for_edge| {
                if [
                    EntityType::DependencyEdgeSequenceForwardDefault,
                    EntityType::DependencyEdgeCyclicForwardDefault,
                    EntityType::InteractionEdgeSequenceForwardDefault,
                    EntityType::InteractionEdgeCyclicForwardDefault,
                ]
                .iter()
                .any(|entity_type_edge_forward| {
                    entity_types_for_edge.contains(entity_type_edge_forward)
                }) {
                    EdgeType::Unpaired
                } else if [
                    EntityType::DependencyEdgeSymmetricForwardDefault,
                    EntityType::InteractionEdgeSymmetricForwardDefault,
                ]
                .iter()
                .any(|entity_type_edge_forward| {
                    entity_types_for_edge.contains(entity_type_edge_forward)
                }) {
                    EdgeType::PairRequest
                } else if [
                    EntityType::DependencyEdgeSymmetricReverseDefault,
                    EntityType::InteractionEdgeSymmetricReverseDefault,
                ]
                .iter()
                .any(|entity_type_edge_reverse| {
                    entity_types_for_edge.contains(entity_type_edge_reverse)
                }) {
                    EdgeType::PairResponse
                } else {
                    EdgeType::Unpaired
                }
            })
            .unwrap_or(EdgeType::Unpaired)
    }

    /// Returns `true` if the two nodes are adjacent siblings with the same
    /// direct parent.
    ///
    /// Two nodes are adjacent siblings when:
    ///
    /// - They are at the same nesting depth (equal `nesting_path` lengths).
    /// - All ancestor chain entries except the last are identical (same
    ///   parent).
    /// - Their sibling indices (last `nesting_path` element) differ by exactly
    ///   1.
    fn nodes_adjacent_siblings_are<'id>(
        node_id_from: &NodeId<'id>,
        node_id_to: &NodeId<'id>,
        node_nesting_infos: &NodeNestingInfos<'id>,
    ) -> bool {
        let Some(info_from) = node_nesting_infos.get(node_id_from) else {
            return false;
        };
        let Some(info_to) = node_nesting_infos.get(node_id_to) else {
            return false;
        };

        let len = info_from.nesting_path.len();
        if len == 0 || len != info_to.nesting_path.len() {
            return false;
        }

        // Same parent: all ancestor chain entries except the last must match.
        let parent_len = len.saturating_sub(1);
        if info_from.ancestor_chain[..parent_len] != info_to.ancestor_chain[..parent_len] {
            return false;
        }

        // Adjacent: sibling indices differ by exactly 1.
        let idx_from = info_from.nesting_path[len - 1];
        let idx_to = info_to.nesting_path[len - 1];
        idx_from.abs_diff(idx_to) == 1
    }

    /// Returns `true` if the divergent ancestors of the two nodes at their LCA
    /// level are adjacent siblings.
    ///
    /// The divergent ancestor of a node is the ancestor that is a direct child
    /// of the LCA of the two nodes. Two divergent ancestors are adjacent
    /// siblings when their sibling indices at the LCA level differ by exactly
    /// 1.
    ///
    /// This detects cross-container edges where the containers are adjacent
    /// (e.g. a node nested inside one container connecting to a node at a
    /// sibling container or at root level). Such edges should use forward face
    /// routing rather than clockwise cycle routing, because the edge path
    /// naturally traverses the gap between the two adjacent containers rather
    /// than routing around the outside of all same-rank nodes.
    ///
    /// Returns `false` when:
    ///
    /// * either node is not found in `node_nesting_infos`,
    /// * one node is an ancestor of the other (contained edge, no divergent
    ///   ancestors),
    /// * sibling index information is missing at the LCA level.
    fn nodes_divergent_ancestors_adjacent_siblings_are<'id>(
        node_id_from: &NodeId<'id>,
        node_id_to: &NodeId<'id>,
        node_nesting_infos: &NodeNestingInfos<'id>,
    ) -> bool {
        let Some(info_from) = node_nesting_infos.get(node_id_from) else {
            return false;
        };
        let Some(info_to) = node_nesting_infos.get(node_id_to) else {
            return false;
        };

        let chain_from = &info_from.ancestor_chain;
        let chain_to = &info_to.ancestor_chain;

        // Find the LCA depth: length of the common ancestor prefix.
        let lca_depth = chain_from
            .iter()
            .zip(chain_to.iter())
            .take_while(|(a, b)| a == b)
            .count();

        // Contained edge: one chain is a prefix of the other -- no divergent ancestors.
        if lca_depth >= chain_from.len() || lca_depth >= chain_to.len() {
            return false;
        }

        // Get sibling indices of the divergent ancestors at the LCA level.
        let Some(&sibling_index_from) = info_from.nesting_path.get(lca_depth) else {
            return false;
        };
        let Some(&sibling_index_to) = info_to.nesting_path.get(lca_depth) else {
            return false;
        };

        // Adjacent: sibling indices differ by exactly 1.
        sibling_index_from.abs_diff(sibling_index_to) == 1
    }

    /// Computes the ranks of the two nodes' divergent ancestors at their
    /// lowest common ancestor (LCA) level.
    ///
    /// This is used to determine whether two nodes are truly at the same
    /// visual rank in the diagram, accounting for hierarchy nesting.
    ///
    /// For nodes in the same container the result is identical to their
    /// local `node_rank_for` values. For cross-container edges the local
    /// ranks can give false positives (both endpoints at rank 0 in their
    /// respective parent contexts even though their containers are at
    /// different root-level ranks).
    ///
    /// Returns `None` when:
    /// * either node is not found in `node_nesting_infos`, or
    /// * one node is an ancestor of the other (contained edge -- handled
    ///   separately by `is_node_contained_in`).
    fn nodes_lca_ranks_compute<'id>(
        node_id_from: &NodeId<'id>,
        node_id_to: &NodeId<'id>,
        node_ranks_nested: &NodeRanksNested<'id>,
        node_nesting_infos: &NodeNestingInfos<'id>,
    ) -> Option<(NodeRank, NodeRank)> {
        let info_from = node_nesting_infos.get(node_id_from)?;
        let info_to = node_nesting_infos.get(node_id_to)?;

        // Find the LCA depth: length of the common prefix of ancestor chains.
        let max_compare = info_from
            .ancestor_chain
            .len()
            .min(info_to.ancestor_chain.len());
        let mut lca_depth = 0;
        for i in 0..max_compare {
            if info_from.ancestor_chain[i] == info_to.ancestor_chain[i] {
                lca_depth = i + 1;
            } else {
                break;
            }
        }

        let divergent_from = info_from.ancestor_chain.get(lca_depth)?;
        let divergent_to = info_to.ancestor_chain.get(lca_depth)?;

        // If both diverge to the same node, one is an ancestor of the other.
        if divergent_from == divergent_to {
            return None;
        }

        // Get the LCA container (None means root level).
        let lca_container = lca_depth
            .checked_sub(1)
            .map(|i| &info_from.ancestor_chain[i]);
        let container_ranks = node_ranks_nested.ranks_for(lca_container)?;

        let rank_from = container_ranks
            .get(divergent_from)
            .copied()
            .unwrap_or_default();
        let rank_to = container_ranks
            .get(divergent_to)
            .copied()
            .unwrap_or_default();

        Some((rank_from, rank_to))
    }

    /// Computes the midpoint of a `BezPath` as the mean of its anchor
    /// points (MoveTo, LineTo, and the final point of CurveTo / QuadTo
    /// elements).
    ///
    /// Returns a `PathMidpoint` in absolute SVG coordinates.
    fn path_midpoint_compute(path: &kurbo::BezPath) -> PathMidpoint {
        let (sum_x, sum_y, point_count) = path
            .elements()
            .iter()
            .filter_map(Self::element_anchor_point)
            .fold((0.0f64, 0.0f64, 0usize), |(sum_x, sum_y, count), point| {
                (sum_x + point.x, sum_y + point.y, count + 1)
            });

        if point_count == 0 {
            PathMidpoint::default()
        } else {
            PathMidpoint {
                x: sum_x / point_count as f64,
                y: sum_y / point_count as f64,
            }
        }
    }

    /// Returns the anchor (end) point of a path element, or `None` for
    /// `ClosePath` which has no anchor point of its own.
    fn element_anchor_point(element: &kurbo::PathEl) -> Option<kurbo::Point> {
        match element {
            kurbo::PathEl::MoveTo(p) | kurbo::PathEl::LineTo(p) => Some(*p),
            kurbo::PathEl::CurveTo(_, _, p) => Some(*p),
            kurbo::PathEl::QuadTo(_, p) => Some(*p),
            kurbo::PathEl::ClosePath => None,
        }
    }

    /// Computes the axis-aligned bounding box of a `BezPath`'s anchor
    /// points (MoveTo, LineTo, and the final point of CurveTo / QuadTo
    /// elements).
    ///
    /// Returns a `PathBounds` in absolute SVG coordinates.
    fn path_bounds_compute(path: &kurbo::BezPath) -> PathBounds {
        path.elements()
            .iter()
            .filter_map(Self::element_anchor_point)
            .fold(None::<PathBounds>, |path_bounds, point| {
                Some(match path_bounds {
                    None => PathBounds {
                        x_min: point.x,
                        x_max: point.x,
                        y_min: point.y,
                        y_max: point.y,
                    },
                    Some(path_bounds) => PathBounds {
                        x_min: path_bounds.x_min.min(point.x),
                        x_max: path_bounds.x_max.max(point.x),
                        y_min: path_bounds.y_min.min(point.y),
                        y_max: path_bounds.y_max.max(point.y),
                    },
                })
            })
            .unwrap_or_default()
    }

    /// Returns the face length (in pixels) for the given node and face.
    ///
    /// For `Top` / `Bottom` this is the node width; for `Left` / `Right`
    /// this is the node collapsed height.
    fn face_length_for_node<'id>(
        node_id: &NodeId<'id>,
        face: NodeFace,
        svg_node_info_map: &SvgNodeInfoByNodeId<'_, 'id>,
    ) -> f32 {
        let Some(node_info) = svg_node_info_map.get(node_id) else {
            return 100.0; // fallback
        };
        match face {
            NodeFace::Top | NodeFace::Bottom => node_info.width,
            NodeFace::Left | NodeFace::Right => node_info.height_collapsed,
        }
    }

    /// Computes the edge contact point offset using the position of the edge's
    /// label taffy node.
    ///
    /// Returns the signed pixel distance from the face midpoint to the label
    /// leaf's entry-side edge along the face axis.  The entry side is the
    /// edge of the label that the path arrives at first, which depends on
    /// `rank_dir` and `face`:
    ///
    /// - `Top`/`Bottom` faces:
    ///   - `TopToBottom`, `LeftToRight`, `RightToLeft`: left x (`label_abs_x`)
    ///   - `BottomToTop`: right x (`label_abs_x + label_width`)
    /// - `Left`/`Right` faces:
    ///   - `LeftToRight`, `TopToBottom`, `BottomToTop`: top y (`label_abs_y`)
    ///   - `RightToLeft`: bottom y (`label_abs_y + label_height`)
    ///
    /// Returns `None` when no label node is recorded for this edge endpoint, or
    /// when the label has zero size along the face axis (indicating no
    /// description text).  The caller should fall back to the slot-based
    /// offset in that case.
    #[allow(clippy::too_many_arguments)]
    fn label_face_offset_compute<'id>(
        face: NodeFace,
        edge_id: &EdgeId<'id>,
        is_from_endpoint: bool,
        edge_label_taffy_nodes: &EdgeIdToEdgeLabelTaffyNodeIds<'id>,
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        svg_node_info_map: &SvgNodeInfoByNodeId<'_, 'id>,
        node_id: &NodeId<'id>,
    ) -> Option<f32> {
        let edge_label_taffy_node_ids = edge_label_taffy_nodes.get(edge_id)?;
        let taffy_node_id = if is_from_endpoint {
            edge_label_taffy_node_ids.from_label_taffy_node_id?
        } else {
            edge_label_taffy_node_ids.to_label_taffy_node_id?
        };
        let layout = taffy_tree.layout(taffy_node_id).ok()?;
        let label_width = layout.size.width;
        let label_height = layout.size.height;
        let node_info = svg_node_info_map.get(node_id)?;
        match face {
            NodeFace::Top | NodeFace::Bottom => {
                if label_width == 0.0 {
                    return None;
                }
                let AbsoluteCoordinates { x: label_abs_x, .. } =
                    TaffyNodeAbsoluteCoordinatesCalculator::calculate(
                        taffy_tree,
                        taffy_node_id,
                        layout,
                    );
                // Route to the entry-side (left x) edge of the label.
                let label_contact_x = label_abs_x;
                let face_midpoint_x = node_info.x + node_info.width / 2.0;
                Some(label_contact_x - face_midpoint_x)
            }
            NodeFace::Left | NodeFace::Right => {
                if label_height == 0.0 {
                    return None;
                }
                let AbsoluteCoordinates { y: label_abs_y, .. } =
                    TaffyNodeAbsoluteCoordinatesCalculator::calculate(
                        taffy_tree,
                        taffy_node_id,
                        layout,
                    );
                // Route to the entry-side (top y) edge of the label.
                let label_contact_y = label_abs_y;
                let face_midpoint_y = node_info.y + node_info.height_collapsed / 2.0;
                Some(label_contact_y - face_midpoint_y)
            }
        }
    }

    /// Returns whether the focus mode bakes a process step that is associated
    /// with this edge as the active focus.
    ///
    /// When true, the edge's animation should run unconditionally in the baked
    /// diagram (rather than being gated on `:focus-within`).
    fn focus_baked_step_associated<'id>(
        focus_mode: TailwindFocusMode<'_, 'id>,
        associated_process_steps: &[&NodeId<'id>],
    ) -> bool {
        matches!(
            focus_mode,
            TailwindFocusMode::Baked {
                active: DiagramFocus::ProcessStep {
                    process_step_id,
                    ..
                },
            } if associated_process_steps
                .iter()
                .any(|node_id| node_id.as_ref() == process_step_id.as_ref())
        )
    }

    fn css_animation_append<'f, 'edge, 'id>(
        css_animation_append_params: CssAnimationAppendParams<'f, 'edge, 'id>,
    ) {
        let CssAnimationAppendParams {
            tailwind_classes,
            css,
            edge_animation_params,
            edge_group_cycle_distance,
            edge_group_animation_duration_total_s,
            edge_path_info,
            edge_animation_active,
            focus_mode,
            associated_process_steps,
        } = css_animation_append_params;
        let edge_animation = EdgeAnimationCalculator::calculate(
            edge_animation_params,
            edge_path_info,
            edge_group_cycle_distance,
            edge_group_animation_duration_total_s,
        );

        // Append dasharray and animate tailwind classes to this
        // edge's existing classes.
        let edge_id_owned: Id<'id> = edge_path_info.edge_id.clone().into_inner();
        let existing = tailwind_classes
            .get(&edge_id_owned)
            .cloned()
            .unwrap_or_default();
        let dasharray = edge_animation.dasharray;
        let animation_name = edge_animation.animation_name;
        let animation_duration =
            EdgeAnimationCalculator::format_duration(edge_animation.edge_animation_duration_s);

        let animation_classes = {
            let mut classes = format!("[stroke-dasharray:{dasharray}]");
            match edge_animation_active {
                EdgeAnimationActive::Always => {
                    classes.push_str(&format!(
                        "\n[&>.edge_body]:animate-[{animation_name}_{animation_duration}s_linear_infinite]"
                    ));
                }
                EdgeAnimationActive::OnProcessStepFocus => match focus_mode {
                    TailwindFocusMode::Interactive => {
                        associated_process_steps.iter().for_each(|process_step_id| {
                            classes.push_str(&format!(
                                "\ngroup-has-[#{process_step_id}:focus-within]:\
                                    [&>.edge_body]:animate-[{animation_name}_{animation_duration}s_linear_infinite]"
                            ));
                        });
                    }
                    TailwindFocusMode::Baked { .. } => {
                        // In baked mode, the focused step's interacting edges
                        // animate unconditionally; all other edges do not
                        // animate.
                        if Self::focus_baked_step_associated(focus_mode, associated_process_steps) {
                            classes.push_str(&format!(
                                "\n[&>.edge_body]:animate-[{animation_name}_{animation_duration}s_linear_infinite]"
                            ));
                        }
                    }
                },
            }
            classes
        };

        let combined = if existing.is_empty() {
            animation_classes
        } else {
            format!("{existing}\n{animation_classes}")
        };
        tailwind_classes.insert(edge_id_owned, combined);

        // Build arrowhead tailwind classes.
        //
        // The edge path already runs from the `from` node to the `to` node, so
        // it can be used directly as the CSS `offset-path` the arrowhead
        // animates along.
        //
        // The arrowhead element needs:
        //   1. `[offset-path:path('...')]` -- the forward edge path
        //   2. `animate-[{arrow_animation_name}_{duration}s_linear_infinite]`
        let arrow_head_offset_path = edge_path_info.path.reverse_subpaths();
        let mut arrow_head_offset_path_svg = arrow_head_offset_path.to_svg();
        // Escape underscores for use inside the tailwind arbitrary value
        // (encre-css transforms these to spaces in the actual CSS value).
        StringCharReplacer::replace_inplace(&mut arrow_head_offset_path_svg, ' ', '_');

        Self::css_animation_append_arrowhead_classes(
            tailwind_classes,
            edge_path_info,
            edge_animation_active,
            focus_mode,
            associated_process_steps,
            &edge_animation.arrow_head_animation_name,
            animation_duration,
            arrow_head_offset_path_svg,
        );

        // Append CSS keyframes for both edge stroke and arrowhead.
        if !css.is_empty() {
            css.push('\n');
        }
        css.push_str(&edge_animation.keyframe_css);
        css.push_str(&edge_animation.arrow_head_keyframe_css);
    }

    /// Appends CSS classes for the arrowhead animation to the diagram's
    /// tailwind classes.
    #[allow(clippy::too_many_arguments)]
    fn css_animation_append_arrowhead_classes<'id>(
        tailwind_classes: &mut EntityTailwindClasses<'id>,
        edge_path_info: &EdgePathInfo<'_, 'id>,
        edge_animation_active: EdgeAnimationActive,
        focus_mode: TailwindFocusMode<'_, 'id>,
        associated_process_steps: &[&NodeId<'id>],
        arrow_head_animation_name: &str,
        animation_duration: String,
        forward_path_svg: String,
    ) {
        let arrow_head_classes = {
            let mut classes = format!(
                "[offset-path:path('{forward_path_svg}')]\n\
                [stroke-dasharray:none]"
            );
            match edge_animation_active {
                EdgeAnimationActive::Always => classes.push_str(&format!(
                    "\nanimate-[{arrow_head_animation_name}_{animation_duration}s_linear_infinite]"
                )),
                EdgeAnimationActive::OnProcessStepFocus => match focus_mode {
                    TailwindFocusMode::Interactive => {
                        associated_process_steps
                            .iter()
                            .for_each(|process_step_id| {
                                classes.push_str(&format!(
                                    "\ngroup-has-[#{process_step_id}:focus-within]:\
                                        animate-[{arrow_head_animation_name}_{animation_duration}s_linear_infinite]"
                                ));
                            });
                    }
                    TailwindFocusMode::Baked { .. } => {
                        // In baked mode, the focused step's interacting edges
                        // animate unconditionally; all other edges do not
                        // animate.
                        if Self::focus_baked_step_associated(focus_mode, associated_process_steps) {
                            classes.push_str(&format!(
                                "\nanimate-[{arrow_head_animation_name}_{animation_duration}s_linear_infinite]"
                            ));
                        }
                    }
                },
            }
            classes
        };

        let edge_id = &edge_path_info.edge_id;
        let arrow_head_entity_id_str = format!("{edge_id}__arrow_head");
        let arrow_head_entity_id: Id<'static> = Id::try_from(arrow_head_entity_id_str)
            .expect("arrow head entity ID should be valid")
            .into_static();
        tailwind_classes.insert(arrow_head_entity_id, arrow_head_classes);
    }
}

// === Supporting types === //

/// A single contact entry used during the per-face sorting phase.
///
/// Tracks which edge touches a given (node, face) together with the
/// rank distance and target node coordinates needed for the
/// rank-and-coordinate sorting, as well as the path midpoint and bounds
/// retained for potential future use.
#[derive(Clone, Copy, Debug, PartialEq)]
struct FaceContactEntry {
    /// Mean anchor point of the edge's zero-offset path.
    ///
    /// Retained for diagnostics and potential future sorting refinements.
    path_midpoint: PathMidpoint,
    /// Axis-aligned bounding box of the edge's zero-offset path.
    ///
    /// Retained for diagnostics and potential future sorting refinements.
    path_bounds: PathBounds,
    /// Absolute difference in `NodeRank` between the edge's `from` and
    /// `to` nodes.
    ///
    /// Used as the primary sort key -- edges spanning fewer ranks are
    /// ordered first so their contact points stay innermost on the
    /// face.
    ///
    /// # Examples
    ///
    /// An edge from rank 0 to rank 1 has `rank_distance = 1`.
    rank_distance: u32,
    /// X coordinate of the edge's `to` node (absolute position).
    ///
    /// Used as a secondary sort key for Top / Bottom faces when two
    /// edges share the same `rank_distance`.
    ///
    /// # Examples
    ///
    /// `120.0` for a node positioned 120 px from the left edge.
    to_node_x: f32,
    /// Y coordinate of the edge's `to` node (absolute position).
    ///
    /// Used as a secondary sort key for Left / Right faces when two
    /// edges share the same `rank_distance`.
    ///
    /// # Examples
    ///
    /// `80.0` for a node positioned 80 px from the top edge.
    to_node_y: f32,
    /// Index into the `all_pass1_groups` vector identifying which edge
    /// group this contact belongs to.
    pass1_group_index: usize,
    /// Index into the group's `pass1_infos` vector identifying which
    /// edge within the group this contact belongs to.
    edge_index: usize,
    /// `true` if this contact is at the "from" endpoint of the edge,
    /// `false` for the "to" endpoint.
    is_from_endpoint: bool,
}

/// Intermediate per-edge data collected in pass 1 and consumed in
/// pass 2.
///
/// Stores everything about an edge that is needed to rebuild its path
/// with face contact offsets, including the rank distance and target
/// node coordinates used for face-contact sorting.
pub(super) struct EdgePass1Info<'edge, 'id> {
    /// Reference to the IR edge (source and target node IDs).
    pub(super) edge: &'edge Edge<'id>,
    /// Generated unique ID for this edge.
    pub(super) edge_id: EdgeId<'id>,
    /// Whether this edge is unpaired or part of a request/response pair.
    pub(super) edge_type: EdgeType,
    /// Which face of the "from" node this edge connects to.
    ///
    /// `None` when the edge connects a contained node (no face offset
    /// applies).
    pub(super) from_face: Option<NodeFace>,
    /// Which face of the "to" node this edge connects to.
    ///
    /// `None` when the edge connects a contained node (no face offset
    /// applies).
    pub(super) to_face: Option<NodeFace>,
    /// Mean anchor point of the zero-offset path.
    ///
    /// Retained for diagnostics and potential future sorting refinements.
    pub(super) path_midpoint: PathMidpoint,
    /// Axis-aligned bounding box of the zero-offset path's anchor
    /// points.
    ///
    /// Retained for diagnostics and potential future sorting refinements.
    pub(super) path_bounds: PathBounds,
    /// Absolute difference in `NodeRank` between the edge's `from` and
    /// `to` nodes.
    ///
    /// # Examples
    ///
    /// An edge from rank 0 to rank 2 has `rank_distance = 2`.
    pub(super) rank_distance: u32,
    /// X coordinate of the edge's `to` node (absolute position).
    ///
    /// # Examples
    ///
    /// `120.0`
    pub(super) to_node_x: f32,
    /// Y coordinate of the edge's `to` node (absolute position).
    ///
    /// # Examples
    ///
    /// `80.0`
    pub(super) to_node_y: f32,
    /// Rank of the edge's `from` node.
    ///
    /// # Examples
    ///
    /// `NodeRank(0)` for the first rank.
    pub(super) rank_from: NodeRank,
    /// Rank of the edge's `to` node.
    ///
    /// # Examples
    ///
    /// `NodeRank(2)` for the third rank.
    pub(super) rank_to: NodeRank,
    /// X coordinate of the edge's `from` node (absolute position).
    ///
    /// Used together with `to_node_x` to determine the cross-axis
    /// direction of the edge for orthogonal protrusion computation.
    ///
    /// # Examples
    ///
    /// `50.0`
    pub(super) from_node_x: f32,
    /// Y coordinate of the edge's `from` node (absolute position).
    ///
    /// # Examples
    ///
    /// `30.0`
    pub(super) from_node_y: f32,
    /// Whether this edge uses cycle (clockwise) face routing.
    ///
    /// `true` when all of the following hold:
    /// - `rank_from == rank_to` (same rank).
    /// - The `from` and `to` nodes are not adjacent siblings with the same
    ///   direct parent.
    /// - Neither endpoint is a tag, process, or process step node.
    ///
    /// When `false` the nearest-face heuristic is used instead.
    pub(super) is_cycle_edge: bool,
}

/// All pass-1 data for a single edge group.
///
/// Contains the per-edge metadata from pass 1 together with the slot
/// index assignments that are filled in by the global
/// `face_offsets_compute` phase.
pub(super) struct EdgeGroupPass1<'edge, 'id> {
    /// The edge group this data belongs to.
    pub(super) edge_group_id: &'edge EdgeGroupId<'id>,
    /// Animation parameters for this edge group.
    pub(super) edge_animation_params: EdgeAnimationParams,
    /// Per-edge pass-1 data.
    pub(super) pass1_infos: Vec<EdgePass1Info<'edge, 'id>>,
    /// Per-edge assigned slot index for the "from" face contact.
    ///
    /// `from_slot_indices[i]` is the slot index assigned to
    /// `pass1_infos[i]`'s "from" endpoint, or `None` if no face offset
    /// applies (e.g. contained edges).
    pub(super) from_slot_indices: Vec<Option<usize>>,
    /// Per-edge assigned slot index for the "to" face contact.
    ///
    /// `to_slot_indices[i]` is the slot index assigned to
    /// `pass1_infos[i]`'s "to" endpoint, or `None` if no face offset
    /// applies (e.g. contained edges).
    pub(super) to_slot_indices: Vec<Option<usize>>,
}

/// Parameters passed to `css_animation_append` to generate edge
/// animation CSS.
struct CssAnimationAppendParams<'f, 'edge, 'id> {
    tailwind_classes: &'f mut EntityTailwindClasses<'id>,
    css: &'f mut Css,
    edge_animation_params: EdgeAnimationParams,
    /// Total `travel` distance of all edges in the group plus the end-of-cycle
    /// pause distance -- the denominator for this edge's keyframe percentages.
    edge_group_cycle_distance: f64,
    edge_group_animation_duration_total_s: f64,
    edge_path_info: &'f EdgePathInfo<'edge, 'id>,
    edge_animation_active: EdgeAnimationActive,
    focus_mode: TailwindFocusMode<'f, 'id>,
    associated_process_steps: &'f [&'f NodeId<'id>],
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds an open path `(0,0) -> (10,0) -> (10,20)` with anchor points
    /// `(0,0)`, `(10,0)`, `(10,20)`.
    fn sample_path() -> kurbo::BezPath {
        let mut path = kurbo::BezPath::new();
        path.move_to((0.0, 0.0));
        path.line_to((10.0, 0.0));
        path.line_to((10.0, 20.0));
        path
    }

    #[test]
    fn path_midpoint_is_anchor_point_average() {
        let path_midpoint = SvgEdgeInfosBuilder::path_midpoint_compute(&sample_path());
        // x = (0 + 10 + 10) / 3, y = (0 + 0 + 20) / 3.
        assert!((path_midpoint.x - 20.0 / 3.0).abs() < 1e-9);
        assert!((path_midpoint.y - 20.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn path_midpoint_empty_path_is_default() {
        let path_midpoint = SvgEdgeInfosBuilder::path_midpoint_compute(&kurbo::BezPath::new());
        assert_eq!(PathMidpoint::default().x, path_midpoint.x);
        assert_eq!(PathMidpoint::default().y, path_midpoint.y);
    }

    #[test]
    fn path_bounds_span_anchor_points() {
        let path_bounds = SvgEdgeInfosBuilder::path_bounds_compute(&sample_path());
        assert_eq!(0.0, path_bounds.x_min);
        assert_eq!(10.0, path_bounds.x_max);
        assert_eq!(0.0, path_bounds.y_min);
        assert_eq!(20.0, path_bounds.y_max);
    }
}
