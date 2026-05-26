use serde::{Deserialize, Serialize};

/// Heading level for markdown text within a diagram node.
///
/// Used in [`MdStyle`] to indicate which heading level a token belongs to.
///
/// [`MdStyle`]: crate::MdStyle
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum MdHeadingLevel {
    /// Heading level 1 (`# Heading`).
    H1,
    /// Heading level 2 (`## Heading`).
    H2,
    /// Heading level 3 (`### Heading`).
    H3,
    /// Heading level 4 (`#### Heading`).
    H4,
    /// Heading level 5 (`##### Heading`).
    H5,
    /// Heading level 6 (`###### Heading`).
    H6,
}

impl MdHeadingLevel {
    /// Returns the font-size scale factor for this heading level.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_taffy_model::MdHeadingLevel;
    ///
    /// assert_eq!(MdHeadingLevel::H1.font_scale(), 2.0);
    /// assert_eq!(MdHeadingLevel::H2.font_scale(), 1.5);
    /// assert_eq!(MdHeadingLevel::H3.font_scale(), 1.25);
    /// assert_eq!(MdHeadingLevel::H4.font_scale(), 1.0);
    /// assert_eq!(MdHeadingLevel::H5.font_scale(), 1.0);
    /// assert_eq!(MdHeadingLevel::H6.font_scale(), 1.0);
    /// ```
    pub fn font_scale(self) -> f32 {
        match self {
            MdHeadingLevel::H1 => 2.0,
            MdHeadingLevel::H2 => 1.5,
            MdHeadingLevel::H3 => 1.25,
            MdHeadingLevel::H4 | MdHeadingLevel::H5 | MdHeadingLevel::H6 => 1.0,
        }
    }
}
