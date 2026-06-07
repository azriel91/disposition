use disposition_lsp::completion::{
    dynamic_completions::DynamicCompletions, id_category::IdCategory, key_category::KeyCategory,
};

#[test]
fn collects_thing_ids_from_things_hierarchy_and_names() {
    let text = "things:\n  t_a:\n    t_a_child: {}\n\
        thing_names:\n  t_a: \"A\"\n  t_b: \"B\"\n";
    let dynamic_completions = DynamicCompletions::from_text(text);

    assert_eq!(
        vec!["t_a", "t_a_child", "t_b"],
        dynamic_completions.ids_for(IdCategory::Thing)
    );
}

#[test]
fn collects_tag_ids() {
    let text = "tags:\n  tag_a: \"A\"\n  tag_b: \"B\"\n";
    let dynamic_completions = DynamicCompletions::from_text(text);

    assert_eq!(
        vec!["tag_a", "tag_b"],
        dynamic_completions.ids_for(IdCategory::Tag)
    );
}

#[test]
fn collects_edge_group_ids_from_dependencies_and_interactions() {
    let text = "thing_dependencies:\n  edge_dep:\n    kind: cyclic\n\
        thing_interactions:\n  edge_int:\n    kind: sequence\n";
    let dynamic_completions = DynamicCompletions::from_text(text);

    assert_eq!(
        vec!["edge_dep", "edge_int"],
        dynamic_completions.ids_for(IdCategory::EdgeGroup)
    );
}

#[test]
fn collects_step_ids_from_nested_steps_block() {
    let text = "processes:\n  proc_a:\n    steps:\n      \
        step_one: \"One\"\n      step_two: \"Two\"\n";
    let dynamic_completions = DynamicCompletions::from_text(text);

    assert_eq!(
        vec!["step_one", "step_two"],
        dynamic_completions.ids_for(IdCategory::ProcessStep)
    );
}

#[test]
fn edge_group_first_level_keys_exclude_nested_kind_and_things() {
    let text = "thing_dependencies:\n  edge_a:\n    kind: cyclic\n    things:\n      - t_a\n";
    let dynamic_completions = DynamicCompletions::from_text(text);

    // Only the edge group ID, not the nested `kind` / `things` keys.
    assert_eq!(
        vec!["edge_a"],
        dynamic_completions.ids_for(IdCategory::EdgeGroup)
    );
}

#[test]
fn key_suggestions_thing_id_offers_defined_things() {
    let text = "things:\n  t_a:\n    t_a_child: {}\n";
    let dynamic_completions = DynamicCompletions::from_text(text);

    assert_eq!(
        vec!["t_a".to_string(), "t_a_child".to_string()],
        dynamic_completions.key_suggestions(KeyCategory::ThingId)
    );
}

#[test]
fn key_suggestions_edge_group_dep_uses_first_two_things() {
    let text = "things:\n  t_a: {}\n  t_b: {}\n  t_c: {}\n";
    let dynamic_completions = DynamicCompletions::from_text(text);

    assert_eq!(
        vec!["edge_dep__t_a_t_b".to_string()],
        dynamic_completions.key_suggestions(KeyCategory::EdgeGroupDep)
    );
}

#[test]
fn key_suggestions_edge_group_interaction_with_single_thing() {
    let text = "things:\n  t_a: {}\n";
    let dynamic_completions = DynamicCompletions::from_text(text);

    assert_eq!(
        vec!["edge_ix__t_a".to_string()],
        dynamic_completions.key_suggestions(KeyCategory::EdgeGroupInteraction)
    );
}

#[test]
fn key_suggestions_edge_group_dep_fallback_without_things() {
    let dynamic_completions = DynamicCompletions::from_text("");

    assert_eq!(
        vec!["edge_dep_".to_string()],
        dynamic_completions.key_suggestions(KeyCategory::EdgeGroupDep)
    );
}

#[test]
fn key_suggestions_tag_name_offers_placeholder() {
    let dynamic_completions = DynamicCompletions::from_text("");

    assert_eq!(
        vec!["tag_example".to_string()],
        dynamic_completions.key_suggestions(KeyCategory::TagName)
    );
}

#[test]
fn key_suggestions_edge_id_appends_zero_index() {
    let text = "thing_dependencies:\n  edge_a:\n    kind: cyclic\n";
    let dynamic_completions = DynamicCompletions::from_text(text);

    assert_eq!(
        vec!["edge_a__0".to_string()],
        dynamic_completions.key_suggestions(KeyCategory::EdgeId)
    );
}

#[test]
fn key_suggestions_entity_includes_things_processes_steps_and_edges() {
    let text = "things:\n  t_a: {}\n\
        processes:\n  proc_a:\n    steps:\n      step_a: \"A\"\n\
        thing_dependencies:\n  edge_a:\n    kind: cyclic\n";
    let dynamic_completions = DynamicCompletions::from_text(text);

    assert_eq!(
        vec![
            "t_a".to_string(),
            "proc_a".to_string(),
            "step_a".to_string(),
            "edge_a__0".to_string(),
        ],
        dynamic_completions.key_suggestions(KeyCategory::Entity)
    );
}

#[test]
fn key_suggestions_theme_styles_includes_defaults_and_ids() {
    let text = "things:\n  t_a: {}\n\
        thing_dependencies:\n  edge_a:\n    kind: cyclic\n";
    let dynamic_completions = DynamicCompletions::from_text(text);

    assert_eq!(
        vec![
            "node_defaults".to_string(),
            "edge_defaults".to_string(),
            "t_a".to_string(),
            "edge_a".to_string(),
            "edge_a__0".to_string(),
        ],
        dynamic_completions.key_suggestions(KeyCategory::ThemeStyles)
    );
}

#[test]
fn key_suggestions_tag_focus_includes_defaults_and_tags() {
    let text = "tags:\n  tag_a: \"A\"\n";
    let dynamic_completions = DynamicCompletions::from_text(text);

    assert_eq!(
        vec!["tag_defaults".to_string(), "tag_a".to_string()],
        dynamic_completions.key_suggestions(KeyCategory::TagFocus)
    );
}

#[test]
fn key_suggestions_style_alias_offers_custom_placeholder() {
    let dynamic_completions = DynamicCompletions::from_text("");

    assert_eq!(
        vec!["style_alias_custom".to_string()],
        dynamic_completions.key_suggestions(KeyCategory::StyleAlias)
    );
}
