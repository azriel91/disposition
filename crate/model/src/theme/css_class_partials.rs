use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::{common::Map, theme::ThemeAttr};

/// Partial CSS class name for each theme attribute. `Map<ThemeAttr,
/// String>` newtype.
///
/// These are *partial* CSS utility class names as an entry may be
/// `StrokeColorNormal: "slate-600"`, whereas the final CSS class name
/// may be `"[&>path]:stroke-slate-600"`.
///
/// Also, one CSS class partial may used to compute multiple CSS classes, such
/// as `StrokeColor: "slate"` mapping to:
///
/// * `"stroke-slate-600"`
/// * `"focus:stroke-slate-500"`
/// * `"hover:stroke-slate-400"`
/// * `"focus:hover:stroke-slate-400"`
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct CssClassPartials(Map<ThemeAttr, String>);

impl CssClassPartials {
    /// Returns a new `CssClassPartials` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `CssClassPartials` map with the given preallocated
    /// capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<ThemeAttr, String> {
        self.0
    }
}

impl Deref for CssClassPartials {
    type Target = Map<ThemeAttr, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CssClassPartials {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Map<ThemeAttr, String>> for CssClassPartials {
    fn from(inner: Map<ThemeAttr, String>) -> Self {
        Self(inner)
    }
}

impl FromIterator<(ThemeAttr, String)> for CssClassPartials {
    fn from_iter<I: IntoIterator<Item = (ThemeAttr, String)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
