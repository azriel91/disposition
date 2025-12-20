use serde::{Deserialize, Serialize};

/// Issue encountered while mapping the input model to the intermediate
/// representation.
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Serialize, Deserialize, thiserror::Error, miette::Diagnostic)]
pub enum ModelToIrIssue {}
