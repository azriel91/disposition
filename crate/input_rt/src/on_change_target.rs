//! Target map selector for generic key-value row mutations.

/// Identifies which field of [`InputDiagram`] a generic key-value row targets.
///
/// [`InputDiagram`]: disposition_input_model::InputDiagram
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OnChangeTarget {
    /// Targets `thing_copy_text`.
    CopyText,
    /// Targets `thing_descs`.
    ThingDesc,
    /// Targets `edge_descs`.
    EdgeDesc,
    /// Targets `entity_tooltips`.
    EntityTooltip,
}
