use serde::{Deserialize, Serialize};

/// Things tagged with the same tag.
///
/// Allows selection / highlighting of things that are related to each other.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct TagThings {
    //
}
