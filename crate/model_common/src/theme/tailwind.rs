//! Canonical Tailwind CSS color names and shade values.
//!
//! These are the value vocabularies for the color- and shade-valued theme
//! attributes (e.g. `shape_color: "slate"`, `fill_shade_normal: "300"`). They
//! live here so they can be shared between the renderer
//! (`disposition_input_ir_rt`, which maps each name+shade to an oklch value) and
//! the language server (`disposition_lsp`, which offers them as value
//! completions) without duplication.
//!
//! The renderer's oklch lookup table is the source of the actual color data; a
//! test asserts the names there stay in sync with [`TAILWIND_COLOR_NAMES`].

/// All Tailwind CSS color names usable as a `*_color` theme attribute value.
///
/// Ordered as in the renderer's oklch table. Example values: `"slate"`,
/// `"blue"`, `"emerald"`.
pub const TAILWIND_COLOR_NAMES: &[&str] = &[
    "slate", "gray", "zinc", "neutral", "stone", "mauve", "olive", "mist", "taupe", "red",
    "orange", "amber", "yellow", "lime", "green", "emerald", "teal", "cyan", "sky", "blue",
    "indigo", "violet", "purple", "fuchsia", "pink", "rose",
];

/// All Tailwind CSS shade values usable as a `*_shade` theme attribute value.
///
/// Ordered lightest (`"50"`) to darkest (`"950"`).
pub const TAILWIND_COLOR_SHADES: &[&str] = &[
    "50", "100", "200", "300", "400", "500", "600", "700", "800", "900", "950",
];
