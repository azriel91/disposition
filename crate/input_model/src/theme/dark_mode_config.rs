use serde::{Deserialize, Serialize};

use crate::theme::{DarkModeCssSelector, DarkModeShadeConfig};

/// Configuration for dark mode behavior.
///
/// Groups the shade adjustment strategy (`shade`) and the CSS selector
/// strategy (`selector`) used when emitting dark-mode styles.
///
/// # Examples
///
/// ```yaml
/// dark_mode_config:
///   shade:
///     mode: invert
///   selector: root_dark_class
/// ```
///
/// ```yaml
/// dark_mode_config:
///   shade:
///     mode: shift
///     levels: 4
///   selector: media_query
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct DarkModeConfig {
    /// How fill, stroke, and text shades are adjusted for dark mode.
    ///
    /// Defaults to `DarkModeShadeConfig::Invert`.
    #[serde(default)]
    pub shade: DarkModeShadeConfig,

    /// CSS selector strategy for the dark-mode variable overrides.
    ///
    /// Defaults to `DarkModeCssSelector::RootDarkClass`.
    #[serde(default)]
    pub selector: DarkModeCssSelector,
}

impl DarkModeConfig {
    /// Returns `true` if all fields are at their default values.
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }
}
