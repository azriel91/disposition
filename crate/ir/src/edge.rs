pub use disposition_model_common::edge::EdgeId;

pub use self::{
    edge::Edge, edge_group::EdgeGroup, edge_group_id::EdgeGroupId, edge_groups::EdgeGroups,
};

#[allow(clippy::module_inception)] // We have an `edge` inner module, but it is intentional.
mod edge;
mod edge_group;
mod edge_group_id;
mod edge_groups;
