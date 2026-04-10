use serde::{Deserialize, Serialize};

// === DarkModeShadeConfig === //

/// Configuration for how shades are adjusted for dark mode.
///
/// Controls the dark-mode counterpart of fill, stroke, and text shades.
///
/// # Variants
///
/// * `Disable` -- no dark mode classes are emitted.
/// * `Invert` -- shades are mirrored around 500 (e.g. 100 becomes 900).
/// * `Shift` -- shades are shifted by a number of levels (e.g. 100 shifted
///   darker by 4 levels becomes 500).
///
/// Defaults to `DarkModeShadeConfig::Invert`.
///
/// # Examples
///
/// ```yaml
/// dark_mode_shade_config:
///   mode: disable
/// ```
///
/// ```yaml
/// dark_mode_shade_config:
///   mode: invert
/// ```
///
/// ```yaml
/// dark_mode_shade_config:
///   mode: shift
///   levels: 4
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
#[derive(Default)]
pub enum DarkModeShadeConfig {
    /// Disables dark mode tailwind classes from being added.
    Disable,

    /// Inverts the shades -- mirrors around 500.
    ///
    /// For example, shade `100` becomes `900`, and `200` becomes `800`.
    #[default]
    Invert,

    /// Shifts shades by a specified number of levels.
    ///
    /// The direction of the shift (darker or lighter) is determined
    /// automatically based on whether the light-mode shades lean light or
    /// dark.
    ///
    /// For example, with `levels: 4`, a light-mode shade of `100` (index 1)
    /// shifted darker becomes `500` (index 5).
    Shift {
        /// Number of shade levels to shift.
        ///
        /// e.g. `4`
        levels: u8,
    },
}

// === Trait Implementations === //
