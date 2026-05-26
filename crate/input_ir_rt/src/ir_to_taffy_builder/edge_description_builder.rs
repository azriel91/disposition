use std::collections::BTreeMap;

use disposition_ir_model::{
    edge::{Edge, EdgeGroups, EdgeId},
    entity::{EntityType, EntityTypes},
    node::{NodeId, NodeNestingInfos, NodeRank, NodeRanksNested},
};
use disposition_model_common::{edge::EdgeDescs, Map};
use disposition_taffy_model::{
    taffy::{self, AlignSelf, Style, TaffyTree},
    EdgeDescriptionCtx, EdgeDescriptionTaffyNodes, TaffyNodeCtx,
};

use crate::EdgeIdGenerator;

use super::edge_spacer_builder::LcaDepthCalculator;

use self::{
    edge_id_and_taffy_description_node::EdgeIdAndTaffyDescriptionNode,
    sibling_index_middle_and_edge_id::SiblingIndexMiddleAndEdgeId,
};

mod edge_id_and_taffy_description_node;
mod sibling_index_middle_and_edge_id;

/// Builds `edge_description_container` and `edge_description` leaf taffy nodes
/// for all described edges at a single LCA level.
///
/// Each described edge at the target LCA level gets:
///
/// 1. A leaf node with `TaffyNodeCtx::EdgeDescription` whose size is measured
///    from the description text (placeholder zero size until Phase 3).
/// 2. A shared container node (styled like a rank container) that wraps all
///    leaf nodes at the same insertion position and is interleaved between
///    existing rank containers.
pub(crate) struct EdgeDescriptionBuilder;

impl EdgeDescriptionBuilder {
    /// Builds `edge_description_container` and leaf taffy nodes for all
    /// described edges at a single LCA level.
    ///
    /// Returns the new taffy node IDs and the positions at which each
    /// `edge_description_container` should be interleaved with rank containers.
    ///
    /// One container node is created per insertion position, holding all
    /// description leaf nodes for edges at that position. Positions with no
    /// described edges produce no container.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn build(
        taffy_tree: &mut TaffyTree<TaffyNodeCtx>,
        edge_descs: &EdgeDescs<'static>,
        edge_groups: &EdgeGroups<'static>,
        node_nesting_infos: &NodeNestingInfos<'static>,
        node_ranks_nested: &NodeRanksNested<'static>,
        entity_types: &EntityTypes<'static>,
        target_entity_type: &EntityType,
        lca_node_id: Option<&NodeId<'static>>,
        rank_container_style: &Style,
    ) -> EdgeDescriptionBuildResult {
        let mut edge_description_taffy_nodes: Map<EdgeId<'static>, EdgeDescriptionTaffyNodes> =
            Map::new();

        // Collect per-edge description leaf nodes grouped by insertion position.
        //
        // Inner BTreeMap key: `MiddleSiblingNodeIndexAndEdgeId` -- sorted so
        // descriptions at the same position are ordered by sibling proximity
        // and then by edge ID as a tiebreaker.
        //
        // Inner BTreeMap value: `EdgeIdAndTaffyDescriptionNode` so we can
        // build `EdgeDescriptionTaffyNodes` after the shared container is
        // created.
        let mut position_to_sorted_descriptions: BTreeMap<
            Option<NodeRank>,
            BTreeMap<SiblingIndexMiddleAndEdgeId, EdgeIdAndTaffyDescriptionNode>,
        > = BTreeMap::new();

        edge_groups.iter().for_each(|(edge_group_id, edge_group)| {
            edge_group
                .iter()
                .enumerate()
                .for_each(|(edge_index, edge)| {
                    let edge_id = EdgeIdGenerator::generate(edge_group_id, edge_index);

                    if let Some((position, sort_key, description_taffy_node_id)) =
                        Self::edge_desc_build(
                            taffy_tree,
                            edge_descs,
                            &edge_id,
                            edge,
                            node_nesting_infos,
                            node_ranks_nested,
                            entity_types,
                            target_entity_type,
                            lca_node_id,
                        )
                    {
                        position_to_sorted_descriptions
                            .entry(position)
                            .or_default()
                            .insert(
                                sort_key,
                                EdgeIdAndTaffyDescriptionNode {
                                    edge_id,
                                    description_taffy_node_id,
                                },
                            );
                    }
                });
        });

        // For each position create one shared container holding all description
        // leaf nodes at that position (in sort order). Then record each edge's
        // `EdgeDescriptionTaffyNodes` with the shared container.
        let position_to_container_ids = position_to_sorted_descriptions
            .into_iter()
            .map(|(position, sorted)| {
                let description_nodes: Vec<EdgeIdAndTaffyDescriptionNode> =
                    sorted.into_values().collect();
                let leaf_node_ids: Vec<taffy::NodeId> = description_nodes
                    .iter()
                    .map(|node| node.description_taffy_node_id)
                    .collect();

                let container_taffy_node_id = taffy_tree
                    .new_with_children(rank_container_style.clone(), &leaf_node_ids)
                    .expect("Expected to create edge_description_container node.");

                for EdgeIdAndTaffyDescriptionNode {
                    edge_id,
                    description_taffy_node_id,
                } in description_nodes
                {
                    edge_description_taffy_nodes.insert(
                        edge_id,
                        EdgeDescriptionTaffyNodes {
                            container_taffy_node_id,
                            description_taffy_node_id,
                        },
                    );
                }

                (position, vec![container_taffy_node_id])
            })
            .collect();

        EdgeDescriptionBuildResult {
            edge_description_taffy_nodes,
            position_to_container_ids,
        }
    }

    /// Builds the description leaf taffy node for a single edge, if applicable.
    ///
    /// Applies the following filters in order:
    ///
    /// 1. The edge must have a description in `edge_descs`.
    /// 2. Both endpoints must have `NodeNestingInfo` entries.
    /// 3. Neither endpoint may be an ancestor of the other (divergent ancestors
    ///    must exist at `lca_depth`).
    /// 4. Both divergent ancestors must match `target_entity_type`.
    /// 5. The edge's LCA must match the `lca_node_id` filter.
    ///
    /// On success returns `(position, sort_key, description_taffy_node_id)`
    /// where:
    /// - `position` -- `None` = before all rank containers; `Some(rank)` =
    ///   after rank_container[rank].
    /// - `sort_key` -- [`MiddleSiblingNodeIndexAndEdgeId`] for deterministic
    ///   ordering at the same position.
    /// - `description_taffy_node_id` -- the newly created leaf node.
    ///
    /// The shared container node is created later in `build` once all leaves
    /// at the same position have been collected.
    #[allow(clippy::too_many_arguments)]
    fn edge_desc_build(
        taffy_tree: &mut TaffyTree<TaffyNodeCtx>,
        edge_descs: &EdgeDescs<'static>,
        edge_id: &EdgeId<'static>,
        edge: &Edge<'static>,
        node_nesting_infos: &NodeNestingInfos<'static>,
        node_ranks_nested: &NodeRanksNested<'static>,
        entity_types: &EntityTypes<'static>,
        target_entity_type: &EntityType,
        lca_node_id: Option<&NodeId<'static>>,
    ) -> Option<(Option<NodeRank>, SiblingIndexMiddleAndEdgeId, taffy::NodeId)> {
        // Step 2.2.1 -- Filter by edge_descs.
        edge_descs.get(edge_id.as_ref())?;

        // Step 2.2.2 -- Resolve nesting infos.
        let info_from = node_nesting_infos.get(&edge.from)?;
        let info_to = node_nesting_infos.get(&edge.to)?;

        // Step 2.2.3 -- Compute LCA depth and divergent ancestors.
        let lca_depth = LcaDepthCalculator::calculate(info_from, info_to);
        let divergent_from = info_from.ancestor_chain.get(lca_depth)?;
        let divergent_to = info_to.ancestor_chain.get(lca_depth)?;

        // Step 2.2.4 -- Entity type filter.
        let from_matches = entity_types
            .get(divergent_from.as_ref())
            .map(|types| types.contains(target_entity_type))
            .unwrap_or(false);
        let to_matches = entity_types
            .get(divergent_to.as_ref())
            .map(|types| types.contains(target_entity_type))
            .unwrap_or(false);
        if !from_matches || !to_matches {
            return None;
        }

        // Step 2.2.5 -- LCA level filter.
        match lca_node_id {
            None => {
                // Top-level: only handle edges whose LCA is the diagram root.
                if lca_depth > 0 {
                    return None;
                }
            }
            Some(expected_lca_node_id) => {
                // Nested: only handle edges whose LCA is this container.
                if lca_depth == 0 {
                    return None;
                }
                let lca_ancestor = info_from.ancestor_chain.get(lca_depth - 1);
                match lca_ancestor {
                    Some(lca_ancestor) if lca_ancestor == expected_lca_node_id => {}
                    _ => return None,
                }
            }
        }

        // Step 2.2.6 -- Look up divergent ancestor ranks.
        let lca_container = lca_depth
            .checked_sub(1)
            .map(|i| &info_from.ancestor_chain[i]);
        let container_ranks = node_ranks_nested.ranks_for(lca_container)?;
        let rank_from = container_ranks
            .get(divergent_from)
            .copied()
            .unwrap_or(NodeRank::new(0));
        let rank_to = container_ranks
            .get(divergent_to)
            .copied()
            .unwrap_or(NodeRank::new(0));

        // Step 2.2.7 -- Compute insertion position.
        let position = if rank_from == rank_to {
            // Cycle edge: insert before the shared rank container.
            if rank_from.value() > 0 {
                Some(NodeRank::new(rank_from.value() - 1))
            } else {
                None
            }
        } else {
            let rank_low = rank_from.min(rank_to);
            let rank_high = rank_from.max(rank_to);
            Some(NodeRank::new(
                rank_low.value() + (rank_high.value() - rank_low.value()) / 2,
            ))
        };

        // Step 2.2.8 -- Compute sibling middle index (sort key).
        let sibling_index_from = info_from.nesting_path.get(lca_depth).copied().unwrap_or(0);
        let sibling_index_to = info_to.nesting_path.get(lca_depth).copied().unwrap_or(0);
        let sibling_index_middle = (sibling_index_from + sibling_index_to) / 2;
        let sort_key = SiblingIndexMiddleAndEdgeId {
            sibling_index_middle,
            edge_id: edge_id.as_str().to_string(),
        };

        // Step 2.2.9 -- Create the description leaf node.
        let description_style = Style {
            align_self: Some(AlignSelf::Stretch),
            ..Default::default()
        };

        let description_taffy_node_id = taffy_tree
            .new_leaf_with_context(
                description_style,
                TaffyNodeCtx::EdgeDescription(EdgeDescriptionCtx {
                    edge_id: edge_id.clone(),
                }),
            )
            .expect("Expected to create edge description leaf node.");

        Some((position, sort_key, description_taffy_node_id))
    }
}

/// Result returned by [`EdgeDescriptionBuilder::build`].
pub(crate) struct EdgeDescriptionBuildResult {
    /// Maps each described edge ID to its newly created taffy node IDs.
    pub(crate) edge_description_taffy_nodes: Map<EdgeId<'static>, EdgeDescriptionTaffyNodes>,

    /// Ordered map from insertion position to the `edge_description_container`
    /// taffy node IDs to insert there.
    ///
    /// Key `None` means before all rank containers; `Some(rank)` means after
    /// `rank_container[rank]`.
    pub(crate) position_to_container_ids: BTreeMap<Option<NodeRank>, Vec<taffy::NodeId>>,
}
