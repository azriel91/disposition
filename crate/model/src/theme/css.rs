use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

/// Additional CSS to place in the SVG's inline `<styles>` section.
///
/// This allows users to add custom CSS rules, such as animations or
/// special styling that isn't covered by the standard theme options.
///
/// # Example
///
/// ```yaml
/// css: |-
///   @keyframes stroke-dashoffset-move {
///     0%   { stroke-dashoffset: 30; }
///     100% { stroke-dashoffset: 0; }
///   }
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct Css(String);

impl Css {
    /// Returns a new empty `Css`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new `Css` from the given string.
    pub fn from_string(css: impl Into<String>) -> Self {
        Self(css.into())
    }

    /// Returns the underlying string.
    pub fn into_inner(self) -> String {
        self.0
    }

    /// Returns true if the CSS string is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the CSS as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for Css {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Css {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<String> for Css {
    fn from(inner: String) -> Self {
        Self(inner)
    }
}

impl From<&str> for Css {
    fn from(inner: &str) -> Self {
        Self(inner.to_string())
    }
}

impl AsRef<str> for Css {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
