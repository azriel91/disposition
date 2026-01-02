use serde::{Deserialize, Serialize};

use crate::node::NodeId;

/// A rendered segment within a line of text.
///
/// Each markdown line is split into segments based on whether it has custom
/// rendering, or anything else. Currently the following markdown nodes have
/// custom rendering:
///
/// * images
/// * links
/// * everything else (treated as text)
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum NodeContentSpan {}
