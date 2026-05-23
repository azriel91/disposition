pub use disposition_model_common::edge::{EdgeId, EdgeLabel, EdgeLabels};

pub use self::{
    edge::Edge, edge_face_assignment::EdgeFaceAssignment,
    edge_face_assignments::EdgeFaceAssignments, edge_group::EdgeGroup, edge_groups::EdgeGroups,
};

#[allow(clippy::module_inception)] // We have an `edge` inner module, but it is intentional.
mod edge;
mod edge_face_assignment;
mod edge_face_assignments;
mod edge_group;
mod edge_groups;
