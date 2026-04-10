use serde::{Deserialize, Serialize};

/// CSS selector strategy for applying dark-mode colour overrides.
///
/// Controls how the dark-mode CSS variable block is targeted in the
/// generated SVG styles.
///
/// # Variants
///
/// * `MediaQuery` -- uses `@media (prefers-color-scheme: dark) { svg { .. } }`.
/// * `RootDarkClass` -- uses `:root.dark svg { .. }`, which allows a
///   surrounding page to toggle dark mode via a `dark` class on `<html>`.
///
/// Defaults to `DarkModeCssSelector::RootDarkClass`.
///
/// # Examples
///
/// ```yaml
/// selector: media_query
/// ```
///
/// ```yaml
/// selector: root_dark_class
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum DarkModeCssSelector {
    /// Uses `@media (prefers-color-scheme: dark) { svg { .. } }`.
    ///
    /// The browser automatically switches based on the OS / browser
    /// preference.
    MediaQuery,

    /// Uses `:root.dark svg { .. }`.
    ///
    /// Allows a surrounding website to control dark mode by toggling a
    /// `dark` class on the `<html>` element.
    #[default]
    RootDarkClass,
}
