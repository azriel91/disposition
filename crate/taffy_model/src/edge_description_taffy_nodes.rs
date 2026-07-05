use std::cmp::Ordering;

use crate::MdNodeTaffyIds;

/// The taffy node IDs created for a single described edge at a single LCA
/// level.
///
/// Two nodes are created per edge description:
///
/// 1. A `container_taffy_node_id` -- a flex container interleaved between rank
///    containers at the LCA level.
/// 2. A `description_taffy_node_id` -- a leaf node (legacy path) or
///    `md_content_node` container (markdown path) whose layout position is used
///    to place the description in the SVG.
///
/// The container uses `TaffyNodeCtx::None` (like rank containers). At
/// `DiagramLod::Simple`, the leaf uses `TaffyNodeCtx::EdgeDescription`.
/// At `DiagramLod::Normal`, the markdown path is active and
/// `md_node_taffy_ids` is populated with the markdown sub-tree IDs.
///
/// # Examples
///
/// ```text
/// // Legacy path (DiagramLod::Simple)
/// EdgeDescriptionTaffyNodes {
///     container_taffy_node_id: NodeId(10),
///     description_taffy_node_id: NodeId(11),
///     md_node_taffy_ids: None,
///     sibling_index_from_cmp_to: Ordering::Less,
/// }
///
/// // Markdown path (DiagramLod::Normal)
/// EdgeDescriptionTaffyNodes {
///     container_taffy_node_id: NodeId(10),
///     description_taffy_node_id: NodeId(11), // points to md_content_node
///     md_node_taffy_ids: Some(MdNodeTaffyIds { ... }),
///     sibling_index_from_cmp_to: Ordering::Greater,
/// }
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct EdgeDescriptionTaffyNodes {
    /// The flex container interleaved between rank containers.
    pub container_taffy_node_id: taffy::NodeId,
    /// The leaf node (legacy) or `md_content_node` (markdown path) whose
    /// layout position is used to place the description in the SVG.
    pub description_taffy_node_id: taffy::NodeId,
    /// Populated at `DiagramLod::Normal`. When `Some`,
    /// `description_taffy_node_id` points to the `md_content_node`
    /// container rather than a bare leaf.
    pub md_node_taffy_ids: Option<MdNodeTaffyIds>,
    /// `sibling_index_from.cmp(&sibling_index_to)` at the edge's LCA depth --
    /// the relative order of the edge's `from`/`to` divergent ancestors among
    /// their siblings.
    ///
    /// Used by `EdgeSpacerCoordinatesCalculator::calculate_description_contact`
    /// to pick which side of the description box this edge's own routing
    /// waypoint sits on, so that edges travelling in opposite directions
    /// (e.g. a `symmetric` interaction group's forward and reverse edges)
    /// don't both clip through the box's center and backtrack.
    ///
    /// `Ordering::Equal` should not occur in practice: two distinct divergent
    /// ancestors always have distinct sibling indices.
    pub sibling_index_from_cmp_to: Ordering,
}
