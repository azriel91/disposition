//! Tests for input to IR diagram mapping.

const EXAMPLE_INPUT: &str = include_str!("example_input.yaml");

use disposition::{
    input_ir_model::IrDiagramAndIssues,
    input_model::InputDiagram,
    ir_model::{
        entity::EntityType,
        layout::{FlexDirection, NodeLayout},
        node::NodeId,
    },
};
use disposition_input_ir_rt::InputToIrDiagramMapper;
use disposition_model_common::{edge::EdgeGroupId, id, Id};

#[test]
fn test_input_to_ir_mapping() {
    let input_diagram = serde_saphyr::from_str::<InputDiagram>(EXAMPLE_INPUT).unwrap();
    let ir_and_issues = InputToIrDiagramMapper::map(input_diagram);

    let IrDiagramAndIssues { diagram, issues } = ir_and_issues;

    // There should be no issues during mapping
    assert!(issues.is_empty(), "Expected no issues, got: {:?}", issues);

    // 1. Verify NodeNames populated from things, tags, processes, and process steps
    // 18 things + 2 tags + 3 processes + 8 process steps = 31 nodes
    assert_eq!(31, diagram.nodes.len());

    // Check specific nodes exist
    let t_aws = NodeId::from(id!("t_aws"));
    let tag_app_development = NodeId::from(id!("tag_app_development"));
    let proc_app_dev = NodeId::from(id!("proc_app_dev"));
    let proc_app_dev_step_repository_clone =
        NodeId::from(id!("proc_app_dev_step_repository_clone"));

    assert!(diagram.nodes.contains_key(&t_aws));
    assert!(diagram.nodes.contains_key(&tag_app_development));
    assert!(diagram.nodes.contains_key(&proc_app_dev));
    assert!(diagram
        .nodes
        .contains_key(&proc_app_dev_step_repository_clone));

    // Verify process name is used (not process id)
    assert_eq!("App Development", diagram.nodes.get(&proc_app_dev).unwrap());

    // 2. Verify NodeCopyText populated from thing_copy_text
    // The input has 18 thing_copy_text entries (from YAML anchor merge)
    assert_eq!(18, diagram.node_copy_text.len());
    let t_localhost_repo = NodeId::from(id!("t_localhost_repo"));
    assert_eq!(
        "~/work/web_app",
        diagram.node_copy_text.get(&t_localhost_repo).unwrap()
    );

    // 3. Verify NodeHierarchy structure
    // Tags should come before processes, processes before things
    let top_level_keys: Vec<&str> = diagram
        .node_hierarchy
        .iter()
        .map(|(node_id, _)| node_id.as_str())
        .collect();

    // First should be tags
    assert!(top_level_keys[0].starts_with("tag_"));
    assert!(top_level_keys[1].starts_with("tag_"));

    // Then processes
    assert!(top_level_keys[2].starts_with("proc_"));
    assert!(top_level_keys[3].starts_with("proc_"));
    assert!(top_level_keys[4].starts_with("proc_"));

    // Then things
    assert!(top_level_keys[5].starts_with("t_"));

    // Verify process steps are nested under processes
    let proc_app_dev_children = diagram.node_hierarchy.get(&proc_app_dev).unwrap();
    assert_eq!(2, proc_app_dev_children.len());
    assert!(proc_app_dev_children.contains_key(&proc_app_dev_step_repository_clone));
    let proc_app_dev_step_project_build = NodeId::from(id!("proc_app_dev_step_project_build"));
    assert!(proc_app_dev_children.contains_key(&proc_app_dev_step_project_build));

    // Verify thing hierarchy is preserved
    let t_aws_children = diagram.node_hierarchy.get(&t_aws).unwrap();
    let t_aws_iam = NodeId::from(id!("t_aws_iam"));
    let t_aws_ecr = NodeId::from(id!("t_aws_ecr"));
    assert!(t_aws_children.contains_key(&t_aws_iam));
    assert!(t_aws_children.contains_key(&t_aws_ecr));

    // 4. Verify EdgeGroups populated from thing_dependencies
    assert_eq!(6, diagram.edge_groups.len());

    // Check cyclic edge expansion
    let pull_edge_group_id = EdgeGroupId::from(id!("edge_t_localhost__t_github_user_repo__pull"));
    let pull_edges = diagram.edge_groups.get(&pull_edge_group_id).unwrap();
    assert_eq!(2, pull_edges.len()); // cyclic with 2 things = 2 edges

    // Check sequence edge expansion
    let push_edge_group_id = EdgeGroupId::from(id!("edge_t_localhost__t_github_user_repo__push"));
    let push_edges = diagram.edge_groups.get(&push_edge_group_id).unwrap();
    assert_eq!(1, push_edges.len()); // sequence with 2 things = 1 edge

    // Verify edge from/to
    assert_eq!("t_localhost", push_edges[0].from.as_str());
    assert_eq!("t_github_user_repo", push_edges[0].to.as_str());

    // 5. Verify EntityDescs includes input entity_descs and process step_descs
    // From example_input: entity_descs has 4 entries, plus step_descs from 3
    // processes
    assert!(diagram.entity_descs.len() >= 4);

    // Check entity desc from input
    let pull_edge_id = id!("edge_t_localhost__t_github_user_repo__pull");
    assert!(diagram.entity_descs.contains_key(&pull_edge_id));

    // Check step desc merged in
    let step_desc_id = id!("proc_app_dev_step_repository_clone");
    assert!(diagram.entity_descs.contains_key(&step_desc_id));

    // 6. Verify EntityTypes with defaults
    // Things should have type_thing_default
    let t_aws_id = id!("t_aws");
    let t_aws_types = diagram.entity_types.get(&t_aws_id).unwrap();
    assert!(t_aws_types
        .iter()
        .any(|entity_type| *entity_type == EntityType::ThingDefault));
    // And custom type if specified
    assert!(t_aws_types
        .iter()
        .any(|entity_type| *entity_type == EntityType::Custom(id!("type_organisation"))));

    // Tags should have tag_type_default
    let tag_id = id!("tag_app_development");
    let tag_types = diagram.entity_types.get(&tag_id).unwrap();
    assert!(tag_types
        .iter()
        .any(|entity_type| *entity_type == EntityType::TagDefault));

    // Processes should have type_process_default
    let proc_id = id!("proc_app_dev");
    let proc_types = diagram.entity_types.get(&proc_id).unwrap();
    assert!(proc_types
        .iter()
        .any(|entity_type| *entity_type == EntityType::ProcessDefault));

    // Process steps should have type_process_step_default
    let step_id = id!("proc_app_dev_step_repository_clone");
    let step_types = diagram.entity_types.get(&step_id).unwrap();
    assert!(step_types
        .iter()
        .any(|entity_type| *entity_type == EntityType::ProcessStepDefault));

    // Edges should have dependency and interaction types
    let edge_id = id!("edge_t_localhost__t_github_user_repo__pull__0");
    let edge_types = diagram.entity_types.get(&edge_id).unwrap();
    assert!(edge_types
        .iter()
        .any(|entity_type| *entity_type == EntityType::EdgeDependencyCyclicDefault));
    assert!(edge_types
        .iter()
        .any(|entity_type| *entity_type == EntityType::EdgeInteractionCyclicDefault));

    // 7. Verify CSS is passed through
    assert!(!diagram.css.is_empty());
}

#[test]
fn test_cyclic_edge_expansion() {
    // Test that cyclic edges create a loop
    let input_diagram = serde_saphyr::from_str::<InputDiagram>(EXAMPLE_INPUT).unwrap();
    let ir_and_issues = InputToIrDiagramMapper::map(input_diagram);
    let diagram = ir_and_issues.diagram;

    // edge_t_localhost__t_github_user_repo__pull is cyclic with:
    //
    // `[t_localhost, t_github_user_repo]`
    //
    // Should create:
    //
    // * `t_localhost -> t_github_user_repo`
    // * `t_github_user_repo -> t_localhost`
    let edge_group_id = EdgeGroupId::from(id!("edge_t_localhost__t_github_user_repo__pull"));
    let edges = diagram.edge_groups.get(&edge_group_id).unwrap();

    assert_eq!(2, edges.len());
    // First edge: t_localhost -> t_github_user_repo
    assert_eq!(id!("t_localhost"), *edges[0].from);
    assert_eq!(id!("t_github_user_repo"), *edges[0].to);
    // Second edge: t_github_user_repo -> t_localhost (cycle back)
    assert_eq!(id!("t_github_user_repo"), *edges[1].from);
    assert_eq!(id!("t_localhost"), *edges[1].to);
}

#[test]
fn test_self_loop_edge() {
    // Test that a cyclic edge with one thing creates a self-loop
    let input_diagram = serde_saphyr::from_str::<InputDiagram>(EXAMPLE_INPUT).unwrap();
    let ir_and_issues = InputToIrDiagramMapper::map(input_diagram);
    let diagram = ir_and_issues.diagram;

    // edge_t_localhost__t_localhost__within is cyclic with `[t_localhost]`
    //
    // Should create:
    //
    // * `t_localhost -> t_localhost` (self-loop)
    let edge_group_id = EdgeGroupId::from(id!("edge_t_localhost__t_localhost__within"));
    let edges = diagram.edge_groups.get(&edge_group_id).unwrap();

    assert_eq!(1, edges.len());
    assert_eq!(id!("t_localhost"), *edges[0].from);
    assert_eq!(id!("t_localhost"), *edges[0].to);
    assert!(edges[0].is_self_loop());
}

#[test]
fn test_sequence_edge_expansion() {
    // Test that sequence edges create a chain without cycling back
    let input_diagram = serde_saphyr::from_str::<InputDiagram>(EXAMPLE_INPUT).unwrap();
    let ir_and_issues = InputToIrDiagramMapper::map(input_diagram);
    let diagram = ir_and_issues.diagram;

    // edge_t_localhost__t_github_user_repo__push is sequence with:
    //
    // `[t_localhost, t_github_user_repo]`
    //
    // Should create:
    //
    // * `t_localhost -> t_github_user_repo` (no cycle back)
    let edge_group_id = EdgeGroupId::from(id!("edge_t_localhost__t_github_user_repo__push"));
    let edges = diagram.edge_groups.get(&edge_group_id).unwrap();

    assert_eq!(1, edges.len());
    assert_eq!("t_localhost", edges[0].from.as_str());
    assert_eq!("t_github_user_repo", edges[0].to.as_str());
}

#[test]
fn test_node_layout_containers() {
    // Test that container nodes get correct flex layouts
    let input_diagram = serde_saphyr::from_str::<InputDiagram>(EXAMPLE_INPUT).unwrap();
    let ir_and_issues = InputToIrDiagramMapper::map(input_diagram);
    let diagram = ir_and_issues.diagram;

    // _root container should have column_reverse direction
    let root_id = NodeId::from(id!("_root"));
    let root_layout = diagram.node_layout.get(&root_id).unwrap();
    if let NodeLayout::Flex(flex) = root_layout {
        assert_eq!(FlexDirection::ColumnReverse, flex.direction);
        assert!(flex.wrap);
        // Padding comes from node_defaults -> padding_normal -> 4.0
        assert_eq!(4.0, flex.padding_top);
        assert_eq!(4.0, flex.padding_right);
        assert_eq!(4.0, flex.padding_bottom);
        assert_eq!(4.0, flex.padding_left);
        assert_eq!(4.0, flex.gap);
    } else {
        panic!("Expected Flex layout for _root");
    }

    // _things_and_processes_container should have row_reverse direction
    let things_and_processes_container_id = NodeId::from(id!("_things_and_processes_container"));
    let things_and_processes_layout = diagram
        .node_layout
        .get(&things_and_processes_container_id)
        .unwrap();
    if let NodeLayout::Flex(flex) = things_and_processes_layout {
        assert_eq!(FlexDirection::RowReverse, flex.direction);
        assert!(flex.wrap);
    } else {
        panic!("Expected Flex layout for _things_and_processes_container");
    }

    // _processes_container should have row direction
    let processes_container_id = NodeId::from(id!("_processes_container"));
    let processes_layout = diagram.node_layout.get(&processes_container_id).unwrap();
    if let NodeLayout::Flex(flex) = processes_layout {
        assert_eq!(FlexDirection::Row, flex.direction);
        assert!(flex.wrap);
    } else {
        panic!("Expected Flex layout for _processes_container");
    }

    // _tags_container should have row direction
    let tags_container_id = NodeId::from(id!("_tags_container"));
    let tags_layout = diagram.node_layout.get(&tags_container_id).unwrap();
    if let NodeLayout::Flex(flex) = tags_layout {
        assert_eq!(FlexDirection::Row, flex.direction);
        assert!(flex.wrap);
    } else {
        panic!("Expected Flex layout for _tags_container");
    }

    // _things_container should have row direction
    let things_container_id = NodeId::from(id!("_things_container"));
    let things_layout = diagram.node_layout.get(&things_container_id).unwrap();
    if let NodeLayout::Flex(flex) = things_layout {
        assert_eq!(FlexDirection::Row, flex.direction);
        assert!(flex.wrap);
    } else {
        panic!("Expected Flex layout for _things_container");
    }
}

#[test]
fn test_node_layout_processes() {
    // Test that processes with steps get flex layout, steps get none
    let input_diagram = serde_saphyr::from_str::<InputDiagram>(EXAMPLE_INPUT).unwrap();
    let ir_and_issues = InputToIrDiagramMapper::map(input_diagram);
    let diagram = ir_and_issues.diagram;

    // proc_app_dev has steps, should have column flex layout
    let proc_id = NodeId::from(id!("proc_app_dev"));
    let proc_layout = diagram.node_layout.get(&proc_id).unwrap();
    if let NodeLayout::Flex(flex) = proc_layout {
        assert_eq!(FlexDirection::Column, flex.direction);
        assert!(!flex.wrap);
        // Padding comes from node_defaults -> padding_normal -> 4.0
        assert_eq!(4.0, flex.padding_top);
        assert_eq!(4.0, flex.gap);
    } else {
        panic!("Expected Flex layout for proc_app_dev");
    }

    // Process steps are leaves, should have None layout
    let step_id = NodeId::from(id!("proc_app_dev_step_repository_clone"));
    let step_layout = diagram.node_layout.get(&step_id).unwrap();
    assert_eq!(&NodeLayout::None, step_layout);

    let step2_id = NodeId::from(id!("proc_app_dev_step_project_build"));
    let step2_layout = diagram.node_layout.get(&step2_id).unwrap();
    assert_eq!(&NodeLayout::None, step2_layout);
}

#[test]
fn test_node_layout_tags() {
    // Test that tags are leaf nodes with no layout
    let input_diagram = serde_saphyr::from_str::<InputDiagram>(EXAMPLE_INPUT).unwrap();
    let ir_and_issues = InputToIrDiagramMapper::map(input_diagram);
    let diagram = ir_and_issues.diagram;

    let tag_0_id = NodeId::from(id!("tag_app_development"));
    let tag_0_layout = diagram.node_layout.get(&tag_0_id).unwrap();
    assert_eq!(&NodeLayout::None, tag_0_layout);

    let tag_1_id = NodeId::from(id!("tag_deployment"));
    let tag_1_layout = diagram.node_layout.get(&tag_1_id).unwrap();
    assert_eq!(&NodeLayout::None, tag_1_layout);
}

#[test]
fn test_node_layout_things_hierarchy() {
    // Test that things with children get flex layout, leaves get none
    let input_diagram = serde_saphyr::from_str::<InputDiagram>(EXAMPLE_INPUT).unwrap();
    let ir_and_issues = InputToIrDiagramMapper::map(input_diagram);
    let diagram = ir_and_issues.diagram;

    // t_aws has children (t_aws_iam, t_aws_ecr, t_aws_ecs), should have column flex
    // (depth 0)
    let t_aws_id = NodeId::from(id!("t_aws"));
    let t_aws_layout = diagram.node_layout.get(&t_aws_id).unwrap();
    if let NodeLayout::Flex(flex) = t_aws_layout {
        assert_eq!(FlexDirection::Column, flex.direction);
        // Padding from node_defaults -> padding_normal -> 4.0
        assert_eq!(4.0, flex.padding_top);
        assert_eq!(4.0, flex.gap);
    } else {
        panic!("Expected Flex layout for t_aws");
    }

    // t_aws_iam has children (t_aws_iam_ecs_policy), should have row flex (depth 1)
    let t_aws_iam_id = NodeId::from(id!("t_aws_iam"));
    let t_aws_iam_layout = diagram.node_layout.get(&t_aws_iam_id).unwrap();
    if let NodeLayout::Flex(flex) = t_aws_iam_layout {
        assert_eq!(FlexDirection::Row, flex.direction);
    } else {
        panic!("Expected Flex layout for t_aws_iam");
    }

    // t_aws_iam_ecs_policy is a leaf, should have None layout
    let leaf_id = NodeId::from(id!("t_aws_iam_ecs_policy"));
    let leaf_layout = diagram.node_layout.get(&leaf_id).unwrap();
    assert_eq!(&NodeLayout::None, leaf_layout);

    // t_aws_ecr_repo has children (images), should have column flex (depth 2)
    let t_aws_ecr_repo_id = NodeId::from(id!("t_aws_ecr_repo"));
    let t_aws_ecr_repo_layout = diagram.node_layout.get(&t_aws_ecr_repo_id).unwrap();
    if let NodeLayout::Flex(flex) = t_aws_ecr_repo_layout {
        assert_eq!(FlexDirection::Column, flex.direction);
    } else {
        panic!("Expected Flex layout for t_aws_ecr_repo");
    }

    // t_aws_ecr_repo_image_1 is a leaf
    let image_id = NodeId::from(id!("t_aws_ecr_repo_image_1"));
    let image_layout = diagram.node_layout.get(&image_id).unwrap();
    assert_eq!(&NodeLayout::None, image_layout);
}

#[test]
fn test_node_layout_padding_from_theme() {
    // Test that padding values are correctly resolved from theme
    let input_diagram = serde_saphyr::from_str::<InputDiagram>(EXAMPLE_INPUT).unwrap();
    let ir_and_issues = InputToIrDiagramMapper::map(input_diagram);
    let diagram = ir_and_issues.diagram;

    // All containers should get padding from node_defaults which uses
    // padding_normal (4.0)
    let t_aws_id = NodeId::from(id!("t_aws"));
    let t_aws_layout = diagram.node_layout.get(&t_aws_id).unwrap();
    if let NodeLayout::Flex(flex) = t_aws_layout {
        assert_eq!(4.0, flex.padding_top);
        assert_eq!(4.0, flex.padding_right);
        assert_eq!(4.0, flex.padding_bottom);
        assert_eq!(4.0, flex.padding_left);
        assert_eq!(0.0, flex.margin_top);
        assert_eq!(0.0, flex.margin_right);
        assert_eq!(0.0, flex.margin_bottom);
        assert_eq!(0.0, flex.margin_left);
        assert_eq!(4.0, flex.gap);
    } else {
        panic!("Expected Flex layout for t_aws");
    }
}

#[test]
fn test_tailwind_classes_generation() {
    let input_diagram = serde_saphyr::from_str::<InputDiagram>(EXAMPLE_INPUT).unwrap();
    let ir_and_issues = InputToIrDiagramMapper::map(input_diagram);
    let diagram = ir_and_issues.diagram;

    // Verify tailwind classes are generated
    assert!(!diagram.tailwind_classes.is_empty());

    // Test tag tailwind classes - should have peer/{id} class
    let tag_id = id!("tag_app_development");
    let tag_classes = diagram.tailwind_classes.get(&tag_id).unwrap();
    assert!(
        tag_classes.contains("\npeer/tag_app_development"),
        "Tag should have peer class. Got: {}",
        tag_classes
    );
    assert!(
        tag_classes.contains("\nvisible"),
        "Tag should have visibility. Got: {}",
        tag_classes
    );

    // Test process tailwind classes - should have group/{id} class
    let proc_id = id!("proc_app_dev");
    let proc_classes = diagram.tailwind_classes.get(&proc_id).unwrap();
    assert!(
        proc_classes.contains("\ngroup/proc_app_dev"),
        "Process should have group class. Got: {}",
        proc_classes
    );

    // Process should also have peer/{step_id} classes for each child step
    // This is because process nodes are sibling elements to thing/edge_group
    // elements, whereas process step nodes are not siblings.
    assert!(
        proc_classes.contains("\npeer/proc_app_dev_step_repository_clone"),
        "Process should have peer class for child step. Got: {}",
        proc_classes
    );
    assert!(
        proc_classes.contains("\npeer/proc_app_dev_step_project_build"),
        "Process should have peer class for child step. Got: {}",
        proc_classes
    );

    // Test process step tailwind classes - should have
    // group-focus-within/{process_id}:visible but NOT peer/{id} (that's on the
    // parent process now)
    let step_id = id!("proc_app_dev_step_repository_clone");
    let step_classes = diagram.tailwind_classes.get(&step_id).unwrap();
    assert!(
        !step_classes.contains("\npeer/proc_app_dev_step_repository_clone"),
        "Process step should NOT have peer class (it's on the parent process now). Got: {}",
        step_classes
    );
    assert!(
        step_classes.contains("\ngroup-focus-within/proc_app_dev:visible"),
        "Process step should have group-focus-within class. Got: {}",
        step_classes
    );

    // Test thing tailwind classes - t_aws should have yellow color from
    // base_styles
    let t_aws_id = id!("t_aws");
    let t_aws_classes = diagram.tailwind_classes.get(&t_aws_id).unwrap();
    assert!(
        t_aws_classes.contains("\nfill-yellow"),
        "t_aws should have yellow fill. Got: {}",
        t_aws_classes
    );
    assert!(
        t_aws_classes.contains("\nstroke-yellow"),
        "t_aws should have yellow stroke. Got: {}",
        t_aws_classes
    );

    // Test edge group tailwind classes
    let edge_group_id = id!("edge_t_localhost__t_github_user_repo__pull");
    let edge_classes = diagram.tailwind_classes.get(&edge_group_id).unwrap();
    assert!(
        edge_classes.contains("\nstroke-"),
        "Edge group should have stroke class. Got: {}",
        edge_classes
    );
    // Should have peer classes for interacting process steps
    assert!(
        edge_classes.contains("\npeer-[:focus-within]/proc_app_dev_step_repository_clone:"),
        "Edge should have peer class for interacting step. Got: {}",
        edge_classes
    );
}

#[test]
fn test_tailwind_classes_shade_resolution() {
    let input_diagram = serde_saphyr::from_str::<InputDiagram>(EXAMPLE_INPUT).unwrap();
    let ir_and_issues = InputToIrDiagramMapper::map(input_diagram);
    let diagram = ir_and_issues.diagram;

    // t_aws has type_organisation which uses shade_pale
    // shade_pale has fill_shade_normal: "100"
    let t_aws_id = id!("t_aws");
    let t_aws_classes = diagram.tailwind_classes.get(&t_aws_id).unwrap();
    assert!(
        t_aws_classes.contains("\nfill-yellow-100"),
        "t_aws should have fill-yellow-100 from shade_pale. Got: {}",
        t_aws_classes
    );
    assert!(
        t_aws_classes.contains("\nhover:fill-yellow-50"),
        "t_aws should have hover:fill-yellow-50 from shade_pale. Got: {}",
        t_aws_classes
    );

    // t_aws_iam has type_thing_default (shade_light) and type_service
    // type_service only specifies stroke_style, so color comes from
    // type_thing_default which is slate. shade_light has fill_shade_normal: "300"
    let t_aws_iam_id = id!("t_aws_iam");
    let t_aws_iam_classes = diagram.tailwind_classes.get(&t_aws_iam_id).unwrap();
    assert!(
        t_aws_iam_classes.contains("\nfill-slate-300"),
        "t_aws_iam should have fill-slate-300 from type_thing_default + shade_light. Got: {}",
        t_aws_iam_classes
    );
}

#[test]
fn test_tailwind_classes_stroke_style() {
    let input_diagram = serde_saphyr::from_str::<InputDiagram>(EXAMPLE_INPUT).unwrap();
    let ir_and_issues = InputToIrDiagramMapper::map(input_diagram);
    let diagram = ir_and_issues.diagram;

    // t_aws has type_organisation which has stroke_style: "dotted"
    // dotted should map to stroke-dasharray:2
    let t_aws_id = id!("t_aws");
    let t_aws_classes = diagram.tailwind_classes.get(&t_aws_id).unwrap();
    assert!(
        t_aws_classes.contains("\n[stroke-dasharray:2]"),
        "t_aws should have stroke-dasharray:2 from dotted stroke_style. Got: {}",
        t_aws_classes
    );

    // t_aws_iam has type_service which has stroke_style: "dashed"
    // dashed should map to stroke-dasharray:3
    let t_aws_iam_id = id!("t_aws_iam");
    let t_aws_iam_classes = diagram.tailwind_classes.get(&t_aws_iam_id).unwrap();
    assert!(
        t_aws_iam_classes.contains("\n[stroke-dasharray:3]"),
        "t_aws_iam should have stroke-dasharray:3 from dashed stroke_style. Got: {}",
        t_aws_iam_classes
    );
}

#[test]
fn test_tailwind_classes_thing_peer_classes() {
    let input_diagram = serde_saphyr::from_str::<InputDiagram>(EXAMPLE_INPUT).unwrap();
    let ir_and_issues = InputToIrDiagramMapper::map(input_diagram);
    let diagram = ir_and_issues.diagram;

    // t_localhost should have peer classes for process steps that interact with
    // edges involving t_localhost:
    // - edge_t_localhost__t_github_user_repo__pull is used by:
    //   - proc_app_dev_step_repository_clone
    //   - proc_app_release_step_pull_request_open
    // - edge_t_localhost__t_github_user_repo__push is used by:
    //   - proc_app_release_step_tag_and_push
    // - edge_t_localhost__t_localhost__within is used by:
    //   - proc_app_dev_step_project_build
    //   - proc_app_release_step_crate_version_update

    let t_localhost_id = id!("t_localhost");
    let t_localhost_classes = diagram.tailwind_classes.get(&t_localhost_id).unwrap();

    // Should have peer class for proc_app_dev_step_repository_clone
    assert!(
        t_localhost_classes.contains("\npeer-[:focus-within]/proc_app_dev_step_repository_clone:"),
        "t_localhost should have peer class for proc_app_dev_step_repository_clone. Got: {}",
        t_localhost_classes
    );

    // Should have peer class for proc_app_dev_step_project_build
    assert!(
        t_localhost_classes.contains("\npeer-[:focus-within]/proc_app_dev_step_project_build:"),
        "t_localhost should have peer class for proc_app_dev_step_project_build. Got: {}",
        t_localhost_classes
    );

    // Should NOT have peer class for proc_app_release_step_gh_actions_build
    // because that step interacts with
    // edge_t_github_user_repo__t_github_user_repo__within which doesn't involve
    // t_localhost
    assert!(
        !t_localhost_classes.contains("\npeer-[:focus-within]/proc_app_release_step_gh_actions_build:"),
        "t_localhost should NOT have peer class for proc_app_release_step_gh_actions_build. Got: {}",
        t_localhost_classes
    );

    // t_github_user_repo should have peer classes for process steps that interact
    // with edges involving t_github_user_repo
    let t_github_user_repo_id = id!("t_github_user_repo");
    let t_github_user_repo_classes = diagram
        .tailwind_classes
        .get(&t_github_user_repo_id)
        .unwrap();

    // Should have peer class for proc_app_release_step_gh_actions_build
    // because that step interacts with
    // edge_t_github_user_repo__t_github_user_repo__within
    assert!(
        t_github_user_repo_classes.contains("\npeer-[:focus-within]/proc_app_release_step_gh_actions_build:"),
        "t_github_user_repo should have peer class for proc_app_release_step_gh_actions_build. Got: {}",
        t_github_user_repo_classes
    );
}

#[test]
fn test_tailwind_classes_tag_peer_classes_for_included_things() {
    // Tests that things included in a tag get proper peer classes based on
    // theme_tag_things_focus NodeDefaults
    let input_diagram = serde_saphyr::from_str::<InputDiagram>(EXAMPLE_INPUT).unwrap();
    let ir_and_issues = InputToIrDiagramMapper::map(input_diagram);
    let diagram = ir_and_issues.diagram;

    // t_localhost is in tag_app_development
    // theme_tag_things_focus has:
    //   tag_defaults.node_defaults: style_aliases_applied: [shade_pale,
    // stroke_dashed_animated]   tag_app_development.node_defaults:
    // style_aliases_applied: [stroke_dashed_animated]
    //
    // The tag_app_development overrides tag_defaults, so it should use
    // stroke_dashed_animated but NOT shade_pale (which would give fill shades)
    // However, stroke_dashed_animated includes animate, so it should have animation

    let t_localhost_id = id!("t_localhost");
    let t_localhost_classes = diagram.tailwind_classes.get(&t_localhost_id).unwrap();

    // t_localhost is IN tag_app_development, so should have full peer classes with
    // animation
    assert!(
        t_localhost_classes
            .contains("peer-[:focus-within]/tag_app_development:animate-[stroke-dashoffset-move_2s_linear_infinite]"),
        "t_localhost should have animation for tag_app_development focus. Got: {}",
        t_localhost_classes
    );

    // Should have fill classes for tag_app_development
    assert!(
        t_localhost_classes.contains("\npeer-[:focus-within]/tag_app_development:fill-slate-"),
        "t_localhost should have fill classes for tag_app_development. Got: {}",
        t_localhost_classes
    );

    // t_localhost is NOT in tag_deployment, so should only have opacity class
    // (from node_excluded_defaults with opacity: 75 from tag_defaults)
    assert!(
        t_localhost_classes.contains("\npeer-[:focus-within]/tag_deployment:opacity-75"),
        "t_localhost should have opacity-75 for tag_deployment (excluded). Got: {}",
        t_localhost_classes
    );

    // Should NOT have full fill/stroke classes for tag_deployment since it's
    // excluded
    assert!(
        !t_localhost_classes.contains("\npeer-[:focus-within]/tag_deployment:fill-slate-"),
        "t_localhost should NOT have fill classes for tag_deployment. Got: {}",
        t_localhost_classes
    );
}

#[test]
fn test_tailwind_classes_tag_peer_classes_for_excluded_things() {
    // Tests that things NOT included in any tag get peer classes based on
    // theme_tag_things_focus NodeExcludedDefaults
    let input_diagram = serde_saphyr::from_str::<InputDiagram>(EXAMPLE_INPUT).unwrap();
    let ir_and_issues = InputToIrDiagramMapper::map(input_diagram);
    let diagram = ir_and_issues.diagram;

    // t_aws is not in any tag (tag_things only has tag_app_development and
    // tag_deployment) theme_tag_things_focus has:
    //   tag_defaults.node_excluded_defaults: opacity: "75"
    //   tag_app_development.node_excluded_defaults: opacity: "50"
    //
    // So t_aws should have:
    //   - peer-[:focus-within]/tag_app_development:opacity-50 (specific override)
    //   - peer-[:focus-within]/tag_deployment:opacity-75 (from tag_defaults)

    let t_aws_id = id!("t_aws");
    let t_aws_classes = diagram.tailwind_classes.get(&t_aws_id).unwrap();

    // t_aws is NOT in tag_app_development, so should have opacity from
    // tag_app_development's node_excluded_defaults (opacity: 50)
    assert!(
        t_aws_classes.contains("\npeer-[:focus-within]/tag_app_development:opacity-50"),
        "t_aws should have opacity-50 for tag_app_development (excluded with specific override). Got: {}",
        t_aws_classes
    );

    // t_aws is NOT in tag_deployment, so should have opacity from
    // tag_defaults.node_excluded_defaults (opacity: 75)
    assert!(
        t_aws_classes.contains("\npeer-[:focus-within]/tag_deployment:opacity-75"),
        "t_aws should have opacity-75 for tag_deployment (excluded, using tag_defaults). Got: {}",
        t_aws_classes
    );

    // Should NOT have animation classes since it's excluded from both tags
    assert!(
        !t_aws_classes.contains("\npeer-[:focus-within]/tag_app_development:animate-"),
        "t_aws should NOT have animation for tag_app_development. Got: {}",
        t_aws_classes
    );
}

#[test]
fn test_tailwind_classes_tag_peer_classes_tag_specific_override() {
    // Tests that tag-specific styles override tag_defaults
    let input_diagram = serde_saphyr::from_str::<InputDiagram>(EXAMPLE_INPUT).unwrap();
    let ir_and_issues = InputToIrDiagramMapper::map(input_diagram);
    let diagram = ir_and_issues.diagram;

    // t_github_user_repo is in BOTH tag_app_development and tag_deployment
    // For tag_app_development:
    //   - tag_app_development.node_defaults overrides tag_defaults.node_defaults
    //   - Should have stroke_dashed_animated but not shade_pale
    // For tag_deployment:
    //   - No tag_deployment.node_defaults, so uses tag_defaults.node_defaults
    //   - Should have shade_pale and stroke_dashed_animated

    let t_github_user_repo_id = id!("t_github_user_repo");
    let t_github_user_repo_classes = diagram
        .tailwind_classes
        .get(&t_github_user_repo_id)
        .unwrap();

    // Both tags should have animation (from stroke_dashed_animated)
    assert!(
        t_github_user_repo_classes
            .contains("peer-[:focus-within]/tag_app_development:animate-[stroke-dashoffset-move_2s_linear_infinite]"),
        "t_github_user_repo should have animation for tag_app_development. Got: {}",
        t_github_user_repo_classes
    );
    assert!(
        t_github_user_repo_classes
            .contains("peer-[:focus-within]/tag_deployment:animate-[stroke-dashoffset-move_2s_linear_infinite]"),
        "t_github_user_repo should have animation for tag_deployment. Got: {}",
        t_github_user_repo_classes
    );

    // Both should have fill classes since t_github_user_repo is included in both
    // tags (uses slate as its shape_color from defaults)
    assert!(
        t_github_user_repo_classes
            .contains("\npeer-[:focus-within]/tag_app_development:fill-slate-"),
        "t_github_user_repo should have fill classes for tag_app_development. Got: {}",
        t_github_user_repo_classes
    );
    assert!(
        t_github_user_repo_classes.contains("\npeer-[:focus-within]/tag_deployment:fill-slate-"),
        "t_github_user_repo should have fill classes for tag_deployment. Got: {}",
        t_github_user_repo_classes
    );
}
