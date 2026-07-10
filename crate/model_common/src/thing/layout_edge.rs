use serde::{Deserialize, Serialize};

use crate::Id;

/// A single invisible edge between two things that affects their rank (and
/// hence layout position), without ever being rendered as a path.
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
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct LayoutEdge<'id> {
    /// The thing this edge starts from -- ranked before `to`.
    pub from: Id<'id>,
    /// The thing this edge points to -- ranked after `from`.
    pub to: Id<'id>,
}

impl<'id> LayoutEdge<'id> {
    /// Returns a new `LayoutEdge`.
    pub fn new(from: Id<'id>, to: Id<'id>) -> Self {
        Self { from, to }
    }

    /// Converts this `LayoutEdge` into one with a `'static` lifetime.
    ///
    /// If any inner `Cow` is borrowed, this will clone the string to create
    /// an owned version.
    pub fn into_static(self) -> LayoutEdge<'static> {
        LayoutEdge {
            from: self.from.into_static(),
            to: self.to.into_static(),
        }
    }
}
