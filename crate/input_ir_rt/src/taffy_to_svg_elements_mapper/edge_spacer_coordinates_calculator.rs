use disposition_model_common::RankDir;
use disposition_taffy_model::TaffyNodeCtx;
use taffy::TaffyTree;

use crate::taffy_to_svg_elements_mapper::edge_path_builder_pass_1::SpacerCoordinates;

/// Computes absolute spacer coordinates for a single taffy node.
///
/// See [`EdgeSpacerCoordinatesCalculator::calculate`] for details.
pub struct EdgeSpacerCoordinatesCalculator;

impl EdgeSpacerCoordinatesCalculator {
    /// Computes absolute spacer coordinates for a single taffy node.
    ///
    /// Walks up the taffy tree to accumulate the absolute position, then
    /// returns `SpacerCoordinates` with entry and exit points that
    /// depend on `rank_dir`:
    ///
    /// * `RankDir::TopToBottom`: entry at top midpoint (smallest y), exit at
    ///   bottom midpoint (largest y).
    /// * `RankDir::BottomToTop`: entry at bottom midpoint (largest y), exit at
    ///   top midpoint (smallest y).
    /// * `RankDir::LeftToRight`: entry at left midpoint (smallest x), exit at
    ///   right midpoint (largest x).
    /// * `RankDir::RightToLeft`: entry at right midpoint (largest x), exit at
    ///   left midpoint (smallest x).
    pub fn calculate(
        rank_dir: RankDir,
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        taffy_node_id: taffy::NodeId,
    ) -> Option<SpacerCoordinates> {
        let layout = taffy_tree.layout(taffy_node_id).ok()?;

        // === Absolute Coordinates === //
        let mut x_acc = layout.location.x;
        let mut y_acc = layout.location.y;
        let mut current_node_id = taffy_node_id;
        while let Some(parent_taffy_node_id) = taffy_tree.parent(current_node_id) {
            let Ok(parent_layout) = taffy_tree.layout(parent_taffy_node_id) else {
                break;
            };
            x_acc += parent_layout.location.x;
            y_acc += parent_layout.location.y;
            current_node_id = parent_taffy_node_id;
        }

        let cx = x_acc + layout.size.width / 2.0;
        let cy = y_acc + layout.size.height / 2.0;
        let left_x = x_acc;
        let right_x = x_acc + layout.size.width;
        let top_y = y_acc;
        let bottom_y = y_acc + layout.size.height;

        let spacer_coordinates = match rank_dir {
            // Vertical flow: entry/exit share the same x (center),
            // differ in y.
            RankDir::TopToBottom => SpacerCoordinates {
                entry_x: cx,
                entry_y: top_y,
                exit_x: cx,
                exit_y: bottom_y,
            },
            RankDir::BottomToTop => SpacerCoordinates {
                entry_x: cx,
                entry_y: bottom_y,
                exit_x: cx,
                exit_y: top_y,
            },
            // Horizontal flow: entry/exit share the same y (center),
            // differ in x.
            RankDir::LeftToRight => SpacerCoordinates {
                entry_x: left_x,
                entry_y: cy,
                exit_x: right_x,
                exit_y: cy,
            },
            RankDir::RightToLeft => SpacerCoordinates {
                entry_x: right_x,
                entry_y: cy,
                exit_x: left_x,
                exit_y: cy,
            },
        };

        Some(spacer_coordinates)
    }
}
