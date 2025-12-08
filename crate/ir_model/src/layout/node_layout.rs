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
///       padding_top: 4.0
///       padding_right: 4.0
///       padding_bottom: 4.0
///       padding_left: 4.0
///       margin_top: 0.0
///       margin_right: 0.0
///       margin_bottom: 0.0
///       margin_left: 0.0
///       gap: 4.0
///   proc_app_dev:
///     flex:
///       direction: "column"
///       wrap: false
///       padding_top: 2.0
///       padding_right: 2.0
///       padding_bottom: 2.0
///       padding_left: 2.0
///       margin_top: 0.0
///       margin_right: 0.0
///       margin_bottom: 0.0
///       margin_left: 0.0
///       gap: 2.0
///
///   # Leaf nodes with no children
///   proc_app_dev_step_repository_clone: none
///   proc_app_dev_step_project_build: none
///   tag_app_development: none
///   tag_deployment: none
///   t_aws_iam_ecs_policy: none
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
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
