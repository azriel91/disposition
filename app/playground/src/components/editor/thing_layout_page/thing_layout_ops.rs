//! Thing layout page mutation helpers.
//!
//! Provides [`ThingLayoutOps`] which groups all mutation operations for the
//! Thing Layout editor page so that related functions are discoverable when
//! sorted by name, per the project's `noun_verb` naming convention.

use dioxus::signals::{ReadableExt, Signal, WritableExt};
use disposition::input_model::InputDiagram;

use super::flat_entry::{hierarchy_flatten, hierarchy_rebuild, FlatEntry};

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
    pub fn entry_move_up(mut input_diagram: Signal<InputDiagram<'static>>, index: usize) {
        let mut entries = {
            let diagram = input_diagram.read();
            hierarchy_flatten(&diagram.thing_hierarchy)
        };

        if index == 0 || index >= entries.len() {
            return;
        }

        let depth = entries[index].depth;

        // Find the previous sibling at the same depth.
        if let Some(prev_sibling_idx) = Self::prev_sibling_index(&entries, index) {
            // Swap the two subtrees: previous sibling subtree and current
            // entry subtree.
            let prev_subtree_end = index; // exclusive
            let prev_subtree_start = prev_sibling_idx;

            let cur_subtree_start = index;
            let cur_subtree_end = Self::subtree_end(&entries, index);

            let prev_subtree: Vec<FlatEntry> =
                entries[prev_subtree_start..prev_subtree_end].to_vec();
            let cur_subtree: Vec<FlatEntry> = entries[cur_subtree_start..cur_subtree_end].to_vec();

            // Replace both subtrees: put current before previous.
            let mut new_entries = Vec::with_capacity(entries.len());
            new_entries.extend_from_slice(&entries[..prev_subtree_start]);
            new_entries.extend(cur_subtree);
            new_entries.extend(prev_subtree);
            new_entries.extend_from_slice(&entries[cur_subtree_end..]);

            entries = new_entries;
        } else {
            // No previous sibling: reparent one level up (become sibling of
            // parent). Only possible if depth > 0.
            if depth == 0 {
                return;
            }

            // Find the parent entry.
            let parent_idx = match Self::parent_index(&entries, index) {
                Some(idx) => idx,
                None => return,
            };

            let cur_subtree_start = index;
            let cur_subtree_end = Self::subtree_end(&entries, index);

            // Extract the current subtree and decrease depth by 1.
            let mut cur_subtree: Vec<FlatEntry> =
                entries[cur_subtree_start..cur_subtree_end].to_vec();
            for entry in &mut cur_subtree {
                entry.depth -= 1;
            }

            // Remove from old position and insert before the parent.
            let mut new_entries = Vec::with_capacity(entries.len());
            new_entries.extend_from_slice(&entries[..parent_idx]);
            new_entries.extend(cur_subtree);
            new_entries.extend_from_slice(&entries[parent_idx..cur_subtree_start]);
            new_entries.extend_from_slice(&entries[cur_subtree_end..]);

            entries = new_entries;
        }

        input_diagram.write().thing_hierarchy = hierarchy_rebuild(&entries);
    }

    /// Move the entry at `index` down within the flattened hierarchy.
    ///
    /// Behaviour:
    /// - If the entry is the last sibling at its depth, it becomes a sibling of
    ///   the parent (moves one level shallower) and is placed just after the
    ///   parent's subtree.
    /// - Otherwise it swaps position with its next sibling (including any
    ///   descendants of that sibling).
    pub fn entry_move_down(mut input_diagram: Signal<InputDiagram<'static>>, index: usize) {
        let mut entries = {
            let diagram = input_diagram.read();
            hierarchy_flatten(&diagram.thing_hierarchy)
        };

        if index >= entries.len() {
            return;
        }

        let depth = entries[index].depth;

        // Find the next sibling at the same depth.
        if let Some(next_sibling_idx) = Self::next_sibling_index(&entries, index) {
            // Swap the two subtrees: current entry subtree and next sibling
            // subtree.
            let cur_subtree_start = index;
            let cur_subtree_end = next_sibling_idx;

            let next_subtree_start = next_sibling_idx;
            let next_subtree_end = Self::subtree_end(&entries, next_sibling_idx);

            let cur_subtree: Vec<FlatEntry> = entries[cur_subtree_start..cur_subtree_end].to_vec();
            let next_subtree: Vec<FlatEntry> =
                entries[next_subtree_start..next_subtree_end].to_vec();

            // Replace both subtrees: put next before current.
            let mut new_entries = Vec::with_capacity(entries.len());
            new_entries.extend_from_slice(&entries[..cur_subtree_start]);
            new_entries.extend(next_subtree);
            new_entries.extend(cur_subtree);
            new_entries.extend_from_slice(&entries[next_subtree_end..]);

            entries = new_entries;
        } else {
            // No next sibling: reparent one level up (become sibling of
            // parent), placed after the parent's full subtree.
            // Only possible if depth > 0.
            if depth == 0 {
                return;
            }

            let parent_idx = match Self::parent_index(&entries, index) {
                Some(idx) => idx,
                None => return,
            };

            let parent_subtree_end = Self::subtree_end(&entries, parent_idx);

            let cur_subtree_start = index;
            let cur_subtree_end = Self::subtree_end(&entries, index);

            // Extract the current subtree and decrease depth by 1.
            let mut cur_subtree: Vec<FlatEntry> =
                entries[cur_subtree_start..cur_subtree_end].to_vec();
            for entry in &mut cur_subtree {
                entry.depth -= 1;
            }

            // Remove from old position and insert after parent subtree.
            let mut new_entries = Vec::with_capacity(entries.len());
            new_entries.extend_from_slice(&entries[..cur_subtree_start]);
            new_entries.extend_from_slice(&entries[cur_subtree_end..parent_subtree_end]);
            new_entries.extend(cur_subtree);
            new_entries.extend_from_slice(&entries[parent_subtree_end..]);

            entries = new_entries;
        }

        input_diagram.write().thing_hierarchy = hierarchy_rebuild(&entries);
    }

    /// Indent the entry at `index` (increase nesting depth by 1).
    ///
    /// The entry becomes a child of the previous sibling at its current
    /// depth. If there is no previous sibling, this is a no-op.
    pub fn entry_indent(mut input_diagram: Signal<InputDiagram<'static>>, index: usize) {
        let mut entries = {
            let diagram = input_diagram.read();
            hierarchy_flatten(&diagram.thing_hierarchy)
        };

        if index >= entries.len() {
            return;
        }

        // Must have a previous sibling to become a child of.
        if Self::prev_sibling_index(&entries, index).is_none() {
            return;
        }

        let cur_subtree_end = Self::subtree_end(&entries, index);

        // Increase depth for the entry and all its descendants.
        for entry in &mut entries[index..cur_subtree_end] {
            entry.depth += 1;
        }

        input_diagram.write().thing_hierarchy = hierarchy_rebuild(&entries);
    }

    /// Outdent the entry at `index` (decrease nesting depth by 1).
    ///
    /// The entry and its subtree move out of their parent and become a
    /// sibling of the parent. Any siblings that follow this entry within the
    /// same parent become children of this entry (they are "adopted").
    ///
    /// If the entry is already at the top level (`depth == 0`), this is a
    /// no-op.
    pub fn entry_outdent(mut input_diagram: Signal<InputDiagram<'static>>, index: usize) {
        let mut entries = {
            let diagram = input_diagram.read();
            hierarchy_flatten(&diagram.thing_hierarchy)
        };

        if index >= entries.len() || entries[index].depth == 0 {
            return;
        }

        let parent_idx = match Self::parent_index(&entries, index) {
            Some(idx) => idx,
            None => return,
        };

        let parent_subtree_end = Self::subtree_end(&entries, parent_idx);
        let cur_subtree_end = Self::subtree_end(&entries, index);

        // The "following siblings" within the parent become children of the
        // entry being outdented. Their depth stays the same relative to
        // the current entry (effectively they stay at current depth, which
        // is now one level deeper relative to the outdented entry's new
        // depth -- which is exactly what we want).

        // Decrease depth for the entry and its current descendants.
        for entry in &mut entries[index..cur_subtree_end] {
            entry.depth -= 1;
        }

        // Move following siblings (cur_subtree_end..parent_subtree_end) so
        // they remain as children -- they keep their depth, which is now
        // correct because the entry moved one level shallower and the
        // followers are one level deeper relative to it.

        // Extract the following siblings block.
        let following: Vec<FlatEntry> = entries[cur_subtree_end..parent_subtree_end].to_vec();

        // Remove from old position and reinsert after the current subtree
        // (which is now at parent level).
        let mut new_entries = Vec::with_capacity(entries.len());
        new_entries.extend_from_slice(&entries[..cur_subtree_end]);
        // skip `following` at old position
        new_entries.extend_from_slice(&entries[parent_subtree_end..]);

        // Insert the following block right after the current subtree
        // (at position cur_subtree_end).
        let insert_pos = cur_subtree_end;
        for (i, entry) in following.into_iter().enumerate() {
            new_entries.insert(insert_pos + i, entry);
        }

        entries = new_entries;

        input_diagram.write().thing_hierarchy = hierarchy_rebuild(&entries);
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
        if from == to {
            return;
        }

        let mut entries = {
            let diagram = input_diagram.read();
            hierarchy_flatten(&diagram.thing_hierarchy)
        };

        if from >= entries.len() || to >= entries.len() {
            return;
        }

        let from_subtree_end = Self::subtree_end(&entries, from);
        // Don't allow dropping onto own subtree.
        if to >= from && to < from_subtree_end {
            return;
        }

        let target_depth = entries[to].depth;
        let source_depth = entries[from].depth;
        let depth_delta = target_depth as isize - source_depth as isize;

        // Extract the subtree.
        let mut subtree: Vec<FlatEntry> = entries[from..from_subtree_end].to_vec();
        for entry in &mut subtree {
            entry.depth = (entry.depth as isize + depth_delta).max(0) as usize;
        }

        // Remove the subtree from entries.
        entries.drain(from..from_subtree_end);

        // Adjust `to` index after removal.
        let adjusted_to = if to > from {
            to - (from_subtree_end - from)
        } else {
            to
        };

        // Insert subtree at the adjusted position.
        for (i, entry) in subtree.into_iter().enumerate() {
            entries.insert(adjusted_to + i, entry);
        }

        input_diagram.write().thing_hierarchy = hierarchy_rebuild(&entries);
    }

    // === Helper functions === //

    /// Returns the exclusive end index of the subtree rooted at `index`.
    ///
    /// The subtree includes `entries[index]` and all following entries with
    /// a strictly greater depth.
    fn subtree_end(entries: &[FlatEntry], index: usize) -> usize {
        let depth = entries[index].depth;
        entries[index + 1..]
            .iter()
            .position(|e| e.depth <= depth)
            .map(|pos| index + 1 + pos)
            .unwrap_or(entries.len())
    }

    /// Returns the index of the previous sibling (same depth, within the
    /// same parent) of the entry at `index`, or `None` if this is the first
    /// sibling.
    fn prev_sibling_index(entries: &[FlatEntry], index: usize) -> Option<usize> {
        let depth = entries[index].depth;
        // Walk backwards looking for an entry at the same depth.
        // If we hit an entry at a shallower depth first, there is no previous
        // sibling.
        for i in (0..index).rev() {
            if entries[i].depth == depth {
                return Some(i);
            }
            if entries[i].depth < depth {
                return None;
            }
        }
        None
    }

    /// Returns the index of the next sibling (same depth, within the same
    /// parent) of the entry at `index`, or `None` if this is the last
    /// sibling.
    fn next_sibling_index(entries: &[FlatEntry], index: usize) -> Option<usize> {
        let depth = entries[index].depth;
        let subtree_end = Self::subtree_end(entries, index);
        if subtree_end < entries.len() && entries[subtree_end].depth == depth {
            Some(subtree_end)
        } else {
            None
        }
    }

    /// Returns the index of the parent entry (nearest preceding entry with
    /// `depth == current_depth - 1`), or `None` if the entry is at the
    /// top level.
    fn parent_index(entries: &[FlatEntry], index: usize) -> Option<usize> {
        if entries[index].depth == 0 {
            return None;
        }
        let parent_depth = entries[index].depth - 1;
        (0..index).rev().find(|&i| entries[i].depth == parent_depth)
    }
}
