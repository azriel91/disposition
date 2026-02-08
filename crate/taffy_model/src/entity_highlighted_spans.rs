use std::ops::{Deref, DerefMut};

use disposition_model_common::{Id, Map};
use serde::{Deserialize, Serialize};

use crate::EntityHighlightedSpan;

/// Highlighted spans of entity descriptions.
///
/// Originally this held the `syntect` highlighted spans, but the performance
/// was too slow so currently there is no highlighting.
///
/// This is computed in `IrToTaffyBuilder`.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct EntityHighlightedSpans<'id>(Map<Id<'id>, Vec<EntityHighlightedSpan>>);

impl<'id> EntityHighlightedSpans<'id> {
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
    pub fn into_inner(self) -> Map<Id<'id>, Vec<EntityHighlightedSpan>> {
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
        IdT: AsRef<Id<'id>>,
    {
        self.0.contains_key(id.as_ref())
    }
}

impl<'id> Deref for EntityHighlightedSpans<'id> {
    type Target = Map<Id<'id>, Vec<EntityHighlightedSpan>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'id> DerefMut for EntityHighlightedSpans<'id> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'id> From<Map<Id<'id>, Vec<EntityHighlightedSpan>>> for EntityHighlightedSpans<'id> {
    fn from(inner: Map<Id<'id>, Vec<EntityHighlightedSpan>>) -> Self {
        Self(inner)
    }
}

impl<'id> FromIterator<(Id<'id>, Vec<EntityHighlightedSpan>)> for EntityHighlightedSpans<'id> {
    fn from_iter<I: IntoIterator<Item = (Id<'id>, Vec<EntityHighlightedSpan>)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
