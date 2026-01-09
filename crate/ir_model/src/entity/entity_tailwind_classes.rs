use std::ops::{Deref, DerefMut};

use disposition_model_common::{Id, Map};
use serde::{Deserialize, Serialize};

/// Tailwind CSS classes for interactive visibility behaviour.
///
/// This map contains computed CSS classes for nodes and edges. The classes
/// control visibility, colors, animations, and interactions based on the
/// diagram's state.
///
/// ## Visibility Patterns
///
/// 1. Process -> Process Steps visibility:
///
///     * Process node: `group/{process_id}` class
///     * Process steps: `invisible group-focus-within/{process_id}:visible`
///     * When process (or any child) has focus, steps become and remain visible
///
/// 2. Process Step -> Edges visibility:
///
///     * Process step: `peer/{step_id}` class
///     * Edges: `invisible peer-focus/{step_id}:visible`
///     * Edges must be DOM siblings AFTER the step element
///
/// 3. **Alternative:** `:target` based visibility:
///
///     * When element ID matches URL fragment (e.g. `#step_id`)
///     * Use `invisible target:visible` on the element
///     * Use `[&:has(~_#step_id:target)]:visible` on edges
///     * Use `peer-[:where([data-step='3']):target]:visible` on edges
///
/// # Example
///
/// ```yaml
/// tailwind_classes:
///   # Tags - act as group containers for highlighting associated things
///   tag_app_development: >-
///     stroke-1
///     visible
///     hover:fill-emerald-400
///     fill-emerald-500
///     focus:fill-emerald-600
///     active:fill-emerald-700
///     peer/tag_app_development
///
///   # Processes - act as group containers for their steps
///   proc_app_dev: >-
///     stroke-1
///     visible
///     hover:fill-blue-200
///     fill-blue-300
///     group/proc_app_dev
///
///   # Process steps - visible when parent process has focus
///   proc_app_dev_step_repository_clone: >-
///     stroke-1
///     invisible
///     peer/proc_app_dev_step_repository_clone
///     group-focus-within/proc_app_dev:visible
///
///   # Things
///   t_aws: >-
///     [stroke-dasharray:2]
///     stroke-1
///     visible
///     hover:fill-yellow-50
///     fill-yellow-100
/// ```
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct EntityTailwindClasses(Map<Id<'static>, String>);

impl EntityTailwindClasses {
    /// Returns a new `TailwindClasses` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `TailwindClasses` map with the given preallocated
    /// capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<Id<'static>, String> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns true if this contains tailwind classes for an entity with the
    /// given ID.
    pub fn contains_key<IdT>(&self, id: &IdT) -> bool
    where
        IdT: AsRef<Id<'static>>,
    {
        self.0.contains_key(id.as_ref())
    }
}

impl Deref for EntityTailwindClasses {
    type Target = Map<Id<'static>, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for EntityTailwindClasses {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Map<Id<'static>, String>> for EntityTailwindClasses {
    fn from(inner: Map<Id<'static>, String>) -> Self {
        Self(inner)
    }
}

impl FromIterator<(Id<'static>, String)> for EntityTailwindClasses {
    fn from_iter<I: IntoIterator<Item = (Id<'static>, String)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
