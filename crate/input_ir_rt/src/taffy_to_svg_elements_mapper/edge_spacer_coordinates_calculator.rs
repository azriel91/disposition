use std::cmp::Ordering;

use disposition_model_common::RankDir;
use disposition_taffy_model::TaffyNodeCtx;
use taffy::TaffyTree;

use crate::{
    taffy_to_svg_elements_mapper::edge_path_builder_pass_1::SpacerCoordinates,
    TaffyNodeAbsoluteCoordinatesCalculator,
};

/// Computes absolute spacer coordinates for a single taffy node.
///
/// See [`EdgeSpacerCoordinatesCalculator::calculate`] for details.
pub struct EdgeSpacerCoordinatesCalculator;

/// Absolute bounding box extremes and center of a taffy node, in SVG
/// coordinate space.
#[derive(Clone, Copy)]
struct NodeRect {
    left_x: f32,
    right_x: f32,
    top_y: f32,
    bottom_y: f32,
    cx: f32,
    cy: f32,
}

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
    ///
    /// Used for spacers the edge path genuinely threads *through* along the
    /// main rank axis (rank-based, cross-container, edge-desc-container, and
    /// text-content spacers). For an edge's own description contact, see
    /// [`Self::calculate_description_contact`] instead, which does not center
    /// on the cross axis.
    pub fn calculate(
        rank_dir: RankDir,
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        taffy_node_id: taffy::NodeId,
    ) -> Option<SpacerCoordinates> {
        let node_rect = Self::node_rect_compute(taffy_tree, taffy_node_id)?;
        let NodeRect {
            left_x,
            right_x,
            top_y,
            bottom_y,
            cx,
            cy,
        } = node_rect;

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

    /// Computes the single routing waypoint (`entry == exit`) for an edge's
    /// own `edge_description_container` leaf, for a **same-rank (cycle edge)**
    /// description box.
    ///
    /// Unlike [`Self::calculate`], the description box is not a corridor the
    /// path threads *through* -- it sits beside the edge's actual travel
    /// path, so entry and exit are the same point: a fixed side of the box
    /// (independent of `RankDir`'s forward/reverse direction),
    /// with the position along the other axis chosen by comparing the
    /// edge's `from`/`to` divergent-ancestor sibling indices
    /// (`sibling_index_from_cmp_to`).
    ///
    /// This intentionally diverges from `EdgeFaceAssigner::cycle_faces`'s
    /// "no flip between forward/reverse RankDir" convention: that function
    /// selects a face on the node itself (an absolute-canvas-position
    /// concept, consistent regardless of rank progression direction), while
    /// this waypoint is biased by which endpoint is genuinely "from" for
    /// *this* edge. Two edges sharing the same box but travelling in
    /// opposite directions (e.g. a `symmetric` interaction group's forward
    /// and reverse edges) need their own bias so neither backtracks through
    /// the box -- confirmed by a real regression: without the
    /// `RightToLeft`/`BottomToTop` flip, a reverse-direction edge's path
    /// visibly looped back on itself when routing through its own
    /// description box (`edge_ix_client_server__1` in
    /// `020_interaction_halo_with_labels.yaml`, whose path back then read
    /// `... 456 -> 245(entry) -> 285(exit) -> 91 ...`, backtracking
    /// rightward from 245 to 285 against its actual right-to-left travel).
    ///
    /// | `RankDir` | fixed axis | `from` before `to` (`Less`) | else (`Greater`) |
    /// |---|---|---|---|
    /// | `LeftToRight` | `x = left_x` | `y = top_y` | `y = bottom_y` |
    /// | `RightToLeft` | `x = left_x` | `y = bottom_y` | `y = top_y` |
    /// | `TopToBottom` | `y = top_y` | `x = left_x` | `x = right_x` |
    /// | `BottomToTop` | `y = top_y` | `x = right_x` | `x = left_x` |
    ///
    /// `Ordering::Equal` should not occur (two distinct divergent ancestors
    /// always have distinct sibling indices); treated the same as `Greater`.
    ///
    /// # Example values
    ///
    /// `rank_dir = LeftToRight`, box `left_x = 245, top_y = 37, bottom_y =
    /// 60`, `sibling_index_from_cmp_to = Ordering::Less` -- returns
    /// `Some(SpacerCoordinates { entry_x: 245.0, entry_y: 37.0, exit_x:
    /// 245.0, exit_y: 37.0 })`.
    pub fn calculate_description_contact(
        rank_dir: RankDir,
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        taffy_node_id: taffy::NodeId,
        sibling_index_from_cmp_to: Ordering,
    ) -> Option<SpacerCoordinates> {
        let node_rect = Self::node_rect_compute(taffy_tree, taffy_node_id)?;
        Some(Self::description_contact_from_rect(
            rank_dir,
            &node_rect,
            sibling_index_from_cmp_to,
        ))
    }

    /// Pure coordinate selection for [`Self::calculate_description_contact`],
    /// separated from taffy tree access so the `RankDir` x `Ordering` table
    /// can be unit tested directly against a constructed [`NodeRect`].
    fn description_contact_from_rect(
        rank_dir: RankDir,
        node_rect: &NodeRect,
        sibling_index_from_cmp_to: Ordering,
    ) -> SpacerCoordinates {
        let NodeRect {
            left_x,
            right_x,
            top_y,
            bottom_y,
            ..
        } = *node_rect;
        let from_before_to = sibling_index_from_cmp_to == Ordering::Less;

        let (x, y) = match rank_dir {
            RankDir::LeftToRight => (left_x, if from_before_to { top_y } else { bottom_y }),
            RankDir::RightToLeft => (left_x, if from_before_to { bottom_y } else { top_y }),
            RankDir::TopToBottom => (if from_before_to { left_x } else { right_x }, top_y),
            RankDir::BottomToTop => (if from_before_to { right_x } else { left_x }, top_y),
        };

        SpacerCoordinates {
            entry_x: x,
            entry_y: y,
            exit_x: x,
            exit_y: y,
        }
    }

    /// Computes the routing waypoint pair (`entry != exit`) for a
    /// **cross-rank** edge's own `edge_description_container` leaf.
    ///
    /// Unlike [`Self::calculate_description_contact`] (used for same-rank
    /// cycle edges, whose description box sits *beside* the path), a
    /// cross-rank edge's description box sits directly on the rank corridor
    /// between its divergent ancestors, so the path should thread *through*
    /// it, the same way [`Self::calculate`] threads through an ordinary
    /// spacer.
    ///
    /// The fixed cross-axis coordinate mirrors [`Self::calculate`]'s
    /// `cx`/`cy` convention (unchanged between a `RankDir` and its reverse
    /// pair -- `top_y` for `LeftToRight`/`RightToLeft`, `left_x` for
    /// `TopToBottom`/`BottomToTop`). `Ordering::Less` (this edge's `from` is
    /// before its `to`, i.e. it travels in the topological-forward
    /// direction) reuses [`Self::calculate`]'s canonical entry/exit
    /// assignment for that `RankDir` (substituting the fixed value for
    /// `cx`/`cy`); `Ordering::Greater` (a reverse-direction edge, e.g. a
    /// `symmetric` interaction group's response edge) swaps entry and exit,
    /// so the waypoint pair always runs in *this edge's own* travel
    /// direction rather than the diagram's canonical one.
    ///
    /// | `RankDir` | fixed axis | `from` before `to` (`Less`) | else (`Greater`) |
    /// |---|---|---|---|
    /// | `LeftToRight` | `y = top_y` | entry=`(left_x,top_y)` exit=`(right_x,top_y)` | entry=`(right_x,top_y)` exit=`(left_x,top_y)` |
    /// | `RightToLeft` | `y = top_y` | entry=`(right_x,top_y)` exit=`(left_x,top_y)` | entry=`(left_x,top_y)` exit=`(right_x,top_y)` |
    /// | `TopToBottom` | `x = left_x` | entry=`(left_x,top_y)` exit=`(left_x,bottom_y)` | entry=`(left_x,bottom_y)` exit=`(left_x,top_y)` |
    /// | `BottomToTop` | `x = left_x` | entry=`(left_x,bottom_y)` exit=`(left_x,top_y)` | entry=`(left_x,top_y)` exit=`(left_x,bottom_y)` |
    ///
    /// `Ordering::Equal` should not occur (two distinct divergent ancestors
    /// always have distinct sibling indices); treated the same as `Greater`.
    ///
    /// # Example values
    ///
    /// `rank_dir = LeftToRight`, box `left_x = 245, right_x = 311, top_y =
    /// 20`, `sibling_index_from_cmp_to = Ordering::Less` -- returns
    /// `Some(SpacerCoordinates { entry_x: 245.0, entry_y: 20.0, exit_x:
    /// 311.0, exit_y: 20.0 })`.
    pub fn calculate_description_thread(
        rank_dir: RankDir,
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        taffy_node_id: taffy::NodeId,
        sibling_index_from_cmp_to: Ordering,
    ) -> Option<SpacerCoordinates> {
        let node_rect = Self::node_rect_compute(taffy_tree, taffy_node_id)?;
        Some(Self::description_thread_from_rect(
            rank_dir,
            &node_rect,
            sibling_index_from_cmp_to,
        ))
    }

    /// Pure coordinate selection for [`Self::calculate_description_thread`],
    /// separated from taffy tree access so the `RankDir` x `Ordering` table
    /// can be unit tested directly against a constructed [`NodeRect`].
    fn description_thread_from_rect(
        rank_dir: RankDir,
        node_rect: &NodeRect,
        sibling_index_from_cmp_to: Ordering,
    ) -> SpacerCoordinates {
        let NodeRect {
            left_x,
            right_x,
            top_y,
            bottom_y,
            ..
        } = *node_rect;
        let from_before_to = sibling_index_from_cmp_to == Ordering::Less;

        let ((entry_x, entry_y), (exit_x, exit_y)) = match rank_dir {
            RankDir::LeftToRight => {
                if from_before_to {
                    ((left_x, top_y), (right_x, top_y))
                } else {
                    ((right_x, top_y), (left_x, top_y))
                }
            }
            RankDir::RightToLeft => {
                if from_before_to {
                    ((right_x, top_y), (left_x, top_y))
                } else {
                    ((left_x, top_y), (right_x, top_y))
                }
            }
            RankDir::TopToBottom => {
                if from_before_to {
                    ((left_x, top_y), (left_x, bottom_y))
                } else {
                    ((left_x, bottom_y), (left_x, top_y))
                }
            }
            RankDir::BottomToTop => {
                if from_before_to {
                    ((left_x, bottom_y), (left_x, top_y))
                } else {
                    ((left_x, top_y), (left_x, bottom_y))
                }
            }
        };

        SpacerCoordinates {
            entry_x,
            entry_y,
            exit_x,
            exit_y,
        }
    }

    /// Computes the absolute bounding box extremes and center of a taffy
    /// node, walking up the tree to accumulate its absolute position.
    fn node_rect_compute(
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        taffy_node_id: taffy::NodeId,
    ) -> Option<NodeRect> {
        let layout = taffy_tree.layout(taffy_node_id).ok()?;
        let absolute_coordinates =
            TaffyNodeAbsoluteCoordinatesCalculator::calculate(taffy_tree, taffy_node_id, layout);

        let left_x = absolute_coordinates.x;
        let top_y = absolute_coordinates.y;
        let right_x = left_x + layout.size.width;
        let bottom_y = top_y + layout.size.height;
        let cx = left_x + layout.size.width / 2.0;
        let cy = top_y + layout.size.height / 2.0;

        Some(NodeRect {
            left_x,
            right_x,
            top_y,
            bottom_y,
            cx,
            cy,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A description box at `x: 10..30` (`left_x: 10, right_x: 30`),
    /// `y: 100..140` (`top_y: 100, bottom_y: 140`).
    fn node_rect() -> NodeRect {
        NodeRect {
            left_x: 10.0,
            right_x: 30.0,
            top_y: 100.0,
            bottom_y: 140.0,
            cx: 20.0,
            cy: 120.0,
        }
    }

    /// Every case returns `entry == exit`: the description box sits beside
    /// the edge's path rather than on a corridor it threads through, so
    /// there is exactly one waypoint, not a pass-through pair.
    fn assert_entry_eq_exit(spacer_coordinates: SpacerCoordinates) {
        assert_eq!(spacer_coordinates.entry_x, spacer_coordinates.exit_x);
        assert_eq!(spacer_coordinates.entry_y, spacer_coordinates.exit_y);
    }

    #[test]
    fn left_to_right_from_before_to_uses_left_x_and_top_y() {
        let spacer_coordinates = EdgeSpacerCoordinatesCalculator::description_contact_from_rect(
            RankDir::LeftToRight,
            &node_rect(),
            Ordering::Less,
        );
        assert_entry_eq_exit(spacer_coordinates);
        assert_eq!(10.0, spacer_coordinates.entry_x);
        assert_eq!(100.0, spacer_coordinates.entry_y);
    }

    #[test]
    fn left_to_right_from_after_to_uses_left_x_and_bottom_y() {
        let spacer_coordinates = EdgeSpacerCoordinatesCalculator::description_contact_from_rect(
            RankDir::LeftToRight,
            &node_rect(),
            Ordering::Greater,
        );
        assert_entry_eq_exit(spacer_coordinates);
        assert_eq!(10.0, spacer_coordinates.entry_x);
        assert_eq!(140.0, spacer_coordinates.entry_y);
    }

    /// `RightToLeft` flips the y choice relative to `LeftToRight`, but keeps
    /// the same fixed `x` side -- confirmed necessary by a real regression
    /// (`edge_ix_client_server__1` in `020_interaction_halo_with_labels.yaml`
    /// looped without this flip).
    #[test]
    fn right_to_left_from_before_to_uses_left_x_and_bottom_y() {
        let spacer_coordinates = EdgeSpacerCoordinatesCalculator::description_contact_from_rect(
            RankDir::RightToLeft,
            &node_rect(),
            Ordering::Less,
        );
        assert_entry_eq_exit(spacer_coordinates);
        assert_eq!(10.0, spacer_coordinates.entry_x);
        assert_eq!(140.0, spacer_coordinates.entry_y);
    }

    #[test]
    fn right_to_left_from_after_to_uses_left_x_and_top_y() {
        let spacer_coordinates = EdgeSpacerCoordinatesCalculator::description_contact_from_rect(
            RankDir::RightToLeft,
            &node_rect(),
            Ordering::Greater,
        );
        assert_entry_eq_exit(spacer_coordinates);
        assert_eq!(10.0, spacer_coordinates.entry_x);
        assert_eq!(100.0, spacer_coordinates.entry_y);
    }

    #[test]
    fn top_to_bottom_from_before_to_uses_top_y_and_left_x() {
        let spacer_coordinates = EdgeSpacerCoordinatesCalculator::description_contact_from_rect(
            RankDir::TopToBottom,
            &node_rect(),
            Ordering::Less,
        );
        assert_entry_eq_exit(spacer_coordinates);
        assert_eq!(100.0, spacer_coordinates.entry_y);
        assert_eq!(10.0, spacer_coordinates.entry_x);
    }

    #[test]
    fn top_to_bottom_from_after_to_uses_top_y_and_right_x() {
        let spacer_coordinates = EdgeSpacerCoordinatesCalculator::description_contact_from_rect(
            RankDir::TopToBottom,
            &node_rect(),
            Ordering::Greater,
        );
        assert_entry_eq_exit(spacer_coordinates);
        assert_eq!(100.0, spacer_coordinates.entry_y);
        assert_eq!(30.0, spacer_coordinates.entry_x);
    }

    /// `BottomToTop` flips the x choice relative to `TopToBottom`, but keeps
    /// the same fixed `y` side.
    #[test]
    fn bottom_to_top_from_before_to_uses_top_y_and_right_x() {
        let spacer_coordinates = EdgeSpacerCoordinatesCalculator::description_contact_from_rect(
            RankDir::BottomToTop,
            &node_rect(),
            Ordering::Less,
        );
        assert_entry_eq_exit(spacer_coordinates);
        assert_eq!(100.0, spacer_coordinates.entry_y);
        assert_eq!(30.0, spacer_coordinates.entry_x);
    }

    #[test]
    fn bottom_to_top_from_after_to_uses_top_y_and_left_x() {
        let spacer_coordinates = EdgeSpacerCoordinatesCalculator::description_contact_from_rect(
            RankDir::BottomToTop,
            &node_rect(),
            Ordering::Greater,
        );
        assert_entry_eq_exit(spacer_coordinates);
        assert_eq!(100.0, spacer_coordinates.entry_y);
        assert_eq!(10.0, spacer_coordinates.entry_x);
    }

    // === `description_thread_from_rect` tests (cross-rank edges) === //
    //
    // Unlike the same-rank `description_contact_from_rect` tests above,
    // entry and exit are expected to differ -- the box is threaded through,
    // not touched at a single point beside the path.

    #[test]
    fn left_to_right_from_before_to_threads_top_y_left_to_right() {
        let spacer_coordinates = EdgeSpacerCoordinatesCalculator::description_thread_from_rect(
            RankDir::LeftToRight,
            &node_rect(),
            Ordering::Less,
        );
        assert_eq!(10.0, spacer_coordinates.entry_x);
        assert_eq!(100.0, spacer_coordinates.entry_y);
        assert_eq!(30.0, spacer_coordinates.exit_x);
        assert_eq!(100.0, spacer_coordinates.exit_y);
    }

    #[test]
    fn left_to_right_from_after_to_threads_top_y_right_to_left() {
        let spacer_coordinates = EdgeSpacerCoordinatesCalculator::description_thread_from_rect(
            RankDir::LeftToRight,
            &node_rect(),
            Ordering::Greater,
        );
        assert_eq!(30.0, spacer_coordinates.entry_x);
        assert_eq!(100.0, spacer_coordinates.entry_y);
        assert_eq!(10.0, spacer_coordinates.exit_x);
        assert_eq!(100.0, spacer_coordinates.exit_y);
    }

    /// `RightToLeft` keeps the same fixed `top_y`, but flips the canonical
    /// entry/exit assignment relative to `LeftToRight` -- mirroring
    /// [`EdgeSpacerCoordinatesCalculator::calculate`]'s convention.
    #[test]
    fn right_to_left_from_before_to_threads_top_y_right_to_left() {
        let spacer_coordinates = EdgeSpacerCoordinatesCalculator::description_thread_from_rect(
            RankDir::RightToLeft,
            &node_rect(),
            Ordering::Less,
        );
        assert_eq!(30.0, spacer_coordinates.entry_x);
        assert_eq!(100.0, spacer_coordinates.entry_y);
        assert_eq!(10.0, spacer_coordinates.exit_x);
        assert_eq!(100.0, spacer_coordinates.exit_y);
    }

    #[test]
    fn right_to_left_from_after_to_threads_top_y_left_to_right() {
        let spacer_coordinates = EdgeSpacerCoordinatesCalculator::description_thread_from_rect(
            RankDir::RightToLeft,
            &node_rect(),
            Ordering::Greater,
        );
        assert_eq!(10.0, spacer_coordinates.entry_x);
        assert_eq!(100.0, spacer_coordinates.entry_y);
        assert_eq!(30.0, spacer_coordinates.exit_x);
        assert_eq!(100.0, spacer_coordinates.exit_y);
    }

    #[test]
    fn top_to_bottom_from_before_to_threads_left_x_top_to_bottom() {
        let spacer_coordinates = EdgeSpacerCoordinatesCalculator::description_thread_from_rect(
            RankDir::TopToBottom,
            &node_rect(),
            Ordering::Less,
        );
        assert_eq!(10.0, spacer_coordinates.entry_x);
        assert_eq!(100.0, spacer_coordinates.entry_y);
        assert_eq!(10.0, spacer_coordinates.exit_x);
        assert_eq!(140.0, spacer_coordinates.exit_y);
    }

    #[test]
    fn top_to_bottom_from_after_to_threads_left_x_bottom_to_top() {
        let spacer_coordinates = EdgeSpacerCoordinatesCalculator::description_thread_from_rect(
            RankDir::TopToBottom,
            &node_rect(),
            Ordering::Greater,
        );
        assert_eq!(10.0, spacer_coordinates.entry_x);
        assert_eq!(140.0, spacer_coordinates.entry_y);
        assert_eq!(10.0, spacer_coordinates.exit_x);
        assert_eq!(100.0, spacer_coordinates.exit_y);
    }

    /// `BottomToTop` keeps the same fixed `left_x`, but flips the canonical
    /// entry/exit assignment relative to `TopToBottom`.
    #[test]
    fn bottom_to_top_from_before_to_threads_left_x_bottom_to_top() {
        let spacer_coordinates = EdgeSpacerCoordinatesCalculator::description_thread_from_rect(
            RankDir::BottomToTop,
            &node_rect(),
            Ordering::Less,
        );
        assert_eq!(10.0, spacer_coordinates.entry_x);
        assert_eq!(140.0, spacer_coordinates.entry_y);
        assert_eq!(10.0, spacer_coordinates.exit_x);
        assert_eq!(100.0, spacer_coordinates.exit_y);
    }

    #[test]
    fn bottom_to_top_from_after_to_threads_left_x_top_to_bottom() {
        let spacer_coordinates = EdgeSpacerCoordinatesCalculator::description_thread_from_rect(
            RankDir::BottomToTop,
            &node_rect(),
            Ordering::Greater,
        );
        assert_eq!(10.0, spacer_coordinates.entry_x);
        assert_eq!(100.0, spacer_coordinates.entry_y);
        assert_eq!(10.0, spacer_coordinates.exit_x);
        assert_eq!(140.0, spacer_coordinates.exit_y);
    }
}
