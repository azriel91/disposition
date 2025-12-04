use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::{
    common::Map,
    theme::{CssClassPartials, IdOrDefaults},
};

/// CSS utility class partials for each element. `Map<IdOrDefaults,
/// CssClassPartials>` newtype.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ThemeStyles(Map<IdOrDefaults, CssClassPartials>);

impl ThemeStyles {
    /// Returns a new `ThemeStyles` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `ThemeStyles` map with the given preallocated
    /// capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<IdOrDefaults, CssClassPartials> {
        self.0
    }
}

impl Deref for ThemeStyles {
    type Target = Map<IdOrDefaults, CssClassPartials>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ThemeStyles {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Map<IdOrDefaults, CssClassPartials>> for ThemeStyles {
    fn from(inner: Map<IdOrDefaults, CssClassPartials>) -> Self {
        Self(inner)
    }
}

impl FromIterator<(IdOrDefaults, CssClassPartials)> for ThemeStyles {
    fn from_iter<I: IntoIterator<Item = (IdOrDefaults, CssClassPartials)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
