//! Target map selector for generic key-value row mutations.

/// Identifies which field of [`InputDiagram`] a generic key-value row targets.
///
/// [`InputDiagram`]: disposition::input_model::InputDiagram
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OnChangeTarget {
    /// Targets `thing_copy_text`.
    CopyText,
    /// Targets `entity_descs`.
    EntityDesc,
    /// Targets `entity_tooltips`.
    EntityTooltip,
}
