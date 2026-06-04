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
    },
}
