use serde::{Deserialize, Serialize};

use crate::layout::FlexDirection;

/// Flex layout configuration for a container node.
///
/// This struct defines how child nodes are arranged within a parent container
/// using CSS flexbox-like layout rules.
///
/// # Example
///
/// ```yaml
/// node_layout:
///   _root:
///     flex:
///       direction: "column_reverse"
///       wrap: true
///       padding_top: 4.0
///       padding_right: 4.0
///       padding_bottom: 4.0
///       padding_left: 4.0
///       margin_top: 0.0
///       margin_right: 0.0
///       margin_bottom: 0.0
///       margin_left: 0.0
///       gap: 4.0
///   _processes_container:
///     flex:
///       direction: "row"
///       wrap: true
///       padding_top: 4.0
///       padding_right: 4.0
///       padding_bottom: 4.0
///       padding_left: 4.0
///       margin_top: 0.0
///       margin_right: 0.0
///       margin_bottom: 0.0
///       margin_left: 0.0
///       gap: 4.0
///   proc_app_dev:
///     flex:
///       direction: "column"
///       wrap: false
///       padding_top: 2.0
///       padding_right: 2.0
///       padding_bottom: 2.0
///       padding_left: 2.0
///       margin_top: 0.0
///       margin_right: 0.0
///       margin_bottom: 0.0
///       margin_left: 0.0
///       gap: 2.0
/// ```
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct FlexLayout {
    /// The direction in which flex items are placed in the flex container.
    #[serde(default)]
    pub direction: FlexDirection,

    /// Whether flex items are forced onto one line or can wrap onto multiple
    /// lines.
    #[serde(default)]
    pub wrap: bool,

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

    /// The gap between flex items, which will be used for the [`Style::gap`].
    ///
    /// [`Style::gap`]: https://docs.rs/taffy/latest/taffy/struct.Style.html#structfield.gap
    #[serde(default)]
    pub gap: f32,
}

impl FlexLayout {
    /// Creates a new `FlexLayout` with default values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_ir_model::layout::{FlexDirection, FlexLayout};
    ///
    /// let layout = FlexLayout::new();
    ///
    /// assert_eq!(layout.direction, FlexDirection::Row);
    /// assert!(!layout.wrap);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the direction.
    pub fn direction(&self) -> FlexDirection {
        self.direction
    }

    /// Returns whether items should wrap.
    pub fn wrap(&self) -> bool {
        self.wrap
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

    /// Returns the gap between items, which will be used for the
    /// [`Style::gap`].
    ///
    /// [`Style::gap`]: https://docs.rs/taffy/latest/taffy/struct.Style.html#structfield.gap
    pub fn gap(&self) -> f32 {
        self.gap
    }
}
