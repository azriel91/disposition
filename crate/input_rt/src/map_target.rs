//! Selector for `thing_dependencies` vs `thing_interactions`.

/// Which edge-group map inside [`InputDiagram`] we are editing.
///
/// Several mutation helpers in [`EdgeGroupCardOps`] operate on either
/// `thing_dependencies` or `thing_interactions`. This enum selects the
/// target so the same logic can be reused for both maps.
///
/// [`InputDiagram`]: disposition_input_model::InputDiagram
/// [`EdgeGroupCardOps`]: crate::edge_group_card_ops::EdgeGroupCardOps
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapTarget {
    /// Targets `thing_dependencies`.
    Dependencies,
    /// Targets `thing_interactions`.
    Interactions,
}
