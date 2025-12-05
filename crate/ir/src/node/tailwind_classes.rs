use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::{common::Map, node::NodeId};

/// Tailwind CSS classes for interactive visibility behaviour.
///
/// This map contains computed CSS classes for each node (and edge). The classes
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
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct TailwindClasses(Map<NodeId, String>);

impl TailwindClasses {
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
    pub fn into_inner(self) -> Map<NodeId, String> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Deref for TailwindClasses {
    type Target = Map<NodeId, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TailwindClasses {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Map<NodeId, String>> for TailwindClasses {
    fn from(inner: Map<NodeId, String>) -> Self {
        Self(inner)
    }
}

impl FromIterator<(NodeId, String)> for TailwindClasses {
    fn from_iter<I: IntoIterator<Item = (NodeId, String)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
