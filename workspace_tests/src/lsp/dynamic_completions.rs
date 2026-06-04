use disposition_lsp::completion::{dynamic_completions::DynamicCompletions, id_category::IdCategory};

#[test]
fn collects_thing_ids_from_things_and_hierarchy() {
    let text = "things:\n  t_a: \"A\"\n  t_b: \"B\"\n\
        thing_hierarchy:\n  t_a:\n    t_a_child: {}\n";
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
    assert_eq!(vec!["edge_a"], dynamic_completions.ids_for(IdCategory::EdgeGroup));
}
