use disposition_ir_model::{
    edge::{EdgeGroup, EdgeGroups, EdgeId},
    entity::EntityTypes,
    node::NodeId,
};
use disposition_model_common::{entity::EntityType, theme::Css, Id, Map};
use disposition_svg_model::{SvgEdgeInfo, SvgNodeInfo};
use kurbo::Shape;

use disposition_ir_model::entity::EntityTailwindClasses;
use disposition_model_common::edge::EdgeGroupId;

use super::{
    edge_model::{EdgeAnimationParams, EdgePathInfo, EdgeType},
    EdgeAnimationCalculator, EdgePathBuilder,
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
    pub(super) fn build<'id>(
        edge_groups: &EdgeGroups<'id>,
        entity_types: &EntityTypes<'id>,
        svg_node_info_map: &Map<&NodeId<'id>, &SvgNodeInfo<'id>>,
        tailwind_classes: &mut EntityTailwindClasses<'id>,
        css: &mut Css,
    ) -> Vec<SvgEdgeInfo<'id>> {
        let mut svg_edge_infos = Vec::new();

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

        /// 1 second per 100 pixels
        const SECONDS_PER_PIXEL: f64 = 0.8 / 100.0;

        edge_groups.iter().for_each(|(edge_group_id, edge_group)| {
            let edge_animation_params = EdgeAnimationParams::default();
            let visible_segments_length = edge_animation_params.visible_segments_length;

            let edge_path_infos = Self::build_edge_path_infos(
                edge_group_id,
                edge_group,
                entity_types,
                svg_node_info_map,
                edge_animation_params,
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
                    Self::css_animation_append(
                        tailwind_classes,
                        css,
                        edge_animation_params,
                        edge_group_path_or_visible_segments_length_max,
                        edge_group_animation_duration_total_s,
                        &edge_path_info,
                    );
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

                svg_edge_infos.push(SvgEdgeInfo::new(
                    edge_id,
                    edge_group_id.clone(),
                    edge.from.clone(),
                    edge.to.clone(),
                    path_d,
                ));
            });
        });

        svg_edge_infos
    }

    /// Returns the `BezPath`s for each edge in the given edge group.
    fn build_edge_path_infos<'edge, 'id>(
        edge_group_id: &'edge EdgeGroupId<'id>,
        edge_group: &'edge EdgeGroup<'id>,
        entity_types: &'edge EntityTypes<'id>,
        svg_node_info_map: &'edge Map<&NodeId<'id>, &SvgNodeInfo<'id>>,
        edge_animation_params: EdgeAnimationParams,
    ) -> Vec<EdgePathInfo<'edge, 'id>> {
        let visible_segments_length = edge_animation_params.visible_segments_length;
        edge_group
            .iter()
            .enumerate()
            .filter_map(|(edge_index, edge)| {
                // Skip edges where either node is not found
                let Some(from_info) = svg_node_info_map.get(&edge.from) else {
                    // TODO: warn user that they probably got a Node ID wrong.
                    return None;
                };
                let Some(to_info) = svg_node_info_map.get(&edge.to) else {
                    // TODO: warn user that they probably got a Node ID wrong.
                    return None;
                };

                let edge_id = Self::generate_edge_id(edge_group_id, edge_index);

                let edge_type = entity_types
                    .get(&*edge_id)
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
                    .unwrap_or(EdgeType::Unpaired);

                let path = EdgePathBuilder::build(from_info, to_info, edge_type);
                let path_length = {
                    // not sure what this is, but I assume it means 1 pixel accuracy
                    let accuracy = 1.0;
                    path.perimeter(accuracy)
                };

                let edge_path_info = EdgePathInfo {
                    edge_id,
                    edge,
                    edge_type,
                    path,
                    path_length,
                    preceding_visible_segments_lengths: edge_index as f64 * visible_segments_length,
                };

                Some(edge_path_info)
            })
            .collect::<Vec<EdgePathInfo>>()
    }

    fn css_animation_append<'edge, 'id>(
        tailwind_classes: &mut EntityTailwindClasses<'id>,
        css: &mut Css,
        edge_animation_params: EdgeAnimationParams,
        edge_group_path_or_visible_segments_length_max: f64,
        edge_group_animation_duration_total_s: f64,
        edge_path_info: &EdgePathInfo<'edge, 'id>,
    ) {
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
        let animation_classes = format!(
            "[stroke-dasharray:{dasharray}]\nanimate-[{animation_name}_{animation_duration}s_linear_infinite]"
        );
        let combined = if existing.is_empty() {
            animation_classes
        } else {
            format!("{existing}\n{animation_classes}")
        };
        tailwind_classes.insert(edge_id_owned, combined);

        // Append CSS keyframes.
        if !css.is_empty() {
            css.push('\n');
        }
        css.push_str(&edge_anim.keyframe_css);
    }

    /// Generates an edge ID from the edge group ID and edge index.
    fn generate_edge_id(edge_group_id: &EdgeGroupId<'_>, edge_index: usize) -> EdgeId<'static> {
        let edge_id_str = format!("{edge_group_id}__{edge_index}");
        Id::try_from(edge_id_str)
            .expect("edge ID should be valid")
            .into()
    }
}
