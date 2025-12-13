use std::ops::{Deref, DerefMut};

use disposition_model_common::Map;
use serde::{Deserialize, Serialize};

use crate::theme::{StyleAlias, ThemeAttr};

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
///
/// # Example
///
/// ```yaml
/// node_defaults: # <-- this is a `CssClassPartials` map
///   style_aliases_applied: [shade_light]
///   shape_color: "slate"
///   stroke_style: "solid"
///   stroke_width: "1"
///   visibility: "visible"
///   fill_shade_normal: "300"
///   fill_shade_hover: "200"
///   fill_shade_focus: "400"
///   fill_shade_active: "500"
///   stroke_shade_normal: "400"
///   text_shade: "900"
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct CssClassPartials {
    /// The style aliases applied to the CSS class partials.
    #[serde(default)]
    style_aliases_applied: Vec<StyleAlias>,
    /// The map of CSS class partials.
    #[serde(default, flatten)]
    partials: Map<ThemeAttr, String>,
}

impl CssClassPartials {
    /// Returns a new `CssClassPartials` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `CssClassPartials` map with the given preallocated
    /// capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            style_aliases_applied: Vec::new(),
            partials: Map::with_capacity(capacity),
        }
    }

    /// Returns the style aliases applied to the CSS class partials.
    pub fn style_aliases_applied(&self) -> &[StyleAlias] {
        &self.style_aliases_applied
    }

    /// Returns a mutable reference to the style aliases applied to the CSS
    /// class partials.
    pub fn style_aliases_applied_mut(&mut self) -> &mut Vec<StyleAlias> {
        &mut self.style_aliases_applied
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> (Vec<StyleAlias>, Map<ThemeAttr, String>) {
        let CssClassPartials {
            style_aliases_applied,
            partials,
        } = self;

        (style_aliases_applied, partials)
    }
}

impl Deref for CssClassPartials {
    type Target = Map<ThemeAttr, String>;

    fn deref(&self) -> &Self::Target {
        &self.partials
    }
}

impl DerefMut for CssClassPartials {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.partials
    }
}

impl From<Map<ThemeAttr, String>> for CssClassPartials {
    fn from(partials: Map<ThemeAttr, String>) -> Self {
        Self {
            style_aliases_applied: Vec::new(),
            partials,
        }
    }
}

impl FromIterator<(ThemeAttr, String)> for CssClassPartials {
    fn from_iter<I: IntoIterator<Item = (ThemeAttr, String)>>(iter: I) -> Self {
        Self {
            style_aliases_applied: Vec::new(),
            partials: Map::from_iter(iter),
        }
    }
}
