//! Thing layout page mutation helpers.
//!
//! This module is a thin Signal-aware wrapper around
//! [`disposition_input_rt::thing_layout_ops::ThingLayoutOps`]. Each method
//! acquires a write guard on the [`Signal`] and delegates to the
//! framework-agnostic implementation.

use dioxus::signals::{Signal, WritableExt};
use disposition::input_model::InputDiagram;

/// Mutation operations for the Thing Layout editor page.
///
/// All methods operate on the flattened representation of the
/// [`ThingHierarchy`](disposition::input_model::thing::ThingHierarchy) and
/// rebuild the tree after each mutation.
pub struct ThingLayoutOps;

impl ThingLayoutOps {
    /// Move the entry at `index` up within the flattened hierarchy.
    ///
    /// Behaviour:
    /// - If the entry is the first sibling at its depth, it becomes a sibling
    ///   of the parent (moves one level shallower) and is placed just before
    ///   the parent.
    /// - Otherwise it swaps position with its previous sibling (including any
    ///   descendants of that sibling).
    ///
    /// Returns the new flat index of the moved entry, or `None` if no
    /// move was performed.
    pub fn entry_move_up(
        mut input_diagram: Signal<InputDiagram<'static>>,
        index: usize,
    ) -> Option<usize> {
        disposition_input_rt::ThingLayoutOps::entry_move_up(&mut input_diagram.write(), index)
    }

    /// Move the entry at `index` down within the flattened hierarchy.
    ///
    /// Behaviour:
    /// - If the entry is the last sibling at its depth, it becomes a sibling of
    ///   the parent (moves one level shallower) and is placed just after the
    ///   parent's subtree.
    /// - Otherwise it swaps position with its next sibling (including any
    ///   descendants of that sibling).
    ///
    /// Returns the new flat index of the moved entry, or `None` if no
    /// move was performed.
    pub fn entry_move_down(
        mut input_diagram: Signal<InputDiagram<'static>>,
        index: usize,
    ) -> Option<usize> {
        disposition_input_rt::ThingLayoutOps::entry_move_down(&mut input_diagram.write(), index)
    }

    /// Indent the entry at `index` (increase nesting depth by 1).
    ///
    /// The entry becomes a child of the previous sibling at its current
    /// depth. If there is no previous sibling, this is a no-op.
    ///
    /// Returns the new flat index of the entry, or `None` if no indent
    /// was performed. The flat index does not change on indent.
    pub fn entry_indent(
        mut input_diagram: Signal<InputDiagram<'static>>,
        index: usize,
    ) -> Option<usize> {
        disposition_input_rt::ThingLayoutOps::entry_indent(&mut input_diagram.write(), index)
    }

    /// Outdent the entry at `index` (decrease nesting depth by 1).
    ///
    /// The entry and its subtree move out of their parent and become a
    /// sibling of the parent, placed immediately after the parent's
    /// subtree. Following siblings within the same parent stay as
    /// children of the parent.
    ///
    /// If the entry is already at the top level (`depth == 0`), this is a
    /// no-op.
    /// Returns the new flat index of the moved entry, or `None` if no
    /// outdent was performed.
    pub fn entry_outdent(
        mut input_diagram: Signal<InputDiagram<'static>>,
        index: usize,
    ) -> Option<usize> {
        disposition_input_rt::ThingLayoutOps::entry_outdent(&mut input_diagram.write(), index)
    }

    /// Moves a dragged entry from flat index `from` to flat index `to`.
    ///
    /// The entry (and its subtree) is removed from its current position and
    /// inserted at the target position, adopting the depth of the target
    /// location (same depth as the entry currently at `to`, or the depth of
    /// the entry just before `to` + 1 if `to` is past the end).
    pub fn entry_drag_move(
        mut input_diagram: Signal<InputDiagram<'static>>,
        from: usize,
        to: usize,
    ) {
        disposition_input_rt::ThingLayoutOps::entry_drag_move(&mut input_diagram.write(), from, to);
    }
}
