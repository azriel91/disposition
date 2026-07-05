use std::{cmp::Ordering, collections::BTreeMap};

use disposition_ir_model::{
    edge::{Edge, EdgeId},
    entity::EntityType,
    node::{NodeId, NodeRank},
};
use disposition_model_common::{edge::EdgeGroupId, Map, RankDir};
use disposition_taffy_model::{
    taffy::{self, style::FlexDirection, AlignSelf, Rect, Style, TaffyTree},
    DiagramLod, EdgeDescriptionCtx, EdgeDescriptionTaffyNodes, TaffyNodeCtx, TEXT_FONT_SIZE,
};
use taffy::LengthPercentageAuto;

use crate::{taffy_to_svg_elements_mapper::EdgeSpacerCoordinatesCalculator, EdgeIdGenerator};

use crate::md_text::md_blocks_parser::MdBlocksParser;

use super::{
    edge_spacer_builder::LcaDepthCalculator, md_node_builder::MdNodeBuilder,
    rank_sibling_inserter::RankSiblingInserter, taffy_build_ctx::TaffyBuildCtx,
    taffy_container_builder::flex_direction_invert,
};

use self::{
    edge_desc_position::EdgeDescPosition,
    edge_id_and_taffy_description_node::EdgeIdAndTaffyDescriptionNode,
    rank_and_sibling_index_middle::RankAndSiblingIndexMiddle,
    sibling_index_middle_and_edge_id::SiblingIndexMiddleAndEdgeId,
};

mod edge_desc_position;
mod edge_id_and_taffy_description_node;
mod rank_and_sibling_index_middle;
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
    ///
    /// At `DiagramLod::Normal`, the single description leaf is replaced by an
    /// `md_content_node` sub-tree built via `MdNodeBuilder`.
    ///
    /// Edges whose divergent ancestors share a rank (cycle edges) do not go
    /// through the position-based interleaving above -- their container is
    /// inserted directly into `rank_to_taffy_ids[rank]`, between the two
    /// divergent ancestors' sibling subtrees, mirroring how
    /// `EdgeSpacerBuilder` places same-level cross-rank spacers. This
    /// requires mutable access to the same `rank_to_taffy_ids` map the caller
    /// passes to `EdgeSpacerBuilder::build`.
    pub(crate) fn build(
        ctx: TaffyBuildCtx<'_>,
        taffy_tree: &mut TaffyTree<TaffyNodeCtx>,
        target_entity_type: &EntityType,
        lca_node_id: Option<&NodeId<'static>>,
        rank_container_style: &Style,
        rank_to_taffy_ids: &mut BTreeMap<NodeRank, Vec<taffy::NodeId>>,
    ) -> EdgeDescriptionBuildResult {
        let edge_groups = ctx.edge_groups;

        let mut edge_description_taffy_nodes: Map<EdgeId<'static>, EdgeDescriptionTaffyNodes> =
            Map::new();

        // Collect per-edge description leaf nodes grouped by insertion position.
        //
        // Inner BTreeMap key: `SiblingIndexMiddleAndEdgeId` -- sorted so
        // descriptions at the same position are ordered by sibling proximity
        // and then by edge ID as a tiebreaker.
        //
        // Inner BTreeMap value: `EdgeIdAndTaffyDescriptionNode` so we can
        // build `EdgeDescriptionTaffyNodes` after the shared container is
        // created.
        //
        // Edges whose divergent ancestors sit at different ranks
        // (`EdgeDescPosition::BetweenRanks`) are grouped here. Edges whose
        // divergent ancestors share a rank (`EdgeDescPosition::SameRank`,
        // cycle edges) are grouped in `same_rank_to_sorted_descriptions`
        // instead, since their container is inserted at a sibling index
        // within a rank rather than interleaved between ranks.
        let mut position_to_sorted_descriptions: BTreeMap<
            Option<NodeRank>,
            BTreeMap<SiblingIndexMiddleAndEdgeId, EdgeIdAndTaffyDescriptionNode>,
        > = BTreeMap::new();
        let mut same_rank_to_sorted_descriptions: BTreeMap<
            RankAndSiblingIndexMiddle,
            BTreeMap<SiblingIndexMiddleAndEdgeId, EdgeIdAndTaffyDescriptionNode>,
        > = BTreeMap::new();

        edge_groups.iter().for_each(|(edge_group_id, edge_group)| {
            edge_group
                .iter()
                .enumerate()
                .for_each(|(edge_index, edge)| {
                    let edge_id = EdgeIdGenerator::generate(edge_group_id, edge_index);

                    if let Some((
                        position,
                        sort_key,
                        description_taffy_node_id,
                        md_node_taffy_ids,
                        sibling_index_from_cmp_to,
                    )) = Self::edge_desc_build(
                        ctx,
                        taffy_tree,
                        edge_group_id,
                        &edge_id,
                        edge,
                        target_entity_type,
                        lca_node_id,
                    ) {
                        let description_node = EdgeIdAndTaffyDescriptionNode {
                            edge_id,
                            description_taffy_node_id,
                            md_node_taffy_ids,
                            sibling_index_from_cmp_to,
                        };
                        match position {
                            EdgeDescPosition::BetweenRanks(rank) => {
                                position_to_sorted_descriptions
                                    .entry(Some(rank))
                                    .or_default()
                                    .insert(sort_key, description_node);
                            }
                            EdgeDescPosition::SameRank(rank_and_sibling_index_middle) => {
                                same_rank_to_sorted_descriptions
                                    .entry(rank_and_sibling_index_middle)
                                    .or_default()
                                    .insert(sort_key, description_node);
                            }
                        }
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
                let container_taffy_node_id = Self::container_from_description_nodes_build(
                    taffy_tree,
                    rank_container_style,
                    ctx.char_width,
                    description_nodes,
                    true, // BetweenRanks: cross-rank edges.
                    &mut edge_description_taffy_nodes,
                );

                (position, vec![container_taffy_node_id])
            })
            .collect();

        // For each same-rank group, create one shared container and insert it
        // directly into the shared rank's sibling list, between the two
        // divergent ancestors -- mirroring how `EdgeSpacerBuilder` inserts
        // same-level cross-rank spacers. Grouping (and inserting) in
        // ascending `(rank, sibling_index_middle)` order, guaranteed by
        // `BTreeMap` iteration, keeps `same_rank_insertion_counts` accounting
        // for earlier insertions correctly.
        let mut same_rank_insertion_counts: BTreeMap<NodeRank, Vec<usize>> = BTreeMap::new();
        same_rank_to_sorted_descriptions.into_iter().for_each(
            |(
                RankAndSiblingIndexMiddle {
                    rank,
                    sibling_index_middle,
                },
                sorted,
            )| {
                let description_nodes: Vec<EdgeIdAndTaffyDescriptionNode> =
                    sorted.into_values().collect();
                let container_taffy_node_id = Self::container_from_description_nodes_build(
                    taffy_tree,
                    rank_container_style,
                    ctx.char_width,
                    description_nodes,
                    false, // SameRank: same-rank/cycle edges.
                    &mut edge_description_taffy_nodes,
                );

                let insertion_base_index = sibling_index_middle + 1;
                RankSiblingInserter::node_insert(
                    rank_to_taffy_ids,
                    &mut same_rank_insertion_counts,
                    rank,
                    insertion_base_index,
                    container_taffy_node_id,
                );
            },
        );

        EdgeDescriptionBuildResult {
            edge_description_taffy_nodes,
            position_to_container_ids,
        }
    }

    /// Builds one shared `edge_description_container` from a group of
    /// same-position description nodes, recording each edge's
    /// `EdgeDescriptionTaffyNodes` against the shared container.
    ///
    /// `is_cross_rank` is uniform across every node in `description_nodes`:
    /// callers pass all-`BetweenRanks` groups (from
    /// `position_to_sorted_descriptions`) with `true`, and all-`SameRank`
    /// groups (from `same_rank_to_sorted_descriptions`) with `false` -- the
    /// two source maps never mix `EdgeDescPosition` variants, so a single
    /// bool per call is sufficient. Stored on `EdgeDescriptionTaffyNodes` so
    /// `SpacerCoordinatesResolver::description_contact_resolve` can pick the
    /// correct routing waypoint calculation per edge.
    fn container_from_description_nodes_build(
        taffy_tree: &mut TaffyTree<TaffyNodeCtx>,
        rank_container_style: &Style,
        char_width: f32,
        description_nodes: Vec<EdgeIdAndTaffyDescriptionNode>,
        is_cross_rank: bool,
        edge_description_taffy_nodes: &mut Map<EdgeId<'static>, EdgeDescriptionTaffyNodes>,
    ) -> taffy::NodeId {
        let leaf_node_ids: Vec<taffy::NodeId> = description_nodes
            .iter()
            .map(|node| node.description_taffy_node_id)
            .collect();

        let container_style =
            Self::container_style_build(rank_container_style, char_width, is_cross_rank);

        let container_taffy_node_id = taffy_tree
            .new_with_children(container_style, &leaf_node_ids)
            .expect("Expected to create edge_description_container node.");

        for EdgeIdAndTaffyDescriptionNode {
            edge_id,
            description_taffy_node_id,
            md_node_taffy_ids,
            sibling_index_from_cmp_to,
        } in description_nodes
        {
            edge_description_taffy_nodes.insert(
                edge_id,
                EdgeDescriptionTaffyNodes {
                    container_taffy_node_id,
                    description_taffy_node_id,
                    md_node_taffy_ids,
                    sibling_index_from_cmp_to,
                    is_cross_rank,
                },
            );
        }

        container_taffy_node_id
    }

    /// Builds the `edge_description_container`'s style from the rank
    /// container style, overriding the gap on whichever axis is the
    /// container's actual stacking axis.
    ///
    /// For a **cross-rank** (`BetweenRanks`) container, this mirrors
    /// `rank_container_style` (same `flex_direction` as ordinary rank
    /// containers): the container sits as a sibling of rank containers
    /// (stacked along the rank axis by its parent), so multiple described
    /// edges sharing that position lay out their own descriptions along the
    /// same axis rank siblings use, keeping the gap compact along the
    /// cross-rank axis instead.
    ///
    /// For a **same-rank** (`SameRank`, cycle edge) container, the roles are
    /// reversed: it is inserted *as a rank sibling itself*, directly between
    /// its two divergent ancestors, which sit side by side along
    /// `rank_container_style.flex_direction`. Mirroring that direction would
    /// stack multiple described edges' boxes along the same axis the two
    /// divergent ancestors are laid out on, widening/heightening the gap
    /// between them per extra description. Since the divergent ancestors'
    /// own edges run *along* that axis, the descriptions should instead
    /// stack along the perpendicular (cross) axis -- so `flex_direction` is
    /// inverted (`taffy_container_builder::flex_direction_invert`) before
    /// the gap axis is chosen.
    ///
    /// After the axis is chosen (inverted or not), any `Reverse` variant is
    /// stripped (`RowReverse` -> `Row`, `ColumnReverse` -> `Column`): the
    /// container's children are already inserted in the correct visual order
    /// (ascending `sibling_index_middle`/`EdgeId`, see [`Self::build`]), so a
    /// reversed direction would render them back to front, crossing over
    /// each other. Ordinary rank containers need `Reverse` (their sibling
    /// insertion order is separately corrected for it, see
    /// `edge_paths.md` -- Sibling order for reversed rank directions), but an
    /// `edge_description_container` has no such correction and does not need
    /// one, since its own children are always freshly sorted here.
    ///
    /// Either way, the gap component matching the *actual* (possibly
    /// inverted, always non-reversed) stacking axis is overridden:
    /// `Row` stacks children horizontally, so the gap lives on `gap.width`;
    /// `Column` stacks children vertically, so the gap lives on
    /// `gap.height`. Using the wrong axis leaves the *other* axis at the
    /// full rank gap, pushing the edge path far from the description text.
    ///
    /// # Example values
    ///
    /// `rank_container_style.flex_direction = Column` (rank_dir:
    /// left_to_right), `char_width = 8.0`, `is_cross_rank = true` --
    /// overrides `gap.height` to `LengthPercentage::length(4.0)`, leaving
    /// `gap.width` unchanged. With `is_cross_rank = false`, `flex_direction`
    /// is inverted to `Row` first, so `gap.width` is overridden instead.
    ///
    /// `rank_container_style.flex_direction = ColumnReverse` (rank_dir:
    /// right_to_left), `is_cross_rank = true` -- `Reverse` is stripped to
    /// `Column`, so `gap.height` is overridden (not `RowReverse`/`gap.width`
    /// as inverting alone, without the strip, would give for the same-rank
    /// case).
    fn container_style_build(
        rank_container_style: &Style,
        char_width: f32,
        is_cross_rank: bool,
    ) -> Style {
        let mut container_style = rank_container_style.clone();
        if !is_cross_rank {
            container_style.flex_direction = flex_direction_invert(container_style.flex_direction);
        }
        // Remove `Reverse` from `flex_direction` to avoid crossing edges.
        container_style.flex_direction = match container_style.flex_direction {
            FlexDirection::RowReverse => FlexDirection::Row,
            FlexDirection::ColumnReverse => FlexDirection::Column,
            _ => container_style.flex_direction,
        };

        let gap_value = taffy::LengthPercentage::length(char_width / 2.0);
        match container_style.flex_direction {
            FlexDirection::Row | FlexDirection::RowReverse => {
                container_style.gap.width = gap_value;
            }
            FlexDirection::Column | FlexDirection::ColumnReverse => {
                container_style.gap.height = gap_value;
            }
        }
        container_style
    }

    /// Builds the description leaf or markdown sub-tree taffy nodes for a
    /// single edge, if applicable.
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
    /// On success returns `(position, sort_key, description_taffy_node_id,
    /// md_node_taffy_ids, sibling_index_from_cmp_to)` where:
    /// - `position` -- an [`EdgeDescPosition`]: `BetweenRanks(rank)` when the
    ///   divergent ancestors sit at different ranks, or `SameRank(..)` when
    ///   they share a rank (cycle edge).
    /// - `sort_key` -- [`SiblingIndexMiddleAndEdgeId`] for deterministic
    ///   ordering at the same position.
    /// - `description_taffy_node_id` -- the newly created leaf node (simple
    ///   path) or `md_content_node` container (markdown path).
    /// - `md_node_taffy_ids` -- populated at `DiagramLod::Normal` with the
    ///   markdown sub-tree IDs.
    /// - `sibling_index_from_cmp_to` --
    ///   `sibling_index_from.cmp(&sibling_index_to)`, carried through to
    ///   [`disposition_taffy_model::EdgeDescriptionTaffyNodes`] for the routing
    ///   waypoint calculation (see
    ///   `EdgeSpacerCoordinatesCalculator::calculate_description_thread`/
    ///   `calculate_description_thread_same_rank`).
    ///
    /// The shared container node is created later in `build` once all leaves
    /// at the same position have been collected.
    fn edge_desc_build(
        ctx: TaffyBuildCtx<'_>,
        taffy_tree: &mut TaffyTree<TaffyNodeCtx>,
        edge_group_id: &EdgeGroupId<'static>,
        edge_id: &EdgeId<'static>,
        edge: &Edge<'static>,
        target_entity_type: &EntityType,
        lca_node_id: Option<&NodeId<'static>>,
    ) -> Option<(
        EdgeDescPosition,
        SiblingIndexMiddleAndEdgeId,
        taffy::NodeId,
        Option<disposition_taffy_model::MdNodeTaffyIds>,
        Ordering,
    )> {
        let edge_descs = ctx.edge_descs;
        let node_nesting_infos = ctx.node_nesting_infos;
        let node_ranks_nested = ctx.node_ranks_nested;
        let entity_types = ctx.entity_types;
        let lod = &ctx.lod;
        let char_width = ctx.char_width;

        // Step 2.2.1 -- Filter by edge_descs (instance ID takes precedence
        // over the edge's group ID).
        let desc_text = edge_descs.get_for_edge(edge_id, edge_group_id)?;

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

        // Step 2.2.7 -- Compute sibling middle index (sort key). Needed by
        // both branches of Step 2.2.8, since the same-rank branch uses it as
        // the sibling insertion position rather than just a sort key.
        let sibling_index_from = info_from.nesting_path.get(lca_depth).copied().unwrap_or(0);
        let sibling_index_to = info_to.nesting_path.get(lca_depth).copied().unwrap_or(0);
        let sibling_index_middle = (sibling_index_from + sibling_index_to) / 2;
        let sibling_index_from_cmp_to = sibling_index_from.cmp(&sibling_index_to);
        let sort_key = SiblingIndexMiddleAndEdgeId {
            sibling_index_middle,
            edge_id: edge_id.as_str().to_string(),
        };

        // Step 2.2.8 -- Compute insertion position.
        let position = if rank_from == rank_to {
            // Cycle edge: insert directly into the shared rank's sibling
            // list, between the two divergent ancestors (see
            // `EdgeDescriptionBuilder::build`'s same-rank handling).
            EdgeDescPosition::SameRank(RankAndSiblingIndexMiddle {
                rank: rank_from,
                sibling_index_middle,
            })
        } else {
            let rank_low = rank_from.min(rank_to);
            let rank_high = rank_from.max(rank_to);
            EdgeDescPosition::BetweenRanks(NodeRank::new(
                rank_low.value() + (rank_high.value() - rank_low.value()) / 2,
            ))
        };

        // Step 2.2.8b -- Compute halo-clearance margin.
        //
        // Mirrors `TaffyEnvelopeBuilder::build`'s edge label margins: the
        // description box gets `margin` (not `padding`) on whichever side the
        // routing path (`EdgeSpacerCoordinatesCalculator::
        // calculate_description_thread`/`calculate_description_thread_same_rank`)
        // runs flush against, so the wide interaction-edge halo doesn't
        // visually overlap the box's rendered content. The routing
        // calculation pulls the same axis back by the same amount (see
        // `EdgeSpacerCoordinatesCalculator::description_thread_from_rect`),
        // so the path stays pinned at the box's pre-margin position while the
        // box itself physically moves away by `halo_pad_px`.
        //
        // That axis is also the axis multiple described edges sharing the
        // same position are packed along (`container_style_build` mirrors --
        // or, for same-rank boxes, inverts -- `rank_container_style`'s
        // `flex_direction` onto the same axis the fixed-axis choice below
        // uses), so as with `TaffyEnvelopeBuilder::build`'s label margins, an
        // additional `label_margin_px` is added on the far side (opposite the
        // routing-path side) so each box reads as visually associated with
        // its own edge rather than crowding the next sibling box.
        //
        // Same-rank (cycle edge) boxes are threaded on the *rotated* axis
        // (see `EdgeSpacerCoordinatesCalculator::rank_dir_same_rank_rotate`),
        // so the margin side is chosen from the rotated direction to match --
        // reusing that same rotation fn (rather than re-deriving the mapping
        // here) so the two can't drift apart. The axis choice below mirrors
        // `description_thread_from_rect`'s fixed-axis selection: `left_x` for
        // `TopToBottom`/`BottomToTop`, `top_y` for `LeftToRight`/`RightToLeft`.
        let margin_rank_dir = match &position {
            EdgeDescPosition::BetweenRanks(_) => ctx.render_options.rank_dir,
            EdgeDescPosition::SameRank(_) => {
                EdgeSpacerCoordinatesCalculator::rank_dir_same_rank_rotate(
                    ctx.render_options.rank_dir,
                )
            }
        };
        let halo_pad_px = ctx.interaction_edge_halo_stroke_width / 2.0;
        let label_margin_px = TEXT_FONT_SIZE / 2.0;
        let description_margin = match margin_rank_dir {
            RankDir::TopToBottom | RankDir::BottomToTop => Rect {
                left: LengthPercentageAuto::length(halo_pad_px),
                right: LengthPercentageAuto::length(halo_pad_px + label_margin_px),
                top: LengthPercentageAuto::length(0.0),
                bottom: LengthPercentageAuto::length(0.0),
            },
            RankDir::LeftToRight | RankDir::RightToLeft => Rect {
                left: LengthPercentageAuto::length(0.0),
                right: LengthPercentageAuto::length(0.0),
                top: LengthPercentageAuto::length(halo_pad_px),
                bottom: LengthPercentageAuto::length(halo_pad_px + label_margin_px),
            },
        };

        // Step 2.2.9 -- Create the description leaf or markdown sub-tree.
        let (description_taffy_node_id, md_node_taffy_ids) = match lod {
            DiagramLod::Simple => {
                // Legacy path: single leaf with EdgeDescription context.
                let description_style = Style {
                    align_self: Some(AlignSelf::Stretch),
                    margin: description_margin,
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

                (description_taffy_node_id, None)
            }
            DiagramLod::Normal => {
                // Markdown path: parse markdown and build token sub-tree.
                let blocks = MdBlocksParser::parse(desc_text);
                let md_node_taffy_ids = MdNodeBuilder::build(taffy_tree, &blocks, char_width);
                let description_taffy_node_id = md_node_taffy_ids.content_node_id;

                // Halo clearance: `MdNodeBuilder::build`'s `content_node_style`
                // has no margin of its own (it's shared with node content and
                // edge labels, which must NOT get this margin unconditionally),
                // so it is re-styled here rather than threading a margin
                // parameter into `MdNodeBuilder::build` -- mirrors
                // `TaffyEnvelopeBuilder::build`'s re-style of
                // `diagram_node_wrapper_node`.
                let content_style = taffy_tree
                    .style(description_taffy_node_id)
                    .expect("Expected md_content_node to have a style.")
                    .clone();
                taffy_tree
                    .set_style(
                        description_taffy_node_id,
                        Style {
                            margin: description_margin,
                            ..content_style
                        },
                    )
                    .expect("Expected to set halo-clearance margin on md_content_node.");

                (description_taffy_node_id, Some(md_node_taffy_ids))
            }
        };

        Some((
            position,
            sort_key,
            description_taffy_node_id,
            md_node_taffy_ids,
            sibling_index_from_cmp_to,
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
