use std::ops::{Deref, DerefMut};

use disposition_model_common::{Id, Map};
use serde::{Deserialize, Serialize};

use crate::EntityHighlightedSpan;

/// `syntect` highlighted spans of entity descriptions.
///
/// This is computed in `IrToTaffyBuilder`.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct EntityHighlightedSpans(Map<Id<'static>, Vec<EntityHighlightedSpan>>);

impl EntityHighlightedSpans {
    /// Returns a new `EntityHighlightedSpans` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `EntityHighlightedSpans` map with the given preallocated
    /// capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<Id<'static>, Vec<EntityHighlightedSpan>> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns true if this contains highlighted spans for an entity with the
    /// given ID.
    pub fn contains_key<IdT>(&self, id: &IdT) -> bool
    where
        IdT: AsRef<Id<'static>>,
    {
        self.0.contains_key(id.as_ref())
    }
}

impl Deref for EntityHighlightedSpans {
    type Target = Map<Id<'static>, Vec<EntityHighlightedSpan>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for EntityHighlightedSpans {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Map<Id<'static>, Vec<EntityHighlightedSpan>>> for EntityHighlightedSpans {
    fn from(inner: Map<Id<'static>, Vec<EntityHighlightedSpan>>) -> Self {
        Self(inner)
    }
}

impl FromIterator<(Id<'static>, Vec<EntityHighlightedSpan>)> for EntityHighlightedSpans {
    fn from_iter<I: IntoIterator<Item = (Id<'static>, Vec<EntityHighlightedSpan>)>>(
        iter: I,
    ) -> Self {
        Self(Map::from_iter(iter))
    }
}
