use serde::{Deserialize, Serialize};

use crate::MdStyle;

/// Context placed on a word-token taffy leaf.
///
/// Used by `node_size_measure` to compute the token width using the token text
/// and heading-level font scale.
///
/// # Examples
///
/// ```rust
/// use disposition_taffy_model::{MdStyle, MdTokenCtx};
///
/// let ctx = MdTokenCtx {
///     text: "hello".to_string(),
///     md_style: MdStyle::default(),
/// };
/// assert_eq!(ctx.text, "hello");
/// ```
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct MdTokenCtx {
    /// The word or text fragment to measure. Contains no leading/trailing
    /// whitespace.
    ///
    /// Example: `"bold"`, `"hello"`.
    pub text: String,
    /// Inline markdown style active when this token was emitted.
    pub md_style: MdStyle,
}
