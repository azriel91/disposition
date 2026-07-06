use disposition_ir_model::{
    edge::EdgeId,
    entity::EntityType,
    node::{NodeFace, NodeFaceEdges, NodeId},
};
use disposition_model_common::{Id, Map};
use disposition_taffy_model::{
    taffy::{
        self,
        style_helpers::{auto, line, max_content},
        Display, FlexDirection, JustifyContent, Rect, Style, TaffyTree,
    },
    DiagramLod, EdgeLabelCtx, MdNodeTaffyIds, TaffyNodeCtx, TaffyNodeKind, TEXT_FONT_SIZE,
};
use taffy::{LengthPercentage, LengthPercentageAuto};

use crate::md_text::md_blocks_parser::MdBlocksParser;

use super::{
    md_node_builder::{MdNodeBuilder, MD_CONTENT_NODE_PADDING},
    taffy_build_ctx::TaffyBuildCtx,
    taffy_node_build_context::EdgeLabelLeafBuilt,
};

// === Constants === //

/// Padding (in pixels) added to the far side of an edge label leaf node.
///
/// The edge path contacts the label at one edge; this padding creates visual
/// separation on the opposite side so the path does not overlap adjacent
/// labels or the face boundary on the exit side.
const EDGE_LABEL_PADDING_PX: f32 = 4.0;

/// Padding (in pixels) added to both sides of a node so edges don't begin at
/// the very edge.
const NODE_SIDE_PADDING_PX: LengthPercentage = LengthPercentage::length(8.0);

// === TaffyEnvelopeBuilder === //

/// Wraps each diagram node in a 3x3 CSS Grid envelope that reserves
/// flex-row/column slots on each face (top, bottom, left, right) for edge
/// label leaf nodes.
pub(crate) struct TaffyEnvelopeBuilder;

impl TaffyEnvelopeBuilder {
    /// Builds an envelope taffy node around `diagram_node_wrapper_node`.
    ///
    /// The envelope adds flex-row/column slots for edge label leaf nodes on
    /// each face of the diagram node. The structure is:
    ///
    /// ```text
    /// envelope_node:               (3x3 CSS Grid)
    ///   edge_wrapper_top:          (flex row,    grid row 1 col 2)
    ///   edge_wrapper_left:         (flex column, grid row 2 col 1)
    ///   diagram_node_wrapper_node: (             grid row 2 col 2)
    ///   edge_wrapper_right:        (flex column, grid row 2 col 3)
    ///   edge_wrapper_bottom:       (flex row,    grid row 3 col 2)
    /// ```
    ///
    /// The four corner cells (row 1/3, col 1/3) are left empty. Each track
    /// size is `auto` so tracks size to their content.
    ///
    /// # Parameters
    ///
    /// * `taffy_tree`: The `TaffyTree` to insert nodes into.
    /// * `node_id`: The diagram node ID for this envelope.
    /// * `diagram_node_wrapper_node`: The existing wrapper node taffy ID to
    ///   wrap.
    /// * `node_face_edges`: The per-node face-to-edge-IDs mapping from
    ///   `IrDiagram`.
    /// * `ctx`: Build context providing the edge label text, endpoint lookup,
    ///   level of detail, and character width needed to build markdown content
    ///   sub-trees for the label slots.
    pub(crate) fn build(
        taffy_tree: &mut TaffyTree<TaffyNodeCtx>,
        node_id: &NodeId<'static>,
        diagram_node_wrapper_node: taffy::NodeId,
        node_face_edges: &NodeFaceEdges<'static>,
        taffy_id_to_kind: &mut Map<taffy::NodeId, TaffyNodeKind<'static>>,
        ctx: TaffyBuildCtx<'_>,
    ) -> (taffy::NodeId, Vec<EdgeLabelLeafBuilt>) {
        let mut edge_label_leaves = Vec::new();

        let md_content_node_padding = LengthPercentage::length(MD_CONTENT_NODE_PADDING);
        let pad = LengthPercentage::length(EDGE_LABEL_PADDING_PX);

        // Halo clearance (`margin`, computed per-edge in `Self::label_margin_build`
        // since it depends on the edge's own dependency/interaction kind) is
        // applied in `Self::label_slot_build`, not baked into these base styles --
        // see `Self::label_margin_build`'s doc comment for the full explanation.
        //
        // For Top/Bottom faces the edge contacts the left x edge of the label
        // for all rank directions (sibling order matches declaration order,
        // see `TaffyContainerBuilder::rank_taffy_ids_reverse_if_direction_reversed`).
        // Padding goes on the opposite side to avoid overlapping adjacent labels.
        let label_leaf_style_top_bottom = Style {
            flex_shrink: 0.0,
            padding: Rect {
                left: md_content_node_padding,
                right: pad,
                top: md_content_node_padding,
                bottom: md_content_node_padding,
            },
            ..Default::default()
        };

        // For Left/Right faces the edge contacts the top y edge of the label
        // for all rank directions. Padding goes on the opposite side.
        let label_leaf_style_left_right = Style {
            flex_shrink: 0.0,
            padding: Rect {
                left: md_content_node_padding,
                right: md_content_node_padding,
                top: md_content_node_padding,
                bottom: pad,
            },
            ..Default::default()
        };

        let top_leaf_ids = Self::build_face_leaves(
            taffy_tree,
            node_id,
            NodeFace::Top,
            node_face_edges.edges_for(node_id, NodeFace::Top),
            &label_leaf_style_top_bottom,
            &mut edge_label_leaves,
            ctx,
        );
        let bottom_leaf_ids = Self::build_face_leaves(
            taffy_tree,
            node_id,
            NodeFace::Bottom,
            node_face_edges.edges_for(node_id, NodeFace::Bottom),
            &label_leaf_style_top_bottom,
            &mut edge_label_leaves,
            ctx,
        );
        let left_leaf_ids = Self::build_face_leaves(
            taffy_tree,
            node_id,
            NodeFace::Left,
            node_face_edges.edges_for(node_id, NodeFace::Left),
            &label_leaf_style_left_right,
            &mut edge_label_leaves,
            ctx,
        );
        let right_leaf_ids = Self::build_face_leaves(
            taffy_tree,
            node_id,
            NodeFace::Right,
            node_face_edges.edges_for(node_id, NodeFace::Right),
            &label_leaf_style_left_right,
            &mut edge_label_leaves,
            ctx,
        );

        // edge_wrapper_top: row 1, col 2 (top-middle cell of the 3x3 grid)
        let edge_wrapper_top = taffy_tree
            .new_with_children(
                Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    grid_row: line(1),
                    grid_column: line(2),
                    justify_content: Some(JustifyContent::SpaceEvenly),
                    padding: Rect {
                        left: NODE_SIDE_PADDING_PX,
                        right: NODE_SIDE_PADDING_PX,
                        top: md_content_node_padding,
                        bottom: md_content_node_padding,
                    },
                    ..Default::default()
                },
                &top_leaf_ids,
            )
            .unwrap_or_else(|e| {
                panic!("Expected to create edge_wrapper_top for {node_id}. Error: {e}")
            });
        taffy_id_to_kind.insert(
            edge_wrapper_top,
            TaffyNodeKind::EnvelopeFaceWrapper {
                node_id: node_id.clone(),
                face: NodeFace::Top,
            },
        );

        // edge_wrapper_left: row 2, col 1 (left-middle cell of the 3x3 grid)
        let edge_wrapper_left = taffy_tree
            .new_with_children(
                Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    grid_row: line(2),
                    grid_column: line(1),
                    justify_content: Some(JustifyContent::SpaceEvenly),
                    padding: Rect {
                        left: md_content_node_padding,
                        right: md_content_node_padding,
                        top: NODE_SIDE_PADDING_PX,
                        bottom: NODE_SIDE_PADDING_PX,
                    },
                    ..Default::default()
                },
                &left_leaf_ids,
            )
            .unwrap_or_else(|e| {
                panic!("Expected to create edge_wrapper_left for {node_id}. Error: {e}")
            });
        taffy_id_to_kind.insert(
            edge_wrapper_left,
            TaffyNodeKind::EnvelopeFaceWrapper {
                node_id: node_id.clone(),
                face: NodeFace::Left,
            },
        );

        // Place diagram_node_wrapper_node at row 2, col 2 (center cell of the 3x3
        // grid).
        let diagram_wrapper_style = taffy_tree
            .style(diagram_node_wrapper_node)
            .unwrap_or_else(|e| {
                panic!(
                    "Expected to get style of diagram_node_wrapper_node for {node_id}. \
                     Error: {e}"
                )
            })
            .clone();
        taffy_tree
            .set_style(
                diagram_node_wrapper_node,
                Style {
                    grid_row: line(2),
                    grid_column: line(2),
                    ..diagram_wrapper_style
                },
            )
            .unwrap_or_else(|e| {
                panic!(
                    "Expected to set grid placement on diagram_node_wrapper_node for \
                     {node_id}. Error: {e}"
                )
            });

        // edge_wrapper_right: row 2, col 3 (right-middle cell of the 3x3 grid)
        let edge_wrapper_right = taffy_tree
            .new_with_children(
                Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    grid_row: line(2),
                    grid_column: line(3),
                    justify_content: Some(JustifyContent::SpaceEvenly),
                    padding: Rect {
                        left: md_content_node_padding,
                        right: md_content_node_padding,
                        top: NODE_SIDE_PADDING_PX,
                        bottom: NODE_SIDE_PADDING_PX,
                    },
                    ..Default::default()
                },
                &right_leaf_ids,
            )
            .unwrap_or_else(|e| {
                panic!("Expected to create edge_wrapper_right for {node_id}. Error: {e}")
            });
        taffy_id_to_kind.insert(
            edge_wrapper_right,
            TaffyNodeKind::EnvelopeFaceWrapper {
                node_id: node_id.clone(),
                face: NodeFace::Right,
            },
        );

        // edge_wrapper_bottom: row 3, col 2 (bottom-middle cell of the 3x3 grid)
        let edge_wrapper_bottom = taffy_tree
            .new_with_children(
                Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    grid_row: line(3),
                    grid_column: line(2),
                    justify_content: Some(JustifyContent::SpaceEvenly),
                    padding: Rect {
                        left: NODE_SIDE_PADDING_PX,
                        right: NODE_SIDE_PADDING_PX,
                        top: md_content_node_padding,
                        bottom: md_content_node_padding,
                    },
                    ..Default::default()
                },
                &bottom_leaf_ids,
            )
            .unwrap_or_else(|e| {
                panic!("Expected to create edge_wrapper_bottom for {node_id}. Error: {e}")
            });
        taffy_id_to_kind.insert(
            edge_wrapper_bottom,
            TaffyNodeKind::EnvelopeFaceWrapper {
                node_id: node_id.clone(),
                face: NodeFace::Bottom,
            },
        );

        // envelope_node: 3x3 CSS Grid -- top-middle, left, center, right,
        // bottom-middle.  The four corner cells are left empty.  Row tracks and
        // the side column tracks are `auto` so they size to their content; the
        // center cell stretches to fill the row/column size allocated by any
        // larger adjacent cell.
        //
        // The center column is `max_content` rather than `auto`. With `auto`,
        // taffy measures the grid's intrinsic height (its flex base size in the
        // surrounding column) by laying the center cell out at its *min-content*
        // width, which makes a wrapping markdown description (node title / body)
        // wrap to one word per line and report a hugely inflated height. The
        // grid then carries that height into the rank container and the whole
        // diagram. Pinning the center column to `max_content` keeps the node at
        // its natural (unwrapped) content width during that measurement, so the
        // height reflects the laid-out content rather than the min-content wrap.
        // The resulting width matches the previous `auto` result for the
        // unconstrained diagrams we render (auto's max is already max-content).
        let envelope_node = taffy_tree
            .new_with_children(
                Style {
                    display: Display::Grid,
                    grid_template_columns: vec![auto(), max_content(), auto()],
                    grid_template_rows: vec![auto(), auto(), auto()],
                    ..Default::default()
                },
                &[
                    edge_wrapper_top,
                    edge_wrapper_left,
                    diagram_node_wrapper_node,
                    edge_wrapper_right,
                    edge_wrapper_bottom,
                ],
            )
            .unwrap_or_else(|e| {
                panic!("Expected to create envelope_node for {node_id}. Error: {e}")
            });

        (envelope_node, edge_label_leaves)
    }

    /// Builds the edge label slot nodes for one face of an envelope node.
    ///
    /// For each edge ID in `edge_ids` a slot node is created and appended to
    /// `label_leaves`. Returns the `taffy::NodeId`s of all created slots in
    /// order.
    ///
    /// At [`DiagramLod::Normal`] with non-empty label text the slot wraps a
    /// markdown content sub-tree (built via [`MdNodeBuilder`]), styled like a
    /// list of [`TaffyNodeCtx::MdToken`] / [`TaffyNodeCtx::MdImage`] leaves, so
    /// the label is rendered with the same markdown styling as node and edge
    /// descriptions. Otherwise the slot is a single placeholder leaf carrying
    /// [`TaffyNodeCtx::EdgeLabel`] context (legacy / [`DiagramLod::Simple`]
    /// path).
    ///
    /// # Parameters
    ///
    /// * `taffy_tree`: The `TaffyTree` to insert nodes into.
    /// * `node_id`: The diagram node ID that owns this face.
    /// * `face`: Which face of the node these labels are on.
    /// * `edge_ids`: The edge IDs that attach to `face` on `node_id`.
    /// * `label_leaf_style`: The taffy `Style` applied to every label slot.
    /// * `label_leaves`: Output accumulator for the built slots.
    /// * `ctx`: Build context providing the label text, endpoint lookup, level
    ///   of detail, and character width.
    fn build_face_leaves(
        taffy_tree: &mut TaffyTree<TaffyNodeCtx>,
        node_id: &NodeId<'static>,
        face: NodeFace,
        edge_ids: &[EdgeId<'static>],
        label_leaf_style: &Style,
        label_leaves: &mut Vec<EdgeLabelLeafBuilt>,
        ctx: TaffyBuildCtx<'_>,
    ) -> Vec<taffy::NodeId> {
        edge_ids
            .iter()
            .map(|edge_id| {
                let (taffy_node_id, md_node_taffy_ids) = Self::label_slot_build(
                    taffy_tree,
                    node_id,
                    face,
                    edge_id,
                    label_leaf_style,
                    ctx,
                );
                label_leaves.push(EdgeLabelLeafBuilt {
                    edge_id: edge_id.clone(),
                    node_id: node_id.clone(),
                    face,
                    taffy_node_id,
                    md_node_taffy_ids,
                });
                taffy_node_id
            })
            .collect()
    }

    /// Builds the slot node for a single edge label, returning the slot node ID
    /// and (at [`DiagramLod::Normal`] with non-empty text) its markdown
    /// sub-tree IDs.
    ///
    /// The label text is the edge's `from` text when `node_id` is the edge's
    /// `from` endpoint, otherwise its `to` text.
    fn label_slot_build(
        taffy_tree: &mut TaffyTree<TaffyNodeCtx>,
        node_id: &NodeId<'static>,
        face: NodeFace,
        edge_id: &EdgeId<'static>,
        label_leaf_style: &Style,
        ctx: TaffyBuildCtx<'_>,
    ) -> (taffy::NodeId, Option<MdNodeTaffyIds>) {
        // Resolve the label text for this slot, building a markdown sub-tree
        // only at `DiagramLod::Normal` when the text is non-empty.
        let label_text = if matches!(ctx.lod, DiagramLod::Normal) {
            ctx.edge_id_to_group_id
                .get(edge_id)
                .and_then(|edge_group_id| ctx.edge_labels.get_for_edge(edge_id, edge_group_id))
                .and_then(|edge_label| {
                    let is_from_endpoint = ctx
                        .edge_id_to_endpoint_node_ids
                        .get(edge_id)
                        .map(|(from_node_id, _to_node_id)| from_node_id == node_id)
                        .unwrap_or(false);
                    let text = if is_from_endpoint {
                        edge_label.from.as_str()
                    } else {
                        edge_label.to.as_str()
                    };
                    (!text.is_empty()).then_some(text)
                })
        } else {
            None
        };

        let label_style = Style {
            margin: Self::label_margin_build(ctx, edge_id, face),
            ..label_leaf_style.clone()
        };

        match label_text {
            Some(label_text) => {
                // Markdown path: the slot wraps a markdown content node so the
                // label is rendered with inline styling (bold, italic, code,
                // links, images). The slot keeps `label_leaf_style` (padding /
                // flex-shrink) so spacing matches the legacy leaf.
                let md_blocks = MdBlocksParser::parse(label_text);
                let md_node_taffy_ids =
                    MdNodeBuilder::build(taffy_tree, &md_blocks, ctx.char_width);
                let slot_taffy_node_id = taffy_tree
                    .new_with_children(label_style, &[md_node_taffy_ids.content_node_id])
                    .unwrap_or_else(|e| {
                        panic!(
                            "Expected to create edge label slot for edge {edge_id} on \
                             face {face:?} of node {node_id}. Error: {e}"
                        )
                    });
                (slot_taffy_node_id, Some(md_node_taffy_ids))
            }
            None => {
                // Legacy / `DiagramLod::Simple` path: a single placeholder leaf
                // sized from the label text during layout measurement.
                let slot_taffy_node_id = taffy_tree
                    .new_leaf_with_context(
                        label_style,
                        TaffyNodeCtx::EdgeLabel(EdgeLabelCtx {
                            edge_id: edge_id.clone(),
                            node_id: node_id.clone(),
                            face,
                        }),
                    )
                    .unwrap_or_else(|e| {
                        panic!(
                            "Expected to create edge label leaf for edge {edge_id} on \
                             face {face:?} of node {node_id}. Error: {e}"
                        )
                    });
                (slot_taffy_node_id, None)
            }
        }
    }

    /// Computes the halo-clearance `margin` for a single edge's label slot.
    ///
    /// The interaction edge halo is a wide path centered on the edge, so the
    /// label needs clearance of half its stroke width ("`halo_pad_px`") on
    /// whichever sides the halo extends into. This must be `margin`, not
    /// `padding` -- the label's background box is sized to include its
    /// padding, so padding-based clearance would still leave the background
    /// overlapping the halo even though the text itself wouldn't.
    ///
    /// Halo orientation follows which face the label sits on, not the
    /// diagram's overall rank direction: Top/Bottom faces lay siblings out in
    /// a row (spread along x), so a straight edge passing through that row
    /// overlaps the label along x regardless of rank_dir -- including
    /// same-rank "cycle" edges, which are assigned Top/Bottom faces even
    /// under a horizontal rank_dir (see `EdgeFaceAssigner::cycle_faces`).
    /// Left/Right faces lay siblings out in a column (spread along y), so the
    /// same reasoning gives y clearance.
    ///
    /// `halo_pad_px` is `0.0` for dependency edges (`EntityType::
    /// is_dependency_edge`): only interaction edges render the wide
    /// interaction-edge halo, so a dependency edge's label has nothing to
    /// clear on the routing-path side. This mirrors
    /// `EdgeDescriptionBuilder::edge_desc_build`'s halo-clearance margin for
    /// edge descriptions.
    ///
    /// Both sides of the packing axis get the *same* margin (`halo_pad_px +
    /// label_margin_px`), not just the far side -- `SvgEdgeInfosBuilder::
    /// label_face_span_compute`'s routing pullback only ever cancels the
    /// `halo_pad_px` component (see its doc comment), so if the entry side
    /// only carried `halo_pad_px` here, a dependency edge (`halo_pad_px ==
    /// 0.0`) would end up with no margin at all on that side, losing
    /// separation from whatever sits before it along the packing axis (e.g.
    /// an adjacent sibling label sharing the same face). Carrying
    /// `label_margin_px` on both sides keeps that separation regardless of
    /// edge kind, and leaves a `label_margin_px`-sized gap between the routed
    /// path and the label even after the pullback cancels `halo_pad_px`, so
    /// the label reads as visually associated with its edge rather than
    /// flush against it.
    fn label_margin_build(
        ctx: TaffyBuildCtx<'_>,
        edge_id: &EdgeId<'static>,
        face: NodeFace,
    ) -> Rect<LengthPercentageAuto> {
        let is_dependency_edge = ctx
            .entity_types
            .get(AsRef::<Id<'_>>::as_ref(edge_id))
            .map(|edge_entity_types| edge_entity_types.iter().any(EntityType::is_dependency_edge))
            .unwrap_or(false);
        let halo_pad_px = if is_dependency_edge {
            0.0
        } else {
            ctx.interaction_edge_halo_stroke_width / 2.0
        };
        let label_margin_px = TEXT_FONT_SIZE / 2.0;
        let margin_px = LengthPercentageAuto::length(halo_pad_px + label_margin_px);

        match face {
            NodeFace::Top | NodeFace::Bottom => Rect {
                left: margin_px,
                right: margin_px,
                top: LengthPercentageAuto::length(0.0),
                bottom: LengthPercentageAuto::length(0.0),
            },
            NodeFace::Left | NodeFace::Right => Rect {
                left: LengthPercentageAuto::length(0.0),
                right: LengthPercentageAuto::length(0.0),
                top: margin_px,
                bottom: margin_px,
            },
        }
    }
}
