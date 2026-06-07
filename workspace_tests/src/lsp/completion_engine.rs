use disposition_lsp::completion::CompletionEngine;

/// Returns the completion labels at the cursor, for terse assertions.
fn labels(text: &str, line: u32, character: u32) -> Vec<String> {
    CompletionEngine::completions(text, line, character)
        .into_iter()
        .map(|item| item.label)
        .collect()
}

#[test]
fn top_level_keys_offered_on_empty_document() {
    let labels = labels("", 0, 0);

    for expected in [
        "things",
        "thing_names",
        "thing_dependencies",
        "render_options",
        "processes",
        "tags",
    ] {
        assert!(
            labels.iter().any(|label| label == expected),
            "expected top-level key `{expected}` in {labels:?}"
        );
    }
}

#[test]
fn nested_render_options_keys() {
    let text = "render_options:\n  ";
    let labels = labels(text, 1, 2);

    assert_eq!(
        vec![
            "edge_curvature".to_string(),
            "process_render_collapse".to_string(),
            "rank_dir".to_string(),
        ],
        sorted(labels)
    );
}

#[test]
fn enum_values_for_rank_dir() {
    let text = "render_options:\n  rank_dir: ";
    let labels = labels(text, 1, 12);

    assert_eq!(
        vec![
            "bottom_to_top".to_string(),
            "left_to_right".to_string(),
            "right_to_left".to_string(),
            "top_to_bottom".to_string(),
        ],
        sorted(labels)
    );
}

#[test]
fn enum_values_for_thing_layout() {
    let text = "thing_layouts:\n  t_a: ";
    let labels = labels(text, 1, 7);

    assert_eq!(
        vec![
            "column".to_string(),
            "column_reverse".to_string(),
            "row".to_string(),
            "row_reverse".to_string(),
        ],
        sorted(labels)
    );
}

#[test]
fn enum_values_for_edge_group_kind() {
    let text = "thing_dependencies:\n  edge_a:\n    kind: ";
    let labels = labels(text, 2, 10);

    assert_eq!(
        vec![
            "cyclic".to_string(),
            "sequence".to_string(),
            "symmetric".to_string(),
        ],
        sorted(labels)
    );
}

#[test]
fn edge_group_struct_keys() {
    let text = "thing_dependencies:\n  edge_a:\n    ";
    let labels = labels(text, 2, 4);

    assert_eq!(
        vec!["kind".to_string(), "things".to_string()],
        sorted(labels)
    );
}

#[test]
fn dynamic_thing_ids_offered_in_edge_things() {
    let text = "things:\n  t_alpha: \"A\"\n  t_beta: \"B\"\n\
        thing_dependencies:\n  edge_a:\n    things:\n      - ";
    let labels = labels(text, 6, 8);

    assert_eq!(
        vec!["t_alpha".to_string(), "t_beta".to_string()],
        sorted(labels)
    );
}

#[test]
fn theme_attr_keys_offered_in_css_class_partials() {
    // `node_defaults` (and any entity key) under a `ThemeStyles` map is a
    // `CssClassPartials`, whose keys are `ThemeAttr`s plus `style_aliases_applied`.
    let text = "theme_types_styles:\n  type_service:\n    node_defaults:\n      ";
    let labels = labels(text, 3, 6);

    for expected in [
        "style_aliases_applied",
        "shape_color",
        "stroke_style",
        "stroke_width",
        "fill_color",
        "fill_shade_normal",
        "text_shade",
        "visibility",
    ] {
        assert!(
            labels.iter().any(|label| label == expected),
            "expected theme attribute key `{expected}` in {labels:?}"
        );
    }
}

#[test]
fn theme_attr_keys_offered_in_theme_default_base_styles() {
    // `theme_default.base_styles` is a `ThemeStyles`, so its entity values are
    // `CssClassPartials` maps keyed by `ThemeAttr`.
    let text = "theme_default:\n  base_styles:\n    edge_defaults:\n      ";
    let labels = labels(text, 3, 6);

    assert!(
        labels.iter().any(|label| label == "shape_color"),
        "expected theme attribute key `shape_color` in {labels:?}"
    );
}

#[test]
fn theme_attr_keys_offered_in_style_aliases() {
    // `theme_default.style_aliases` is a `StyleAliases` map, whose values are
    // `CssClassPartials` keyed by `ThemeAttr`.
    let text = "theme_default:\n  style_aliases:\n    my_alias:\n      ";
    let labels = labels(text, 3, 6);

    assert!(
        labels.iter().any(|label| label == "stroke_shade_normal"),
        "expected theme attribute key `stroke_shade_normal` in {labels:?}"
    );
}

#[test]
fn style_alias_values_offered_in_style_aliases_applied() {
    // `style_aliases_applied` is a `Vec<StyleAlias>`; its list values are the
    // well-known style aliases, offered in their serialized snake_case form
    // (not the PascalCase Rust variant names).
    let text = "theme_types_styles:\n  type_service:\n    node_defaults:\n      \
        style_aliases_applied:\n        - ";
    let labels = labels(text, 4, 10);

    for expected in [
        "circle_xs",
        "padding_normal",
        "rounded_2xl",
        "shade_light",
        "stroke_dashed_animated",
        "focus_outline",
    ] {
        assert!(
            labels.iter().any(|label| label == expected),
            "expected style alias `{expected}` in {labels:?}"
        );
    }

    // The PascalCase Rust variant names must not leak into completions.
    assert!(
        !labels.iter().any(|label| label == "CircleXs"),
        "did not expect PascalCase style alias `CircleXs` in {labels:?}"
    );
}

#[test]
fn dynamic_thing_ids_offered_as_thing_names_keys() {
    let text = "things:\n  t_alpha: {}\n  t_beta: {}\n\
        thing_names:\n  ";
    let labels = labels(text, 4, 2);

    for expected in ["t_alpha", "t_beta"] {
        assert!(
            labels.iter().any(|label| label == expected),
            "expected thing id key `{expected}` in {labels:?}"
        );
    }
}

#[test]
fn edge_group_template_offered_as_thing_dependencies_key() {
    let text = "things:\n  t_alpha: {}\n  t_beta: {}\n\
        thing_dependencies:\n  ";
    let labels = labels(text, 4, 2);

    assert!(
        labels
            .iter()
            .any(|label| label == "edge_dep__t_alpha_t_beta"),
        "expected edge group template in {labels:?}"
    );
}

#[test]
fn tag_example_offered_as_tags_key() {
    let text = "tags:\n  ";
    let labels = labels(text, 1, 2);

    assert!(
        labels.iter().any(|label| label == "tag_example"),
        "expected `tag_example` in {labels:?}"
    );
}

#[test]
fn edge_id_offered_as_edge_descs_key() {
    let text = "thing_dependencies:\n  edge_a:\n    kind: cyclic\n\
        edge_descs:\n  ";
    let labels = labels(text, 4, 2);

    assert!(
        labels.iter().any(|label| label == "edge_a__0"),
        "expected edge id `edge_a__0` in {labels:?}"
    );
}

#[test]
fn entity_ids_offered_as_entity_types_keys() {
    let text = "things:\n  t_a: {}\n\
        thing_dependencies:\n  edge_a:\n    kind: cyclic\n\
        entity_types:\n  ";
    let labels = labels(text, 6, 2);

    for expected in ["t_a", "edge_a__0"] {
        assert!(
            labels.iter().any(|label| label == expected),
            "expected entity key `{expected}` in {labels:?}"
        );
    }
}

#[test]
fn style_alias_keys_offered_in_style_aliases() {
    let text = "theme_default:\n  style_aliases:\n    ";
    let labels = labels(text, 2, 4);

    for expected in ["padding_normal", "shade_light", "style_alias_custom"] {
        assert!(
            labels.iter().any(|label| label == expected),
            "expected style alias key `{expected}` in {labels:?}"
        );
    }
}

#[test]
fn theme_styles_keys_offered_in_base_styles() {
    let text = "things:\n  t_a: {}\n\
        theme_default:\n  base_styles:\n    ";
    let labels = labels(text, 4, 4);

    for expected in ["node_defaults", "edge_defaults", "t_a"] {
        assert!(
            labels.iter().any(|label| label == expected),
            "expected theme styles key `{expected}` in {labels:?}"
        );
    }
}

#[test]
fn builtin_entity_types_offered_in_theme_types_styles() {
    let text = "theme_types_styles:\n  ";
    let labels = labels(text, 1, 2);

    for expected in [
        "type_thing_default",
        "type_dependency_edge_sequence_default",
    ] {
        assert!(
            labels.iter().any(|label| label == expected),
            "expected built-in entity type `{expected}` in {labels:?}"
        );
    }

    // The PascalCase Rust variant names must not leak into completions.
    assert!(
        !labels.iter().any(|label| label == "ThingDefault"),
        "did not expect PascalCase entity type `ThingDefault` in {labels:?}"
    );
}

#[test]
fn already_declared_keys_filtered_from_suggestions() {
    // `t_a` is already a `thing_names` key, so only `t_b` should remain.
    let text = "things:\n  t_a: {}\n  t_b: {}\n\
        thing_names:\n  t_a: \"A\"\n  ";
    let labels = labels(text, 5, 2);

    assert!(
        labels.iter().any(|label| label == "t_b"),
        "expected undeclared thing id `t_b` in {labels:?}"
    );
    assert!(
        !labels.iter().any(|label| label == "t_a"),
        "did not expect already-declared `t_a` in {labels:?}"
    );
}

#[test]
fn already_declared_struct_fields_filtered() {
    // `kind` is already set on the edge group, so only `things` should remain.
    let text = "thing_dependencies:\n  edge_a:\n    kind: cyclic\n    ";
    let labels = labels(text, 3, 4);

    assert_eq!(vec!["things".to_string()], labels);
}

#[test]
fn color_values_offered_for_shape_color() {
    let text = "theme_default:\n  base_styles:\n    node_defaults:\n      shape_color: ";
    let labels = labels(text, 3, 19);

    for expected in ["slate", "blue", "emerald", "rose"] {
        assert!(
            labels.iter().any(|label| label == expected),
            "expected color value `{expected}` in {labels:?}"
        );
    }
}

#[test]
fn shade_values_offered_for_fill_shade_normal() {
    let text =
        "theme_types_styles:\n  type_service:\n    node_defaults:\n      fill_shade_normal: ";
    let labels = labels(text, 3, 25);

    for expected in ["50", "300", "950"] {
        assert!(
            labels.iter().any(|label| label == expected),
            "expected shade value `{expected}` in {labels:?}"
        );
    }
}

#[test]
fn style_values_offered_for_stroke_style() {
    let text = "theme_default:\n  style_aliases:\n    my_alias:\n      stroke_style: ";
    let labels = labels(text, 3, 20);

    assert_eq!(
        vec![
            "dashed".to_string(),
            "dotted".to_string(),
            "none".to_string(),
            "solid".to_string(),
        ],
        sorted(labels)
    );
}

#[test]
fn visibility_values_offered_for_visibility() {
    let text = "theme_default:\n  base_styles:\n    edge_defaults:\n      visibility: ";
    let labels = labels(text, 3, 18);

    assert_eq!(
        vec![
            "collapse".to_string(),
            "invisible".to_string(),
            "visible".to_string(),
        ],
        sorted(labels)
    );
}

#[test]
fn no_values_offered_for_numeric_theme_attr() {
    // `stroke_width` is freeform numeric, so no value suggestions are offered.
    let text = "theme_default:\n  base_styles:\n    node_defaults:\n      stroke_width: ";
    let labels = labels(text, 3, 20);

    assert!(
        labels.is_empty(),
        "expected no suggestions for numeric `stroke_width`, got {labels:?}"
    );
}

#[test]
fn edge_group_things_value_inserts_flow_list() {
    // At `things: <cursor>` (not a `- ` item), the array element is inserted as
    // a flow list so the YAML stays a valid sequence.
    let text = "things:\n  t_alpha: {}\n  t_beta: {}\n\
        thing_dependencies:\n  edge_a:\n    things: ";
    let items = CompletionEngine::completions(text, 5, 12);

    let t_alpha = items
        .iter()
        .find(|item| item.label == "t_alpha")
        .expect("expected `t_alpha` completion");
    assert_eq!(Some("[t_alpha]"), t_alpha.insert_text.as_deref());
}

#[test]
fn edge_group_things_list_item_inserts_bare_id() {
    // In a `- ` list item, the element is inserted bare (no flow-list wrapper).
    let text = "things:\n  t_alpha: {}\n  t_beta: {}\n\
        thing_dependencies:\n  edge_a:\n    things:\n      - ";
    let items = CompletionEngine::completions(text, 6, 8);

    let t_alpha = items
        .iter()
        .find(|item| item.label == "t_alpha")
        .expect("expected `t_alpha` completion");
    assert_eq!(None, t_alpha.insert_text.as_deref());
}

#[test]
fn edge_group_things_offered_for_same_indent_block_sequence() {
    // The `- ` is at the same indent as `things:`; thing IDs are still offered.
    let text = "things:\n  t_alpha: {}\n  t_beta: {}\n\
        thing_dependencies:\n  edge_a:\n    things:\n    - ";
    let labels = labels(text, 6, 6);

    assert_eq!(
        vec!["t_alpha".to_string(), "t_beta".to_string()],
        sorted(labels)
    );
}

#[test]
fn edge_group_things_after_bare_dash_inserts_leading_space() {
    // `-` with no space -- the selected id is inserted as ` t_alpha` (-> `-
    // t_alpha`).
    let text = "things:\n  t_alpha: {}\n\
        thing_dependencies:\n  edge_a:\n    things:\n      -";
    let items = CompletionEngine::completions(text, 5, 7);

    let t_alpha = items
        .iter()
        .find(|item| item.label == "t_alpha")
        .expect("expected `t_alpha` completion");
    assert_eq!(Some(" t_alpha"), t_alpha.insert_text.as_deref());
}

#[test]
fn edge_group_things_inside_flow_list_inserts_bare_id() {
    // Caret inside `[ .. ]`; the element must not be wrapped again.
    let text = "things:\n  t_alpha: {}\n  t_beta: {}\n\
        thing_dependencies:\n  edge_a:\n    things: [t_alpha, ";
    let items = CompletionEngine::completions(text, 5, 22);

    let t_beta = items
        .iter()
        .find(|item| item.label == "t_beta")
        .expect("expected `t_beta` completion");
    assert_eq!(None, t_beta.insert_text.as_deref());
}

#[test]
fn edge_group_things_after_bare_colon_inserts_space_and_list() {
    // `things:` with no following space -- insert a separator space then list.
    let text = "things:\n  t_alpha: {}\n\
        thing_dependencies:\n  edge_a:\n    things:";
    let items = CompletionEngine::completions(text, 4, 11);

    let t_alpha = items
        .iter()
        .find(|item| item.label == "t_alpha")
        .expect("expected `t_alpha` completion");
    assert_eq!(Some(" [t_alpha]"), t_alpha.insert_text.as_deref());
}

fn sorted(mut labels: Vec<String>) -> Vec<String> {
    labels.sort();
    labels
}
