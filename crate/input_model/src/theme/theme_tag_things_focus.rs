use std::ops::{Deref, DerefMut};

use disposition_model_common::Map;
use serde::{Deserialize, Serialize};

use crate::theme::{TagIdOrDefaults, ThemeStyles};

/// Styles when a tag is focused, applied to all tags or specific tags.
///
/// When a tag is focused, the things listed for it in `tag_things` are
/// highlighted and the rest are dimmed. This map controls both sides:
///
/// * `node_defaults` styles the *included* things (those in the focused tag).
/// * `node_excluded_defaults` styles the *excluded* things (those not in it) --
///   e.g. dimming them with `opacity`.
///
/// Edges are highlighted / dimmed based on whether their endpoint things are in
/// the focused tag; tags currently hold things only, so adding an edge group to
/// a tag directly is not yet supported.
///
/// The `tag_defaults` key applies styles to all tags uniformly. Specific tag
/// IDs can be used to override defaults for particular tags.
///
/// # Example
///
/// ```yaml
/// theme_tag_things_focus:
///   tag_defaults:
///     # things in the focused tag
///     node_defaults:
///       style_aliases_applied: [shade_pale, stroke_dashed_animated]
///     # things not in the focused tag
///     node_excluded_defaults:
///       opacity: "0.5"
///
///   tag_app_development:
///     node_defaults:
///       style_aliases_applied: [stroke_dashed_animated]
///     node_excluded_defaults:
///       opacity: "0.3"
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(bound(
    deserialize = "TagIdOrDefaults<'id>: Deserialize<'de>, ThemeStyles<'id>: Deserialize<'de>"
))]
pub struct ThemeTagThingsFocus<'id>(Map<TagIdOrDefaults<'id>, ThemeStyles<'id>>);

impl<'id> ThemeTagThingsFocus<'id> {
    /// Returns a new empty `ThemeTagThingsFocus` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `ThemeTagThingsFocus` map with the given
    /// preallocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<TagIdOrDefaults<'id>, ThemeStyles<'id>> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<'id> Deref for ThemeTagThingsFocus<'id> {
    type Target = Map<TagIdOrDefaults<'id>, ThemeStyles<'id>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'id> DerefMut for ThemeTagThingsFocus<'id> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'id> From<Map<TagIdOrDefaults<'id>, ThemeStyles<'id>>> for ThemeTagThingsFocus<'id> {
    fn from(inner: Map<TagIdOrDefaults<'id>, ThemeStyles<'id>>) -> Self {
        Self(inner)
    }
}

impl<'id> FromIterator<(TagIdOrDefaults<'id>, ThemeStyles<'id>)> for ThemeTagThingsFocus<'id> {
    fn from_iter<I: IntoIterator<Item = (TagIdOrDefaults<'id>, ThemeStyles<'id>)>>(
        iter: I,
    ) -> Self {
        Self(Map::from_iter(iter))
    }
}
