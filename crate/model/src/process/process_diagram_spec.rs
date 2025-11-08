use serde::{Deserialize, Serialize};

/// Represents the relationships between steps in a process.
///
/// This can be associated with [`ThingId`]s in a [`ThingDiagramSpec`].
///
/// This isn't named "SequenceDiagramSpec" to reduce overloading the term and
/// creating ambiguity with sequence diagrams.
///
/// [`ThingId`]: crate::thing::ThingId
/// [`ThingDiagramSpec`]: crate::thing::ThingDiagramSpec
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ProcessDiagramSpec {
    //
}
