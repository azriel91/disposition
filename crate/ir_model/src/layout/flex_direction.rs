use serde::{Deserialize, Serialize};

/// Direction for flex layout of child nodes.
///
/// This enum maps to CSS flexbox `flex-direction` values and determines
/// how child nodes are arranged within their parent container.
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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FlexDirection {
    /// Items are placed left to right in a row.
    #[default]
    Row,

    /// Items are placed right to left in a row.
    RowReverse,

    /// Items are placed top to bottom in a column.
    Column,

    /// Items are placed bottom to top in a column.
    ColumnReverse,
}

impl From<FlexDirection> for taffy::style::FlexDirection {
    fn from(direction: FlexDirection) -> Self {
        match direction {
            FlexDirection::Row => taffy::style::FlexDirection::Row,
            FlexDirection::RowReverse => taffy::style::FlexDirection::RowReverse,
            FlexDirection::Column => taffy::style::FlexDirection::Column,
            FlexDirection::ColumnReverse => taffy::style::FlexDirection::ColumnReverse,
        }
    }
}
