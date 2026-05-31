/// Absolute x and y coordinates of a taffy node in the SVG coordinate space.
///
/// Taffy node coordinates are stored relative to each node's parent, whereas
/// these are the accumulated absolute coordinates used when rendering the SVG.
///
/// # Examples
///
/// ```text
/// AbsoluteCoordinates { x: 150.0, y: 80.0 }
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct AbsoluteCoordinates {
    /// Absolute x coordinate.
    pub(crate) x: f32,
    /// Absolute y coordinate.
    pub(crate) y: f32,
}
