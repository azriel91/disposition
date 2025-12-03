use serde::{Deserialize, Serialize};

use crate::thing::ThingId;

/// Specifies the kind of edge and the things it connects.
///
/// Edges can be either cyclic (forming a loop) or sequential (one-way chain).
///
/// # Examples
///
/// ```yaml
/// # Cyclic edge - last thing connects back to first
/// edge_t_localhost__t_github_user_repo__pull:
///   cyclic:
///     - t_localhost
///     - t_github_user_repo
///
/// # Sequential edge - one-way chain from first to last
/// edge_t_localhost__t_github_user_repo__push:
///   sequence:
///     - t_localhost
///     - t_github_user_repo
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    /// Last thing in the list has an edge back to first thing.
    ///
    /// Should have at least one `thing`. When there is only one thing,
    /// it represents a self-loop.
    Cyclic(Vec<ThingId>),

    /// A sequence of 2 or more things forming a one-way chain.
    ///
    /// The edge goes from the first thing to the second, second to third, etc.
    Sequence(Vec<ThingId>),
}

impl EdgeKind {
    /// Returns the things involved in this edge.
    pub fn things(&self) -> &[ThingId] {
        match self {
            EdgeKind::Cyclic(things) => things,
            EdgeKind::Sequence(things) => things,
        }
    }

    /// Returns true if this is a cyclic edge.
    pub fn is_cyclic(&self) -> bool {
        matches!(self, EdgeKind::Cyclic(_))
    }

    /// Returns true if this is a sequential edge.
    pub fn is_sequence(&self) -> bool {
        matches!(self, EdgeKind::Sequence(_))
    }
}
