use disposition_input_ir_model::EdgeAnimationActive;
use disposition_ir_model::{
    edge::{Edge, EdgeGroup, EdgeGroups, EdgeId},
    entity::EntityTypes,
    node::NodeId,
    process::ProcessStepEntities,
};
use disposition_model_common::{entity::EntityType, theme::Css, Id, Map};
use disposition_svg_model::{SvgEdgeInfo, SvgNodeInfo};
use kurbo::Shape;

use disposition_ir_model::entity::EntityTailwindClasses;
use disposition_model_common::edge::EdgeGroupId;

use crate::taffy_to_svg_elements_mapper::{
    edge_face_contact_tracker::EdgeFaceContactTracker,
    edge_model::{EdgeAnimationParams, EdgePathInfo, EdgeType, NodeFace},
    edge_path_builder::EdgeFaceOffset,
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
    pub(super) fn build<'id>(
        edge_groups: &EdgeGroups<'id>,
        entity_types: &EntityTypes<'id>,
        svg_node_info_map: &Map<&NodeId<'id>, &SvgNodeInfo<'id>>,
        tailwind_classes: &mut EntityTailwindClasses<'id>,
        css: &mut Css,
        edge_animation_active: EdgeAnimationActive,
        process_step_entities: &ProcessStepEntities<'id>,
    ) -> Vec<SvgEdgeInfo<'id>> {
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
            let pass1 = Self::build_edge_pass1_infos(
                edge_group_id,
                edge_group,
                entity_types,
                svg_node_info_map,
                &mut face_contact_tracker,
            );
            all_pass1_groups.push(pass1);
        }

        // === Global sort and offset computation === //

        let face_offsets = Self::face_offsets_compute(
            &mut all_pass1_groups,
            svg_node_info_map,
            &mut face_contact_tracker,
        );

        // === Global Pass 2: rebuild paths with offsets, emit SvgEdgeInfos === //

        let mut svg_edge_infos = Vec::new();

        for pass1_group in all_pass1_groups {
            let EdgeGroupPass1 {
                edge_group_id,
                edge_animation_params,
                pass1_infos,
                from_slot,
                to_slot,
            } = pass1_group;

            let visible_segments_length = edge_animation_params.visible_segments_length;

            let edge_path_infos = Self::build_edge_path_infos_with_offsets(
                &pass1_infos,
                &from_slot,
                &to_slot,
                &face_offsets,
                svg_node_info_map,
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
            let path = EdgePathBuilder::build(from_info, to_info, edge_type);
            let faces = EdgePathBuilder::faces_select(from_info, to_info);

            let (from_face, to_face) = match faces {
                Some((ff, tf)) => (Some(ff), Some(tf)),
                None => (None, None),
            };

            // Register contacts.
            if let Some(ff) = from_face {
                face_contact_tracker.contact_register(edge.from.clone(), ff);
            }
            if let Some(tf) = to_face {
                face_contact_tracker.contact_register(edge.to.clone(), tf);
            }

            // Compute path midpoint for curvature-center sorting.
            let path_midpoint = Self::path_midpoint_compute(&path);

            pass1_infos.push(EdgePass1Info {
                edge_index,
                edge,
                edge_id,
                edge_type,
                from_face,
                to_face,
                path_midpoint,
            });
        }

        // === Sort contacts per face within this group === //
        //
        // Slot assignment is deferred to the global phase; here we just
        // prepare per-face entry lists that the global phase will merge.

        let from_slot: Vec<Option<usize>> = vec![None; pass1_infos.len()];
        let to_slot: Vec<Option<usize>> = vec![None; pass1_infos.len()];

        // These will be filled globally by `face_offsets_compute`.

        EdgeGroupPass1 {
            edge_group_id,
            edge_animation_params,
            pass1_infos,
            from_slot,
            to_slot,
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
    ) -> Map<(NodeId<'id>, NodeFace), Vec<f32>> {
        // Collect (path_midpoint, group_idx, edge_idx, is_from) per (node, face).
        let mut face_entries: Map<(NodeId<'id>, NodeFace), Vec<FaceContactEntry>> = Map::new();

        for (group_idx, group) in all_pass1_groups.iter().enumerate() {
            for (edge_idx, info) in group.pass1_infos.iter().enumerate() {
                if let Some(ff) = info.from_face {
                    face_entries
                        .entry((info.edge.from.clone(), ff))
                        .or_default()
                        .push(FaceContactEntry {
                            midpoint: info.path_midpoint,
                            group_idx,
                            edge_idx,
                            is_from: true,
                        });
                }
                if let Some(tf) = info.to_face {
                    face_entries
                        .entry((info.edge.to.clone(), tf))
                        .or_default()
                        .push(FaceContactEntry {
                            midpoint: info.path_midpoint,
                            group_idx,
                            edge_idx,
                            is_from: false,
                        });
                }
            }
        }

        // Sort each face's entries using curvature-center ordering, then
        // assign slot indices.
        for ((node_id, face), entries) in face_entries.iter_mut() {
            Self::face_entries_sort_by_curvature(node_id, *face, entries, svg_node_info_map);

            for (slot_idx, entry) in entries.iter().enumerate() {
                if entry.is_from {
                    all_pass1_groups[entry.group_idx].from_slot[entry.edge_idx] = Some(slot_idx);
                } else {
                    all_pass1_groups[entry.group_idx].to_slot[entry.edge_idx] = Some(slot_idx);
                }
            }
        }

        // Reset tracker indices so `offset_calculate` hands out slots in
        // the order we request them.
        face_contact_tracker.indices_reset();

        // Pre-compute per-face ordered offset values so we can index by
        // slot rather than relying on call order.
        let mut face_offsets: Map<(NodeId<'id>, NodeFace), Vec<f32>> = Map::new();
        for ((node_id, face), entries) in &face_entries {
            let count = entries.len();
            let face_length = Self::face_length_for_node(node_id, *face, svg_node_info_map);
            let offsets: Vec<f32> = (0..count)
                .map(|_| face_contact_tracker.offset_calculate(node_id, *face, face_length))
                .collect();
            face_offsets.insert((node_id.clone(), *face), offsets);
        }

        face_offsets
    }

    /// Sorts the entries for a single (node, face) so that edges with a
    /// tighter curve (smaller radius to the common curvature center) are
    /// placed closer to the curvature center along the face.
    ///
    /// # Algorithm
    ///
    /// 1. Compute the curvature center as the mean of all path midpoints.
    /// 2. Determine which direction along the face axis the center lies
    ///    relative to the node face midpoint.
    /// 3. For each entry, compute its radius (distance from its midpoint to the
    ///    curvature center).
    /// 4. Sort by radius ascending -- smallest radius first.
    /// 5. Assign sorted entries to slots starting from the slot nearest the
    ///    curvature center and progressing outward.
    ///
    /// The slot ordering produced is:
    ///
    /// * If the center is in the "negative" direction along the face (left for
    ///   Top/Bottom, up for Left/Right), then the smallest radius gets slot 0
    ///   (most negative offset) and subsequent radii get slots 1, 2, ...
    /// * If the center is in the "positive" direction, the smallest radius gets
    ///   the last slot and subsequent radii get decreasing slot indices.
    fn face_entries_sort_by_curvature<'id>(
        node_id: &NodeId<'id>,
        face: NodeFace,
        entries: &mut Vec<FaceContactEntry>,
        svg_node_info_map: &Map<&NodeId<'id>, &SvgNodeInfo<'id>>,
    ) {
        let count = entries.len();
        if count <= 1 {
            return;
        }

        // Compute curvature center = mean of all midpoints.
        let center_x = entries.iter().map(|e| e.midpoint.0).sum::<f64>() / count as f64;
        let center_y = entries.iter().map(|e| e.midpoint.1).sum::<f64>() / count as f64;

        // Face midpoint in absolute coordinates.
        let (face_mid_x, face_mid_y) =
            Self::face_midpoint_absolute(node_id, face, svg_node_info_map);

        // Determine whether the curvature center lies in the "negative"
        // direction along the face axis relative to the face midpoint.
        //
        // For Left/Right faces the axis is vertical (y increases downward):
        //   center above face midpoint => center_y < face_mid_y => negative.
        // For Top/Bottom faces the axis is horizontal (x increases rightward):
        //   center to the left => center_x < face_mid_x => negative.
        let center_toward_negative = match face {
            NodeFace::Left | NodeFace::Right => center_y < face_mid_y,
            NodeFace::Top | NodeFace::Bottom => center_x < face_mid_x,
        };

        // Sort by radius ascending (smallest / tightest curve first).
        entries.sort_by(|a, b| {
            let ra = Self::distance(a.midpoint, (center_x, center_y));
            let rb = Self::distance(b.midpoint, (center_x, center_y));
            ra.partial_cmp(&rb).unwrap_or(std::cmp::Ordering::Equal)
        });

        // If the center is toward the negative direction, the tightest
        // edge (index 0 after the sort above) should get slot 0, which
        // maps to the most-negative offset -- already in the correct
        // order.
        //
        // If the center is toward the positive direction, the tightest
        // edge should get the *last* slot (most-positive offset), so we
        // reverse the order.
        if !center_toward_negative {
            entries.reverse();
        }
    }

    /// **Pass 2** for a single edge group: rebuilds every path using the
    /// globally computed face offsets.
    fn build_edge_path_infos_with_offsets<'edge, 'id>(
        pass1_infos: &[EdgePass1Info<'edge, 'id>],
        from_slot: &[Option<usize>],
        to_slot: &[Option<usize>],
        face_offsets: &Map<(NodeId<'id>, NodeFace), Vec<f32>>,
        svg_node_info_map: &Map<&NodeId<'id>, &SvgNodeInfo<'id>>,
        visible_segments_length: f64,
    ) -> Vec<EdgePathInfo<'edge, 'id>> {
        pass1_infos
            .iter()
            .enumerate()
            .map(|(idx, info)| {
                let from_info = svg_node_info_map
                    .get(&info.edge.from)
                    .expect("from node validated in pass 1");
                let to_info = svg_node_info_map
                    .get(&info.edge.to)
                    .expect("to node validated in pass 1");

                let from_offset = info
                    .from_face
                    .and_then(|ff| {
                        let slot = from_slot[idx]?;
                        let offsets = face_offsets.get(&(info.edge.from.clone(), ff))?;
                        Some(offsets[slot])
                    })
                    .unwrap_or(0.0);

                let to_offset = info
                    .to_face
                    .and_then(|tf| {
                        let slot = to_slot[idx]?;
                        let offsets = face_offsets.get(&(info.edge.to.clone(), tf))?;
                        Some(offsets[slot])
                    })
                    .unwrap_or(0.0);

                let face_offset = EdgeFaceOffset {
                    from_offset,
                    to_offset,
                };

                let path = EdgePathBuilder::build_with_offsets(
                    from_info,
                    to_info,
                    info.edge_type,
                    face_offset,
                );
                let path_length = {
                    let accuracy = 1.0;
                    path.perimeter(accuracy)
                };

                EdgePathInfo {
                    edge_id: info.edge_id.clone(),
                    edge: info.edge,
                    edge_type: info.edge_type,
                    path,
                    path_length,
                    preceding_visible_segments_lengths: info.edge_index as f64
                        * visible_segments_length,
                }
            })
            .collect::<Vec<EdgePathInfo>>()
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
    /// Returns `(mean_x, mean_y)` in absolute SVG coordinates.
    fn path_midpoint_compute(path: &kurbo::BezPath) -> (f64, f64) {
        let mut sum_x: f64 = 0.0;
        let mut sum_y: f64 = 0.0;
        let mut count: usize = 0;

        for el in path.elements() {
            let pt = match el {
                kurbo::PathEl::MoveTo(p) | kurbo::PathEl::LineTo(p) => Some(p),
                kurbo::PathEl::CurveTo(_, _, p) => Some(p),
                kurbo::PathEl::QuadTo(_, p) => Some(p),
                kurbo::PathEl::ClosePath => None,
            };
            if let Some(p) = pt {
                sum_x += p.x;
                sum_y += p.y;
                count += 1;
            }
        }

        if count == 0 {
            (0.0, 0.0)
        } else {
            (sum_x / count as f64, sum_y / count as f64)
        }
    }

    /// Returns the absolute midpoint of a node face.
    fn face_midpoint_absolute<'id>(
        node_id: &NodeId<'id>,
        face: NodeFace,
        svg_node_info_map: &Map<&NodeId<'id>, &SvgNodeInfo<'id>>,
    ) -> (f64, f64) {
        let Some(info) = svg_node_info_map.get(node_id) else {
            return (0.0, 0.0);
        };
        match face {
            NodeFace::Top => ((info.x + info.width / 2.0) as f64, info.y as f64),
            NodeFace::Bottom => (
                (info.x + info.width / 2.0) as f64,
                (info.y + info.height_collapsed) as f64,
            ),
            NodeFace::Left => (info.x as f64, (info.y + info.height_collapsed / 2.0) as f64),
            NodeFace::Right => (
                (info.x + info.width) as f64,
                (info.y + info.height_collapsed / 2.0) as f64,
            ),
        }
    }

    /// Euclidean distance between two 2D points.
    fn distance(a: (f64, f64), b: (f64, f64)) -> f64 {
        let dx = a.0 - b.0;
        let dy = a.1 - b.1;
        (dx * dx + dy * dy).sqrt()
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
        let Some(info) = svg_node_info_map.get(node_id) else {
            return 100.0; // fallback
        };
        match face {
            NodeFace::Top | NodeFace::Bottom => info.width,
            NodeFace::Left | NodeFace::Right => info.height_collapsed,
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

/// A single contact entry for the per-face sorting phase.
///
/// Stored in `face_offsets_compute` to track which edge touches a given
/// (node, face) and the path midpoint needed for curvature-center
/// sorting.
struct FaceContactEntry {
    /// Mean anchor point of the edge's zero-offset path.
    midpoint: (f64, f64),
    /// Index into `all_pass1_groups`.
    group_idx: usize,
    /// Index into the group's `pass1_infos`.
    edge_idx: usize,
    /// `true` if this contact is at the "from" endpoint, `false` for "to".
    is_from: bool,
}

/// Intermediate per-edge data collected in pass 1 and consumed in pass 2.
struct EdgePass1Info<'edge, 'id> {
    edge_index: usize,
    edge: &'edge Edge<'id>,
    edge_id: EdgeId<'id>,
    edge_type: EdgeType,
    /// `None` when the edge connects a contained node (no face offset
    /// applies).
    from_face: Option<NodeFace>,
    to_face: Option<NodeFace>,
    /// Mean anchor point of the zero-offset path, used to determine
    /// curvature-center distance during the sort phase.
    path_midpoint: (f64, f64),
}

/// All pass-1 data for a single edge group.
struct EdgeGroupPass1<'edge, 'id> {
    edge_group_id: &'edge EdgeGroupId<'id>,
    edge_animation_params: EdgeAnimationParams,
    pass1_infos: Vec<EdgePass1Info<'edge, 'id>>,
    /// Per-edge assigned slot index for the "from" face contact.
    from_slot: Vec<Option<usize>>,
    /// Per-edge assigned slot index for the "to" face contact.
    to_slot: Vec<Option<usize>>,
}

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
