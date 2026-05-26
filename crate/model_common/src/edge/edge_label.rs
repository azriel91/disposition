use serde::{Deserialize, Serialize};

/// Text labels displayed at each endpoint of a diagram edge.
///
/// Both endpoints may show different text, allowing the source and destination
/// context to be described independently.
///
/// # Examples
///
/// ```yaml
/// edge_labels:
///   edge_t_localhost__t_github_user_repo__pull__0:
///     from: "local branch"
///     to: "remote branch"
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct EdgeLabel {
    /// Text label displayed near the `from` endpoint of the edge.
    ///
    /// May be empty if no label is needed at the source.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub from: String,

    /// Text label displayed near the `to` endpoint of the edge.
    ///
    /// May be empty if no label is needed at the destination.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub to: String,
}
