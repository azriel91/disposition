//! A snapshot of a single `CssClassPartials` entry, captured as owned strings.
//!
//! Replaces the weakly typed `(String, Vec<String>, Vec<(String, String)>)`
//! tuple with named fields so that call sites clearly communicate the role
//! of each element.

use super::theme_attr_entry::ThemeAttrEntry;

// === CssClassPartialsSnapshot === //

/// Owned snapshot of one `IdOrDefaults -> CssClassPartials` (or
/// `StyleAlias -> CssClassPartials`) map entry.
///
/// The snapshot converts domain types into plain strings so that the
/// signal borrow can be dropped before event handlers run.
///
/// # Examples
///
/// ```rust,ignore
/// let snapshot = CssClassPartialsSnapshot {
///     entry_key: "node_defaults".to_owned(),
///     style_aliases_applied: vec!["shade_light".to_owned()],
///     theme_attrs: vec![
///         ThemeAttrEntry {
///             attr_name: "fill_color".to_owned(),
///             attr_value: "#e8f0fe".to_owned(),
///         },
///     ],
/// };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CssClassPartialsSnapshot {
    /// The serialised key for this entry.
    ///
    /// For `ThemeStyles` maps this is an `IdOrDefaults` key such as
    /// `"node_defaults"` or a custom entity ID like `"app_server"`.
    ///
    /// For `style_aliases` maps this is the `StyleAlias` key such as
    /// `"shade_light"`.
    pub entry_key: String,

    /// The `snake_case` names of the style aliases applied to this entry.
    ///
    /// Valid values: `["shade_light", "padding_normal"]`.
    pub style_aliases_applied: Vec<String>,

    /// The theme attribute name-value pairs (the `partials` map).
    pub theme_attrs: Vec<ThemeAttrEntry>,
}
