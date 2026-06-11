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
    EdgeLabelCtx, TaffyNodeCtx,
};
use taffy::LengthPercentage;

use super::taffy_node_build_context::EdgeLabelLeafBuilt;

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
    pub(crate) fn build(
        taffy_tree: &mut TaffyTree<TaffyNodeCtx>,
        node_id: &NodeId<'static>,
        diagram_node_wrapper_node: taffy::NodeId,
        node_face_edges: &NodeFaceEdges<'static>,
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
        );
        let bottom_leaf_ids = Self::build_face_leaves(
            taffy_tree,
            node_id,
            NodeFace::Bottom,
            node_face_edges.edges_for(node_id, NodeFace::Bottom),
            &label_leaf_style_top_bottom,
            &mut edge_label_leaves,
        );
        let left_leaf_ids = Self::build_face_leaves(
            taffy_tree,
            node_id,
            NodeFace::Left,
            node_face_edges.edges_for(node_id, NodeFace::Left),
            &label_leaf_style_left_right,
            &mut edge_label_leaves,
        );
        let right_leaf_ids = Self::build_face_leaves(
            taffy_tree,
            node_id,
            NodeFace::Right,
            node_face_edges.edges_for(node_id, NodeFace::Right),
            &label_leaf_style_left_right,
            &mut edge_label_leaves,
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
                        top: zero,
                        bottom: zero,
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

    /// Builds the edge label leaf nodes for one face of an envelope node.
    ///
    /// For each edge ID in `edge_ids`, a leaf node is created with
    /// [`TaffyNodeCtx::EdgeLabel`] context and appended to `label_leaves`.
    /// Returns the `taffy::NodeId`s of all created leaves in order.
    ///
    /// # Parameters
    ///
    /// * `taffy_tree`: The `TaffyTree` to insert nodes into.
    /// * `node_id`: The diagram node ID that owns this face.
    /// * `face`: Which face of the node these labels are on.
    /// * `edge_ids`: The edge IDs that attach to `face` on `node_id`.
    /// * `label_leaf_style`: The taffy `Style` applied to every label leaf.
    /// * `label_leaves`: Output accumulator for the built leaves.
    fn build_face_leaves(
        taffy_tree: &mut TaffyTree<TaffyNodeCtx>,
        node_id: &NodeId<'static>,
        face: NodeFace,
        edge_ids: &[EdgeId<'static>],
        label_leaf_style: &Style,
        label_leaves: &mut Vec<EdgeLabelLeafBuilt>,
    ) -> Vec<taffy::NodeId> {
        edge_ids
            .iter()
            .map(|edge_id| {
                let taffy_node_id = taffy_tree
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
                label_leaves.push(EdgeLabelLeafBuilt {
                    edge_id: edge_id.clone(),
                    node_id: node_id.clone(),
                    face,
                    taffy_node_id,
                });
                taffy_node_id
            })
            .collect()
    }
}
