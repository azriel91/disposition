use serde::{Deserialize, Serialize};

/// Level of detail to render in a diagram.
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum DiagramLod {
    /// Entity names are shown, no descriptions.
    Simple,
    /// Entity names and descriptions are shown.
    Normal,
}
