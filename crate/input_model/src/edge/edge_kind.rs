use serde::{Deserialize, Serialize};

use crate::thing::ThingId;

/// Specifies the kind of edge and the things it connects.
///
/// Edges can be either cyclic (forming a loop) or sequential (one-way chain).
///
/// # Examples
///
/// ```yaml
/// thing_dependencies:
///   # Cyclic edge - last thing connects back to first
///   edge_dep_t_localhost__t_github_user_repo__pull: # <-- value is an `EdgeKind::Cyclic`
///     cyclic:
///       - t_localhost
///       - t_github_user_repo
///
///   # Sequential edge - one-way chain from first to last
///   edge_dep_t_localhost__t_github_user_repo__push: # <-- value is an `EdgeKind::Sequence`
///     sequence:
///       - t_localhost
///       - t_github_user_repo
///
///   # Symmetric edge - forward chain then reverse chain back to first
///   edge_dep_t_github_user_repo__t_github_user_repo__within: # <-- value is an `EdgeKind::Symmetric`
///     symmetric:
///       - t_github_user_repo
/// ```
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind<'id> {
    /// Last thing in the list has an edge back to first thing.
    ///
    /// Should have at least one `thing`. When there is only one thing,
    /// it represents a self-loop.
    Cyclic(Vec<ThingId<'id>>),

    /// A sequence of 2 or more things forming a one-way chain.
    ///
    /// The edge goes from the first thing to the second, second to third, etc.
    Sequence(Vec<ThingId<'id>>),

    /// A symmetric edge where things connect forward then back.
    ///
    /// For a list of things A, B, C, the edges are: A -> B -> C -> B -> A.
    /// Should have at least one `thing`. When there is only one thing,
    /// it represents a request and response to itself.
    Symmetric(Vec<ThingId<'id>>),
}

impl<'id> EdgeKind<'id> {
    /// Returns the things involved in this edge.
    pub fn things(&self) -> &[ThingId<'id>] {
        match self {
            EdgeKind::Cyclic(things) => things,
            EdgeKind::Sequence(things) => things,
            EdgeKind::Symmetric(things) => things,
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

    /// Returns true if this is a symmetric edge.
    pub fn is_symmetric(&self) -> bool {
        matches!(self, EdgeKind::Symmetric(_))
    }
}
