//! Tests for `disposition_input_rt::style_aliases_section_ops::StyleAliasesSectionOps`.

use disposition::input_model::{theme::CssClassPartials, InputDiagram};
use disposition_input_rt::{id_parse::parse_style_alias, StyleAliasesSectionOps};

fn empty_diagram() -> InputDiagram<'static> {
    InputDiagram::default()
}

fn diagram_with_style_alias(alias_str: &str, applied: &[&str]) -> InputDiagram<'static> {
    let mut input_diagram = empty_diagram();
    let style_alias = parse_style_alias(alias_str).unwrap();
    let mut css_class_partials = CssClassPartials::default();
    css_class_partials.style_aliases_applied = applied
        .iter()
        .map(|s| parse_style_alias(s).unwrap())
        .collect();
    input_diagram
        .theme_default
        .style_aliases
        .insert(style_alias, css_class_partials);
    input_diagram
}

// === style_alias_rename === //

#[test]
fn style_alias_rename_renames_key_in_style_aliases() {
    let mut input_diagram = diagram_with_style_alias("shade_light", &[]);

    StyleAliasesSectionOps::style_alias_rename(&mut input_diagram, "shade_light", "shade_pale");

    let style_alias_old = parse_style_alias("shade_light").unwrap();
    let style_alias_new = parse_style_alias("shade_pale").unwrap();
    assert!(!input_diagram
        .theme_default
        .style_aliases
        .contains_key(&style_alias_old));
    assert!(input_diagram
        .theme_default
        .style_aliases
        .contains_key(&style_alias_new));
}

#[test]
fn style_alias_rename_noop_when_same_alias() {
    let mut input_diagram = diagram_with_style_alias("shade_light", &[]);

    StyleAliasesSectionOps::style_alias_rename(&mut input_diagram, "shade_light", "shade_light");

    let style_alias = parse_style_alias("shade_light").unwrap();
    assert!(input_diagram
        .theme_default
        .style_aliases
        .contains_key(&style_alias));
}

#[test]
fn style_alias_rename_noop_for_invalid_old_alias() {
    let mut input_diagram = diagram_with_style_alias("shade_light", &[]);

    StyleAliasesSectionOps::style_alias_rename(&mut input_diagram, "", "shade_pale");

    let style_alias = parse_style_alias("shade_light").unwrap();
    assert!(input_diagram
        .theme_default
        .style_aliases
        .contains_key(&style_alias));
    assert_eq!(input_diagram.theme_default.style_aliases.len(), 1);
}

#[test]
fn style_alias_rename_noop_for_invalid_new_alias() {
    let mut input_diagram = diagram_with_style_alias("shade_light", &[]);

    StyleAliasesSectionOps::style_alias_rename(&mut input_diagram, "shade_light", "");

    let style_alias = parse_style_alias("shade_light").unwrap();
    assert!(input_diagram
        .theme_default
        .style_aliases
        .contains_key(&style_alias));
    assert_eq!(input_diagram.theme_default.style_aliases.len(), 1);
}

#[test]
fn style_alias_rename_noop_when_old_key_not_found() {
    let mut input_diagram = diagram_with_style_alias("shade_light", &[]);

    StyleAliasesSectionOps::style_alias_rename(
        &mut input_diagram,
        "shade_nonexistent",
        "shade_pale",
    );

    // Original unchanged, no new key added.
    let style_alias = parse_style_alias("shade_light").unwrap();
    assert!(input_diagram
        .theme_default
        .style_aliases
        .contains_key(&style_alias));
    assert_eq!(input_diagram.theme_default.style_aliases.len(), 1);
}

#[test]
fn style_alias_rename_noop_when_target_key_already_exists() {
    let mut input_diagram = diagram_with_style_alias("shade_light", &[]);
    // Insert a second alias that is the target of the rename.
    let style_alias_existing = parse_style_alias("shade_pale").unwrap();
    input_diagram
        .theme_default
        .style_aliases
        .insert(style_alias_existing.clone(), CssClassPartials::default());

    StyleAliasesSectionOps::style_alias_rename(&mut input_diagram, "shade_light", "shade_pale");

    // Both original keys should still be present -- rename was rejected.
    let style_alias_old = parse_style_alias("shade_light").unwrap();
    assert!(input_diagram
        .theme_default
        .style_aliases
        .contains_key(&style_alias_old));
    assert!(input_diagram
        .theme_default
        .style_aliases
        .contains_key(&style_alias_existing));
    assert_eq!(input_diagram.theme_default.style_aliases.len(), 2);
}

#[test]
fn style_alias_rename_updates_applied_in_style_aliases_values() {
    // Create a diagram with two aliases: shade_a applies shade_b.
    let mut input_diagram = empty_diagram();
    let style_alias_a = parse_style_alias("shade_a").unwrap();
    let style_alias_b = parse_style_alias("shade_b").unwrap();

    let mut css_partials_a = CssClassPartials::default();
    css_partials_a
        .style_aliases_applied
        .push(style_alias_b.clone());
    input_diagram
        .theme_default
        .style_aliases
        .insert(style_alias_a.clone(), css_partials_a);

    let css_partials_b = CssClassPartials::default();
    input_diagram
        .theme_default
        .style_aliases
        .insert(style_alias_b.clone(), css_partials_b);

    StyleAliasesSectionOps::style_alias_rename(&mut input_diagram, "shade_b", "shade_c");

    let style_alias_c = parse_style_alias("shade_c").unwrap();

    // shade_a's applied list should now reference shade_c instead of shade_b.
    let css_partials_a = input_diagram
        .theme_default
        .style_aliases
        .get(&style_alias_a)
        .unwrap();
    assert!(css_partials_a
        .style_aliases_applied
        .contains(&style_alias_c));
    assert!(!css_partials_a
        .style_aliases_applied
        .contains(&style_alias_b));
}

#[test]
fn style_alias_rename_updates_base_styles() {
    use disposition::{input_model::theme::IdOrDefaults, model_common::Id};

    let mut input_diagram = diagram_with_style_alias("shade_light", &[]);

    // Add a base style entry whose CssClassPartials applies shade_light.
    let entity_id = Id::new("entity_0").unwrap().into_static();
    let key = IdOrDefaults::Id(entity_id);
    let mut css_partials = CssClassPartials::default();
    let style_alias_old = parse_style_alias("shade_light").unwrap();
    css_partials
        .style_aliases_applied
        .push(style_alias_old.clone());
    input_diagram
        .theme_default
        .base_styles
        .insert(key.clone(), css_partials);

    StyleAliasesSectionOps::style_alias_rename(&mut input_diagram, "shade_light", "shade_pale");

    let style_alias_new = parse_style_alias("shade_pale").unwrap();
    let css_partials = input_diagram.theme_default.base_styles.get(&key).unwrap();
    assert!(css_partials
        .style_aliases_applied
        .contains(&style_alias_new));
    assert!(!css_partials
        .style_aliases_applied
        .contains(&style_alias_old));
}

#[test]
fn style_alias_rename_updates_process_step_selected_styles() {
    use disposition::{input_model::theme::IdOrDefaults, model_common::Id};

    let mut input_diagram = diagram_with_style_alias("shade_light", &[]);

    let entity_id = Id::new("step_0").unwrap().into_static();
    let key = IdOrDefaults::Id(entity_id);
    let mut css_partials = CssClassPartials::default();
    let style_alias_old = parse_style_alias("shade_light").unwrap();
    css_partials
        .style_aliases_applied
        .push(style_alias_old.clone());
    input_diagram
        .theme_default
        .process_step_selected_styles
        .insert(key.clone(), css_partials);

    StyleAliasesSectionOps::style_alias_rename(&mut input_diagram, "shade_light", "shade_pale");

    let style_alias_new = parse_style_alias("shade_pale").unwrap();
    let css_partials = input_diagram
        .theme_default
        .process_step_selected_styles
        .get(&key)
        .unwrap();
    assert!(css_partials
        .style_aliases_applied
        .contains(&style_alias_new));
    assert!(!css_partials
        .style_aliases_applied
        .contains(&style_alias_old));
}

#[test]
fn style_alias_rename_updates_theme_types_styles() {
    use disposition::{
        input_model::theme::IdOrDefaults,
        model_common::{entity::EntityTypeId, Id},
    };

    let mut input_diagram = diagram_with_style_alias("shade_light", &[]);

    let entity_type_id = EntityTypeId::from(Id::new("type_org").unwrap().into_static());
    let entity_id = Id::new("entity_0").unwrap().into_static();
    let key = IdOrDefaults::Id(entity_id);
    let mut css_partials = CssClassPartials::default();
    let style_alias_old = parse_style_alias("shade_light").unwrap();
    css_partials
        .style_aliases_applied
        .push(style_alias_old.clone());
    let mut theme_styles = disposition::input_model::theme::ThemeStyles::new();
    theme_styles.insert(key.clone(), css_partials);
    input_diagram
        .theme_types_styles
        .insert(entity_type_id, theme_styles);

    StyleAliasesSectionOps::style_alias_rename(&mut input_diagram, "shade_light", "shade_pale");

    let style_alias_new = parse_style_alias("shade_pale").unwrap();
    for theme_styles in input_diagram.theme_types_styles.values() {
        if let Some(css_partials) = theme_styles.get(&key) {
            assert!(css_partials
                .style_aliases_applied
                .contains(&style_alias_new));
            assert!(!css_partials
                .style_aliases_applied
                .contains(&style_alias_old));
        }
    }
}

#[test]
fn style_alias_rename_updates_theme_thing_dependencies_styles() {
    use disposition::{input_model::theme::IdOrDefaults, model_common::Id};

    let mut input_diagram = diagram_with_style_alias("shade_light", &[]);

    let entity_id = Id::new("entity_0").unwrap().into_static();
    let key = IdOrDefaults::Id(entity_id);
    let mut css_partials = CssClassPartials::default();
    let style_alias_old = parse_style_alias("shade_light").unwrap();
    css_partials
        .style_aliases_applied
        .push(style_alias_old.clone());
    input_diagram
        .theme_thing_dependencies_styles
        .things_included_styles
        .insert(key.clone(), css_partials.clone());
    input_diagram
        .theme_thing_dependencies_styles
        .things_excluded_styles
        .insert(key.clone(), css_partials);

    StyleAliasesSectionOps::style_alias_rename(&mut input_diagram, "shade_light", "shade_pale");

    let style_alias_new = parse_style_alias("shade_pale").unwrap();

    let included_partials = input_diagram
        .theme_thing_dependencies_styles
        .things_included_styles
        .get(&key)
        .unwrap();
    assert!(included_partials
        .style_aliases_applied
        .contains(&style_alias_new));
    assert!(!included_partials
        .style_aliases_applied
        .contains(&style_alias_old));

    let excluded_partials = input_diagram
        .theme_thing_dependencies_styles
        .things_excluded_styles
        .get(&key)
        .unwrap();
    assert!(excluded_partials
        .style_aliases_applied
        .contains(&style_alias_new));
    assert!(!excluded_partials
        .style_aliases_applied
        .contains(&style_alias_old));
}

#[test]
fn style_alias_rename_updates_theme_tag_things_focus() {
    use disposition::{
        input_model::theme::{IdOrDefaults, TagIdOrDefaults},
        model_common::Id,
    };

    let mut input_diagram = diagram_with_style_alias("shade_light", &[]);

    let tag_key = TagIdOrDefaults::TagDefaults;
    let entity_id = Id::new("entity_0").unwrap().into_static();
    let key = IdOrDefaults::Id(entity_id);
    let mut css_partials = CssClassPartials::default();
    let style_alias_old = parse_style_alias("shade_light").unwrap();
    css_partials
        .style_aliases_applied
        .push(style_alias_old.clone());
    let mut theme_styles = disposition::input_model::theme::ThemeStyles::new();
    theme_styles.insert(key.clone(), css_partials);
    input_diagram
        .theme_tag_things_focus
        .insert(tag_key, theme_styles);

    StyleAliasesSectionOps::style_alias_rename(&mut input_diagram, "shade_light", "shade_pale");

    let style_alias_new = parse_style_alias("shade_pale").unwrap();
    for theme_styles in input_diagram.theme_tag_things_focus.values() {
        if let Some(css_partials) = theme_styles.get(&key) {
            assert!(css_partials
                .style_aliases_applied
                .contains(&style_alias_new));
            assert!(!css_partials
                .style_aliases_applied
                .contains(&style_alias_old));
        }
    }
}
