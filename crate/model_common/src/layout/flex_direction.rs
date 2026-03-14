use serde::{Deserialize, Serialize};

/// Direction for flex layout of child nodes.
///
/// This enum maps to CSS flexbox `flex-direction` values and determines
/// how child nodes are arranged within their parent container.
///
/// # YAML values
///
/// * `"row"` -- Items are placed left to right.
/// * `"row_reverse"` -- Items are placed right to left.
/// * `"column"` -- Items are placed top to bottom.
/// * `"column_reverse"` -- Items are placed bottom to top.
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
