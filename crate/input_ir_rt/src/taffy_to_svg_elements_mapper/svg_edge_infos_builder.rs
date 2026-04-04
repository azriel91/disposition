use disposition_input_ir_model::EdgeAnimationActive;
use disposition_ir_model::{
    edge::{Edge, EdgeGroup, EdgeId},
    entity::EntityTypes,
    node::{NodeId, NodeRank},
    IrDiagram,
};
use disposition_model_common::{entity::EntityType, theme::Css, Id, Map, RankDir};
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
    edge_path_builder::{EdgeFaceOffset, SpacerCoordinates},
    ArrowHeadBuilder, EdgeAnimationCalculator, EdgePathBuilder, StringCharReplacer,
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
    ///    offsets, register face contacts, and store path midpoints.
    /// 2. Sort contacts per face globally using curvature-center-based ordering
    ///    and compute offsets.
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
            rank_dir,
            ..
        } = ir_diagram;
        let rank_dir = *rank_dir;

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
                &mut face_contact_tracker,
            );
            all_pass1_groups.push(edge_group_pass1);
        }

        // === Global sort and offset computation === //

        let face_offsets_by_node_face = Self::face_offsets_compute(
            &mut all_pass1_groups,
            svg_node_info_map,
            &mut face_contact_tracker,
        );

        // === Global Pass 2: rebuild paths with offsets, emit SvgEdgeInfos === //

        let mut svg_edge_infos = Vec::new();

        for edge_group_pass1 in all_pass1_groups {
            let EdgeGroupPass1 {
                edge_group_id,
                edge_animation_params,
                pass1_infos,
                from_slot_indices,
                to_slot_indices,
            } = edge_group_pass1;

            let visible_segments_length = edge_animation_params.visible_segments_length;

            let edge_path_infos = Self::build_edge_path_infos_with_offsets(
                rank_dir,
                &pass1_infos,
                &from_slot_indices,
                &to_slot_indices,
                &face_offsets_by_node_face,
                svg_node_info_map,
                taffy_tree,
                edge_spacer_taffy_nodes,
                visible_segments_length,
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
                            .any(EntityType::is_interaction_edge_type)
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
                let arrow_head_path_d = if is_interaction_edge {
                    // Origin-centred V-shape; CSS offset-path handles
                    // positioning and rotation.
                    ArrowHeadBuilder::build_origin_arrow_head()
                } else {
                    // Positioned V-shape at the `to` node end of the edge.
                    ArrowHeadBuilder::build_static_arrow_head(&path)
                };

                svg_edge_infos.push(SvgEdgeInfo::new(
                    edge_id,
                    edge_group_id.clone(),
                    edge.from.clone(),
                    edge.to.clone(),
                    path_d,
                    arrow_head_path_d,
                ));
            });
        }

        svg_edge_infos
    }

    /// **Pass 1** for a single edge group: determines edge types, builds
    /// zero-offset paths, registers face contacts, and stores the path
    /// midpoint for each endpoint so that the global sort phase can
    /// compute curvature-center-based ordering.
    ///
    /// The returned `EdgeGroupPass1` contains everything needed for
    /// pass 2 to rebuild the paths with offsets.
    fn build_edge_pass1_infos<'edge, 'id>(
        rank_dir: RankDir,
        edge_group_id: &'edge EdgeGroupId<'id>,
        edge_group: &'edge EdgeGroup<'id>,
        entity_types: &'edge EntityTypes<'id>,
        svg_node_info_map: &'edge Map<&NodeId<'id>, &SvgNodeInfo<'id>>,
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
            let path = EdgePathBuilder::build(rank_dir, from_info, to_info, edge_type);
            let faces = EdgePathBuilder::faces_select(rank_dir, from_info, to_info);

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

            pass1_infos.push(EdgePass1Info {
                edge_index,
                edge,
                edge_id,
                edge_type,
                from_face,
                to_face,
                path_midpoint,
                path_bounds,
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
    /// curvature-center-based sorting.
    ///
    /// For each (node, face) the algorithm:
    ///
    /// 1. Gathers path midpoints from every edge touching that face.
    /// 2. Computes a common curvature center -- the mean of all midpoints.
    /// 3. Projects the curvature center onto the face axis to determine which
    ///    direction along the face is "toward the center".
    /// 4. Sorts edges by radius (distance from their midpoint to the curvature
    ///    center), smallest first.
    /// 5. Assigns offset slots so that the tightest-radius edge gets the slot
    ///    nearest to the curvature center along the face, and each successive
    ///    edge (with a larger radius) gets the next slot away from the center.
    ///
    /// This prevents edges from crossing each other: inner curves stay
    /// on the inside and outer curves stay on the outside.
    fn face_offsets_compute<'edge, 'id>(
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
                            pass1_group_index,
                            edge_index,
                            is_from_endpoint: false,
                        });
                }
            }
        }

        // Sort each face's entries using curvature-center ordering, then
        // assign slot indices.
        for (node_id_and_face, face_contact_entries) in face_contact_entries_by_node_face.iter_mut()
        {
            Self::face_entries_sort_by_curvature(
                &node_id_and_face.node_id,
                node_id_and_face.face,
                face_contact_entries,
                svg_node_info_map,
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
            let offsets: Vec<f32> = (0..contact_count)
                .map(|_| {
                    face_contact_tracker.offset_calculate(
                        &node_id_and_face.node_id,
                        node_id_and_face.face,
                        face_length,
                    )
                })
                .collect();
            face_offsets_by_node_face.insert(
                node_id_and_face.clone(),
                EdgeContactPointOffsets::new(offsets),
            );
        }

        face_offsets_by_node_face
    }

    /// Sorts the entries for a single (node, face) by the angle from the
    /// curvature center to each entry's path midpoint, so that edges
    /// connecting to the same face are ordered to reduce crossovers.
    ///
    /// # Algorithm
    ///
    /// 1. Compute the curvature center via `curvature_center_compute`, which
    ///    uses the extremal path-bounds coordinates in the direction the edges
    ///    extend away from the face, falling back to the mean along the
    ///    parallel axis when the edges don't overshoot the node extent.
    /// 2. For each entry, compute the angle from the curvature center to its
    ///    path midpoint, using an `atan2` formulation chosen per face so that
    ///    the angle increases along the face axis (top-to-bottom for
    ///    Left/Right, left-to-right for Top/Bottom).
    /// 3. Sort by angle ascending and assign to slots in that order.
    ///
    /// The per-face `atan2` formulations place the `atan2` discontinuity
    /// (at +/- pi) behind the node -- opposite the face normal -- where
    /// no path midpoints should lie, avoiding wrap-around artefacts.
    ///
    /// | Face   | Midpoints  | `atan2` args      | Increases     |
    /// |--------|------------|-------------------|---------------|
    /// | Left   | dx < 0     | `(dy, -dx)`       | top to bottom |
    /// | Right  | dx > 0     | `(dy,  dx)`       | top to bottom |
    /// | Top    | dy < 0     | `(dx, -dy)`       | left to right |
    /// | Bottom | dy > 0     | `(dx,  dy)`       | left to right |
    fn face_entries_sort_by_curvature<'id>(
        node_id: &NodeId<'id>,
        face: NodeFace,
        face_contact_entries: &mut [FaceContactEntry],
        svg_node_info_map: &Map<&NodeId<'id>, &SvgNodeInfo<'id>>,
    ) {
        let contact_count = face_contact_entries.len();
        if contact_count <= 1 {
            return;
        }

        let curvature_center =
            Self::curvature_center_compute(node_id, face, face_contact_entries, svg_node_info_map);

        // Sort by angle from the curvature center to each midpoint.
        //
        // The `atan2` argument pair is chosen per face so that:
        //
        // * The angle increases along the face axis (top-to-bottom for vertical faces,
        //   left-to-right for horizontal faces).
        // * The atan2 discontinuity at +/- pi sits behind the node, opposite the
        //   direction midpoints extend, so no midpoint straddles the wrap-around.
        face_contact_entries.sort_by(|entry_a, entry_b| {
            let angle_a = Self::face_curvature_angle(face, entry_a.path_midpoint, curvature_center);
            let angle_b = Self::face_curvature_angle(face, entry_b.path_midpoint, curvature_center);
            angle_a
                .partial_cmp(&angle_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// Computes the curvature center used for angle-based sorting of
    /// face contact entries.
    ///
    /// Instead of using the simple mean of all path midpoints, the
    /// curvature center is placed at the extremal (min or max)
    /// coordinate of the edges' full path bounds in the direction the
    /// paths extend away from the node face. This pulls the center
    /// toward the "apex" of the fan of edges, producing more
    /// discriminating angles.
    ///
    /// # Per-axis logic
    ///
    /// For the axis **perpendicular** to the face (the direction edges
    /// extend away from the node):
    ///
    /// * If the mean of the path midpoints is on the outward side of the face,
    ///   the extremal coordinate (min or max) of all path bounds in that
    ///   direction is used.
    /// * Otherwise (edges curving back toward the node) the opposite extremal
    ///   coordinate is used.
    ///
    /// For the axis **parallel** to the face:
    ///
    /// * If the mean is outside the node's extent on that axis, the
    ///   corresponding extremal coordinate (min or max) of the path bounds is
    ///   used.
    /// * If the mean falls within the node's extent, the mean itself is kept.
    fn curvature_center_compute<'id>(
        node_id: &NodeId<'id>,
        face: NodeFace,
        face_contact_entries: &[FaceContactEntry],
        svg_node_info_map: &Map<&NodeId<'id>, &SvgNodeInfo<'id>>,
    ) -> PathMidpoint {
        let contact_count = face_contact_entries.len() as f64;

        // Mean of path midpoints.
        let mean_x = face_contact_entries
            .iter()
            .map(|e| e.path_midpoint.x)
            .sum::<f64>()
            / contact_count;
        let mean_y = face_contact_entries
            .iter()
            .map(|e| e.path_midpoint.y)
            .sum::<f64>()
            / contact_count;

        // Aggregate path bounds across all entries.
        let bounds_x_min = face_contact_entries
            .iter()
            .map(|e| e.path_bounds.x_min)
            .fold(f64::INFINITY, f64::min);
        let bounds_x_max = face_contact_entries
            .iter()
            .map(|e| e.path_bounds.x_max)
            .fold(f64::NEG_INFINITY, f64::max);
        let bounds_y_min = face_contact_entries
            .iter()
            .map(|e| e.path_bounds.y_min)
            .fold(f64::INFINITY, f64::min);
        let bounds_y_max = face_contact_entries
            .iter()
            .map(|e| e.path_bounds.y_max)
            .fold(f64::NEG_INFINITY, f64::max);

        // Node geometry.
        let (node_x, node_y, node_w, node_h) = if let Some(info) = svg_node_info_map.get(node_id) {
            (
                info.x as f64,
                info.y as f64,
                info.width as f64,
                info.height_collapsed as f64,
            )
        } else {
            return PathMidpoint {
                x: mean_x,
                y: mean_y,
            };
        };

        let (center_x, center_y) = match face {
            // Left / Right faces: edges extend along X (perpendicular),
            // spread along Y (parallel).
            NodeFace::Left | NodeFace::Right => {
                let face_x = if face == NodeFace::Left {
                    node_x
                } else {
                    node_x + node_w
                };

                // Perpendicular axis (X): use extremal bound opposite
                // to the direction the edges extend.
                let cx = if mean_x < face_x {
                    bounds_x_max
                } else {
                    bounds_x_min
                };

                // Parallel axis (Y): use extremal bound when outside
                // the node's vertical extent, otherwise keep the mean.
                let node_y_min = node_y;
                let node_y_max = node_y + node_h;
                let cy = if mean_y < node_y_min {
                    bounds_y_max
                } else if mean_y > node_y_max {
                    bounds_y_min
                } else {
                    mean_y
                };

                (cx, cy)
            }

            // Top / Bottom faces: edges extend along Y (perpendicular),
            // spread along X (parallel).
            NodeFace::Top | NodeFace::Bottom => {
                let face_y = if face == NodeFace::Top {
                    node_y
                } else {
                    node_y + node_h
                };

                // Perpendicular axis (Y): use extremal bound opposite
                // to the direction the edges extend.
                let cy = if mean_y < face_y {
                    bounds_y_max
                } else {
                    bounds_y_min
                };

                // Parallel axis (X): use extremal bound when outside
                // the node's horizontal extent, otherwise keep the mean.
                let node_x_min = node_x;
                let node_x_max = node_x + node_w;
                let cx = if mean_x < node_x_min {
                    bounds_x_max
                } else if mean_x > node_x_max {
                    bounds_x_min
                } else {
                    mean_x
                };

                (cx, cy)
            }
        };

        PathMidpoint {
            x: center_x,
            y: center_y,
        }
    }

    /// Returns the angle from `curvature_center` to `midpoint`, using an
    /// `atan2` formulation specific to `face`.
    ///
    /// The formulation is chosen so that the returned angle increases
    /// along the face axis (top-to-bottom for Left/Right, left-to-right
    /// for Top/Bottom) and the atan2 discontinuity is placed behind the
    /// node where no midpoints should be.
    fn face_curvature_angle(
        face: NodeFace,
        midpoint: PathMidpoint,
        curvature_center: PathMidpoint,
    ) -> f64 {
        let dx = midpoint.x - curvature_center.x;
        let dy = midpoint.y - curvature_center.y;
        match face {
            // Left face: midpoints extend to the left (dx < 0).
            // atan2(dy, -dx) increases top-to-bottom, discontinuity to the right.
            NodeFace::Left => f64::atan2(dy, -dx),
            // Right face: midpoints extend to the right (dx > 0).
            // atan2(dy, dx) increases top-to-bottom, discontinuity to the left.
            NodeFace::Right => f64::atan2(dy, dx),
            // Top face: midpoints extend upward (dy < 0).
            // atan2(dx, -dy) increases left-to-right, discontinuity below.
            NodeFace::Top => f64::atan2(dx, -dy),
            // Bottom face: midpoints extend downward (dy > 0).
            // atan2(dx, dy) increases left-to-right, discontinuity above.
            NodeFace::Bottom => f64::atan2(dx, dy),
        }
    }

    /// **Pass 2** for a single edge group: rebuilds every path using the
    /// globally computed face offsets.
    #[allow(clippy::too_many_arguments)]
    fn build_edge_path_infos_with_offsets<'edge, 'id>(
        rank_dir: RankDir,
        pass1_infos: &[EdgePass1Info<'edge, 'id>],
        from_slot_indices: &[Option<usize>],
        to_slot_indices: &[Option<usize>],
        face_offsets_by_node_face: &Map<NodeIdAndFace<'id>, EdgeContactPointOffsets>,
        svg_node_info_map: &Map<&NodeId<'id>, &SvgNodeInfo<'id>>,
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        edge_spacer_taffy_nodes: &Map<EdgeId<'id>, EdgeSpacerTaffyNodes>,
        visible_segments_length: f64,
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
                    &pass1_info.edge_id,
                    taffy_tree,
                    edge_spacer_taffy_nodes,
                );

                let path = EdgePathBuilder::build_with_offsets_and_spacers(
                    rank_dir,
                    from_info,
                    to_info,
                    pass1_info.edge_type,
                    face_offset,
                    &spacer_coordinates,
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
    /// Returns an empty `Vec` when the edge has no spacer nodes.
    fn spacer_coordinates_from_spacers<'id>(
        edge_id: &EdgeId<'id>,
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        edge_spacer_taffy_nodes: &Map<EdgeId<'id>, EdgeSpacerTaffyNodes>,
    ) -> Vec<SpacerCoordinates> {
        let Some(spacer_nodes) = edge_spacer_taffy_nodes.get(edge_id) else {
            return Vec::new();
        };

        // Collect spacer coordinates sorted by rank so they are in the
        // correct order from the from-node towards the to-node.
        let mut rank_spacers: Vec<(NodeRank, SpacerCoordinates)> = spacer_nodes
            .rank_to_spacer_taffy_node_id
            .iter()
            .filter_map(|(rank, &taffy_node_id)| {
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

                // Slice the spacer in half vertically: the entry is the
                // top midpoint and the exit is the bottom midpoint.
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
        let edge_anim = EdgeAnimationCalculator::calculate(
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
        let dasharray = edge_anim.dasharray;
        let animation_name = edge_anim.animation_name;
        let animation_duration =
            EdgeAnimationCalculator::format_duration(edge_anim.edge_animation_duration_s);

        let animation_classes = {
            let mut classes = format!("[stroke-dasharray:{dasharray}]");
            match edge_animation_active {
                EdgeAnimationActive::Always => {
                    classes.push_str(&format!(
                        "\nanimate-[{animation_name}_{animation_duration}s_linear_infinite]"
                    ));
                }
                EdgeAnimationActive::OnProcessStepFocus => {
                    associated_process_steps.iter().for_each(|process_step_id| {
                        classes.push_str(&format!(
                            "\ngroup-has-[#{process_step_id}:focus-within]:\
                                animate-[{animation_name}_{animation_duration}s_linear_infinite]"
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

        let arrow_head_animation_name = &edge_anim.arrow_head_animation_name;

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

        let arrow_head_entity_id_str = format!("{}_arrow_head", edge_path_info.edge_id.as_str());
        let arrow_head_entity_id: Id<'id> = Id::try_from(arrow_head_entity_id_str)
            .expect("arrow head entity ID should be valid")
            .into_static();
        tailwind_classes.insert(arrow_head_entity_id, arrow_head_classes);

        // Append CSS keyframes for both edge stroke and arrowhead.
        if !css.is_empty() {
            css.push('\n');
        }
        css.push_str(&edge_anim.keyframe_css);
        css.push_str(&edge_anim.arrow_head_keyframe_css);
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

/// A single contact entry used during the per-face curvature-center
/// sorting phase.
///
/// Tracks which edge touches a given (node, face) and the path midpoint
/// and bounds needed to compute the curvature center for sorting.
#[derive(Clone, Copy, Debug, PartialEq)]
struct FaceContactEntry {
    /// Mean anchor point of the edge's zero-offset path, used to
    /// determine curvature angle during sorting.
    path_midpoint: PathMidpoint,
    /// Axis-aligned bounding box of the edge's zero-offset path, used
    /// to compute the curvature center extremal coordinates.
    path_bounds: PathBounds,
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
/// with face contact offsets.
struct EdgePass1Info<'edge, 'id> {
    /// The edge's position within its edge group (0-based).
    edge_index: usize,
    /// Reference to the IR edge (source and target node IDs).
    edge: &'edge Edge<'id>,
    /// Generated unique ID for this edge.
    edge_id: EdgeId<'id>,
    /// Whether this edge is unpaired or part of a request/response pair.
    edge_type: EdgeType,
    /// Which face of the "from" node this edge connects to.
    ///
    /// `None` when the edge connects a contained node (no face offset
    /// applies).
    from_face: Option<NodeFace>,
    /// Which face of the "to" node this edge connects to.
    ///
    /// `None` when the edge connects a contained node (no face offset
    /// applies).
    to_face: Option<NodeFace>,
    /// Mean anchor point of the zero-offset path, used to determine
    /// curvature-center distance during the sort phase.
    path_midpoint: PathMidpoint,
    /// Axis-aligned bounding box of the zero-offset path's anchor
    /// points, used to compute the curvature center extremal
    /// coordinates.
    path_bounds: PathBounds,
}

/// All pass-1 data for a single edge group.
///
/// Contains the per-edge metadata from pass 1 together with the slot
/// index assignments that are filled in by the global
/// `face_offsets_compute` phase.
struct EdgeGroupPass1<'edge, 'id> {
    /// The edge group this data belongs to.
    edge_group_id: &'edge EdgeGroupId<'id>,
    /// Animation parameters for this edge group.
    edge_animation_params: EdgeAnimationParams,
    /// Per-edge pass-1 data.
    pass1_infos: Vec<EdgePass1Info<'edge, 'id>>,
    /// Per-edge assigned slot index for the "from" face contact.
    ///
    /// `from_slot_indices[i]` is the slot index assigned to
    /// `pass1_infos[i]`'s "from" endpoint, or `None` if no face offset
    /// applies (e.g. contained edges).
    from_slot_indices: Vec<Option<usize>>,
    /// Per-edge assigned slot index for the "to" face contact.
    ///
    /// `to_slot_indices[i]` is the slot index assigned to
    /// `pass1_infos[i]`'s "to" endpoint, or `None` if no face offset
    /// applies (e.g. contained edges).
    to_slot_indices: Vec<Option<usize>>,
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
