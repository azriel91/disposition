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

impl MdStyle {
    /// Converts this markdown style into a list of Tailwind CSS class names.
    ///
    /// Headings are rendered as bold text without font size adjustments.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_taffy_model::{MdHeadingLevel, MdStyle};
    ///
    /// let style = MdStyle {
    ///     bold: false,
    ///     italic: true,
    ///     heading_level: Some(MdHeadingLevel::H2),
    ///     ..MdStyle::default()
    /// };
    /// let classes = style.to_tailwind_classes();
    /// assert!(classes.contains(&"font-bold".to_string()));
    /// assert!(classes.contains(&"italic".to_string()));
    /// ```
    pub fn to_tailwind_classes(&self) -> Vec<String> {
        let mut classes = Vec::new();

        if self.bold || self.heading_level.is_some() {
            classes.push("font-bold".to_string());
        }
        if self.italic {
            classes.push("italic".to_string());
        }
        if self.strikethrough {
            classes.push("line-through".to_string());
        }
        if self.link_dest.is_some() {
            classes.push("underline".to_string());
        }

        classes
    }
}
