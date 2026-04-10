use serde::{Deserialize, Serialize};

/// Issue encountered while mapping the input model to the intermediate
/// representation.
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Debug, Serialize, Deserialize, thiserror::Error, miette::Diagnostic)]
pub enum ModelToIrIssue {}
