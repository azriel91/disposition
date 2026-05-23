use std::collections::BTreeMap;

use disposition_ir_model::{
    edge::{Edge, EdgeGroups, EdgeId},
    entity::{EntityType, EntityTypes},
    node::{NodeId, NodeNestingInfos, NodeRank, NodeRanksNested},
};
use disposition_model_common::{entity::EntityDescs, Map};
use disposition_taffy_model::{
    taffy::{self, AlignSelf, Style, TaffyTree},
    EdgeDescriptionCtx, EdgeDescriptionTaffyNodes, TaffyNodeCtx,
};

use crate::EdgeIdGenerator;

use super::edge_spacer_builder::LcaDepthCalculator;

/// Builds `edge_description_container` and `edge_description` leaf taffy nodes
/// for all described edges at a single LCA level.
///
/// Each described edge at the target LCA level gets:
///
/// 1. A leaf node with `TaffyNodeCtx::EdgeDescription` whose size is measured
///    from the description text (placeholder zero size until Phase 3).
/// 2. A container node (styled like a rank container) that wraps the leaf and
///    is interleaved between existing rank containers.
pub(crate) struct EdgeDescriptionBuilder;

impl EdgeDescriptionBuilder {
    /// Builds `edge_description_container` and leaf taffy nodes for all
    /// described edges at a single LCA level.
    ///
    /// Returns the new taffy node IDs and the positions at which each
    /// `edge_description_container` should be interleaved with rank containers.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn build(
        taffy_tree: &mut TaffyTree<TaffyNodeCtx>,
        entity_descs: &EntityDescs<'static>,
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

        // Inner BTreeMap key: `(sibling_middle, edge_id_str)` -- sorted so
        // containers at the same position are ordered by sibling proximity
        // and then by edge ID as a tiebreaker.
        let mut position_to_sorted_containers: BTreeMap<
            Option<NodeRank>,
            BTreeMap<(usize, String), taffy::NodeId>,
        > = BTreeMap::new();

        edge_groups.iter().for_each(|(edge_group_id, edge_group)| {
            edge_group
                .iter()
                .enumerate()
                .for_each(|(edge_index, edge)| {
                    let edge_id = EdgeIdGenerator::generate(edge_group_id, edge_index);

                    if let Some((position, sort_key, edge_description_nodes)) =
                        Self::edge_desc_build(
                            taffy_tree,
                            entity_descs,
                            &edge_id,
                            edge,
                            node_nesting_infos,
                            node_ranks_nested,
                            entity_types,
                            target_entity_type,
                            lca_node_id,
                            rank_container_style,
                        )
                    {
                        position_to_sorted_containers
                            .entry(position)
                            .or_default()
                            .insert(sort_key, edge_description_nodes.container_taffy_node_id);
                        edge_description_taffy_nodes.insert(edge_id, edge_description_nodes);
                    }
                });
        });

        // Flatten each inner BTreeMap to a Vec, preserving sort order.
        let position_to_container_ids = position_to_sorted_containers
            .into_iter()
            .map(|(position, sorted)| (position, sorted.into_values().collect()))
            .collect();

        EdgeDescriptionBuildResult {
            edge_description_taffy_nodes,
            position_to_container_ids,
        }
    }

    /// Builds the two taffy nodes for a single edge description, if applicable.
    ///
    /// Applies the following filters in order:
    ///
    /// 1. The edge must have a description in `entity_descs`.
    /// 2. Both endpoints must have `NodeNestingInfo` entries.
    /// 3. Neither endpoint may be an ancestor of the other (divergent ancestors
    ///    must exist at `lca_depth`).
    /// 4. Both divergent ancestors must match `target_entity_type`.
    /// 5. The edge's LCA must match the `lca_node_id` filter.
    ///
    /// On success returns `(position, sort_key, EdgeDescriptionTaffyNodes)`
    /// where:
    /// - `position` -- `None` = before all rank containers; `Some(rank)` =
    ///   after rank_container[rank].
    /// - `sort_key` -- `(sibling_middle, edge_id_str)` for deterministic
    ///   ordering at the same position.
    #[allow(clippy::too_many_arguments)]
    fn edge_desc_build(
        taffy_tree: &mut TaffyTree<TaffyNodeCtx>,
        entity_descs: &EntityDescs<'static>,
        edge_id: &EdgeId<'static>,
        edge: &Edge<'static>,
        node_nesting_infos: &NodeNestingInfos<'static>,
        node_ranks_nested: &NodeRanksNested<'static>,
        entity_types: &EntityTypes<'static>,
        target_entity_type: &EntityType,
        lca_node_id: Option<&NodeId<'static>>,
        rank_container_style: &Style,
    ) -> Option<(Option<NodeRank>, (usize, String), EdgeDescriptionTaffyNodes)> {
        // Step 2.2.1 -- Filter by entity_descs.
        entity_descs.get(edge_id.as_ref())?;

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
        let sibling_middle = (sibling_index_from + sibling_index_to) / 2;
        let sort_key = (sibling_middle, edge_id.as_str().to_string());

        // Step 2.2.8 -- Create the leaf and container taffy nodes.
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

        let container_taffy_node_id = taffy_tree
            .new_with_children(rank_container_style.clone(), &[description_taffy_node_id])
            .expect("Expected to create edge_description_container node.");

        Some((
            position,
            sort_key,
            EdgeDescriptionTaffyNodes {
                container_taffy_node_id,
                description_taffy_node_id,
            },
        ))
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
