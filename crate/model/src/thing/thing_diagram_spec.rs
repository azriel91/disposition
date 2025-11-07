use serde::{Deserialize, Serialize};

/// Diagram specification for things / objects.
///
/// Use this when you want to create a diagram that represents the relationships
/// between things or objects.
///
/// This isn't named "EntityDiagramSpec" to reduce overloading the term and
/// creating ambiguity with entity relationship diagrams.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ThingDiagramSpec {
    //
}
