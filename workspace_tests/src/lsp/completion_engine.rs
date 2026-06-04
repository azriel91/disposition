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
        "thing_hierarchy",
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

    assert_eq!(vec!["kind".to_string(), "things".to_string()], sorted(labels));
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

fn sorted(mut labels: Vec<String>) -> Vec<String> {
    labels.sort();
    labels
}
