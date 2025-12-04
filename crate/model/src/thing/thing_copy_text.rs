use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::{common::Map, thing::ThingId};

/// Text to copy to clipboard when a thing's copy button is clicked.
///
/// This allows things to have different copy text than their display label.
/// For example, a directory thing might display as "📂 ~/work/web_app" but
/// copy as "~/work/web_app".
///
/// # Example
///
/// ```yaml
/// thing_copy_text:
///   t_localhost_repo: "~/work/web_app"
///   t_localhost_repo_src: "~/work/web_app/src"
///   t_localhost_repo_target: "~/work/web_app/target"
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ThingCopyText(Map<ThingId, String>);

impl ThingCopyText {
    /// Returns a new `ThingCopyText` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `ThingCopyText` map with the given preallocated
    /// capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<ThingId, String> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Deref for ThingCopyText {
    type Target = Map<ThingId, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ThingCopyText {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Map<ThingId, String>> for ThingCopyText {
    fn from(inner: Map<ThingId, String>) -> Self {
        Self(inner)
    }
}

impl FromIterator<(ThingId, String)> for ThingCopyText {
    fn from_iter<I: IntoIterator<Item = (ThingId, String)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
