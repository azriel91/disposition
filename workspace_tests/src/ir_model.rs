const EXAMPLE_IR: &str = include_str!("example_ir.yaml");

use disposition::ir_model::IrDiagram;

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
    assert_eq!(18, diagram.node_copy_text.len());
    assert_eq!(
        &["t_localhost", "t_localhost_repo", "t_localhost_repo_src"],
        diagram
            .node_copy_text
            .iter()
            .take(3)
            .map(|(node_id, _)| node_id.as_str())
            .collect::<Vec<_>>()
            .as_slice()
    );
    assert_eq!(1, diagram.thing_descs.len());
    assert_eq!(4, diagram.edge_descs.len());
    assert_eq!(8, diagram.entity_tooltips.len());
    assert_eq!(
        &["t_localhost"],
        diagram
            .thing_descs
            .iter()
            .map(|(id, _)| id.as_str())
            .collect::<Vec<_>>()
            .as_slice()
    );
    assert_eq!(
        &[
            "edge_ix_t_localhost__t_github_user_repo__pull",
            "edge_ix_t_localhost__t_github_user_repo__push",
        ],
        diagram
            .edge_descs
            .iter()
            .take(2)
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
    assert_eq!(12, diagram.edge_groups.len());
    assert_eq!(64, diagram.entity_types.len());
    // 52 base entries + 8 interaction edges each getting a `__halo` entry
    // + 8 interaction edges each getting a `__halo_outline` entry
    // + 8 interaction edges each getting a `__label_bg` entry
    // + 8 interaction edges each getting a `__desc_bg` entry
    // + 8 dependency edges each getting a `__label_bg` entry
    // + 8 dependency edges each getting a `__desc_bg` entry.
    assert_eq!(100, diagram.tailwind_classes.len());
    assert_eq!(36, diagram.node_layouts.len());
    assert!(!diagram.css.is_empty());
}
