//! A snapshot of a single theme attribute name-value pair.
//!
//! Uses the strongly typed [`ThemeAttr`] for the key so that call sites
//! no longer need fallible `parse_theme_attr()` round-trips.

use disposition::input_model::theme::ThemeAttr;

// === ThemeAttrEntry === //

/// Snapshot of a single `ThemeAttr -> value` pair.
///
/// The attribute key is kept as a [`ThemeAttr`] rather than a `String` so
/// that downstream components can use it directly without re-parsing.
///
/// # Examples
///
/// ```rust,ignore
/// let entry = ThemeAttrEntry {
///     theme_attr: ThemeAttr::FillColor,
///     attr_value: "#e8f0fe".to_owned(),
/// };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThemeAttrEntry {
    /// The theme attribute variant.
    ///
    /// Example values: `ThemeAttr::FillColor`, `ThemeAttr::StrokeWidth`,
    /// `ThemeAttr::Opacity`.
    pub theme_attr: ThemeAttr,

    /// The CSS value associated with this attribute.
    ///
    /// Valid values: `"#e8f0fe"`, `"2px"`, `"0.5"`.
    pub attr_value: String,
}
