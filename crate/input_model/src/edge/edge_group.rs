use serde::{Deserialize, Serialize};

use crate::thing::ThingId;

use super::EdgeKind;

/// An edge group combining an [`EdgeKind`] with the list of things it connects.
///
/// # Examples
///
/// ```yaml
/// thing_dependencies:
///   edge_dep_t_localhost__t_github_user_repo__pull:
///     kind: cyclic
///     things:
///       - t_localhost
///       - t_github_user_repo
///
///   edge_dep_t_localhost__t_github_user_repo__push:
///     kind: sequence
///     things:
///       - t_localhost
///       - t_github_user_repo
///
///   edge_dep_t_github_user_repo__t_github_user_repo__within:
///     kind: symmetric
///     things:
///       - t_github_user_repo
/// ```
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct EdgeGroup<'id> {
    /// The kind of edge -- cyclic, sequence, or symmetric.
    pub kind: EdgeKind,

    /// The things involved in this edge group.
    #[serde(default)]
    pub things: Vec<ThingId<'id>>,
}

impl<'id> EdgeGroup<'id> {
    /// Returns a new `EdgeGroup` with the given kind and things.
    pub fn new(kind: EdgeKind, things: Vec<ThingId<'id>>) -> Self {
        Self { kind, things }
    }
}
