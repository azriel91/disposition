//! Undo / redo history for the playground editor.
//!
//! [`UndoHistory`] maintains a stack of [`InputDiagram`] snapshots and a
//! cursor pointing at the "current" entry. Pushing a new snapshot discards
//! any redo entries beyond the cursor.
//!
//! The history is stored inside a Dioxus [`Signal`] so that it can be shared
//! via the context API and mutated from any component.
//!
//! ## Capacity
//!
//! The history keeps at most [`UndoHistory::MAX_ENTRIES`] snapshots. When
//! the limit is reached the oldest entry is dropped.

use dioxus::signals::{Signal, WritableExt};
use disposition::input_model::InputDiagram;

/// Maximum number of snapshots retained in the undo history.
const MAX_ENTRIES: usize = 200;

/// Undo / redo history for [`InputDiagram`] snapshots.
///
/// # Fields
///
/// * `entries` -- the ordered list of snapshots, oldest first.
/// * `cursor` -- index into `entries` that represents the *current* state. Undo
///   moves the cursor backwards; redo moves it forwards.
/// * `skip_next_push` -- flag used to prevent the memo that watches the
///   `input_diagram` signal from recording a snapshot when the change was
///   caused by an undo/redo operation itself.
#[derive(Clone, Debug)]
pub struct UndoHistory {
    entries: Vec<InputDiagram<'static>>,
    cursor: usize,
    skip_next_push: bool,
}

impl UndoHistory {
    /// Maximum number of snapshots retained.
    #[allow(dead_code)]
    pub const MAX_ENTRIES: usize = MAX_ENTRIES;

    /// Create a new history seeded with the given initial diagram.
    pub fn new(initial: InputDiagram<'static>) -> Self {
        Self {
            entries: vec![initial],
            cursor: 0,
            skip_next_push: false,
        }
    }

    /// Push a new snapshot onto the history.
    ///
    /// * Duplicate pushes (where the new diagram equals the current one) are
    ///   silently ignored.
    /// * Any redo entries beyond the cursor are discarded.
    /// * When the history exceeds [`MAX_ENTRIES`] the oldest entry is dropped.
    pub fn push(&mut self, diagram: InputDiagram<'static>) {
        // If the `skip_next_push` flag is set, clear it and return -- the
        // change was triggered by an undo/redo operation.
        if self.skip_next_push {
            self.skip_next_push = false;
            return;
        }

        // Ignore duplicate pushes.
        if self.entries.get(self.cursor) == Some(&diagram) {
            return;
        }

        // Discard any redo entries beyond the cursor.
        self.entries.truncate(self.cursor + 1);

        // Append the new snapshot.
        self.entries.push(diagram);
        self.cursor = self.entries.len() - 1;

        // Enforce capacity limit.
        if self.entries.len() > MAX_ENTRIES {
            let excess = self.entries.len() - MAX_ENTRIES;
            self.entries.drain(..excess);
            // `cursor` was pointing at the last element which is still valid
            // after the drain, but we need to adjust the index.
            self.cursor = self.entries.len() - 1;
        }
    }

    /// Whether an undo operation is available.
    pub fn can_undo(&self) -> bool {
        self.cursor > 0
    }

    /// Whether a redo operation is available.
    pub fn can_redo(&self) -> bool {
        self.cursor + 1 < self.entries.len()
    }

    /// Move the cursor one step back and return the diagram at that position.
    ///
    /// Returns `None` if already at the oldest entry.
    pub fn undo(&mut self) -> Option<&InputDiagram<'static>> {
        if self.can_undo() {
            self.cursor -= 1;
            self.skip_next_push = true;
            Some(&self.entries[self.cursor])
        } else {
            None
        }
    }

    /// Move the cursor one step forward and return the diagram at that
    /// position.
    ///
    /// Returns `None` if already at the newest entry.
    pub fn redo(&mut self) -> Option<&InputDiagram<'static>> {
        if self.can_redo() {
            self.cursor += 1;
            self.skip_next_push = true;
            Some(&self.entries[self.cursor])
        } else {
            None
        }
    }

    /// The current snapshot (the one the cursor points at).
    #[allow(dead_code)]
    pub fn current(&self) -> &InputDiagram<'static> {
        &self.entries[self.cursor]
    }

    /// Number of undo steps available (steps before the cursor).
    pub fn undo_depth(&self) -> usize {
        self.cursor
    }

    /// Number of redo steps available (steps after the cursor).
    pub fn redo_depth(&self) -> usize {
        self.entries.len() - 1 - self.cursor
    }
}

// === Signal helper functions === //

/// Perform an undo on the history signal, returning the restored diagram
/// (if any).
///
/// This is a convenience wrapper so callers don't need to manually
/// `.write()` the signal.
pub fn history_undo(mut history: Signal<UndoHistory>) -> Option<InputDiagram<'static>> {
    let mut h = history.write();
    h.undo().cloned()
}

/// Perform a redo on the history signal, returning the restored diagram
/// (if any).
pub fn history_redo(mut history: Signal<UndoHistory>) -> Option<InputDiagram<'static>> {
    let mut h = history.write();
    h.redo().cloned()
}

/// Push a new snapshot onto the history signal.
pub fn history_push(mut history: Signal<UndoHistory>, diagram: InputDiagram<'static>) {
    history.write().push(diagram);
}

// === Tests === //

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create a minimal `InputDiagram` that is distinguishable by
    /// its serialised YAML (we just check equality).
    fn make_diagram(label: &str) -> InputDiagram<'static> {
        let yaml = format!("things:\n  {label}: {label}");
        serde_saphyr::from_str(&yaml).expect("valid yaml")
    }

    #[test]
    fn new_history_has_one_entry() {
        let h = UndoHistory::new(InputDiagram::default());
        assert_eq!(h.undo_depth(), 0);
        assert_eq!(h.redo_depth(), 0);
        assert!(!h.can_undo());
        assert!(!h.can_redo());
    }

    #[test]
    fn push_and_undo() {
        let d0 = make_diagram("a");
        let d1 = make_diagram("b");

        let mut h = UndoHistory::new(d0.clone());
        h.push(d1.clone());

        assert!(h.can_undo());
        assert!(!h.can_redo());

        let undone = h.undo().cloned();
        assert_eq!(undone.as_ref(), Some(&d0));
        assert_eq!(h.current(), &d0);

        assert!(!h.can_undo());
        assert!(h.can_redo());
    }

    #[test]
    fn redo_after_undo() {
        let d0 = make_diagram("a");
        let d1 = make_diagram("b");

        let mut h = UndoHistory::new(d0.clone());
        h.push(d1.clone());
        h.undo();

        let redone = h.redo().cloned();
        assert_eq!(redone.as_ref(), Some(&d1));
        assert_eq!(h.current(), &d1);
    }

    #[test]
    fn push_after_undo_discards_redo() {
        let d0 = make_diagram("a");
        let d1 = make_diagram("b");
        let d2 = make_diagram("c");

        let mut h = UndoHistory::new(d0.clone());
        h.push(d1.clone());
        h.undo(); // cursor at d0, skip_next_push = true

        // Simulate the memo firing after undo: it pushes the undone diagram
        // which clears the skip_next_push flag without adding an entry.
        h.push(d0.clone());

        // Now a genuine new user edit arrives.
        h.push(d2.clone()); // d1 should be gone

        assert!(!h.can_redo());
        assert!(h.can_undo());

        let undone = h.undo().cloned();
        assert_eq!(undone.as_ref(), Some(&d0));
    }

    #[test]
    fn duplicate_push_is_ignored() {
        let d0 = make_diagram("a");

        let mut h = UndoHistory::new(d0.clone());
        h.push(d0.clone());

        assert_eq!(h.undo_depth(), 0);
    }

    #[test]
    fn skip_next_push_flag() {
        let d0 = make_diagram("a");
        let d1 = make_diagram("b");

        let mut h = UndoHistory::new(d0.clone());
        h.push(d1.clone());

        // Undo sets skip_next_push.
        let undone = h.undo().cloned().unwrap();

        // The next push should be skipped (simulating the memo
        // re-serializing the undone diagram).
        h.push(undone);
        // Cursor should still be at d0, not have added a new entry.
        assert_eq!(h.current(), &d0);
        assert_eq!(h.undo_depth(), 0);
        assert_eq!(h.redo_depth(), 1);
    }

    #[test]
    fn capacity_limit() {
        let d0 = make_diagram("seed");
        let mut h = UndoHistory::new(d0);

        for i in 0..MAX_ENTRIES + 50 {
            h.push(make_diagram(&format!("d{i}")));
        }

        // Should never exceed MAX_ENTRIES.
        assert!(h.entries.len() <= MAX_ENTRIES);
        // Cursor should be at the end.
        assert_eq!(h.cursor, h.entries.len() - 1);
    }
}
