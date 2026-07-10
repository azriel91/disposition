use disposition_ir_model::{
    edge::{EdgeGroups, EdgeId, EdgeRouteReversals},
    entity::{EntityType, EntityTypes},
    node::{NodeNestingInfos, NodeRanksNested},
};
use disposition_model_common::{
    edge::{EdgeCurvature, EdgeGroupId, EdgeLabel, EdgeLabels},
    Id, RenderOptions,
};

use crate::{
    divergent_ancestor_ranks_calculator::DivergentAncestorRanksCalculator, EdgeIdGenerator,
};

/// Normalizes edge routing direction for cleaner paths.
///
/// An edge whose effective [`EdgeCurvature`] is `Curved`, and whose `from`
/// node's divergent ancestor rank at the LCA level is strictly greater than
/// its `to` node's, is routed through far more spacer waypoints than the same
/// edge in the opposite direction -- the routing algorithm's spacer placement
/// and protrusion bands are tuned for ascending-rank travel. Such an edge's
/// path is much cleaner when computed as though its endpoints were swapped.
///
/// This normalizer swaps `from`/`to` in the stored [`EdgeGroups`] entry (and
/// the edge's [`EdgeLabels`] entry, so each label stays on its real node) for
/// every qualifying edge, and records the edge's ID in the returned
/// [`EdgeRouteReversals`]. Every downstream stage (spacer construction, face
/// assignment, offsets, protrusions, path building) then computes the mirror
/// geometry; at SVG emission the path is reversed so the drawn path still
/// runs from the real `from` node to the real `to` node, with the arrow head
/// on the real `to` node.
///
/// Edges are left untouched when any of the following hold:
///
/// * The effective curvature is not [`EdgeCurvature::Curved`] or
///   [`EdgeCurvature::Orthogonal`] -- `Direct*` edges bypass spacers entirely.
/// * The edge is a self-loop.
/// * One endpoint is an ancestor of the other (no divergent ancestors).
/// * The divergent ancestor ranks are equal (same-rank / cycle edges).
pub(crate) struct EdgeRouteNormalizer;

impl EdgeRouteNormalizer {
    /// Reverses the stored direction of descending-rank `Curved` edges.
    ///
    /// Returns the IDs of the edges that were reversed.
    pub(crate) fn normalize<'id>(
        edge_groups: &mut EdgeGroups<'id>,
        edge_labels: &mut EdgeLabels<'id>,
        entity_types: &EntityTypes<'id>,
        node_nesting_infos: &NodeNestingInfos<'id>,
        node_ranks_nested: &NodeRanksNested<'id>,
        render_options: &RenderOptions,
    ) -> EdgeRouteReversals<'id> {
        let mut edge_route_reversals = EdgeRouteReversals::new();

        edge_groups
            .iter_mut()
            .for_each(|(edge_group_id, edge_group)| {
                edge_group
                    .iter_mut()
                    .enumerate()
                    .for_each(|(edge_index, edge)| {
                        let edge_id = EdgeIdGenerator::generate(edge_group_id, edge_index);

                        let edge_curvature =
                            Self::edge_curvature_effective(entity_types, render_options, &edge_id);
                        match edge_curvature {
                            EdgeCurvature::Curved | EdgeCurvature::Orthogonal => {}
                            EdgeCurvature::DirectStraight | EdgeCurvature::DirectCurved => return,
                        }
                        if edge.is_self_loop() {
                            return;
                        }

                        let Some(info_from) = node_nesting_infos.get(&edge.from) else {
                            return;
                        };
                        let Some(info_to) = node_nesting_infos.get(&edge.to) else {
                            return;
                        };
                        let Some((rank_from, rank_to)) =
                            DivergentAncestorRanksCalculator::divergent_ancestor_ranks_from_to(
                                info_from,
                                info_to,
                                node_ranks_nested,
                            )
                        else {
                            return;
                        };

                        if rank_from > rank_to {
                            *edge = edge.reversed();
                            Self::edge_label_swap(edge_labels, edge_group_id, &edge_id);
                            edge_route_reversals.insert(edge_id);
                        }
                    });
            });

        edge_route_reversals
    }

    /// Returns the effective curvature for an edge.
    ///
    /// Interaction edges use [`RenderOptions::interactions_edge_curvature`];
    /// all other edges use [`RenderOptions::dependencies_edge_curvature`].
    fn edge_curvature_effective<'id>(
        entity_types: &EntityTypes<'id>,
        render_options: &RenderOptions,
        edge_id: &EdgeId<'id>,
    ) -> EdgeCurvature {
        let is_interaction_edge = entity_types
            .get(AsRef::<Id<'_>>::as_ref(edge_id))
            .map(|edge_entity_types| {
                edge_entity_types
                    .iter()
                    .any(EntityType::is_interaction_edge)
            })
            .unwrap_or(false);

        if is_interaction_edge {
            render_options.interactions_edge_curvature
        } else {
            render_options.dependencies_edge_curvature
        }
    }

    /// Swaps the `from`/`to` labels of a reversed edge.
    ///
    /// When only a group-level label entry exists, a swapped edge-specific
    /// entry is materialized so the group entry stays valid for the group's
    /// non-reversed edges.
    fn edge_label_swap<'id>(
        edge_labels: &mut EdgeLabels<'id>,
        edge_group_id: &EdgeGroupId<'id>,
        edge_id: &EdgeId<'id>,
    ) {
        let edge_label_swapped =
            edge_labels
                .get_for_edge(edge_id, edge_group_id)
                .map(|edge_label| EdgeLabel {
                    from: edge_label.to.clone(),
                    to: edge_label.from.clone(),
                });

        if let Some(edge_label_swapped) = edge_label_swapped {
            edge_labels.insert(edge_id.clone(), edge_label_swapped);
        }
    }
}
