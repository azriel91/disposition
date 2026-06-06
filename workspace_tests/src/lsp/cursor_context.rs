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
            key: "rank_dir".to_string()
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
            key: "things".to_string()
        },
        cursor_context.target
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
