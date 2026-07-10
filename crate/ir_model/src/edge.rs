pub use disposition_model_common::edge::{EdgeDescs, EdgeId, EdgeLabel, EdgeLabels};

pub use self::{
    edge::Edge, edge_face_assignment::EdgeFaceAssignment,
    edge_face_assignments::EdgeFaceAssignments, edge_group::EdgeGroup, edge_groups::EdgeGroups,
    edge_route_reversals::EdgeRouteReversals,
};

#[allow(clippy::module_inception)] // We have an `edge` inner module, but it is intentional.
mod edge;
mod edge_face_assignment;
mod edge_face_assignments;
mod edge_group;
mod edge_groups;
mod edge_route_reversals;
