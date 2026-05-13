/// A face/side of a rectangular diagram node.
///
/// Used to identify which side of a node an edge exits or enters,
/// for routing edge paths and placing edge label slots.
///
/// # Examples
///
/// Valid values: `Top`, `Bottom`, `Left`, `Right`
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum NodeFace {
    /// The top edge of the node rectangle.
    Top,
    /// The bottom edge of the node rectangle.
    Bottom,
    /// The left edge of the node rectangle.
    Left,
    /// The right edge of the node rectangle.
    Right,
}
