use std::ops::{Deref, DerefMut};

use disposition_model_common::{Id, Map};
use serde::{Deserialize, Serialize};

use crate::thing::ThingId;

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
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ThingCopyText<'id>(Map<ThingId<'id>, String>);

impl<'id> ThingCopyText<'id> {
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
    pub fn into_inner(self) -> Map<ThingId<'id>, String> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns true if this contains copy text for a thing with the given ID.
    pub fn contains_key<IdT>(&self, id: &IdT) -> bool
    where
        IdT: AsRef<Id<'id>>,
    {
        self.0.contains_key(id.as_ref())
    }
}

impl<'id> Deref for ThingCopyText<'id> {
    type Target = Map<ThingId<'id>, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'id> DerefMut for ThingCopyText<'id> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'id> From<Map<ThingId<'id>, String>> for ThingCopyText<'id> {
    fn from(inner: Map<ThingId<'id>, String>) -> Self {
        Self(inner)
    }
}

impl<'id> FromIterator<(ThingId<'id>, String)> for ThingCopyText<'id> {
    fn from_iter<I: IntoIterator<Item = (ThingId<'id>, String)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
