use std::ops::{Deref, DerefMut};

use disposition_model_common::Set;
use serde::{Deserialize, Serialize};

use crate::edge::EdgeId;

/// Edge IDs whose routing geometry is computed for the mirror (`to` -> `from`)
/// orientation.
///
/// An edge whose effective [`EdgeCurvature`] is `Curved`, and whose `from`
/// node's divergent ancestor rank at the LCA level is strictly greater than
/// its `to` node's, routes more cleanly when computed as though the endpoints
/// were swapped. For such edges:
///
/// * The [`EdgeGroups`] entry is stored with `from`/`to` swapped (and the
///   edge's [`EdgeLabels`] entry swapped to match), so every downstream stage
///   computes the mirror geometry.
/// * The edge's ID is recorded here, and the SVG path is reversed at emission
///   so the drawn path still runs from the real `from` node to the real `to`
///   node, with the arrow head on the real `to` node.
///
/// [`EdgeCurvature`]: disposition_model_common::edge::EdgeCurvature
/// [`EdgeGroups`]: crate::edge::EdgeGroups
/// [`EdgeLabels`]: crate::edge::EdgeLabels
///
/// # Example
///
/// ```yaml
/// edge_route_reversals:
/// - edge_ix__t_aws_s3_tier_footage__t_aws_rds_tier__0
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct EdgeRouteReversals<'id>(Set<EdgeId<'id>>);

impl<'id> EdgeRouteReversals<'id> {
    /// Returns a new empty `EdgeRouteReversals` set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `EdgeRouteReversals` set with the given preallocated
    /// capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Set::with_capacity(capacity))
    }

    /// Returns the underlying set.
    pub fn into_inner(self) -> Set<EdgeId<'id>> {
        self.0
    }

    /// Returns true if the set is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the number of edge IDs in this set.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Converts this `EdgeRouteReversals` into one with a `'static` lifetime.
    ///
    /// If any inner `Cow` is borrowed, this will clone the string to create
    /// an owned version.
    pub fn into_static(self) -> EdgeRouteReversals<'static> {
        EdgeRouteReversals(
            self.0
                .into_iter()
                .map(|edge_id| edge_id.into_static())
                .collect(),
        )
    }
}

impl<'id> Deref for EdgeRouteReversals<'id> {
    type Target = Set<EdgeId<'id>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'id> DerefMut for EdgeRouteReversals<'id> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'id> From<Set<EdgeId<'id>>> for EdgeRouteReversals<'id> {
    fn from(inner: Set<EdgeId<'id>>) -> Self {
        Self(inner)
    }
}

impl<'id> FromIterator<EdgeId<'id>> for EdgeRouteReversals<'id> {
    fn from_iter<I: IntoIterator<Item = EdgeId<'id>>>(iter: I) -> Self {
        Self(Set::from_iter(iter))
    }
}
