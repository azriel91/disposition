//! What the cursor is positioned to complete.

/// Whether the cursor is completing a map key or a value, within its container.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompletionTarget {
    /// The cursor is typing a map key -- offer the container's known fields.
    Key,
    /// The cursor is typing a value for `key` -- offer enum values or IDs.
    Value {
        /// The key whose value is being completed, e.g. `"kind"` or `"things"`.
        key: String,
        /// Whether the cursor is already inside a sequence value -- either a
        /// block `- ` item line, or within `[ .. ]` flow-list brackets. When
        /// `false`, an array-valued field completed at the `key:` position has
        /// its element values wrapped in flow-list syntax (e.g. `[t_a]`).
        in_sequence: bool,
        /// Whether a separator space must precede the inserted value (the
        /// cursor is immediately after `key:` with no space yet).
        needs_space: bool,
    },
}
