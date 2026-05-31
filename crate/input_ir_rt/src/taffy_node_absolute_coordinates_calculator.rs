use disposition_taffy_model::{
    taffy::{self, TaffyTree},
    TaffyNodeCtx,
};

use crate::AbsoluteCoordinates;

/// Calculates the absolute coordinates of a taffy node.
#[derive(Clone, Copy, Debug)]
pub(crate) struct TaffyNodeAbsoluteCoordinatesCalculator;

impl TaffyNodeAbsoluteCoordinatesCalculator {
    /// Calculates the absolute x and y coordinates of a taffy node.
    ///
    /// The coordinates of a taffy node in the Taffy tree are relative to each
    /// node's parent, whereas we need them to be absolute when rendering the
    /// SVG. This walks up the parent chain and accumulates each parent's
    /// location offset.
    pub(crate) fn calculate(
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        taffy_node_id: taffy::NodeId,
        layout: &taffy::Layout,
    ) -> AbsoluteCoordinates {
        // We don't use the content_box here because these are coordinates for
        // the `<rect>` element.
        let mut x = layout.location.x;
        let mut y = layout.location.y;
        let mut current_node_id = taffy_node_id;
        while let Some(parent_taffy_node_id) = taffy_tree.parent(current_node_id) {
            let Ok(parent_layout) = taffy_tree.layout(parent_taffy_node_id) else {
                break;
            };
            // `content_box_x/y` places the inner nodes to align to the bottom
            // right of the parent nodes instead of having appropriate padding
            // around the inner node.
            x += parent_layout.location.x;
            y += parent_layout.location.y;
            current_node_id = parent_taffy_node_id;
        }
        AbsoluteCoordinates { x, y }
    }
}
