use serde::{Deserialize, Serialize};

use crate::layout::FlexLayout;

/// Layout configuration for a node.
///
/// A node can either have a flex layout (for container nodes with children)
/// or no layout (for leaf nodes without children).
///
/// # Example
///
/// ```yaml
/// node_layout:
///   # Container with flex layout
///   _root:
///     flex:
///       direction: "column_reverse"
///       wrap: true
///       gap: "4"
///   proc_app_dev:
///     flex:
///       direction: "column"
///       wrap: false
///       gap: "2"
///
///   # Leaf nodes with no children
///   proc_app_dev_step_repository_clone: none
///   proc_app_dev_step_project_build: none
///   tag_app_development: none
///   tag_deployment: none
///   t_aws_iam_ecs_policy: none
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeLayout {
    /// Flex layout for container nodes with children.
    Flex(FlexLayout),

    /// No layout for leaf nodes (nodes without children to lay out).
    #[default]
    None,
}

impl From<FlexLayout> for NodeLayout {
    fn from(flex: FlexLayout) -> Self {
        NodeLayout::Flex(flex)
    }
}
