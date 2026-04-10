use serde::{Deserialize, Serialize};

/// Leaf layout configuration for a container node.
///
/// This struct defines how child nodes are arranged within a parent container
/// using CSS Leafbox-like layout rules.
///
/// # Example
///
/// ```yaml
/// node_layout:
///   t_localhost:
///     leaf:
///       padding_top: 4.0
///       padding_right: 4.0
///       padding_bottom: 4.0
///       padding_left: 4.0
///       margin_top: 0.0
///       margin_right: 0.0
///       margin_bottom: 0.0
///       margin_left: 0.0
///   proc_app_dev:
///     leaf:
///       padding_top: 2.0
///       padding_right: 2.0
///       padding_bottom: 2.0
///       padding_left: 2.0
///       margin_top: 0.0
///       margin_right: 0.0
///       margin_bottom: 0.0
///       margin_left: 0.0
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct LeafLayout {
    /// The top padding within this node, which will be used for the
    /// [`Style::padding`].
    ///
    /// [`Style::padding`]: https://docs.rs/taffy/latest/taffy/struct.Style.html#structfield.padding
    #[serde(default)]
    pub padding_top: f32,

    /// The right padding within this node, which will be used for the
    /// [`Style::padding`].
    ///
    /// [`Style::padding`]: https://docs.rs/taffy/latest/taffy/struct.Style.html#structfield.padding
    #[serde(default)]
    pub padding_right: f32,

    /// The bottom padding within this node, which will be used for the
    /// [`Style::padding`].
    ///
    /// [`Style::padding`]: https://docs.rs/taffy/latest/taffy/struct.Style.html#structfield.padding
    #[serde(default)]
    pub padding_bottom: f32,

    /// The left padding within this node, which will be used for the
    /// [`Style::padding`].
    ///
    /// [`Style::padding`]: https://docs.rs/taffy/latest/taffy/struct.Style.html#structfield.padding
    #[serde(default)]
    pub padding_left: f32,

    /// The top margin around this node, which will be used for the
    /// [`Style::margin`].
    ///
    /// [`Style::margin`]: https://docs.rs/taffy/latest/taffy/struct.Style.html#structfield.margin
    #[serde(default)]
    pub margin_top: f32,

    /// The right margin around this node, which will be used for the
    /// [`Style::margin`].
    ///
    /// [`Style::margin`]: https://docs.rs/taffy/latest/taffy/struct.Style.html#structfield.margin
    #[serde(default)]
    pub margin_right: f32,

    /// The bottom margin around this node, which will be used for the
    /// [`Style::margin`].
    ///
    /// [`Style::margin`]: https://docs.rs/taffy/latest/taffy/struct.Style.html#structfield.margin
    #[serde(default)]
    pub margin_bottom: f32,

    /// The left margin around this node, which will be used for the
    /// [`Style::margin`].
    ///
    /// [`Style::margin`]: https://docs.rs/taffy/latest/taffy/struct.Style.html#structfield.margin
    #[serde(default)]
    pub margin_left: f32,
}

impl LeafLayout {
    /// Creates a new `LeafLayout` with default values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_ir_model::layout::LeafLayout;
    ///
    /// let layout = LeafLayout::new();
    ///
    /// assert_eq!(layout.padding_top(), 0.0);
    /// assert_eq!(layout.padding_right(), 0.0);
    /// assert_eq!(layout.padding_bottom(), 0.0);
    /// assert_eq!(layout.padding_left(), 0.0);
    /// assert_eq!(layout.margin_top(), 0.0);
    /// assert_eq!(layout.margin_right(), 0.0);
    /// assert_eq!(layout.margin_bottom(), 0.0);
    /// assert_eq!(layout.margin_left(), 0.0);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the top padding within this node, which will be used for the
    /// [`Style::padding`].
    ///
    /// [`Style::padding`]: https://docs.rs/taffy/latest/taffy/struct.Style.html#structfield.padding
    pub fn padding_top(&self) -> f32 {
        self.padding_top
    }

    /// Returns the right padding within this node, which will be used for the
    /// [`Style::padding`].
    ///
    /// [`Style::padding`]: https://docs.rs/taffy/latest/taffy/struct.Style.html#structfield.padding
    pub fn padding_right(&self) -> f32 {
        self.padding_right
    }

    /// Returns the bottom padding within this node, which will be used for the
    /// [`Style::padding`].
    ///
    /// [`Style::padding`]: https://docs.rs/taffy/latest/taffy/struct.Style.html#structfield.padding
    pub fn padding_bottom(&self) -> f32 {
        self.padding_bottom
    }

    /// Returns the left padding within this node, which will be used for the
    /// [`Style::padding`].
    ///
    /// [`Style::padding`]: https://docs.rs/taffy/latest/taffy/struct.Style.html#structfield.padding
    pub fn padding_left(&self) -> f32 {
        self.padding_left
    }

    /// Returns the top margin around this node, which will be used for the
    /// [`Style::margin`].
    ///
    /// [`Style::margin`]: https://docs.rs/taffy/latest/taffy/struct.Style.html#structfield.margin
    pub fn margin_top(&self) -> f32 {
        self.margin_top
    }

    /// Returns the right margin around this node, which will be used for the
    /// [`Style::margin`].
    ///
    /// [`Style::margin`]: https://docs.rs/taffy/latest/taffy/struct.Style.html#structfield.margin
    pub fn margin_right(&self) -> f32 {
        self.margin_right
    }

    /// Returns the bottom margin around this node, which will be used for the
    /// [`Style::margin`].
    ///
    /// [`Style::margin`]: https://docs.rs/taffy/latest/taffy/struct.Style.html#structfield.margin
    pub fn margin_bottom(&self) -> f32 {
        self.margin_bottom
    }

    /// Returns the left margin around this node, which will be used for the
    /// [`Style::margin`].
    ///
    /// [`Style::margin`]: https://docs.rs/taffy/latest/taffy/struct.Style.html#structfield.margin
    pub fn margin_left(&self) -> f32 {
        self.margin_left
    }
}
