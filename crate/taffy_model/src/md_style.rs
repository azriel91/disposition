use serde::{Deserialize, Serialize};

use crate::MdHeadingLevel;

/// Inline markdown formatting active when a token is emitted.
///
/// Derives `Default` -- all fields `false` / `None`.
///
/// # Examples
///
/// ```rust
/// use disposition_taffy_model::{MdHeadingLevel, MdStyle};
///
/// let style = MdStyle {
///     bold: true,
///     heading_level: Some(MdHeadingLevel::H2),
///     ..MdStyle::default()
/// };
/// assert!(style.bold);
/// assert_eq!(style.heading_level, Some(MdHeadingLevel::H2));
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct MdStyle {
    /// Whether the token is inside a `**strong**` / `__strong__` run.
    pub bold: bool,
    /// Whether the token is inside an `*emphasis*` / `_emphasis_` run.
    pub italic: bool,
    /// Whether the token is inside a `~~strikethrough~~` run.
    pub strikethrough: bool,
    /// Whether the token is an inline code fragment (`` `code` ``).
    pub code: bool,
    /// Non-`None` when the token is inside a heading block.
    pub heading_level: Option<MdHeadingLevel>,
    /// Non-`None` when the token is inside a `[link](url)` run.
    ///
    /// Contains the destination URL string, e.g. `"https://example.com"`.
    pub link_dest: Option<String>,
}
