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
