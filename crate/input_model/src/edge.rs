pub use disposition_model_common::edge::{EdgeDescs, EdgeLabel, EdgeLabels};

pub use self::{
    edge_group::EdgeGroup,
    edge_kind::{EdgeKind, EdgeKindParseError},
};

mod edge_group;
mod edge_kind;
