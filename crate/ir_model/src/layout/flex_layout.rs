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
///       gap: "4"
///   _processes_container:
///     flex:
///       direction: "row"
///       wrap: true
///       gap: "4"
///   proc_app_dev:
///     flex:
///       direction: "column"
///       wrap: false
///       gap: "2"
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct FlexLayout {
    /// The direction in which flex items are placed in the flex container.
    #[serde(default)]
    pub direction: FlexDirection,

    /// Whether flex items are forced onto one line or can wrap onto multiple
    /// lines.
    #[serde(default)]
    pub wrap: bool,

    /// The gap between flex items, as a Tailwind spacing value (e.g., "1",
    /// "2", "4").
    #[serde(default)]
    pub gap: String,
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
    /// assert!(layout.gap.is_empty());
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

    /// Returns the gap between items.
    pub fn gap(&self) -> &str {
        &self.gap
    }
}
