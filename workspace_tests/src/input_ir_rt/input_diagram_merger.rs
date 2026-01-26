//! Tests for `InputDiagramMerger`.

use disposition::{
    input_model::InputDiagram,
    model_common::{id, Id},
};
use disposition_input_ir_rt::InputDiagramMerger;
use pretty_assertions::assert_eq;

use crate::input_ir_rt::EXAMPLE_INPUT;

/// Tests that merging `example_input.yaml` over `InputDiagram::base()` produces
/// a result with base style aliases, overlay things, and properly merged
/// themes.
#[test]
fn test_merge_example_input_over_base_produces_merged() {
    let base_diagram = InputDiagram::base();
    let overlay_diagram = serde_saphyr::from_str::<InputDiagram>(EXAMPLE_INPUT).unwrap();

    let merged = InputDiagramMerger::merge(base_diagram, &overlay_diagram);

    // Verify overlay things are present
    assert_eq!(overlay_diagram.things.len(), merged.things.len());
    let t_aws = id!("t_aws");
    assert!(merged.things.contains_key(&t_aws));

    // Verify base style_aliases are present
    assert!(merged
        .theme_default
        .style_aliases
        .contains_key(&disposition::input_model::theme::StyleAlias::PaddingNormal));
    assert!(merged
        .theme_default
        .style_aliases
        .contains_key(&disposition::input_model::theme::StyleAlias::ShadePale));

    // Verify base theme_types_styles are present
    let type_thing_default = id!("type_thing_default");
    assert!(merged.theme_types_styles.contains_key(&type_thing_default));

    // Verify overlay theme_types_styles are present
    let type_organisation = id!("type_organisation");
    assert!(merged.theme_types_styles.contains_key(&type_organisation));

    // Verify CSS is present (from base, preserved since overlay has same content)
    assert!(!merged.css.is_empty());
    assert!(merged.css.as_str().contains("stroke-dashoffset-move"));

    // Verify overlay processes are present
    assert_eq!(overlay_diagram.processes.len(), merged.processes.len());

    // Verify overlay thing_hierarchy is present
    assert_eq!(
        overlay_diagram.thing_hierarchy.len(),
        merged.thing_hierarchy.len()
    );

    // Verify base theme_tag_things_focus is present
    assert!(!merged.theme_tag_things_focus.is_empty());
}

/// Tests that merging an empty overlay over base returns the base values.
#[test]
fn test_merge_empty_overlay_preserves_base() {
    let base_diagram = InputDiagram::base();
    let overlay_diagram = InputDiagram::new();

    let merged = InputDiagramMerger::merge(base_diagram, &overlay_diagram);

    // The merged diagram should have all base values
    assert!(!merged.theme_default.style_aliases.is_empty());
    assert!(!merged.theme_types_styles.is_empty());
    assert!(!merged.css.is_empty());
}

/// Tests that overlay values override base values for the same key.
#[test]
fn test_merge_overlay_overrides_base() {
    let base_diagram = InputDiagram::base();
    let overlay_diagram = serde_saphyr::from_str::<InputDiagram>(EXAMPLE_INPUT).unwrap();

    let merged = InputDiagramMerger::merge(base_diagram, &overlay_diagram);

    // Overlay things should be present
    let t_aws = id!("t_aws");
    assert!(merged.things.contains_key(&t_aws));
    assert_eq!("☁️ Amazon Web Services", merged.things.get(&t_aws).unwrap());

    // Base style_aliases should still be present (padding_normal from base)
    assert!(merged
        .theme_default
        .style_aliases
        .contains_key(&disposition::input_model::theme::StyleAlias::PaddingNormal));
}

/// Tests that theme_default.base_styles are merged correctly.
#[test]
fn test_merge_theme_default_base_styles() {
    let base_diagram = InputDiagram::base();
    let overlay_diagram = serde_saphyr::from_str::<InputDiagram>(EXAMPLE_INPUT).unwrap();

    let merged = InputDiagramMerger::merge(base_diagram, &overlay_diagram);

    // The overlay has custom base_styles for t_aws and t_github
    // Check that they're present in the merged result
    let t_aws_id = disposition::input_model::theme::IdOrDefaults::Id(id!("t_aws").into());
    assert!(merged.theme_default.base_styles.contains_key(&t_aws_id));

    let t_github_id = disposition::input_model::theme::IdOrDefaults::Id(id!("t_github").into());
    assert!(merged.theme_default.base_styles.contains_key(&t_github_id));

    // Base node_defaults should also be present
    assert!(merged
        .theme_default
        .base_styles
        .contains_key(&disposition::input_model::theme::IdOrDefaults::NodeDefaults));
}

/// Tests that theme_types_styles are merged correctly.
#[test]
fn test_merge_theme_types_styles() {
    let base_diagram = InputDiagram::base();
    let overlay_diagram = serde_saphyr::from_str::<InputDiagram>(EXAMPLE_INPUT).unwrap();

    let merged = InputDiagramMerger::merge(base_diagram, &overlay_diagram);

    // Base types should be present
    let type_thing_default = id!("type_thing_default");
    assert!(merged.theme_types_styles.contains_key(&type_thing_default));

    // Overlay custom types should be present
    let type_organisation = id!("type_organisation");
    assert!(merged.theme_types_styles.contains_key(&type_organisation));
}

/// Tests that thing_hierarchy is merged correctly.
#[test]
fn test_merge_thing_hierarchy() {
    let base_diagram = InputDiagram::base();
    let overlay_diagram = serde_saphyr::from_str::<InputDiagram>(EXAMPLE_INPUT).unwrap();

    let merged = InputDiagramMerger::merge(base_diagram, &overlay_diagram);

    // The overlay defines the entire thing_hierarchy
    let t_aws = id!("t_aws");
    assert!(merged.thing_hierarchy.contains_key(&t_aws));

    let t_localhost = id!("t_localhost");
    assert!(merged.thing_hierarchy.contains_key(&t_localhost));
}

/// Tests that processes are merged correctly.
#[test]
fn test_merge_processes() {
    let base_diagram = InputDiagram::base();
    let overlay_diagram = serde_saphyr::from_str::<InputDiagram>(EXAMPLE_INPUT).unwrap();

    let merged = InputDiagramMerger::merge(base_diagram, &overlay_diagram);

    // Overlay processes should be present
    let proc_app_dev = id!("proc_app_dev");
    assert!(merged.processes.contains_key(&proc_app_dev));

    let proc_app_release = id!("proc_app_release");
    assert!(merged.processes.contains_key(&proc_app_release));
}

/// Tests that entity_types are merged correctly.
#[test]
fn test_merge_entity_types() {
    let base_diagram = InputDiagram::base();
    let overlay_diagram = serde_saphyr::from_str::<InputDiagram>(EXAMPLE_INPUT).unwrap();

    let merged = InputDiagramMerger::merge(base_diagram, &overlay_diagram);

    // Overlay entity_types should be present
    let t_aws = id!("t_aws");
    assert!(merged.entity_types.contains_key(&t_aws));
}

/// Tests that CSS is handled correctly (overlay replaces base if non-empty).
#[test]
fn test_merge_css_overlay_replaces_if_nonempty() {
    let base_diagram = InputDiagram::base();
    let overlay_diagram = serde_saphyr::from_str::<InputDiagram>(EXAMPLE_INPUT).unwrap();

    let merged = InputDiagramMerger::merge(base_diagram, &overlay_diagram);

    // The overlay has CSS (same content as base in this case)
    assert!(!merged.css.is_empty());

    // Verify the CSS content matches what we expect
    assert!(merged.css.as_str().contains("stroke-dashoffset-move"));
}

/// Tests that an empty base with overlay values returns overlay values.
#[test]
fn test_merge_empty_base_with_overlay() {
    let base_diagram = InputDiagram::new();
    let overlay_diagram = serde_saphyr::from_str::<InputDiagram>(EXAMPLE_INPUT).unwrap();

    let merged = InputDiagramMerger::merge(base_diagram, &overlay_diagram);

    // All overlay values should be present
    assert_eq!(overlay_diagram.things.len(), merged.things.len());
    assert_eq!(
        overlay_diagram.thing_hierarchy.len(),
        merged.thing_hierarchy.len()
    );
    assert_eq!(overlay_diagram.processes.len(), merged.processes.len());
}
