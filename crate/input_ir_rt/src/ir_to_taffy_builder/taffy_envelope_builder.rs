use disposition_ir_model::{
    edge::EdgeId,
    node::{NodeFace, NodeFaceEdges, NodeId},
};
use disposition_taffy_model::{
    taffy::{
        self,
        style_helpers::{auto, line},
        Display, FlexDirection, JustifyContent, Rect, Style, TaffyTree,
    },
    DiagramLod, EdgeLabelCtx, MdNodeTaffyIds, TaffyNodeCtx, TEXT_LINE_HEIGHT,
};
use taffy::{LengthPercentage, LengthPercentageAuto};

use crate::md_text::md_blocks_parser::MdBlocksParser;

use super::{
    md_node_builder::MdNodeBuilder, taffy_build_ctx::TaffyBuildCtx,
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
        ctx: TaffyBuildCtx<'_>,
    ) -> (taffy::NodeId, Vec<EdgeLabelLeafBuilt>) {
        let mut edge_label_leaves = Vec::new();

        let pad = LengthPercentage::length(EDGE_LABEL_PADDING_PX);
        let zero = LengthPercentage::length(0.0);

        // For Top/Bottom faces the edge contacts the left x edge of the label
        // for all rank directions (sibling order matches declaration order,
        // see `TaffyContainerBuilder::rank_taffy_ids_reverse_if_direction_reversed`).
        // Padding goes on the opposite side to avoid overlapping adjacent labels.
        let label_leaf_style_top_bottom = Style {
            flex_shrink: 0.0,
            padding: Rect {
                left: zero,
                right: pad,
                top: zero,
                bottom: zero,
            },
            ..Default::default()
        };

        // For Left/Right faces the edge contacts the top y edge of the label
        // for all rank directions. Padding goes on the opposite side.
        let label_leaf_style_left_right = Style {
            flex_shrink: 0.0,
            padding: Rect {
                left: zero,
                right: zero,
                top: zero,
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
        //
        // When this face carries any labels, add a one-line bottom margin so
        // the label lifts off the node below it. A Top-face label sits directly
        // above the node, and its text baseline is at the line-box bottom (so
        // glyph descenders would otherwise spill into the node). The margin is
        // gated on the wrapper having labels so label-less nodes (0-height top
        // wrapper) gain no spurious vertical space.
        let edge_wrapper_top_margin_bottom = if top_leaf_ids.is_empty() {
            LengthPercentageAuto::length(0.0)
        } else {
            LengthPercentageAuto::length(TEXT_LINE_HEIGHT)
        };
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
                        top: zero,
                        bottom: zero,
                    },
                    margin: Rect {
                        left: LengthPercentageAuto::length(0.0),
                        right: LengthPercentageAuto::length(0.0),
                        top: LengthPercentageAuto::length(0.0),
                        bottom: edge_wrapper_top_margin_bottom,
                    },
                    ..Default::default()
                },
                &top_leaf_ids,
            )
            .unwrap_or_else(|e| {
                panic!("Expected to create edge_wrapper_top for {node_id}. Error: {e}")
            });

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
                        left: zero,
                        right: zero,
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
                        left: zero,
                        right: zero,
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
                        top: zero,
                        bottom: zero,
                    },
                    ..Default::default()
                },
                &bottom_leaf_ids,
            )
            .unwrap_or_else(|e| {
                panic!("Expected to create edge_wrapper_bottom for {node_id}. Error: {e}")
            });

        // envelope_node: 3x3 CSS Grid -- top-middle, left, center, right,
        // bottom-middle.  The four corner cells are left empty.  Column and
        // row track sizes are `auto` so each track sizes to its content; the
        // center cell stretches to fill the row/column size allocated by any
        // larger adjacent cell.
        let envelope_node = taffy_tree
            .new_with_children(
                Style {
                    display: Display::Grid,
                    grid_template_columns: vec![auto(), auto(), auto()],
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
            ctx.edge_labels.get(edge_id).and_then(|edge_label| {
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
                    .new_with_children(
                        label_leaf_style.clone(),
                        &[md_node_taffy_ids.content_node_id],
                    )
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
                        label_leaf_style.clone(),
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
}
