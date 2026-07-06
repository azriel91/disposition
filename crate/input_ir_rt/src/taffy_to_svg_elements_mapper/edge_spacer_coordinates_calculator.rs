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
    /// [`Self::calculate_description_thread`] /
    /// [`Self::calculate_description_thread_same_rank`] instead, which are
    /// biased along the cross axis by `sibling_index_from_cmp_to` rather
    /// than centered on it.
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

    /// Computes the routing waypoint pair (`entry != exit`) for a
    /// **cross-rank** edge's own `edge_description_container` leaf.
    ///
    /// Unlike [`Self::calculate_description_thread_same_rank`] (used for
    /// same-rank cycle edges, whose description box sits between two
    /// divergent ancestors laid out side by side *within* their shared
    /// rank), a cross-rank edge's description box sits directly on the rank
    /// corridor *between ranks*, so the path should thread *through*
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
        interaction_edge_halo_stroke_width: f32,
    ) -> Option<SpacerCoordinates> {
        let node_rect = Self::node_rect_compute(taffy_tree, taffy_node_id)?;
        Some(Self::description_thread_from_rect(
            rank_dir,
            &node_rect,
            sibling_index_from_cmp_to,
            interaction_edge_halo_stroke_width,
        ))
    }

    /// Pure coordinate selection for [`Self::calculate_description_thread`],
    /// separated from taffy tree access so the `RankDir` x `Ordering` table
    /// can be unit tested directly against a constructed [`NodeRect`].
    ///
    /// `interaction_edge_halo_stroke_width` pulls the fixed-axis coordinate
    /// back by half its value (`halo_pad_px`), mirroring
    /// `SvgEdgeInfosBuilder::label_face_span_compute`'s `- halo_pad_px`
    /// pullback for edge labels: `EdgeDescriptionBuilder::edge_desc_build`
    /// gives the description box a matching `halo_pad_px` margin on the same
    /// fixed-axis side, so the pullback here cancels that margin's push for
    /// routing purposes, keeping the path pinned at the box's pre-margin
    /// position while the rendered box (and its content) has physically
    /// moved away -- opening real clearance for the halo. Only the fixed
    /// axis moves; the free axis (spanning the box's own width/height) is
    /// untouched.
    fn description_thread_from_rect(
        rank_dir: RankDir,
        node_rect: &NodeRect,
        sibling_index_from_cmp_to: Ordering,
        interaction_edge_halo_stroke_width: f32,
    ) -> SpacerCoordinates {
        let NodeRect {
            left_x,
            right_x,
            top_y,
            bottom_y,
            ..
        } = *node_rect;
        let from_before_to = sibling_index_from_cmp_to == Ordering::Less;
        let halo_pad_px = interaction_edge_halo_stroke_width / 2.0;

        let ((entry_x, entry_y), (exit_x, exit_y)) = match rank_dir {
            RankDir::LeftToRight => {
                let top_y = top_y - halo_pad_px;
                if from_before_to {
                    ((left_x, top_y), (right_x, top_y))
                } else {
                    ((right_x, top_y), (left_x, top_y))
                }
            }
            RankDir::RightToLeft => {
                let top_y = top_y - halo_pad_px;
                if from_before_to {
                    ((right_x, top_y), (left_x, top_y))
                } else {
                    ((left_x, top_y), (right_x, top_y))
                }
            }
            RankDir::TopToBottom => {
                let left_x = left_x - halo_pad_px;
                if from_before_to {
                    ((left_x, top_y), (left_x, bottom_y))
                } else {
                    ((left_x, bottom_y), (left_x, top_y))
                }
            }
            RankDir::BottomToTop => {
                let left_x = left_x - halo_pad_px;
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

    /// Computes the routing waypoint pair (`entry != exit`) for a
    /// **same-rank** (cycle edge) edge's own `edge_description_container`
    /// leaf.
    ///
    /// A same-rank edge's divergent ancestors are laid out side by side
    /// *within* their shared rank -- horizontally when the rank's own
    /// children stack via `Row`/`RowReverse` (`RankDir::TopToBottom`/
    /// `BottomToTop`), vertically when they stack via `Column`/
    /// `ColumnReverse` (`RankDir::LeftToRight`/`RightToLeft`). The
    /// description box sits directly between them, on that within-rank
    /// axis, so (like [`Self::calculate_description_thread`]'s cross-rank
    /// corridor) the path should thread *through* it rather than touch a
    /// single point beside it.
    ///
    /// Because within-rank sibling order always matches declaration order
    /// regardless of `RankDir`'s forward/reverse convention (see
    /// `edge_paths.md` -- Sibling order for reversed rank directions),
    /// `Ordering::Less`/`Greater` here means the same thing (`from`'s
    /// divergent ancestor sits earlier/later along the shared rank) for
    /// both members of a forward/reverse `RankDir` pair -- unlike
    /// [`Self::calculate_description_thread`], where the physical meaning
    /// of `Less`/`Greater` flips between a `RankDir` and its reverse pair.
    /// Only the horizontal-vs-vertical layout axis depends on `RankDir`.
    ///
    /// This reuses [`Self::description_thread_from_rect`]'s table by
    /// rotating `rank_dir` onto whichever of its two canonical rows matches
    /// the axis same-rank siblings are actually laid out on:
    /// `TopToBottom`/`BottomToTop` (horizontal siblings) both use the
    /// `LeftToRight` row (fixed `y = top_y`); `LeftToRight`/`RightToLeft`
    /// (vertical siblings) both use the `TopToBottom` row (fixed
    /// `x = left_x`).
    ///
    /// # Example values
    ///
    /// `rank_dir = TopToBottom`, box `left_x = 132, right_x = 290,
    /// top_y = 25`, `sibling_index_from_cmp_to = Ordering::Less` -- rotates
    /// to the `LeftToRight` row, returning `Some(SpacerCoordinates {
    /// entry_x: 132.0, entry_y: 25.0, exit_x: 290.0, exit_y: 25.0 })`.
    pub fn calculate_description_thread_same_rank(
        rank_dir: RankDir,
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        taffy_node_id: taffy::NodeId,
        sibling_index_from_cmp_to: Ordering,
        interaction_edge_halo_stroke_width: f32,
    ) -> Option<SpacerCoordinates> {
        let node_rect = Self::node_rect_compute(taffy_tree, taffy_node_id)?;
        Some(Self::description_thread_from_rect(
            Self::rank_dir_same_rank_rotate(rank_dir),
            &node_rect,
            sibling_index_from_cmp_to,
            interaction_edge_halo_stroke_width,
        ))
    }

    /// Rotates `rank_dir` onto the canonical `RankDir` whose
    /// [`Self::description_thread_from_rect`] row matches the axis
    /// same-rank siblings are laid out on, for
    /// [`Self::calculate_description_thread_same_rank`].
    ///
    /// `TopToBottom`/`BottomToTop` (horizontal within-rank siblings) both
    /// rotate to `LeftToRight`; `LeftToRight`/`RightToLeft` (vertical
    /// within-rank siblings) both rotate to `TopToBottom`. Separated from
    /// [`Self::calculate_description_thread_same_rank`] so the mapping can be
    /// unit tested without constructing a taffy tree.
    ///
    /// Also reused by `EdgeDescriptionBuilder::edge_desc_build` (a sibling
    /// module, not a descendant of this one -- re-exported `pub(crate)` from
    /// `taffy_to_svg_elements_mapper` for this) to choose which side of an
    /// edge description's halo-clearance margin to apply, so the build-time
    /// margin side and the routing-time pullback axis
    /// (`description_thread_from_rect`) can't drift apart.
    pub(crate) fn rank_dir_same_rank_rotate(rank_dir: RankDir) -> RankDir {
        match rank_dir {
            RankDir::TopToBottom | RankDir::BottomToTop => RankDir::LeftToRight,
            RankDir::LeftToRight | RankDir::RightToLeft => RankDir::TopToBottom,
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

    // === `description_thread_from_rect` tests (cross-rank edges) === //

    #[test]
    fn left_to_right_from_before_to_threads_top_y_left_to_right() {
        let spacer_coordinates = EdgeSpacerCoordinatesCalculator::description_thread_from_rect(
            RankDir::LeftToRight,
            &node_rect(),
            Ordering::Less,
            0.0,
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
            0.0,
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
            0.0,
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
            0.0,
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
            0.0,
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
            0.0,
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
            0.0,
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
            0.0,
        );
        assert_eq!(10.0, spacer_coordinates.entry_x);
        assert_eq!(100.0, spacer_coordinates.entry_y);
        assert_eq!(10.0, spacer_coordinates.exit_x);
        assert_eq!(140.0, spacer_coordinates.exit_y);
    }

    // === `description_thread_from_rect` halo-pullback tests === //

    /// `LeftToRight` fixed axis is `y = top_y`; a non-zero
    /// `interaction_edge_halo_stroke_width` pulls `top_y` back by
    /// `halo_pad_px`, leaving the free axis (`left_x`/`right_x`) untouched.
    #[test]
    fn left_to_right_pulls_back_top_y_by_half_halo_stroke_width() {
        let spacer_coordinates = EdgeSpacerCoordinatesCalculator::description_thread_from_rect(
            RankDir::LeftToRight,
            &node_rect(),
            Ordering::Less,
            8.0,
        );
        assert_eq!(10.0, spacer_coordinates.entry_x);
        assert_eq!(96.0, spacer_coordinates.entry_y);
        assert_eq!(30.0, spacer_coordinates.exit_x);
        assert_eq!(96.0, spacer_coordinates.exit_y);
    }

    /// `TopToBottom` fixed axis is `x = left_x`; a non-zero
    /// `interaction_edge_halo_stroke_width` pulls `left_x` back by
    /// `halo_pad_px`, leaving the free axis (`top_y`/`bottom_y`) untouched.
    #[test]
    fn top_to_bottom_pulls_back_left_x_by_half_halo_stroke_width() {
        let spacer_coordinates = EdgeSpacerCoordinatesCalculator::description_thread_from_rect(
            RankDir::TopToBottom,
            &node_rect(),
            Ordering::Less,
            8.0,
        );
        assert_eq!(6.0, spacer_coordinates.entry_x);
        assert_eq!(100.0, spacer_coordinates.entry_y);
        assert_eq!(6.0, spacer_coordinates.exit_x);
        assert_eq!(140.0, spacer_coordinates.exit_y);
    }

    // === `rank_dir_same_rank_rotate` tests (same-rank edges) === //

    #[test]
    fn same_rank_rotate_top_to_bottom_to_left_to_right() {
        assert_eq!(
            RankDir::LeftToRight,
            EdgeSpacerCoordinatesCalculator::rank_dir_same_rank_rotate(RankDir::TopToBottom)
        );
    }

    #[test]
    fn same_rank_rotate_bottom_to_top_to_left_to_right() {
        assert_eq!(
            RankDir::LeftToRight,
            EdgeSpacerCoordinatesCalculator::rank_dir_same_rank_rotate(RankDir::BottomToTop)
        );
    }

    #[test]
    fn same_rank_rotate_left_to_right_to_top_to_bottom() {
        assert_eq!(
            RankDir::TopToBottom,
            EdgeSpacerCoordinatesCalculator::rank_dir_same_rank_rotate(RankDir::LeftToRight)
        );
    }

    #[test]
    fn same_rank_rotate_right_to_left_to_top_to_bottom() {
        assert_eq!(
            RankDir::TopToBottom,
            EdgeSpacerCoordinatesCalculator::rank_dir_same_rank_rotate(RankDir::RightToLeft)
        );
    }
}
