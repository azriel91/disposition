//! SVG diagram generator intermediate representation.
//!
//! The intermediate representation is the computed data structure from
//! combining the layered values from the input data. It is used to generate
//! the final SVG output.

#[macro_use]
extern crate id_newtype;

// Re-exports
// This allows consumers to not need to depend on `utoipa` manually.
#[cfg(feature = "openapi")]
pub use utoipa;

#[cfg(feature = "openapi")]
pub use crate::api_doc::ApiDoc;
pub use crate::ir_diagram::IrDiagram;

pub mod common;
pub mod edge;
pub mod entity;
pub mod layout;
pub mod node;

#[cfg(feature = "openapi")]
mod api_doc;
mod ir_diagram;

#[cfg(test)]
mod tests {
    const EXAMPLE_IR: &str = include_str!("example_ir.yaml");

    use crate::IrDiagram;

    #[test]
    fn test_parse_example_ir() {
        let diagram = serde_saphyr::from_str::<IrDiagram>(EXAMPLE_IR).unwrap();
        assert_eq!(31, diagram.nodes.len());
        assert_eq!(
            &["t_aws", "t_aws_iam", "t_aws_iam_ecs_policy",],
            diagram
                .nodes
                .iter()
                .take(3)
                .map(|(node_id, _)| node_id.as_str())
                .collect::<Vec<_>>()
                .as_slice()
        );
        assert_eq!(31, diagram.node_copy_text.len());
        // Verifies that order is maintained from the merge key, even when we override
        // values.
        assert_eq!(
            &["t_aws", "t_aws_iam", "t_aws_iam_ecs_policy",],
            diagram
                .node_copy_text
                .iter()
                .take(3)
                .map(|(node_id, _)| node_id.as_str())
                .collect::<Vec<_>>()
                .as_slice()
        );
        assert_eq!(10, diagram.entity_descs.len());
        assert_eq!(
            &[
                "proc_app_release_step_crate_version_update",
                "proc_app_release_step_pull_request_open",
                "proc_app_release_step_gh_actions_build",
            ],
            diagram
                .entity_descs
                .iter()
                .take(3)
                .map(|(id, _)| id.as_str())
                .collect::<Vec<_>>()
                .as_slice()
        );
        // We care that tags come before processes, and processes come before things
        assert_eq!(
            &[
                "tag_app_development",
                "tag_deployment",
                "proc_app_dev",
                "proc_app_release",
                "proc_i12e_region_tier_app_deploy",
                "t_aws",
                "t_github",
                "t_localhost",
            ],
            diagram
                .node_hierarchy
                .iter()
                .map(|(node_id, _node_hierarchy)| node_id.as_str())
                .collect::<Vec<_>>()
                .as_slice()
        );
        assert_eq!(6, diagram.edge_groups.len());
        assert_eq!(38, diagram.entity_types.len());
        assert_eq!(10, diagram.tailwind_classes.len());
        assert_eq!(16, diagram.node_layout.len());
        assert!(!diagram.css.is_empty());
    }
}
