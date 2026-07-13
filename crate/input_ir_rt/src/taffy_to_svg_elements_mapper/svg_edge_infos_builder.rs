use disposition_input_ir_model::EdgeAnimationActive;
use disposition_input_model::DiagramFocus;
use disposition_ir_model::{
    edge::{Edge, EdgeFaceAssignments, EdgeGroup, EdgeId, EdgeRouteReversals},
    entity::EntityTypes,
    node::{NodeFace, NodeId, NodeNestingInfos, NodeRank, NodeRanksNested},
    IrDiagram,
};
use disposition_model_common::{
    edge::EdgeCurvature, entity::EntityType, theme::Css, Id, Map, RankDir,
};
use disposition_svg_model::{
    EdgePathBounds, EdgePathMidpoint, EdgeRoutingDiagnostic, EdgeRoutingDiagnostics,
    OrthoProtrusionParams, RankGapEntryDiagnostic, SvgEdgeInfo, SvgNodeInfo,
};
use disposition_taffy_model::{
    taffy::TaffyTree, EdgeIdToEdgeDescriptionTaffyNodes, EdgeIdToEdgeLabelTaffyNodeIds,
    EdgeIdToEdgeSpacerTaffyNodes, TaffyNodeCtx,
};
use kurbo::Shape;

use disposition_ir_model::entity::EntityTailwindClasses;
use disposition_model_common::edge::EdgeGroupId;

use crate::{
    input_to_ir_diagram_mapper::tailwind_focus_mode::TailwindFocusMode,
    taffy_to_svg_elements_mapper::{
        edge_face_contact_tracker::{EdgeFaceContactTracker, CONTACT_GAP_MIN_PX},
        edge_halo_outline_calculator::{EdgeHaloOutlineCalculator, EdgeHaloOutlineRails},
        edge_model::{
            EdgeAnimationParams, EdgeContactPointOffsets, EdgeHaloWindow, EdgePathInfo, EdgeType,
            HaloAnimationParams, NodeIdAndFace, NodeIdAndFaceToContactPointOffsets, PathBounds,
            PathMidpoint,
        },
        edge_path_builder_pass_1::{EdgeFaceOffset, SpacerCoordinates},
        ortho_protrusion_calculator::{OrthoProtrusionCalculator, OrthoProtrusionOutcome},
        ArrowHeadBuilder, EdgeAnimationCalculator, EdgePathBuilderPass1, EdgePathBuilderPass2,
        EdgePathLocusCalculator, SpacerCoordinatesResolver, StringCharReplacer,
        SvgNodeInfoByNodeId,
    },
    AbsoluteCoordinates, EdgeFaceAssigner, EdgeHaloIdGenerator, EdgeHaloOutlineIdGenerator,
    EdgeIdGenerator, TaffyNodeAbsoluteCoordinatesCalculator,
};

/// Builds [`SvgEdgeInfo`]s for all edges in the diagram from edge groups and
/// node layout information.
#[derive(Clone, Copy, Debug)]
pub(super) struct SvgEdgeInfosBuilder;

/// Output of [`SvgEdgeInfosBuilder::build`].
///
/// Carries the rendered edge infos plus the edge-routing diagnostics
/// captured during their computation.
pub(super) struct SvgEdgeInfosBuilt<'id> {
    /// The rendered SVG edge infos.
    pub(super) svg_edge_infos: Vec<SvgEdgeInfo<'id>>,
    /// Diagnostic snapshot of the pass-1, offset, and protrusion values.
    pub(super) edge_routing_diagnostics: EdgeRoutingDiagnostics<'id>,
}

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
        edge_description_taffy_nodes: &EdgeIdToEdgeDescriptionTaffyNodes<'id>,
        tailwind_classes: &mut EntityTailwindClasses<'id>,
        css: &mut Css,
        edge_animation_active: EdgeAnimationActive,
        focus_mode: TailwindFocusMode<'_, 'id>,
    ) -> SvgEdgeInfosBuilt<'id> {
        let IrDiagram {
            edge_groups,
            edge_route_reversals,
            entity_types,
            edge_face_assignments,
            process_step_entities,
            render_options,
            interaction_edge_halo,
            ..
        } = ir_diagram;
        let rank_dir = render_options.rank_dir;
        let seconds_per_px = render_options.interaction_edge_animation_millis_per_px / 1000.0;
        let interaction_edge_halo_opacity_base = f64::from(interaction_edge_halo.opacity);
        let interaction_edge_halo_outline_opacity_base =
            f64::from(interaction_edge_halo.outline_opacity);

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
        // 2. The `total_animation_time` should be a constant `seconds_per_px *
        //    total_length`.
        // 3. The `start_pct` ("request start") will be `preceding_edge_lengths_sum /
        //    total_length`.
        // 4. The `end_pct` ("request end") will be `(preceding_edge_lengths_sum +
        //    current_edge_length) / total_length`.
        // 5. The `duration` for each edge's animation will be the `total_animation_time
        //    * (edge_length / total_length)`.

        // === Global Pass 1: collect metadata and register face contacts === //

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
            );
            all_pass1_groups.push(edge_group_pass1);
        }

        // === Global sort and offset computation === //

        let mut face_offsets_by_node_face = Self::face_offsets_compute(
            &mut all_pass1_groups,
            svg_node_info_map,
            &ir_diagram.node_nesting_infos,
            edge_label_taffy_nodes,
            taffy_tree,
            ir_diagram.interaction_edge_halo.stroke_width,
        );

        // Nudge a container's face contact away from edges that transit the
        // same inter-rank gap to reach a node nested inside that container, so
        // their near-parallel legs do not visually touch.
        Self::face_offsets_gap_transit_separate(
            rank_dir,
            &all_pass1_groups,
            svg_node_info_map,
            &ir_diagram.node_nesting_infos,
            taffy_tree,
            edge_spacer_taffy_nodes,
            edge_description_taffy_nodes,
            &mut face_offsets_by_node_face,
            ir_diagram.interaction_edge_halo.stroke_width,
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

        // Whether each group's effective edge curvature is direct (bypasses
        // spacers and protrusions). Direct groups are excluded from protrusion
        // band sizing. Edge groups are exclusively one kind, so the curvature is
        // selected once per group from the group's edges.
        let group_is_direct: Vec<bool> = all_pass1_groups
            .iter()
            .map(|edge_group_pass1| {
                let is_interaction_group = edge_group_pass1
                    .pass1_infos
                    .iter()
                    .any(|pass1_info| pass1_info.is_interaction);
                let edge_curvature = if is_interaction_group {
                    render_options.interaction_edge_curvature
                } else {
                    render_options.dependency_edge_curvature
                };
                edge_curvature.is_direct()
            })
            .collect();

        let OrthoProtrusionOutcome {
            protrusion_params: ortho_protrusions_all,
            rank_gap_entry_diagnostics,
        } = OrthoProtrusionCalculator::calculate(
            rank_dir,
            &all_pass1_groups,
            &from_slot_indices_all,
            &to_slot_indices_all,
            &face_offsets_by_node_face,
            svg_node_info_map,
            taffy_tree,
            edge_spacer_taffy_nodes,
            edge_description_taffy_nodes,
            &ir_diagram.node_nesting_infos,
            &ir_diagram.node_ranks_nested,
            entity_types,
            &group_is_direct,
            ir_diagram.interaction_edge_halo.stroke_width,
        );

        // Assemble the per-edge routing diagnostics while the pass-1 groups,
        // slot indices, offsets, and protrusion results are all still
        // available (the pass-2 loop below consumes `all_pass1_groups`).
        let edge_routing_diagnostics = Self::edge_routing_diagnostics_build(
            &all_pass1_groups,
            &from_slot_indices_all,
            &to_slot_indices_all,
            &face_offsets_by_node_face,
            &ortho_protrusions_all,
            rank_gap_entry_diagnostics,
            edge_route_reversals,
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

            // Dependency and interaction edges can be drawn with independent
            // curvatures. Edge groups are exclusively one kind, so the curvature
            // is selected once per group from the group's edges.
            let is_interaction_group = pass1_infos.iter().any(|pass1_info| {
                entity_types
                    .get(AsRef::<Id<'_>>::as_ref(&pass1_info.edge_id))
                    .map(|edge_entity_types| {
                        edge_entity_types
                            .iter()
                            .any(EntityType::is_interaction_edge)
                    })
                    .unwrap_or(false)
            });
            let edge_curvature = if is_interaction_group {
                render_options.interaction_edge_curvature
            } else {
                render_options.dependency_edge_curvature
            };

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
                edge_description_taffy_nodes,
                visible_segments_length,
                ortho_protrusions,
                ir_diagram.interaction_edge_halo.stroke_width,
                edge_route_reversals,
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
                edge_animation_params.pause_duration_secs / seconds_per_px;
            let edge_group_cycle_distance = edge_group_travel_total + edge_group_pause_distance;
            let edge_group_animation_duration_total_s = seconds_per_px * edge_group_travel_total
                + edge_animation_params.pause_duration_secs;

            // Look up the process steps associated with this edge group (by its
            // inner Id) once per group so every edge in the group can reuse the
            // result.
            let edge_group_id_inner: &Id<'id> = edge_group_id.as_ref();
            let associated_process_steps: &[&NodeId<'id>] = entity_to_process_steps
                .get(edge_group_id_inner)
                .map(Vec::as_slice)
                .unwrap_or(&[]);

            // Precomputed per-edge keyframe windows, so a Forward edge's halo
            // animation can reference the next edge's window, and a Reverse
            // edge's halo animation can reference the previous edge's
            // window, without re-deriving them mid-iteration.
            //
            // Halo pairing uses the edge's raw `EntityType` (whether
            // `InteractionEdgeSymmetricReverseDefault` was assigned to it),
            // not `EdgePathInfo::edge_type` / `EdgeType::PairResponse`: a
            // `sequence`-kind edge that is manually tagged
            // `InteractionEdgeSymmetricReverseDefault` (e.g. to mark a
            // request/response round trip within an otherwise-forward
            // sequence) also keeps its
            // `InteractionEdgeSequenceForwardDefault` default, and
            // `edge_type_determine` -- correctly, for dasharray/path
            // direction purposes -- classifies that combination as
            // `EdgeType::Unpaired` rather than `PairResponse`. This mirrors
            // the same raw-`EntityType` check `tailwind_classes_builder.rs`
            // already uses to pick the halo's static forward/reverse colour.
            let edge_halo_windows: Vec<EdgeHaloWindow> = edge_path_infos
                .iter()
                .map(|edge_path_info| {
                    let (start_pct, end_pct) = EdgeAnimationCalculator::active_window_pct(
                        edge_animation_params,
                        edge_path_info,
                        edge_group_cycle_distance,
                    );
                    let is_reverse = entity_types
                        .get(AsRef::<Id<'_>>::as_ref(&edge_path_info.edge_id))
                        .is_some_and(|edge_entity_types| {
                            edge_entity_types
                                .contains(&EntityType::InteractionEdgeSymmetricReverseDefault)
                        });
                    EdgeHaloWindow {
                        is_reverse,
                        start_pct,
                        end_pct,
                    }
                })
                .collect();

            edge_path_infos
                .into_iter()
                .enumerate()
                .for_each(|(edge_index, edge_path_info)| {
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
                        let halo_animation_params = HaloAnimationParams {
                            is_reverse: edge_halo_windows[edge_index].is_reverse,
                            prev_window: (edge_index > 0)
                                .then(|| edge_halo_windows[edge_index - 1]),
                            next_window: edge_halo_windows.get(edge_index + 1).copied(),
                            opacity_base: interaction_edge_halo_opacity_base,
                            outline_opacity_base: interaction_edge_halo_outline_opacity_base,
                        };
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
                            halo_animation_params,
                            interaction_edge_halo_enabled: render_options
                                .interaction_edge_halo
                                .is_enabled(),
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
                        let locus_path =
                            EdgePathLocusCalculator::calculate(&path, &arrow_head_path);

                        (arrow_head_path, locus_path)
                    };
                    let arrow_head_path_d = arrow_head_path.to_svg();
                    let locus_path_d = locus_path.to_svg();

                    let (halo_outline_rail_a_path_d, halo_outline_rail_b_path_d) =
                        if is_interaction_edge && render_options.interaction_edge_halo.is_enabled()
                        {
                            let EdgeHaloOutlineRails { rail_a, rail_b } =
                                EdgeHaloOutlineCalculator::calculate(
                                    &path,
                                    f64::from(ir_diagram.interaction_edge_halo.stroke_width),
                                );
                            (rail_a.to_svg(), rail_b.to_svg())
                        } else {
                            (String::new(), String::new())
                        };

                    let tooltip = ir_diagram
                        .entity_tooltips
                        .get(edge_id.as_ref())
                        .cloned()
                        .unwrap_or_default();

                    // Route-reversed edges are stored mirrored, so swap the
                    // endpoints back to the user-declared orientation for
                    // consumers (focus / hover / diagnostics).
                    let (node_id_from, node_id_to) = if edge_route_reversals.contains(&edge_id) {
                        (edge.to.clone(), edge.from.clone())
                    } else {
                        (edge.from.clone(), edge.to.clone())
                    };

                    svg_edge_infos.push(SvgEdgeInfo::new(
                        edge_id,
                        edge_group_id.clone(),
                        node_id_from,
                        node_id_to,
                        path_d,
                        arrow_head_path_d,
                        locus_path_d,
                        halo_outline_rail_a_path_d,
                        halo_outline_rail_b_path_d,
                        tooltip,
                        ortho_protrusion_params,
                    ));
                });
        }

        SvgEdgeInfosBuilt {
            svg_edge_infos,
            edge_routing_diagnostics,
        }
    }

    /// Assembles the [`EdgeRoutingDiagnostics`] from the pass-1 groups,
    /// slot indices, resolved face offsets, and computed protrusion
    /// parameters.
    ///
    /// One [`EdgeRoutingDiagnostic`] is produced per edge, in edge-group
    /// then edge order (parallel to `all_pass1_groups`). The rank-gap
    /// entries are taken as-is from the protrusion calculator's snapshot.
    fn edge_routing_diagnostics_build<'id>(
        all_pass1_groups: &[EdgeGroupPass1<'_, 'id>],
        from_slot_indices_all: &[Vec<Option<usize>>],
        to_slot_indices_all: &[Vec<Option<usize>>],
        face_offsets_by_node_face: &NodeIdAndFaceToContactPointOffsets<'id>,
        ortho_protrusions_all: &[Vec<OrthoProtrusionParams>],
        rank_gap_entries: Vec<RankGapEntryDiagnostic<'id>>,
        edge_route_reversals: &EdgeRouteReversals<'id>,
    ) -> EdgeRoutingDiagnostics<'id> {
        let face_offset_resolve =
            |node_id: &NodeId<'id>, face: Option<NodeFace>, slot_index: Option<usize>| -> f32 {
                let (Some(face), Some(slot_index)) = (face, slot_index) else {
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
            };

        let edge_entries = all_pass1_groups
            .iter()
            .enumerate()
            .flat_map(|(group_index, group)| {
                let from_slot_indices = &from_slot_indices_all[group_index];
                let to_slot_indices = &to_slot_indices_all[group_index];
                let ortho_protrusions = &ortho_protrusions_all[group_index];

                group
                    .pass1_infos
                    .iter()
                    .enumerate()
                    .map(move |(edge_index, pass1_info)| {
                        let from_slot_index = from_slot_indices[edge_index];
                        let to_slot_index = to_slot_indices[edge_index];

                        EdgeRoutingDiagnostic {
                            edge_id: pass1_info.edge_id.clone(),
                            edge_group_id: (*group.edge_group_id).clone(),
                            from_node_id: pass1_info.edge.from.clone(),
                            to_node_id: pass1_info.edge.to.clone(),
                            from_face: pass1_info.from_face,
                            to_face: pass1_info.to_face,
                            rank_from: pass1_info.rank_from,
                            rank_to: pass1_info.rank_to,
                            rank_distance: pass1_info.rank_distance,
                            is_cycle_edge: pass1_info.is_cycle_edge,
                            is_interaction: pass1_info.is_interaction,
                            route_reversed: edge_route_reversals.contains(&pass1_info.edge_id),
                            from_node_x: pass1_info.from_node_x,
                            from_node_y: pass1_info.from_node_y,
                            to_node_x: pass1_info.to_node_x,
                            to_node_y: pass1_info.to_node_y,
                            from_slot_index,
                            to_slot_index,
                            from_face_offset: face_offset_resolve(
                                &pass1_info.edge.from,
                                pass1_info.from_face,
                                from_slot_index,
                            ),
                            to_face_offset: face_offset_resolve(
                                &pass1_info.edge.to,
                                pass1_info.to_face,
                                to_slot_index,
                            ),
                            ortho_protrusion_params: ortho_protrusions[edge_index].clone(),
                            path_midpoint: EdgePathMidpoint {
                                x: pass1_info.path_midpoint.x,
                                y: pass1_info.path_midpoint.y,
                            },
                            path_bounds: EdgePathBounds {
                                x_min: pass1_info.path_bounds.x_min,
                                x_max: pass1_info.path_bounds.x_max,
                                y_min: pass1_info.path_bounds.y_min,
                                y_max: pass1_info.path_bounds.y_max,
                            },
                        }
                    })
            })
            .collect();

        EdgeRoutingDiagnostics::new(edge_entries, rank_gap_entries)
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
            let is_interaction = entity_types
                .get(AsRef::<Id<'_>>::as_ref(&edge_id))
                .map(|edge_entity_types| {
                    edge_entity_types
                        .iter()
                        .any(EntityType::is_interaction_edge)
                })
                .unwrap_or(false);

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
                is_interaction,
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
        node_nesting_infos: &NodeNestingInfos<'id>,
        edge_label_taffy_nodes: &EdgeIdToEdgeLabelTaffyNodeIds<'id>,
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        interaction_edge_halo_stroke_width: f32,
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
                                    from_node_x: pass1_info.from_node_x,
                                    from_node_y: pass1_info.from_node_y,
                                    pass1_group_index,
                                    edge_index,
                                    is_from_endpoint: true,
                                    is_interaction: pass1_info.is_interaction,
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
                                    from_node_x: pass1_info.from_node_x,
                                    from_node_y: pass1_info.from_node_y,
                                    pass1_group_index,
                                    edge_index,
                                    is_from_endpoint: false,
                                    is_interaction: pass1_info.is_interaction,
                                });
                        }
                    },
                );
            });

        // Sort each face's entries by rank distance and approach coordinate,
        // then assign slot indices.
        //
        // Dependency and interaction edges are spread in separate slot pools:
        // each face's entries are partitioned by kind, each kind is sorted
        // independently, then the dependency entries are placed before the
        // interaction entries. This way a co-located interaction edge (e.g. an
        // `txn_*` edge running between the same two nodes as a dependency edge)
        // does not push the dependency edge's contact off the face midpoint.
        face_contact_entries_by_node_face.iter_mut().for_each(
            |(node_id_and_face, face_contact_entries)| {
                let (mut dependency_entries, mut interaction_entries): (
                    Vec<FaceContactEntry>,
                    Vec<FaceContactEntry>,
                ) = face_contact_entries
                    .iter()
                    .copied()
                    .partition(|face_contact_entry| !face_contact_entry.is_interaction);
                Self::face_entries_sort_by_rank_and_coordinate(
                    node_id_and_face.face,
                    &mut dependency_entries,
                );
                Self::face_entries_sort_by_rank_and_coordinate(
                    node_id_and_face.face,
                    &mut interaction_entries,
                );
                face_contact_entries.clear();
                face_contact_entries.extend(dependency_entries);
                face_contact_entries.extend(interaction_entries);

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

        // Pre-compute per-face ordered offset values so we can index by
        // slot rather than relying on call order.
        let mut face_offsets_by_node_face: NodeIdAndFaceToContactPointOffsets<'id> = Map::new();

        for (node_id_and_face, face_contact_entries) in &face_contact_entries_by_node_face {
            let face_length = Self::face_length_for_node(
                &node_id_and_face.node_id,
                node_id_and_face.face,
                svg_node_info_map,
            );

            // Resolve every entry's own label span up front, regardless of
            // kind. An entry with real label content uses its own `near`
            // offset directly (below); every resolved span (from either
            // kind) is also kept as `label_spans_occupied` so the slot-based
            // fallback can steer clear of real label geometry it would
            // otherwise be blind to (see
            // `Self::face_offsets_slot_offset_labels_clear`).
            let label_spans: Vec<Option<LabelFaceSpan>> = face_contact_entries
                .iter()
                .map(|entry| {
                    let edge_id = &all_pass1_groups[entry.pass1_group_index].pass1_infos
                        [entry.edge_index]
                        .edge_id;
                    // Dependency edges have no interaction edge halo, so
                    // their label contact needs no halo-clearance pullback
                    // (see `TaffyEnvelopeBuilder::label_margin_build`, which
                    // likewise omits the halo-clearance margin component for
                    // dependency edges).
                    let label_halo_stroke_width = if entry.is_interaction {
                        interaction_edge_halo_stroke_width
                    } else {
                        0.0
                    };
                    Self::label_face_span_compute(
                        node_id_and_face.face,
                        edge_id,
                        entry.is_from_endpoint,
                        edge_label_taffy_nodes,
                        taffy_tree,
                        svg_node_info_map,
                        &node_id_and_face.node_id,
                        label_halo_stroke_width,
                    )
                })
                .collect();
            let label_spans_occupied: Vec<LabelFaceSpan> =
                label_spans.iter().filter_map(|span| *span).collect();

            // Compute slot-based fallback offsets per kind so dependency and
            // interaction contacts each fan symmetrically around the face
            // midpoint. Entries are ordered `[dependencies.., interactions..]`,
            // so an entry's within-kind index is its slot index for
            // dependencies, or `slot_index - dependency_count` for interactions.
            //
            // No direction-based negation is needed: sibling nodes are
            // inserted in reversed order for reversed rank directions (see
            // `TaffyContainerBuilder::rank_taffy_ids_reverse_if_direction_reversed`),
            // so visual order matches declaration order for all directions.
            let dependency_count = face_contact_entries
                .iter()
                .filter(|face_contact_entry| !face_contact_entry.is_interaction)
                .count();
            let interaction_count = face_contact_entries.len() - dependency_count;

            // Substitute label-based offsets where the edge has a
            // non-zero description label on this face; otherwise fall back
            // to the per-kind slot arithmetic, nudged clear of any real
            // label span from either kind on this face. Also record whether
            // each fallback offset was actually moved by that clearing step
            // (`was_label_cleared`), so `face_offsets_label_cleared_collisions_separate`
            // below can re-separate entries that independently converged on
            // the same or a nearby label-span boundary.
            let mut offsets: Vec<f32> = Vec::with_capacity(face_contact_entries.len());
            let mut was_label_cleared: Vec<bool> = Vec::with_capacity(face_contact_entries.len());
            for (slot_index, (face_contact_entry, label_span)) in
                face_contact_entries.iter().zip(label_spans).enumerate()
            {
                if let Some(label_span) = label_span {
                    offsets.push(label_span.near);
                    was_label_cleared.push(false);
                    continue;
                }
                let (within_kind_index, kind_count) = if face_contact_entry.is_interaction {
                    (slot_index - dependency_count, interaction_count)
                } else {
                    (slot_index, dependency_count)
                };
                let slot_offset = EdgeFaceContactTracker::offset_for_index(
                    within_kind_index,
                    kind_count,
                    face_length,
                );
                let cleared_offset =
                    Self::face_offsets_slot_offset_labels_clear(slot_offset, &label_spans_occupied);
                offsets.push(cleared_offset);
                was_label_cleared.push(cleared_offset != slot_offset);
            }

            // Self-loop from/to contacts may come from different sources
            // (label-aligned from vs slot-based to); enforce the face contact
            // gap between them so the from segment clears the arrow head at
            // the to contact.
            Self::face_offsets_self_loop_separation_enforce(
                face_contact_entries,
                face_length,
                &mut offsets,
            );

            // Multiple fallback offsets independently nudged clear of real
            // label spans (possibly the same span, or different but
            // adjacent spans) can still converge on the same or a nearby
            // coordinate; re-separate just those entries.
            Self::face_offsets_label_cleared_collisions_separate(&mut offsets, &was_label_cleared);

            face_offsets_by_node_face.insert(
                node_id_and_face.clone(),
                EdgeContactPointOffsets::new(offsets),
            );
        }

        // Separate contacts on the same face that resolve to the same absolute
        // coordinate across *different* nodes (e.g. an edge from a container
        // and an edge from a node nested and centered within it). The
        // per-(node, face) spreading above cannot see these, so their
        // protrusion stubs would otherwise be drawn on top of each other.
        Self::face_offsets_collisions_separate(
            &face_contact_entries_by_node_face,
            svg_node_info_map,
            node_nesting_infos,
            &mut face_offsets_by_node_face,
        );

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

    /// Enforces a minimum separation between fallback offsets that were
    /// independently nudged clear of *different* real label spans by
    /// `face_offsets_slot_offset_labels_clear`.
    ///
    /// That function considers one entry and one span at a time, so two
    /// unrelated entries on the same face -- possibly from different
    /// edge-kind pools -- can each be nudged to the same or a near-identical
    /// boundary when their respective label spans sit close together (or are
    /// the same span). This pass re-separates only the entries that were
    /// actually nudged (`was_label_cleared[i] == true`); every other offset
    /// -- real label positions, and untouched slot-fallback offsets -- is
    /// treated as a fixed anchor and never moved.
    ///
    /// Untouched slot-fallback offsets must remain fixed anchors here too,
    /// not just real labels: a dependency edge and an interaction edge that
    /// are each the sole occupant of their own kind-pool on one face both
    /// independently compute the fallback centre `0.0` -- an intentional,
    /// documented coincidence (the interaction edge's curved path bows away
    /// from it). A face with no real labels at all has `was_label_cleared`
    /// all-`false`, so this function returns immediately and is a no-op.
    fn face_offsets_label_cleared_collisions_separate(
        offsets: &mut [f32],
        was_label_cleared: &[bool],
    ) {
        let contact_count = offsets.len();
        if contact_count < 2 || !was_label_cleared.iter().any(|&cleared| cleared) {
            return;
        }

        // Sort all slots on this face by current offset, ascending; stable
        // tie-break on slot index for determinism.
        let mut order: Vec<usize> = (0..contact_count).collect();
        order.sort_by(|&a, &b| {
            offsets[a]
                .partial_cmp(&offsets[b])
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.cmp(&b))
        });

        // Walk the sorted order, splitting into runs of consecutive
        // label-cleared slots bounded by fixed slots (a real label, or an
        // untouched fallback offset) or the ends of the face's contact list.
        let mut run_start = 0;
        for index in 0..=order.len() {
            let at_boundary = index == order.len() || !was_label_cleared[order[index]];
            if at_boundary {
                let run = &order[run_start..index];
                if !run.is_empty() {
                    let left_bound = (run_start > 0).then(|| offsets[order[run_start - 1]]);
                    let right_bound = (index < order.len()).then(|| offsets[order[index]]);
                    Self::face_offsets_label_cleared_run_separate(
                        run,
                        offsets,
                        left_bound,
                        right_bound,
                    );
                }
                run_start = index + 1;
            }
        }
    }

    /// Spreads one run of consecutive label-cleared slots so each is at
    /// least `CONTACT_GAP_MIN_PX` from its neighbours and from the bounding
    /// fixed offsets, while disturbing the run's own clearance-derived
    /// positions as little as possible.
    fn face_offsets_label_cleared_run_separate(
        run: &[usize],
        offsets: &mut [f32],
        left_bound: Option<f32>,
        right_bound: Option<f32>,
    ) {
        let run_len = run.len();

        // Already separated from each other and from both bounds?
        let mut prev = left_bound;
        let mut already_ok = true;
        for &index in run {
            if let Some(previous_offset) = prev
                && offsets[index] - previous_offset < CONTACT_GAP_MIN_PX
            {
                already_ok = false;
                break;
            }
            prev = Some(offsets[index]);
        }
        if already_ok && let (Some(right_bound), Some(&last)) = (right_bound, run.last()) {
            already_ok = right_bound - offsets[last] >= CONTACT_GAP_MIN_PX;
        }
        if already_ok {
            return;
        }

        // Redistribute evenly around the run's own centroid (minimising
        // displacement from the clearance-derived positions), clamped to
        // fit between whichever fixed bounds exist.
        let centroid = run.iter().map(|&index| offsets[index]).sum::<f32>() / run_len as f32;
        let span = (run_len as f32 - 1.0) * CONTACT_GAP_MIN_PX;
        let mut start = centroid - span / 2.0;
        if let Some(left_bound) = left_bound {
            start = start.max(left_bound + CONTACT_GAP_MIN_PX);
        }
        if let Some(right_bound) = right_bound {
            start = start.min(right_bound - CONTACT_GAP_MIN_PX - span);
            if let Some(left_bound) = left_bound {
                // Bounds are tighter than the run needs; re-clamp so the run
                // stays as close to the near bound as possible rather than
                // overshooting past the far one.
                start = start.max(left_bound + CONTACT_GAP_MIN_PX);
            }
        }
        for (position, &index) in run.iter().enumerate() {
            offsets[index] = start + position as f32 * CONTACT_GAP_MIN_PX;
        }
    }

    /// Separates edge contacts on the same `NodeFace` that resolve to the
    /// same absolute coordinate but belong to different nodes.
    ///
    /// The per-`(node, face)` spreading in `face_offsets_compute` only sees
    /// contacts that share a single node face, so two edges exiting the same
    /// face of *different* nodes -- e.g. an edge from a container and an edge
    /// from a node nested and centered within it -- can land their contact
    /// points at the identical absolute coordinate, drawing their protrusion
    /// stubs on top of each other.
    ///
    /// This pass flattens every contact, clusters contacts on the same face by
    /// proximity along the face axis, and -- for clusters that span two or more
    /// distinct nodes -- redistributes them symmetrically around the cluster's
    /// shared midpoint. Clusters of a single contact are left untouched, so
    /// layouts without cross-node coincidence are byte-for-byte unchanged.
    fn face_offsets_collisions_separate<'id>(
        face_contact_entries_by_node_face: &Map<NodeIdAndFace<'id>, Vec<FaceContactEntry>>,
        svg_node_info_map: &SvgNodeInfoByNodeId<'_, 'id>,
        node_nesting_infos: &NodeNestingInfos<'id>,
        face_offsets_by_node_face: &mut NodeIdAndFaceToContactPointOffsets<'id>,
    ) {
        // Group records by exact `NodeFace` so opposite-direction stubs (e.g. a
        // `Top` contact and a `Bottom` contact) are never merged.
        let mut records_by_face: Map<NodeFace, Vec<FaceContactCollisionRecord<'id>>> = Map::new();

        for (node_id_and_face, face_contact_entries) in face_contact_entries_by_node_face {
            let Some(midpoint) = Self::face_midpoint(
                &node_id_and_face.node_id,
                node_id_and_face.face,
                svg_node_info_map,
            ) else {
                continue;
            };
            let face_length = Self::face_length_for_node(
                &node_id_and_face.node_id,
                node_id_and_face.face,
                svg_node_info_map,
            );
            let Some(main_axis_coord) = Self::face_main_axis_coord(
                &node_id_and_face.node_id,
                node_id_and_face.face,
                svg_node_info_map,
            ) else {
                continue;
            };
            let Some(offsets) = face_offsets_by_node_face.get(node_id_and_face) else {
                continue;
            };

            face_contact_entries
                .iter()
                .enumerate()
                .for_each(|(slot_index, face_contact_entry)| {
                    let Some(offset) = offsets.get(slot_index) else {
                        return;
                    };
                    records_by_face
                        .entry(node_id_and_face.face)
                        .or_default()
                        .push(FaceContactCollisionRecord {
                            node_id_and_face: node_id_and_face.clone(),
                            slot_index,
                            midpoint,
                            abs_coord: midpoint + offset,
                            main_axis_coord,
                            face_length,
                            rank_distance: face_contact_entry.rank_distance,
                            pass1_group_index: face_contact_entry.pass1_group_index,
                            edge_index: face_contact_entry.edge_index,
                        });
                });
        }

        for (_face, mut records) in records_by_face {
            if records.len() < 2 {
                continue;
            }

            // Sort by absolute coordinate so adjacent records cluster together.
            records.sort_by(|record_a, record_b| {
                record_a
                    .abs_coord
                    .partial_cmp(&record_b.abs_coord)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            // Walk records, splitting into clusters wherever the gap to the
            // previous record exceeds the contact gap. Same-(node, face)
            // contacts spread by the slot logic are at least the contact gap
            // apart, so they split into single-record clusters (skipped) unless
            // a contact from another node falls within the gap.
            let mut cluster_start = 0usize;
            for index in 1..=records.len() {
                let split = index == records.len()
                    || (records[index].abs_coord - records[index - 1].abs_coord)
                        > CONTACT_GAP_MIN_PX;
                if split {
                    Self::face_offsets_collision_cluster_separate(
                        &records[cluster_start..index],
                        node_nesting_infos,
                        face_offsets_by_node_face,
                    );
                    cluster_start = index;
                }
            }
        }
    }

    /// Splits one abs-coordinate cluster into the sub-groups that genuinely
    /// collide, and redistributes each.
    ///
    /// Sharing a face-axis coordinate is necessary but not sufficient for two
    /// contacts to overlap: their stubs must also protrude into the same
    /// inter-rank gap. That holds when the nodes are in the same rank row
    /// (equal `main_axis_coord`) or one is nested inside the other (the
    /// stubs are collinear through the container boundary).
    /// Vertically-stacked siblings at different ranks share a face-axis
    /// coordinate but protrude into different gaps, so they are kept apart
    /// here. The cluster is partitioned into connected components under
    /// this relation, and each component is redistributed independently.
    fn face_offsets_collision_cluster_separate<'id>(
        cluster: &[FaceContactCollisionRecord<'id>],
        node_nesting_infos: &NodeNestingInfos<'id>,
        face_offsets_by_node_face: &mut NodeIdAndFaceToContactPointOffsets<'id>,
    ) {
        if cluster.len() < 2 {
            return;
        }

        let component_of = Self::collision_components_assign(cluster, node_nesting_infos);
        let component_count = component_of.iter().copied().max().map_or(0, |max| max + 1);
        for component_id in 0..component_count {
            let component: Vec<&FaceContactCollisionRecord<'id>> = cluster
                .iter()
                .zip(component_of.iter())
                .filter_map(|(record, &record_component)| {
                    (record_component == component_id).then_some(record)
                })
                .collect();
            Self::face_offsets_collision_component_separate(&component, face_offsets_by_node_face);
        }
    }

    /// Assigns each record in `cluster` a connected-component id under the
    /// collision-compatibility relation (see
    /// [`face_offsets_collision_cluster_separate`](Self::face_offsets_collision_cluster_separate)).
    fn collision_components_assign<'id>(
        cluster: &[FaceContactCollisionRecord<'id>],
        node_nesting_infos: &NodeNestingInfos<'id>,
    ) -> Vec<usize> {
        let n = cluster.len();
        let mut parent: Vec<usize> = (0..n).collect();

        for i in 0..n {
            for j in (i + 1)..n {
                if Self::collision_records_compatible(&cluster[i], &cluster[j], node_nesting_infos)
                {
                    let root_i = Self::union_find_root(&parent, i);
                    let root_j = Self::union_find_root(&parent, j);
                    if root_i != root_j {
                        parent[root_i] = root_j;
                    }
                }
            }
        }

        // Relabel each record by its component root into contiguous ids.
        let mut root_to_label: Vec<Option<usize>> = vec![None; n];
        let mut next_label = 0usize;
        (0..n)
            .map(|i| {
                let root = Self::union_find_root(&parent, i);
                match root_to_label[root] {
                    Some(label) => label,
                    None => {
                        let label = next_label;
                        next_label += 1;
                        root_to_label[root] = Some(label);
                        label
                    }
                }
            })
            .collect()
    }

    /// Follows the union-find parent chain to the root of `i`.
    fn union_find_root(parent: &[usize], mut i: usize) -> usize {
        while parent[i] != i {
            i = parent[i];
        }
        i
    }

    /// Returns whether two coincident face contacts actually share an
    /// inter-rank gap (and so their protrusion stubs would overlap).
    fn collision_records_compatible<'id>(
        record_a: &FaceContactCollisionRecord<'id>,
        record_b: &FaceContactCollisionRecord<'id>,
        node_nesting_infos: &NodeNestingInfos<'id>,
    ) -> bool {
        /// Tolerance for treating two faces as being in the same rank row.
        /// Taffy aligns same-rank siblings to an identical main-axis
        /// coordinate, so a sub-pixel epsilon is sufficient.
        const MAIN_AXIS_EPS: f32 = 1.0;

        let node_a = &record_a.node_id_and_face.node_id;
        let node_b = &record_b.node_id_and_face.node_id;
        node_a == node_b
            || (record_a.main_axis_coord - record_b.main_axis_coord).abs() < MAIN_AXIS_EPS
            || Self::node_is_descendant_of(node_a, node_b, node_nesting_infos)
            || Self::node_is_descendant_of(node_b, node_a, node_nesting_infos)
    }

    /// Redistributes one collision component symmetrically around its shared
    /// midpoint.
    ///
    /// A component is only adjusted when it spans two or more distinct
    /// `(node, face)` groups; a component wholly within one group is already
    /// spread by the slot logic in `face_offsets_compute`, and a single-record
    /// component needs no separation.
    fn face_offsets_collision_component_separate<'id>(
        component: &[&FaceContactCollisionRecord<'id>],
        face_offsets_by_node_face: &mut NodeIdAndFaceToContactPointOffsets<'id>,
    ) {
        let distinct_node_faces = component
            .iter()
            .map(|record| &record.node_id_and_face)
            .collect::<std::collections::HashSet<_>>()
            .len();
        if component.len() < 2 || distinct_node_faces < 2 {
            return;
        }

        let contact_count = component.len();
        let center =
            component.iter().map(|record| record.abs_coord).sum::<f32>() / contact_count as f32;
        // Size the fan from the narrowest node's face so it fits within all
        // nodes in the component.
        let min_face_length = component
            .iter()
            .map(|record| record.face_length)
            .fold(f32::INFINITY, f32::min);
        let gap = EdgeFaceContactTracker::gap_calculate(contact_count, min_face_length);

        // Order deterministically: closer-ranked edges innermost, then by stable
        // edge identity.
        let mut ordered_records: Vec<&FaceContactCollisionRecord<'id>> = component.to_vec();
        ordered_records.sort_by(|record_a, record_b| {
            record_a
                .rank_distance
                .cmp(&record_b.rank_distance)
                .then(record_a.pass1_group_index.cmp(&record_b.pass1_group_index))
                .then(record_a.edge_index.cmp(&record_b.edge_index))
        });

        let center_index = (contact_count as f32 - 1.0) / 2.0;
        ordered_records
            .iter()
            .enumerate()
            .for_each(|(slot, record)| {
                let abs_pos = center + (slot as f32 - center_index) * gap;
                let offset = abs_pos - record.midpoint;
                if let Some(offsets) = face_offsets_by_node_face.get_mut(&record.node_id_and_face) {
                    offsets.offset_set(record.slot_index, offset);
                }
            });
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

            // Secondary: the cross-axis coordinate of the *other* endpoint,
            // i.e. the side the edge approaches this face from. For a
            // from-endpoint contact the other endpoint is the `to` node; for a
            // to-endpoint contact it is the `from` node. Using the other
            // endpoint matters when several edges enter the *same* target face:
            // their `to` coordinate is identical, so ordering by it collapses to
            // input order and produces crossings. Ordering by the `from`
            // coordinate instead fans the contacts in the spatial order of
            // their sources.
            Self::face_entry_approach_coord(face, entry_a)
                .partial_cmp(&Self::face_entry_approach_coord(face, entry_b))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// Returns the cross-axis coordinate of the endpoint *opposite* to this
    /// contact -- the side the edge approaches the face from.
    ///
    /// * For a from-endpoint contact: the `to` node's coordinate (x for
    ///   Top/Bottom, y for Left/Right).
    /// * For a to-endpoint contact: the `from` node's coordinate.
    fn face_entry_approach_coord(face: NodeFace, entry: &FaceContactEntry) -> f32 {
        match (face, entry.is_from_endpoint) {
            (NodeFace::Top | NodeFace::Bottom, true) => entry.to_node_x,
            (NodeFace::Top | NodeFace::Bottom, false) => entry.from_node_x,
            (NodeFace::Left | NodeFace::Right, true) => entry.to_node_y,
            (NodeFace::Left | NodeFace::Right, false) => entry.from_node_y,
        }
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
        edge_description_taffy_nodes: &EdgeIdToEdgeDescriptionTaffyNodes<'id>,
        visible_segments_length: f64,
        ortho_protrusions: &[OrthoProtrusionParams],
        interaction_edge_halo_stroke_width: f32,
        edge_route_reversals: &EdgeRouteReversals<'id>,
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

                // Dependency edges have no interaction edge halo, so their
                // description contact needs no halo-clearance pullback (see
                // `EdgeDescriptionBuilder::edge_desc_build`, which likewise
                // omits the halo-clearance margin for dependency edges).
                let description_halo_stroke_width = if pass1_info.is_interaction {
                    interaction_edge_halo_stroke_width
                } else {
                    0.0
                };

                // Compute spacer coordinates from spacer taffy nodes if
                // this edge has any intermediate-rank spacers. The edge's own
                // description contact (if any) is already folded into this
                // list, so `Curved`/`Orthogonal` routing picks it up
                // unconditionally.
                let spacer_coordinates = SpacerCoordinatesResolver::resolve(
                    rank_dir,
                    &pass1_info.edge_id,
                    taffy_tree,
                    edge_spacer_taffy_nodes,
                    edge_description_taffy_nodes,
                    description_halo_stroke_width,
                );

                // Direct-curvature edges ignore `spacer_coordinates` entirely
                // (see `EdgePathBuilderPass2::build`'s `Direct*` arms), so the
                // description contact must also be passed separately -- this
                // is the one waypoint applied regardless of curvature.
                let description_contact = SpacerCoordinatesResolver::description_contact_resolve(
                    rank_dir,
                    &pass1_info.edge_id,
                    taffy_tree,
                    edge_description_taffy_nodes,
                    description_halo_stroke_width,
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
                    description_contact,
                );

                // Route-reversed edges are stored mirrored (`from`/`to`
                // swapped by `EdgeRouteNormalizer`), so the path above runs
                // from the real `to` node to the real `from` node. Reverse it
                // here -- before path length, arrow head, locus, halo, and
                // animation are computed -- so every downstream consumer sees
                // a path that runs real-from -> real-to.
                let path = if edge_route_reversals.contains(&pass1_info.edge_id) {
                    path.reverse_subpaths()
                } else {
                    path
                };
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
    /// Returns the midpoint coordinate of a node face along the face axis.
    ///
    /// For `Top`/`Bottom` faces this is the horizontal midpoint
    /// (`x + width / 2`); for `Left`/`Right` faces the vertical midpoint
    /// (`y + height_collapsed / 2`). Returns `None` when the node has no layout
    /// info.
    fn face_midpoint<'id>(
        node_id: &NodeId<'id>,
        face: NodeFace,
        svg_node_info_map: &SvgNodeInfoByNodeId<'_, 'id>,
    ) -> Option<f32> {
        let node_info = svg_node_info_map.get(node_id)?;
        Some(match face {
            NodeFace::Top | NodeFace::Bottom => node_info.x + node_info.width / 2.0,
            NodeFace::Left | NodeFace::Right => node_info.y + node_info.height_collapsed / 2.0,
        })
    }

    /// Returns the coordinate of `face` along the **rank** axis -- the axis
    /// perpendicular to the face axis (y for Top/Bottom, x for Left/Right).
    ///
    /// This is the outward edge of the face: the node's top edge for `Top`, its
    /// bottom edge for `Bottom`, and so on. Used by
    /// `face_offsets_collisions_separate` to tell whether two contacts that
    /// share a face-axis coordinate actually protrude into the same inter-rank
    /// gap (same rank row) or merely line up across different ranks.
    fn face_main_axis_coord<'id>(
        node_id: &NodeId<'id>,
        face: NodeFace,
        svg_node_info_map: &SvgNodeInfoByNodeId<'_, 'id>,
    ) -> Option<f32> {
        let node_info = svg_node_info_map.get(node_id)?;
        Some(match face {
            NodeFace::Top => node_info.y,
            NodeFace::Bottom => node_info.y + node_info.height_collapsed,
            NodeFace::Left => node_info.x,
            NodeFace::Right => node_info.x + node_info.width,
        })
    }

    /// Nudges a container node's face contact away from edges that **transit**
    /// the same inter-rank gap on their way to a node nested inside that
    /// container.
    ///
    /// When an edge enters a container `C` at face `F`, and another edge routes
    /// through the gap just before `C`'s `F` face to reach a node nested inside
    /// `C`, the two run near-parallel one cross-axis step apart. The
    /// per-`(node, face)` offset logic cannot see this, because the transiting
    /// edge does not contact `C`. This pass detects the transit (via the
    /// transiting edge's spacer in that gap) and shifts `C`'s contact along the
    /// face axis so the two legs are at least `CONTACT_GAP_MIN_PX` apart.
    ///
    /// Only container to-faces with a transiting descendant edge within the
    /// contact gap are adjusted, so layouts without this pattern are unchanged.
    #[allow(clippy::too_many_arguments)]
    fn face_offsets_gap_transit_separate<'edge, 'id>(
        rank_dir: RankDir,
        all_pass1_groups: &[EdgeGroupPass1<'edge, 'id>],
        svg_node_info_map: &SvgNodeInfoByNodeId<'_, 'id>,
        node_nesting_infos: &NodeNestingInfos<'id>,
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        edge_spacer_taffy_nodes: &EdgeIdToEdgeSpacerTaffyNodes<'id>,
        edge_description_taffy_nodes: &EdgeIdToEdgeDescriptionTaffyNodes<'id>,
        face_offsets_by_node_face: &mut NodeIdAndFaceToContactPointOffsets<'id>,
        interaction_edge_halo_stroke_width: f32,
    ) {
        for group in all_pass1_groups {
            for (edge_index, pass1_info) in group.pass1_infos.iter().enumerate() {
                let Some(face) = pass1_info.to_face else {
                    continue;
                };
                let container = &pass1_info.edge.to;

                let Some(midpoint) = Self::face_midpoint(container, face, svg_node_info_map) else {
                    continue;
                };
                let Some(slot) = group.to_slot_indices[edge_index] else {
                    continue;
                };
                let node_id_and_face = NodeIdAndFace {
                    node_id: container.clone(),
                    face,
                };
                let Some(current_offset) = face_offsets_by_node_face
                    .get(&node_id_and_face)
                    .and_then(|offsets| offsets.get(slot))
                else {
                    continue;
                };
                let contact = midpoint + current_offset;

                let Some(&container_info) = svg_node_info_map.get(container) else {
                    continue;
                };
                let near_face_main = Self::face_coord_main_axis(container_info, face);

                // Cross-axis positions of edges transiting the gap just before
                // this container's face to reach a node nested inside it.
                let transit_positions: Vec<f32> = all_pass1_groups
                    .iter()
                    .flat_map(|other_group| other_group.pass1_infos.iter())
                    .filter(|other| {
                        Self::node_is_descendant_of(&other.edge.to, container, node_nesting_infos)
                    })
                    .filter_map(|other| {
                        // Dependency edges have no interaction edge halo, so
                        // their description contact needs no halo-clearance
                        // pullback (see
                        // `EdgeDescriptionBuilder::edge_desc_build`).
                        let description_halo_stroke_width = if other.is_interaction {
                            interaction_edge_halo_stroke_width
                        } else {
                            0.0
                        };
                        let spacer_coordinates = SpacerCoordinatesResolver::resolve(
                            rank_dir,
                            &other.edge_id,
                            taffy_tree,
                            edge_spacer_taffy_nodes,
                            edge_description_taffy_nodes,
                            description_halo_stroke_width,
                        );
                        Self::transit_cross_axis_before_face(
                            face,
                            near_face_main,
                            &spacer_coordinates,
                        )
                    })
                    .collect();

                if transit_positions.is_empty() {
                    continue;
                }

                // The approach origin: the cross-axis coordinate the edge's
                // routing sweeps from toward the container face. Using the
                // from-node face midpoint keeps the contact on the from-node's
                // side of any transit, so the approach sweep does not cross it.
                let Some(from_face) = pass1_info.from_face else {
                    continue;
                };
                let Some(from_cross) =
                    Self::face_midpoint(&pass1_info.edge.from, from_face, svg_node_info_map)
                else {
                    continue;
                };

                let face_length = Self::face_length_for_node(container, face, svg_node_info_map);
                if let Some(new_offset) = Self::contact_offset_cleared_from_transits(
                    contact,
                    from_cross,
                    midpoint,
                    face_length,
                    &transit_positions,
                ) && let Some(offsets) = face_offsets_by_node_face.get_mut(&node_id_and_face)
                {
                    offsets.offset_set(slot, new_offset);
                }
            }
        }
    }

    /// Returns the main-axis (rank-direction) coordinate of a node's near face.
    ///
    /// `Left` -> left x, `Right` -> right x, `Top` -> top y, `Bottom` ->
    /// bottom y.
    fn face_coord_main_axis(node_info: &SvgNodeInfo<'_>, face: NodeFace) -> f32 {
        match face {
            NodeFace::Left => node_info.x,
            NodeFace::Right => node_info.x + node_info.width,
            NodeFace::Top => node_info.y,
            NodeFace::Bottom => node_info.y + node_info.height_collapsed,
        }
    }

    /// Returns whether `node_id` is a strict descendant of `ancestor_id`.
    ///
    /// The node's `ancestor_chain` ends with the node itself, so the ancestor
    /// is searched in all but the last element.
    fn node_is_descendant_of<'id>(
        node_id: &NodeId<'id>,
        ancestor_id: &NodeId<'id>,
        node_nesting_infos: &NodeNestingInfos<'id>,
    ) -> bool {
        node_nesting_infos
            .get(node_id)
            .map(|node_nesting_info| {
                let ancestor_chain = &node_nesting_info.ancestor_chain;
                let prefix_len = ancestor_chain.len().saturating_sub(1);
                ancestor_chain[..prefix_len]
                    .iter()
                    .any(|chain_node_id| chain_node_id == ancestor_id)
            })
            .unwrap_or(false)
    }

    /// Returns the cross-axis coordinate of the spacer routing an edge through
    /// the gap **just before** a container's `face` on the approach side, or
    /// `None` when the edge has no such spacer.
    ///
    /// The approach side is the outward direction of `face`; the relevant
    /// spacer is the one nearest the face among those whose main-axis position
    /// lies before it.
    fn transit_cross_axis_before_face(
        face: NodeFace,
        near_face_main: f32,
        spacer_coordinates: &[SpacerCoordinates],
    ) -> Option<f32> {
        // `Left`/`Top` faces are entered from the smaller-coordinate side, so a
        // transiting spacer sits at a smaller main-axis coordinate (closest =
        // largest). `Right`/`Bottom` are the mirror.
        let approach_from_small = matches!(face, NodeFace::Left | NodeFace::Top);
        let spacer_main = |spacer: &SpacerCoordinates| match face {
            NodeFace::Left | NodeFace::Right => spacer.exit_x,
            NodeFace::Top | NodeFace::Bottom => spacer.exit_y,
        };
        let spacer_cross = |spacer: &SpacerCoordinates| match face {
            NodeFace::Left | NodeFace::Right => spacer.entry_y,
            NodeFace::Top | NodeFace::Bottom => spacer.entry_x,
        };

        spacer_coordinates
            .iter()
            .filter(|spacer| {
                if approach_from_small {
                    spacer_main(spacer) < near_face_main
                } else {
                    spacer_main(spacer) > near_face_main
                }
            })
            .reduce(|nearest, spacer| {
                let closer = if approach_from_small {
                    spacer_main(spacer) > spacer_main(nearest)
                } else {
                    spacer_main(spacer) < spacer_main(nearest)
                };
                if closer {
                    spacer
                } else {
                    nearest
                }
            })
            .map(spacer_cross)
    }

    /// Computes a new face offset that keeps `contact` on the **same side of
    /// each transit as `from_cross`** (the approach origin), clearing every
    /// transit by `CONTACT_GAP_MIN_PX` while staying within the node face.
    ///
    /// This prevents the edge's approach sweep (from `from_cross` toward the
    /// contact) from crossing a transit leg, and also separates a contact that
    /// merely sits too close to one. Returns `None` when no move is needed, the
    /// constraints conflict, or the move would leave the face.
    fn contact_offset_cleared_from_transits(
        contact: f32,
        from_cross: f32,
        midpoint: f32,
        face_length: f32,
        transits: &[f32],
    ) -> Option<f32> {
        let gap = CONTACT_GAP_MIN_PX;

        // Keep the contact within the face, leaving an arrow-head margin.
        let half = (face_length / 2.0 - 4.0).max(0.0);
        let mut lower = midpoint - half;
        let mut upper = midpoint + half;

        for &transit in transits {
            // Constrain the contact to the from-node's side of this transit, so
            // the approach neither crosses nor touches it. If the from-node sits
            // on the transit, it cannot be cleared -- skip.
            if from_cross < transit - 1e-3 {
                upper = upper.min(transit - gap);
            } else if from_cross > transit + 1e-3 {
                lower = lower.max(transit + gap);
            }
        }

        if lower > upper {
            // Transits on both sides leave no clear band; leave the contact.
            return None;
        }

        let new_contact = contact.clamp(lower, upper);
        if (new_contact - contact).abs() < 1e-3 {
            return None;
        }
        Some(new_contact - midpoint)
    }

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

    /// Computes the signed offset span (from the face midpoint) occupied by
    /// an edge's own label taffy node on `face`, or `None` when the label has
    /// no real content (i.e. no description text on this face).
    ///
    /// `near` is the entry-side edge of the label -- the edge of the label
    /// the path arrives at first, always the left x (`Top`/`Bottom` faces)
    /// or top y (`Left`/`Right` faces): sibling insertion order is reversed
    /// for reversed rank directions (see `TaffyContainerBuilder::
    /// rank_taffy_ids_reverse_if_direction_reversed`), so the entry side is
    /// the same for all `RankDir` values. `far` is the opposite edge of the
    /// label. Both are already adjusted by half
    /// `interaction_edge_halo_stroke_width` ("`halo_pad_px`") -- `near`
    /// pulled back, `far` pushed out -- so `[near, far]` covers the label's
    /// full rendered footprint plus halo clearance, and a contact placed
    /// exactly at `near` stops short of the label rather than terminating
    /// flush against it.
    ///
    /// Without the `halo_pad_px` pullback, the contact point would be the
    /// label's own (post-layout) coordinate, so any clearance added via the
    /// label's own margin (see `TaffyEnvelopeBuilder::label_margin_build`)
    /// would shift both the label and the path by the same amount and never
    /// open up a visible gap -- the halo, being centered on the path, would
    /// still overlap the label by half its stroke width.
    ///
    /// `label_margin_build` gives the label slot `margin` of `halo_pad_px +
    /// label_margin_px` on *both* sides of the packing axis (not just the far
    /// side), but this pullback only ever cancels the `halo_pad_px`
    /// component -- so after cancellation, the routed path still ends up
    /// `label_margin_px` further from the face midpoint than the label's
    /// pre-margin position, leaving the label visibly associated with its
    /// edge rather than flush against the contact point. This is intentional
    /// and mirrors `EdgeDescriptionBuilder::edge_desc_build`'s equivalent
    /// margin/pullback split for edge descriptions.
    ///
    /// The caller (`Self::face_offsets_compute`) passes `0.0` for
    /// `interaction_edge_halo_stroke_width` when the edge is a dependency
    /// edge (`!FaceContactEntry::is_interaction`), since dependency edges
    /// render no interaction edge halo and so have nothing to pull back from
    /// -- only the `label_margin_px` component of the label's own margin
    /// applies in that case, matching `label_margin_build`'s halo-clearance
    /// exception for dependency edges.
    #[allow(clippy::too_many_arguments)]
    fn label_face_span_compute<'id>(
        face: NodeFace,
        edge_id: &EdgeId<'id>,
        is_from_endpoint: bool,
        edge_label_taffy_nodes: &EdgeIdToEdgeLabelTaffyNodeIds<'id>,
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        svg_node_info_map: &SvgNodeInfoByNodeId<'_, 'id>,
        node_id: &NodeId<'id>,
        interaction_edge_halo_stroke_width: f32,
    ) -> Option<LabelFaceSpan> {
        let halo_pad_px = interaction_edge_halo_stroke_width / 2.0;
        let edge_label_taffy_node_ids = edge_label_taffy_nodes.get(edge_id)?;
        // Only treat the label as real when it actually has content. Every
        // edge -- even one without a description -- gets a padded label leaf
        // (non-zero width), so a width check alone would always treat the
        // leaf as a real label and pin the contact to the leaf's pre-layout
        // position (ordered structurally by `NodeFaceEdges`, not by where the
        // edge geometrically approaches the face). Descriptionless edges
        // therefore fall back to the coordinate-aware slot logic in
        // `face_offsets_compute`, which knows the real layout positions and
        // spreads dependency and interaction edges in separate pools --
        // nudged clear of any real label span returned here (see
        // `Self::face_offsets_slot_offset_labels_clear`). `*_md_node_taffy_ids`
        // is `Some` only when the corresponding label text is non-empty.
        let (taffy_node_id, label_md_node_taffy_ids) = if is_from_endpoint {
            (
                edge_label_taffy_node_ids.from_label_taffy_node_id?,
                &edge_label_taffy_node_ids.from_label_md_node_taffy_ids,
            )
        } else {
            (
                edge_label_taffy_node_ids.to_label_taffy_node_id?,
                &edge_label_taffy_node_ids.to_label_md_node_taffy_ids,
            )
        };
        label_md_node_taffy_ids.as_ref()?;
        let layout = taffy_tree.layout(taffy_node_id).ok()?;
        let label_width = layout.size.width;
        let label_height = layout.size.height;
        let face_midpoint = Self::face_midpoint(node_id, face, svg_node_info_map)?;
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
                Some(LabelFaceSpan {
                    near: (label_abs_x - halo_pad_px) - face_midpoint,
                    far: (label_abs_x + label_width + halo_pad_px) - face_midpoint,
                })
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
                Some(LabelFaceSpan {
                    near: (label_abs_y - halo_pad_px) - face_midpoint,
                    far: (label_abs_y + label_height + halo_pad_px) - face_midpoint,
                })
            }
        }
    }

    /// Nudges a slot-based fallback offset clear of any real label span on
    /// the same face.
    ///
    /// The per-kind slot arithmetic
    /// (`EdgeFaceContactTracker::offset_for_index`) has no awareness of
    /// where any edge's *real* label rendered -- including a co-located
    /// edge from the *other* kind pool (see the "Edge-kind pools" doc
    /// comment on `Self::face_offsets_compute`) -- so a fallback
    /// contact can land inside a real label's box. When `slot_offset` falls
    /// inside one of `label_spans_occupied` (with `CONTACT_GAP_MIN_PX`
    /// clearance), this shifts it to whichever edge of that span is nearer,
    /// so the contact clears the label while staying as close as possible to
    /// its originally-computed (symmetrically fanned) position.
    ///
    /// Returns `slot_offset` unchanged when it is already clear of every
    /// span -- in particular, a face with no real labels at all (e.g.
    /// `0044_edge_offsets_and_protrusion_complex_2`, where every edge uses
    /// the fallback) never enters the nudging loop, so it is byte-for-byte
    /// unchanged.
    fn face_offsets_slot_offset_labels_clear(
        slot_offset: f32,
        label_spans_occupied: &[LabelFaceSpan],
    ) -> f32 {
        let mut offset = slot_offset;
        // A handful of iterations is enough to step clear of multiple labels
        // stacked on the same face; real diagrams rarely have more than one
        // or two per face, and each iteration strictly increases separation
        // from the offending span, so this cannot oscillate.
        for _ in 0..4 {
            let Some(span) = label_spans_occupied.iter().find(|span| {
                offset > span.near - CONTACT_GAP_MIN_PX && offset < span.far + CONTACT_GAP_MIN_PX
            }) else {
                break;
            };
            let near_side = span.near - CONTACT_GAP_MIN_PX;
            let far_side = span.far + CONTACT_GAP_MIN_PX;
            offset = if (near_side - offset).abs() <= (far_side - offset).abs() {
                near_side
            } else {
                far_side
            };
        }
        offset
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
            halo_animation_params,
            interaction_edge_halo_enabled,
        } = css_animation_append_params;
        let edge_animation = EdgeAnimationCalculator::calculate(
            edge_animation_params,
            edge_path_info,
            edge_group_cycle_distance,
            edge_group_animation_duration_total_s,
            halo_animation_params,
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
        let mut forward_path_svg = edge_path_info.path.to_svg();
        // Escape underscores for use inside the tailwind arbitrary value
        // (encre-css transforms these to spaces in the actual CSS value).
        StringCharReplacer::replace_inplace(&mut forward_path_svg, ' ', '_');

        Self::css_animation_append_arrowhead_classes(
            tailwind_classes,
            edge_path_info,
            edge_animation_active,
            focus_mode,
            associated_process_steps,
            &edge_animation.arrow_head_animation_name,
            animation_duration.clone(),
            forward_path_svg,
        );

        // Append CSS keyframes for edge stroke and arrowhead.
        if !css.is_empty() {
            css.push('\n');
        }
        css.push_str(&edge_animation.keyframe_css);
        css.push_str(&edge_animation.arrow_head_keyframe_css);

        // The halo's (and its outline's) tailwind-classes entity keys only
        // exist when halo rendering is enabled -- attaching an animate class
        // (or pushing keyframes) when disabled would spuriously create those
        // keys and make `svg_elements_to_svg_mapper.rs` render halo/outline
        // paths that should not exist.
        if interaction_edge_halo_enabled {
            Self::css_animation_append_halo_classes(
                tailwind_classes,
                edge_path_info,
                edge_animation_active,
                focus_mode,
                associated_process_steps,
                &edge_animation.halo_animation_name,
                &animation_duration,
            );
            css.push_str(&edge_animation.halo_keyframe_css);

            Self::css_animation_append_halo_outline_classes(
                tailwind_classes,
                edge_path_info,
                edge_animation_active,
                focus_mode,
                associated_process_steps,
                &edge_animation.halo_outline_animation_name,
                &animation_duration,
            );
            css.push_str(&edge_animation.halo_outline_keyframe_css);
        }
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

    /// Appends CSS classes for the halo opacity animation to the diagram's
    /// tailwind classes.
    ///
    /// Unlike the arrowhead's synthetic entity (which starts with no prior
    /// classes), `{edge_id}__halo` already holds static forward/reverse
    /// classes written earlier by `tailwind_classes_builder.rs`'s
    /// `interaction_edge_halo_classes_build` (colour, stroke width, base
    /// opacity), so the animate class is appended to the existing classes
    /// rather than overwriting them -- mirroring how the edge body's
    /// existing classes are appended to in `Self::css_animation_append`.
    #[allow(clippy::too_many_arguments)]
    fn css_animation_append_halo_classes<'id>(
        tailwind_classes: &mut EntityTailwindClasses<'id>,
        edge_path_info: &EdgePathInfo<'_, 'id>,
        edge_animation_active: EdgeAnimationActive,
        focus_mode: TailwindFocusMode<'_, 'id>,
        associated_process_steps: &[&NodeId<'id>],
        halo_animation_name: &str,
        animation_duration: &str,
    ) {
        let halo_classes = Self::animate_classes_build(
            halo_animation_name,
            animation_duration,
            edge_animation_active,
            focus_mode,
            associated_process_steps,
        );
        if halo_classes.is_empty() {
            return;
        }

        let halo_id = EdgeHaloIdGenerator::generate(&edge_path_info.edge_id);
        Self::tailwind_classes_append(tailwind_classes, halo_id, halo_classes);
    }

    /// Appends CSS classes for the halo outline's opacity animation to the
    /// diagram's tailwind classes.
    ///
    /// Mirrors `Self::css_animation_append_halo_classes`, but targets the
    /// `{edge_id}__halo_outline` entity (already holding static
    /// forward/reverse classes written by `tailwind_classes_builder.rs`)
    /// rather than `{edge_id}__halo`.
    #[allow(clippy::too_many_arguments)]
    fn css_animation_append_halo_outline_classes<'id>(
        tailwind_classes: &mut EntityTailwindClasses<'id>,
        edge_path_info: &EdgePathInfo<'_, 'id>,
        edge_animation_active: EdgeAnimationActive,
        focus_mode: TailwindFocusMode<'_, 'id>,
        associated_process_steps: &[&NodeId<'id>],
        halo_outline_animation_name: &str,
        animation_duration: &str,
    ) {
        let halo_outline_classes = Self::animate_classes_build(
            halo_outline_animation_name,
            animation_duration,
            edge_animation_active,
            focus_mode,
            associated_process_steps,
        );
        if halo_outline_classes.is_empty() {
            return;
        }

        let halo_outline_id = EdgeHaloOutlineIdGenerator::generate(&edge_path_info.edge_id);
        Self::tailwind_classes_append(tailwind_classes, halo_outline_id, halo_outline_classes);
    }

    /// Builds the `animate-[{animation_name}_{duration}s_linear_infinite]`
    /// tailwind class (with the `EdgeAnimationActive::OnProcessStepFocus`
    /// focus-mode branching applied), for an entity that needs no other
    /// classes alongside it (unlike the arrowhead, which also carries
    /// `offset-path` / `stroke-dasharray` classes).
    ///
    /// Returns an empty string when nothing should animate (`Baked` focus
    /// mode with no associated, currently-focused process step).
    fn animate_classes_build<'id>(
        animation_name: &str,
        animation_duration: &str,
        edge_animation_active: EdgeAnimationActive,
        focus_mode: TailwindFocusMode<'_, 'id>,
        associated_process_steps: &[&NodeId<'id>],
    ) -> String {
        let mut classes = String::new();
        match edge_animation_active {
            EdgeAnimationActive::Always => classes.push_str(&format!(
                "animate-[{animation_name}_{animation_duration}s_linear_infinite]"
            )),
            EdgeAnimationActive::OnProcessStepFocus => match focus_mode {
                TailwindFocusMode::Interactive => {
                    associated_process_steps
                        .iter()
                        .for_each(|process_step_id| {
                            classes.push_str(&format!(
                                "\ngroup-has-[#{process_step_id}:focus-within]:\
                                    animate-[{animation_name}_{animation_duration}s_linear_infinite]"
                            ));
                        });
                }
                TailwindFocusMode::Baked { .. } => {
                    // In baked mode, the focused step's interacting edges
                    // animate unconditionally; all other edges do not
                    // animate.
                    if Self::focus_baked_step_associated(focus_mode, associated_process_steps) {
                        classes.push_str(&format!(
                            "animate-[{animation_name}_{animation_duration}s_linear_infinite]"
                        ));
                    }
                }
            },
        }
        classes
    }

    /// Appends `new_classes` onto an entity's existing tailwind classes
    /// (rather than overwriting them), inserting a blank-separated join when
    /// the entity already has classes.
    ///
    /// Used for entities (like `{edge_id}__halo` / `{edge_id}__halo_outline`)
    /// that already hold static classes written earlier in the pipeline by
    /// `tailwind_classes_builder.rs`.
    fn tailwind_classes_append<'id>(
        tailwind_classes: &mut EntityTailwindClasses<'id>,
        entity_id: Id<'static>,
        new_classes: String,
    ) {
        let existing = tailwind_classes
            .get(&entity_id)
            .cloned()
            .unwrap_or_default();
        let combined = if existing.is_empty() {
            new_classes
        } else {
            format!("{existing}\n{new_classes}")
        };
        tailwind_classes.insert(entity_id, combined);
    }
}

// === Supporting types === //

/// The span occupied by an edge's own real label taffy node along the face
/// axis, expressed as signed pixel offsets from the face midpoint -- the
/// same units as `EdgeContactPointOffsets` -- so it can be compared directly
/// against a slot-based fallback offset.
///
/// `near <= far` always holds: `near` is the entry-side edge of the label (the
/// side a contact is routed to, see
/// `SvgEdgeInfosBuilder::label_face_span_compute`), `far` is the opposite edge,
/// and both are already adjusted outward by the halo clearance.
#[derive(Clone, Copy, Debug, PartialEq)]
struct LabelFaceSpan {
    near: f32,
    far: f32,
}

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
    /// X coordinate of the edge's `from` node (absolute position).
    ///
    /// Used as the secondary sort key for Top / Bottom faces when this
    /// contact is at the edge's `to` endpoint -- contacts entering a
    /// shared target face are ordered by where each edge approaches from.
    ///
    /// # Examples
    ///
    /// `50.0` for a node positioned 50 px from the left edge.
    from_node_x: f32,
    /// Y coordinate of the edge's `from` node (absolute position).
    ///
    /// Used as the secondary sort key for Left / Right faces when this
    /// contact is at the edge's `to` endpoint.
    ///
    /// # Examples
    ///
    /// `30.0` for a node positioned 30 px from the top edge.
    from_node_y: f32,
    /// Index into the `all_pass1_groups` vector identifying which edge
    /// group this contact belongs to.
    pass1_group_index: usize,
    /// Index into the group's `pass1_infos` vector identifying which
    /// edge within the group this contact belongs to.
    edge_index: usize,
    /// `true` if this contact is at the "from" endpoint of the edge,
    /// `false` for the "to" endpoint.
    is_from_endpoint: bool,
    /// `true` if the edge is an interaction edge, `false` for a
    /// dependency edge.
    ///
    /// Dependency and interaction edges are spread in separate slot pools
    /// so an interaction edge sharing a face does not push a dependency
    /// edge's contact off the face midpoint.
    is_interaction: bool,
}

/// A flattened face contact used by `face_offsets_collisions_separate` to
/// detect and resolve coincident contacts across different nodes.
///
/// Unlike [`FaceContactEntry`], which is grouped per `(node, face)`, this
/// record carries the resolved absolute coordinate of the contact so contacts
/// from different nodes that land at the same place can be clustered and
/// re-spread.
#[derive(Clone, Debug)]
struct FaceContactCollisionRecord<'id> {
    /// The `(node, face)` group this contact belongs to.
    node_id_and_face: NodeIdAndFace<'id>,
    /// Slot index of this contact within its group's offsets vector.
    slot_index: usize,
    /// Midpoint of the node face along the face axis (x for Top/Bottom, y for
    /// Left/Right).
    midpoint: f32,
    /// Absolute coordinate of the contact along the face axis
    /// (`midpoint + offset`).
    abs_coord: f32,
    /// Coordinate of the node face along the **rank** axis (y for Top/Bottom,
    /// x for Left/Right).
    ///
    /// Two contacts sharing an `abs_coord` only actually collide when their
    /// stubs protrude into the same inter-rank gap. Same-rank siblings have an
    /// equal `main_axis_coord`; vertically-stacked siblings at different ranks
    /// do not, so this distinguishes a genuine cross-node coincidence from two
    /// contacts that merely line up along the face axis.
    main_axis_coord: f32,
    /// Length of the node face (width for Top/Bottom, collapsed height for
    /// Left/Right), used to size the fan so it fits within the face.
    face_length: f32,
    /// Absolute rank difference between the edge's endpoints, used as the
    /// primary deterministic ordering key within a cluster.
    rank_distance: u32,
    /// Index into `all_pass1_groups`, used as a stable ordering tie-breaker.
    pass1_group_index: usize,
    /// Index into the group's `pass1_infos`, used as a stable ordering
    /// tie-breaker.
    edge_index: usize,
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
    /// Whether this edge is an interaction edge (`true`) or a dependency
    /// edge (`false`).
    ///
    /// Used to spread dependency and interaction contacts in separate
    /// slot pools during offset computation, so a co-located interaction
    /// edge does not push a dependency edge's contact off the face
    /// midpoint.
    pub(super) is_interaction: bool,
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
    /// Parameters for this edge's halo (and halo outline) opacity animation.
    halo_animation_params: HaloAnimationParams,
    /// Whether `RenderOptions::interaction_edge_halo` is enabled.
    ///
    /// The halo's tailwind-classes entity key (`{edge_id}__halo`) only
    /// exists in `tailwind_classes` when this is `true` -- its mere presence
    /// is what `svg_elements_to_svg_mapper.rs` uses to decide whether to
    /// render the halo `<path>` at all, so the halo animation classes and
    /// keyframes must not be attached when this is `false`.
    interaction_edge_halo_enabled: bool,
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
