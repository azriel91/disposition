use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::{Id, Map};

/// Tooltips for entities.
///
/// This map contains text (typically markdown) that provides additional
/// context about entities in the diagram. These tooltips are displayed
/// when an entity is hovered.
///
/// # Example
///
/// ```yaml
/// entity_tooltips:
///   # process_steps
///   proc_app_dev_step_repository_clone: |-
///     ```bash
///     git clone https://github.com/azriel91/web_app.git
///     ```
/// ```
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct EntityTooltips<'id>(Map<Id<'id>, String>);

impl<'id> EntityTooltips<'id> {
    /// Returns a new `EntityTooltips` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `EntityTooltips` map with the given preallocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<Id<'id>, String> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Converts this `EntityTooltips` into one with a `'static` lifetime.
    ///
    /// If any inner `Cow` is borrowed, this will clone the string to create
    /// an owned version.
    pub fn into_static(self) -> EntityTooltips<'static> {
        EntityTooltips(
            self.0
                .into_iter()
                .map(|(id, desc)| (id.into_static(), desc))
                .collect(),
        )
    }

    /// Returns true if this contains a description for an entity with the given
    /// ID.
    pub fn contains_key<IdT>(&self, id: &IdT) -> bool
    where
        IdT: AsRef<Id<'id>>,
    {
        self.0.contains_key(id.as_ref())
    }
}

impl<'id> Deref for EntityTooltips<'id> {
    type Target = Map<Id<'id>, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'id> DerefMut for EntityTooltips<'id> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'id> From<Map<Id<'id>, String>> for EntityTooltips<'id> {
    fn from(inner: Map<Id<'id>, String>) -> Self {
        Self(inner)
    }
}

impl<'id> FromIterator<(Id<'id>, String)> for EntityTooltips<'id> {
    fn from_iter<I: IntoIterator<Item = (Id<'id>, String)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
