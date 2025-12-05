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
