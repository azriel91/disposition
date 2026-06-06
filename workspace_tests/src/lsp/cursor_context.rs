use disposition_lsp::completion::{
    completion_target::CompletionTarget, cursor_context::CursorContext,
};

#[test]
fn top_level_empty_line_is_key_at_root() {
    let cursor_context = CursorContext::at("", 0, 0);

    assert_eq!(Vec::<String>::new(), cursor_context.path);
    assert_eq!(CompletionTarget::Key, cursor_context.target);
}

#[test]
fn nested_key_resolves_container_path() {
    let text = "render_options:\n  ";
    let cursor_context = CursorContext::at(text, 1, 2);

    assert_eq!(vec!["render_options".to_string()], cursor_context.path);
    assert_eq!(CompletionTarget::Key, cursor_context.target);
}

#[test]
fn value_after_colon_is_value_target() {
    let text = "render_options:\n  rank_dir: ";
    let cursor_context = CursorContext::at(text, 1, 12);

    assert_eq!(vec!["render_options".to_string()], cursor_context.path);
    assert_eq!(
        CompletionTarget::Value {
            key: "rank_dir".to_string(),
            in_sequence: false,
            needs_space: false,
        },
        cursor_context.target
    );
}

#[test]
fn list_item_is_value_of_enclosing_key() {
    let text = "thing_dependencies:\n  edge_a:\n    things:\n      - ";
    let cursor_context = CursorContext::at(text, 3, 8);

    assert_eq!(
        vec!["thing_dependencies".to_string(), "edge_a".to_string()],
        cursor_context.path
    );
    assert_eq!(
        CompletionTarget::Value {
            key: "things".to_string(),
            in_sequence: true,
            needs_space: false,
        },
        cursor_context.target
    );
}

#[test]
fn block_sequence_at_same_indent_resolves_owning_key() {
    // The `- ` items sit at the same indent as `things:` (a block sequence not
    // indented under its key), so `things` must still be the owning key.
    let text = "thing_dependencies:\n  edge_a:\n    things:\n    - ";
    let cursor_context = CursorContext::at(text, 3, 6);

    assert_eq!(
        vec!["thing_dependencies".to_string(), "edge_a".to_string()],
        cursor_context.path
    );
    assert_eq!(
        CompletionTarget::Value {
            key: "things".to_string(),
            in_sequence: true,
            needs_space: false,
        },
        cursor_context.target
    );
}

#[test]
fn bare_dash_list_item_needs_space() {
    // `-` with no following space -- a selected value needs a leading space.
    let text = "thing_dependencies:\n  edge_a:\n    things:\n      -";
    let cursor_context = CursorContext::at(text, 3, 7);

    assert_eq!(
        CompletionTarget::Value {
            key: "things".to_string(),
            in_sequence: true,
            needs_space: true,
        },
        cursor_context.target
    );
}

#[test]
fn caret_inside_flow_list_is_in_sequence() {
    let text = "thing_dependencies:\n  edge_a:\n    things: [t_a, ";
    let cursor_context = CursorContext::at(text, 2, 17);

    assert_eq!(
        CompletionTarget::Value {
            key: "things".to_string(),
            in_sequence: true,
            needs_space: false,
        },
        cursor_context.target
    );
}

#[test]
fn caret_immediately_after_colon_needs_space() {
    let text = "thing_dependencies:\n  edge_a:\n    things:";
    let cursor_context = CursorContext::at(text, 2, 11);

    assert_eq!(
        CompletionTarget::Value {
            key: "things".to_string(),
            in_sequence: false,
            needs_space: true,
        },
        cursor_context.target
    );
}

#[test]
fn sibling_keys_collected_for_key_target() {
    let text = "thing_names:\n  t_a: \"A\"\n  t_b: \"B\"\n  ";
    let cursor_context = CursorContext::at(text, 3, 2);

    assert_eq!(CompletionTarget::Key, cursor_context.target);
    assert_eq!(
        vec!["t_a".to_string(), "t_b".to_string()],
        cursor_context.sibling_keys.into_iter().collect::<Vec<_>>()
    );
}

#[test]
fn sibling_keys_exclude_descendants_and_other_blocks() {
    // `t_a`'s nested child and the `things` block keys must not be collected as
    // siblings of the cursor under `thing_names`.
    let text = "things:\n  t_x: {}\n\
        thing_names:\n  t_a:\n    nested: \"n\"\n  ";
    let cursor_context = CursorContext::at(text, 5, 2);

    assert_eq!(
        vec!["t_a".to_string()],
        cursor_context.sibling_keys.into_iter().collect::<Vec<_>>()
    );
}

#[test]
fn deeply_nested_key_chain() {
    let text = "processes:\n  proc_a:\n    steps:\n      ";
    let cursor_context = CursorContext::at(text, 3, 6);

    assert_eq!(
        vec![
            "processes".to_string(),
            "proc_a".to_string(),
            "steps".to_string()
        ],
        cursor_context.path
    );
    assert_eq!(CompletionTarget::Key, cursor_context.target);
}
