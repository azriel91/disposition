use serde::{Deserialize, Serialize};

use crate::{DiagramLod, Dimension};

/// The width and height of a diagram, and the level of detail to render.
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct DimensionAndLod {
    /// The size of the diagram.
    pub dimension: Dimension,
    /// The level of detail to render in the diagram.
    pub lod: DiagramLod,
}

impl DimensionAndLod {
    /// Returns a new `DimensionAndLod` with the given dimension and level of
    /// detail.
    pub fn new(dimension: Dimension, lod: DiagramLod) -> Self {
        Self { dimension, lod }
    }

    /// Returns a new `DimensionAndLod` with [`Dimension::Sm`] and
    /// [`DiagramLod::Simple`].
    pub fn default_sm() -> Self {
        Self::new(Dimension::Sm, DiagramLod::Simple)
    }

    /// Returns a new `DimensionAndLod` with [`Dimension::Md`] and
    /// [`DiagramLod::Normal`].
    pub fn default_md() -> Self {
        Self::new(Dimension::Md, DiagramLod::Normal)
    }

    /// Returns a new `DimensionAndLod` with [`Dimension::Lg`] and
    /// [`DiagramLod::Normal`].
    pub fn default_lg() -> Self {
        Self::new(Dimension::Lg, DiagramLod::Normal)
    }

    /// Returns a new `DimensionAndLod` with [`Dimension::_2xl`] and
    /// [`DiagramLod::Normal`].
    pub fn default_2xl() -> Self {
        Self::new(Dimension::_2xl, DiagramLod::Normal)
    }
}
