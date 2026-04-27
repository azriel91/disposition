use disposition_input_ir_model::EdgeAnimationActive;
use disposition_ir_model::{
    edge::{Edge, EdgeGroup, EdgeId},
    entity::EntityTypes,
    node::{NodeId, NodeRank, NodeRanks},
    IrDiagram,
};
use disposition_model_common::{
    edge::EdgeCurvature, entity::EntityType, theme::Css, Id, Map, RankDir,
};
use disposition_svg_model::{SvgEdgeInfo, SvgNodeInfo};
use disposition_taffy_model::{taffy::TaffyTree, EdgeSpacerTaffyNodes, TaffyNodeCtx};
use kurbo::Shape;

use disposition_ir_model::entity::EntityTailwindClasses;
use disposition_model_common::edge::EdgeGroupId;

use crate::taffy_to_svg_elements_mapper::{
    edge_face_contact_tracker::EdgeFaceContactTracker,
    edge_model::{
        EdgeAnimationParams, EdgeContactPointOffsets, EdgePathInfo, EdgeType, NodeFace,
        NodeIdAndFace, PathBounds, PathMidpoint,
    },
    edge_path_builder_pass_1::{EdgeFaceOffset, SpacerCoordinates},
    edge_path_builder_pass_2::edge_path_builder_pass_2_ortho::OrthoProtrusionParams,
    ortho_protrusion_calculator::OrthoProtrusionCalculator,
    ArrowHeadBuilder, EdgeAnimationCalculator, EdgePathBuilderPass1, EdgePathBuilderPass2,
    EdgePathLocusCalculator, StringCharReplacer,
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
        svg_node_info_map: &Map<&NodeId<'id>, &SvgNodeInfo<'id>>,
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        edge_spacer_taffy_nodes: &Map<EdgeId<'id>, EdgeSpacerTaffyNodes>,
        tailwind_classes: &mut EntityTailwindClasses<'id>,
        css: &mut Css,
        edge_animation_active: EdgeAnimationActive,
    ) -> Vec<SvgEdgeInfo<'id>> {
        let IrDiagram {
            edge_groups,
            entity_types,
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

        /// 0.8 seconds per 100 pixels
        const SECONDS_PER_PIXEL: f64 = 0.8 / 100.0;

        // === Global Pass 1: collect metadata and register face contacts === //

        let mut face_contact_tracker = EdgeFaceContactTracker::new();
        let mut all_pass1_groups: Vec<EdgeGroupPass1<'_, 'id>> = Vec::new();

        for (edge_group_id, edge_group) in edge_groups.iter() {
            let edge_group_pass1 = Self::build_edge_pass1_infos(
                rank_dir,
                edge_group_id,
                edge_group,
                entity_types,
                svg_node_info_map,
                &ir_diagram.node_ranks,
                &mut face_contact_tracker,
            );
            all_pass1_groups.push(edge_group_pass1);
        }

        // === Global sort and offset computation === //

        let face_offsets_by_node_face = Self::face_offsets_compute(
            rank_dir,
            &mut all_pass1_groups,
            svg_node_info_map,
            &mut face_contact_tracker,
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

            let edge_group_path_length_total = edge_path_infos
                .iter()
                .map(|edge_path_info| edge_path_info.path_length)
                .sum::<f64>();
            let edge_group_visible_segments_length_total =
                edge_path_infos.len() as f64 * visible_segments_length;
            let edge_group_path_or_visible_segments_length_max =
                edge_group_visible_segments_length_total.max(edge_group_path_length_total);
            let edge_group_animation_duration_total_s = SECONDS_PER_PIXEL
                * edge_group_path_or_visible_segments_length_max
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
                        edge_group_path_or_visible_segments_length_max,
                        edge_group_animation_duration_total_s,
                        edge_path_info: &edge_path_info,
                        edge_animation_active,
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
                    preceding_visible_segments_lengths: _,
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
    fn build_edge_pass1_infos<'edge, 'id>(
        rank_dir: RankDir,
        edge_group_id: &'edge EdgeGroupId<'id>,
        edge_group: &'edge EdgeGroup<'id>,
        entity_types: &'edge EntityTypes<'id>,
        svg_node_info_map: &'edge Map<&NodeId<'id>, &SvgNodeInfo<'id>>,
        node_ranks: &NodeRanks<'id>,
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

            let edge_id = Self::generate_edge_id(edge_group_id, edge_index);
            let edge_type = Self::edge_type_determine(&edge_id, entity_types);

            // Build the path with zero offsets to determine natural coordinates.
            let path = EdgePathBuilderPass1::build(rank_dir, from_info, to_info, edge_type);
            let faces = EdgePathBuilderPass1::faces_select(rank_dir, from_info, to_info);

            let (from_face, to_face) = match faces {
                Some((from_face, to_face)) => (Some(from_face), Some(to_face)),
                None => (None, None),
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

            // Compute rank distance between from and to nodes.
            let rank_from = node_ranks.get(&edge.from).copied().unwrap_or_default();
            let rank_to = node_ranks.get(&edge.to).copied().unwrap_or_default();
            let rank_distance = rank_to.value().abs_diff(rank_from.value());

            // Store to-node coordinates for tie-breaking during sorting.
            let to_node_x = to_info.x;
            let to_node_y = to_info.y;
            let from_node_x = from_info.x;
            let from_node_y = from_info.y;

            pass1_infos.push(EdgePass1Info {
                edge_index,
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
    /// 4. For reversed directions (`BottomToTop`, `RightToLeft`), negates all
    ///    offsets so the spatial ordering mirrors the reversed visual flow,
    ///    reducing edge crossover.
    ///
    /// This prevents edges from crossing each other: short-range edges
    /// stay tight against the node and long-range edges arc around them.
    fn face_offsets_compute<'edge, 'id>(
        rank_dir: RankDir,
        all_pass1_groups: &mut Vec<EdgeGroupPass1<'edge, 'id>>,
        svg_node_info_map: &Map<&NodeId<'id>, &SvgNodeInfo<'id>>,
        face_contact_tracker: &mut EdgeFaceContactTracker<'id>,
    ) -> Map<NodeIdAndFace<'id>, EdgeContactPointOffsets> {
        // Collect face contact entries per (node, face) across all groups.
        let mut face_contact_entries_by_node_face: Map<NodeIdAndFace<'id>, Vec<FaceContactEntry>> =
            Map::new();

        for (pass1_group_index, edge_group_pass1) in all_pass1_groups.iter().enumerate() {
            for (edge_index, pass1_info) in edge_group_pass1.pass1_infos.iter().enumerate() {
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
            }
        }

        // Sort each face's entries by rank distance and target coordinate,
        // then assign slot indices.
        for (node_id_and_face, face_contact_entries) in face_contact_entries_by_node_face.iter_mut()
        {
            Self::face_entries_sort_by_rank_and_coordinate(
                node_id_and_face.face,
                face_contact_entries,
            );

            for (slot_index, face_contact_entry) in face_contact_entries.iter().enumerate() {
                if face_contact_entry.is_from_endpoint {
                    all_pass1_groups[face_contact_entry.pass1_group_index].from_slot_indices
                        [face_contact_entry.edge_index] = Some(slot_index);
                } else {
                    all_pass1_groups[face_contact_entry.pass1_group_index].to_slot_indices
                        [face_contact_entry.edge_index] = Some(slot_index);
                }
            }
        }

        // Reset tracker indices so `offset_calculate` hands out slots in
        // the order we request them.
        face_contact_tracker.indices_reset();

        // Pre-compute per-face ordered offset values so we can index by
        // slot rather than relying on call order.
        let mut face_offsets_by_node_face: Map<NodeIdAndFace<'id>, EdgeContactPointOffsets> =
            Map::new();

        for (node_id_and_face, face_contact_entries) in &face_contact_entries_by_node_face {
            let contact_count = face_contact_entries.len();
            let face_length = Self::face_length_for_node(
                &node_id_and_face.node_id,
                node_id_and_face.face,
                svg_node_info_map,
            );
            // For reversed directions (BottomToTop, RightToLeft) negate
            // the offsets so that the spatial ordering of contact points
            // mirrors the reversed visual flow, reducing edge crossover.
            let negate_offsets = matches!(rank_dir, RankDir::BottomToTop | RankDir::RightToLeft);

            let offsets: Vec<f32> = (0..contact_count)
                .map(|_| {
                    let offset = face_contact_tracker.offset_calculate(
                        &node_id_and_face.node_id,
                        node_id_and_face.face,
                        face_length,
                    );
                    if negate_offsets {
                        -offset
                    } else {
                        offset
                    }
                })
                .collect();
            face_offsets_by_node_face.insert(
                node_id_and_face.clone(),
                EdgeContactPointOffsets::new(offsets),
            );
        }

        face_offsets_by_node_face
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
        face_offsets_by_node_face: &Map<NodeIdAndFace<'id>, EdgeContactPointOffsets>,
        svg_node_info_map: &Map<&NodeId<'id>, &SvgNodeInfo<'id>>,
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        edge_spacer_taffy_nodes: &Map<EdgeId<'id>, EdgeSpacerTaffyNodes>,
        visible_segments_length: f64,
        ortho_protrusions: &[OrthoProtrusionParams],
    ) -> Vec<EdgePathInfo<'edge, 'id>> {
        pass1_infos
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
                let spacer_coordinates = Self::spacer_coordinates_from_spacers(
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
                    preceding_visible_segments_lengths: pass1_info.edge_index as f64
                        * visible_segments_length,
                }
            })
            .collect::<Vec<EdgePathInfo>>()
    }

    /// Computes spacer coordinates from spacer taffy nodes for an edge.
    ///
    /// If the edge has spacer nodes at intermediate ranks, their layout
    /// positions are returned in rank order as `SpacerCoordinates`. Each
    /// spacer has an entry point and an exit point that slice the spacer
    /// in half, so the edge path is perfectly straight while passing
    /// through the spacer area.
    ///
    /// Cross-container spacer nodes (inserted alongside sibling
    /// containers for edges that cross container boundaries) are also
    /// included. All spacer coordinates are sorted by absolute
    /// main-axis coordinate so they appear in the correct visual order along
    /// the edge path.
    ///
    /// Returns an empty `Vec` when the edge has no spacer nodes.
    fn spacer_coordinates_from_spacers<'id>(
        rank_dir: RankDir,
        edge_id: &EdgeId<'id>,
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        edge_spacer_taffy_nodes: &Map<EdgeId<'id>, EdgeSpacerTaffyNodes>,
    ) -> Vec<SpacerCoordinates> {
        let Some(spacer_nodes) = edge_spacer_taffy_nodes.get(edge_id) else {
            return Vec::new();
        };

        // Collect rank-based spacer coordinates.
        let rank_spacers: Vec<(NodeRank, SpacerCoordinates)> = spacer_nodes
            .rank_to_spacer_taffy_node_id
            .iter()
            .filter_map(|(rank, &taffy_node_id)| {
                let coords =
                    Self::spacer_absolute_coordinates(rank_dir, taffy_tree, taffy_node_id)?;
                Some((*rank, coords))
            })
            .collect();

        // Collect cross-container spacer coordinates.
        let cross_container_spacers: Vec<SpacerCoordinates> = spacer_nodes
            .cross_container_spacer_taffy_node_ids
            .iter()
            .filter_map(|&taffy_node_id| {
                Self::spacer_absolute_coordinates(rank_dir, taffy_tree, taffy_node_id)
            })
            .collect();

        if cross_container_spacers.is_empty() {
            // Fast path: only rank-based spacers -- sort by rank as before.
            let mut rank_spacers = rank_spacers;
            rank_spacers.sort_by_key(|(rank, _)| *rank);
            return rank_spacers.into_iter().map(|(_, coords)| coords).collect();
        }

        // Merge both kinds and sort by absolute coordinate along the
        // main axis so the spacers appear in the correct visual order
        // along the edge path.
        let mut all_spacers: Vec<SpacerCoordinates> = rank_spacers
            .into_iter()
            .map(|(_, coords)| coords)
            .chain(cross_container_spacers)
            .collect();

        all_spacers.sort_by(|a, b| {
            let a_key = match rank_dir {
                RankDir::TopToBottom | RankDir::BottomToTop => a.entry_y,
                RankDir::LeftToRight | RankDir::RightToLeft => a.entry_x,
            };
            let b_key = match rank_dir {
                RankDir::TopToBottom | RankDir::BottomToTop => b.entry_y,
                RankDir::LeftToRight | RankDir::RightToLeft => b.entry_x,
            };
            a_key
                .partial_cmp(&b_key)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        all_spacers
    }

    /// Computes absolute spacer coordinates for a single taffy node.
    ///
    /// Walks up the taffy tree to accumulate the absolute position, then
    /// returns `SpacerCoordinates` with entry and exit points that
    /// depend on `rank_dir`:
    ///
    /// * `RankDir::TopToBottom`: entry at top midpoint (smallest y), exit at
    ///   bottom midpoint (largest y).
    /// * `RankDir::BottomToTop`: entry at bottom midpoint (largest y), exit at
    ///   top midpoint (smallest y).
    /// * `RankDir::LeftToRight`: entry at left midpoint (smallest x), exit at
    ///   right midpoint (largest x).
    /// * `RankDir::RightToLeft`: entry at right midpoint (largest x), exit at
    ///   left midpoint (smallest x).
    fn spacer_absolute_coordinates(
        rank_dir: RankDir,
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        taffy_node_id: taffy::NodeId,
    ) -> Option<SpacerCoordinates> {
        let layout = taffy_tree.layout(taffy_node_id).ok()?;

        // === Absolute Coordinates === //
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
        let cy = y_acc + layout.size.height / 2.0;
        let left_x = x_acc;
        let right_x = x_acc + layout.size.width;
        let top_y = y_acc;
        let bottom_y = y_acc + layout.size.height;

        let spacer_coordinates = match rank_dir {
            // Vertical flow: entry/exit share the same x (center),
            // differ in y.
            RankDir::TopToBottom => SpacerCoordinates {
                entry_x: cx,
                entry_y: top_y,
                exit_x: cx,
                exit_y: bottom_y,
            },
            RankDir::BottomToTop => SpacerCoordinates {
                entry_x: cx,
                entry_y: bottom_y,
                exit_x: cx,
                exit_y: top_y,
            },
            // Horizontal flow: entry/exit share the same y (center),
            // differ in x.
            RankDir::LeftToRight => SpacerCoordinates {
                entry_x: left_x,
                entry_y: cy,
                exit_x: right_x,
                exit_y: cy,
            },
            RankDir::RightToLeft => SpacerCoordinates {
                entry_x: right_x,
                entry_y: cy,
                exit_x: left_x,
                exit_y: cy,
            },
        };

        Some(spacer_coordinates)
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

    /// Computes the midpoint of a `BezPath` as the mean of its anchor
    /// points (MoveTo, LineTo, and the final point of CurveTo / QuadTo
    /// elements).
    ///
    /// Returns a `PathMidpoint` in absolute SVG coordinates.
    fn path_midpoint_compute(path: &kurbo::BezPath) -> PathMidpoint {
        let mut sum_x: f64 = 0.0;
        let mut sum_y: f64 = 0.0;
        let mut point_count: usize = 0;

        for element in path.elements() {
            let point = match element {
                kurbo::PathEl::MoveTo(p) | kurbo::PathEl::LineTo(p) => Some(p),
                kurbo::PathEl::CurveTo(_, _, p) => Some(p),
                kurbo::PathEl::QuadTo(_, p) => Some(p),
                kurbo::PathEl::ClosePath => None,
            };
            if let Some(p) = point {
                sum_x += p.x;
                sum_y += p.y;
                point_count += 1;
            }
        }

        if point_count == 0 {
            PathMidpoint::default()
        } else {
            PathMidpoint {
                x: sum_x / point_count as f64,
                y: sum_y / point_count as f64,
            }
        }
    }

    /// Computes the axis-aligned bounding box of a `BezPath`'s anchor
    /// points (MoveTo, LineTo, and the final point of CurveTo / QuadTo
    /// elements).
    ///
    /// Returns a `PathBounds` in absolute SVG coordinates.
    fn path_bounds_compute(path: &kurbo::BezPath) -> PathBounds {
        let mut x_min = f64::INFINITY;
        let mut x_max = f64::NEG_INFINITY;
        let mut y_min = f64::INFINITY;
        let mut y_max = f64::NEG_INFINITY;

        for element in path.elements() {
            let point = match element {
                kurbo::PathEl::MoveTo(p) | kurbo::PathEl::LineTo(p) => Some(p),
                kurbo::PathEl::CurveTo(_, _, p) => Some(p),
                kurbo::PathEl::QuadTo(_, p) => Some(p),
                kurbo::PathEl::ClosePath => None,
            };
            if let Some(p) = point {
                x_min = x_min.min(p.x);
                x_max = x_max.max(p.x);
                y_min = y_min.min(p.y);
                y_max = y_max.max(p.y);
            }
        }

        if x_min.is_infinite() {
            PathBounds::default()
        } else {
            PathBounds {
                x_min,
                x_max,
                y_min,
                y_max,
            }
        }
    }

    /// Returns the face length (in pixels) for the given node and face.
    ///
    /// For `Top` / `Bottom` this is the node width; for `Left` / `Right`
    /// this is the node collapsed height.
    fn face_length_for_node<'id>(
        node_id: &NodeId<'id>,
        face: NodeFace,
        svg_node_info_map: &Map<&NodeId<'id>, &SvgNodeInfo<'id>>,
    ) -> f32 {
        let Some(node_info) = svg_node_info_map.get(node_id) else {
            return 100.0; // fallback
        };
        match face {
            NodeFace::Top | NodeFace::Bottom => node_info.width,
            NodeFace::Left | NodeFace::Right => node_info.height_collapsed,
        }
    }

    fn css_animation_append<'f, 'edge, 'id>(
        css_animation_append_params: CssAnimationAppendParams<'f, 'edge, 'id>,
    ) {
        let CssAnimationAppendParams {
            tailwind_classes,
            css,
            edge_animation_params,
            edge_group_path_or_visible_segments_length_max,
            edge_group_animation_duration_total_s,
            edge_path_info,
            edge_animation_active,
            associated_process_steps,
        } = css_animation_append_params;
        let edge_animation = EdgeAnimationCalculator::calculate(
            edge_animation_params,
            edge_path_info,
            edge_group_path_or_visible_segments_length_max,
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
                EdgeAnimationActive::OnProcessStepFocus => {
                    associated_process_steps.iter().for_each(|process_step_id| {
                        classes.push_str(&format!(
                            "\ngroup-has-[#{process_step_id}:focus-within]:\
                                [&>.edge_body]:animate-[{animation_name}_{animation_duration}s_linear_infinite]"
                        ));
                    });
                }
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
        // For some reason the CSS path needs the *forward* edge path, even though the
        // SVG path is reversed.
        //
        // The arrowhead element needs:
        //   1. `[offset-path:path('...')]` -- the forward edge path
        //   2. `animate-[{arrow_animation_name}_{duration}s_linear_infinite]`
        let forward_path = edge_path_info.path.reverse_subpaths();
        let mut forward_path_svg = forward_path.to_svg();
        // Escape underscores for use inside the tailwind arbitrary value
        // (encre-css transforms these to spaces in the actual CSS value).
        StringCharReplacer::replace_inplace(&mut forward_path_svg, ' ', '_');

        Self::css_animation_append_arrowhead_classes(
            tailwind_classes,
            edge_path_info,
            edge_animation_active,
            associated_process_steps,
            &edge_animation.arrow_head_animation_name,
            animation_duration,
            forward_path_svg,
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
    fn css_animation_append_arrowhead_classes<'id>(
        tailwind_classes: &mut EntityTailwindClasses<'id>,
        edge_path_info: &EdgePathInfo<'_, 'id>,
        edge_animation_active: EdgeAnimationActive,
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
                EdgeAnimationActive::OnProcessStepFocus => {
                    associated_process_steps
                        .iter()
                        .for_each(|process_step_id| {
                            classes.push_str(&format!(
                                "\ngroup-has-[#{process_step_id}:focus-within]:\
                                    animate-[{arrow_head_animation_name}_{animation_duration}s_linear_infinite]"
                            ));
                        });
                }
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

    /// Generates an edge ID from the edge group ID and edge index.
    fn generate_edge_id(edge_group_id: &EdgeGroupId<'_>, edge_index: usize) -> EdgeId<'static> {
        let edge_id_str = format!("{edge_group_id}__{edge_index}");
        Id::try_from(edge_id_str)
            .expect("edge ID should be valid")
            .into()
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
    /// The edge's position within its edge group (0-based).
    pub(super) edge_index: usize,
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
    edge_group_path_or_visible_segments_length_max: f64,
    edge_group_animation_duration_total_s: f64,
    edge_path_info: &'f EdgePathInfo<'edge, 'id>,
    edge_animation_active: EdgeAnimationActive,
    associated_process_steps: &'f [&'f NodeId<'id>],
}
