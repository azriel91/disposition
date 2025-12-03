use serde::{Deserialize, Serialize};

use crate::{process::Processes, tag::TagNames, thing::ThingNames};

/// The kinds of diagrams that can be generated.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct InputDiagram {
    /// Things in the diagram and their display labels.
    things: ThingNames,
    /// Processes are groupings of interactions between things sequenced over
    /// time.
    processes: Processes,
    /// Tags are labels that can be associated with things, so that the things
    /// can be highlighted when the tag is focused.
    tags: TagNames,
}
