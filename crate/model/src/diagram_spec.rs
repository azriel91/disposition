use indexmap::IndexMap;

use crate::{
    group::{GroupId, GroupSpec},
    process::{ProcessDiagramSpec, ProcessId},
    thing::ThingDiagramSpec,
};

/// The kinds of diagrams that can be generated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagramSpec {
    /// Specification for things / objects.
    things: ThingDiagramSpec,
    /// Specification for processes.
    ///
    /// These may be related to the things in the diagram.
    processes: IndexMap<ProcessId, ProcessDiagramSpec>,
    ///
    groups: IndexMap<GroupId, GroupSpec>,
}
