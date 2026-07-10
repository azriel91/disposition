use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::{edge::EdgeId, thing::LayoutEdge, Map};

/// Invisible edges between things that affect rank/layout, keyed by their own
/// [`EdgeId`].
///
/// Unlike dependency or interaction edges, these never produce an SVG
/// `<path>` -- they exist purely to influence node rank (and hence position)
/// using the same LCA-aware rank computation as dependency edges.
///
/// # Example
///
/// ```yaml
/// thing_layout_edges:
///   edge_layout_app__db:
///     from: app
///     to: db
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ThingLayoutEdges<'id>(Map<EdgeId<'id>, LayoutEdge<'id>>);

impl<'id> ThingLayoutEdges<'id> {
    /// Returns a new `ThingLayoutEdges` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `ThingLayoutEdges` map with the given preallocated
    /// capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<EdgeId<'id>, LayoutEdge<'id>> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Converts this `ThingLayoutEdges` into one with a `'static` lifetime.
    ///
    /// If any inner `Cow` is borrowed, this will clone the string to create
    /// an owned version.
    pub fn into_static(self) -> ThingLayoutEdges<'static> {
        ThingLayoutEdges(
            self.0
                .into_iter()
                .map(|(edge_id, layout_edge)| (edge_id.into_static(), layout_edge.into_static()))
                .collect(),
        )
    }
}

impl<'id> Deref for ThingLayoutEdges<'id> {
    type Target = Map<EdgeId<'id>, LayoutEdge<'id>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'id> DerefMut for ThingLayoutEdges<'id> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'id> From<Map<EdgeId<'id>, LayoutEdge<'id>>> for ThingLayoutEdges<'id> {
    fn from(inner: Map<EdgeId<'id>, LayoutEdge<'id>>) -> Self {
        Self(inner)
    }
}

impl<'id> FromIterator<(EdgeId<'id>, LayoutEdge<'id>)> for ThingLayoutEdges<'id> {
    fn from_iter<I: IntoIterator<Item = (EdgeId<'id>, LayoutEdge<'id>)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
