//! A snapshot of a single theme attribute name-value pair.
//!
//! Replaces the weakly typed `(String, String)` tuple with named fields so
//! that call sites clearly communicate which element is the attribute name
//! and which is the value.

// === ThemeAttrEntry === //

/// Snapshot of a single `ThemeAttr -> value` pair, serialised to strings.
///
/// # Examples
///
/// ```rust,ignore
/// let entry = ThemeAttrEntry {
///     attr_name: "fill_color".to_owned(),
///     attr_value: "#e8f0fe".to_owned(),
/// };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThemeAttrEntry {
    /// The `snake_case` name of the
    /// [`ThemeAttr`](disposition::input_model::theme::ThemeAttr) variant.
    ///
    /// Valid values: `"fill_color"`, `"stroke_width"`, `"opacity"`.
    pub attr_name: String,

    /// The CSS value associated with this attribute.
    ///
    /// Valid values: `"#e8f0fe"`, `"2px"`, `"0.5"`.
    pub attr_value: String,
}
