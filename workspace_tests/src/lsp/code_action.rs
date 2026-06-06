use disposition_lsp::code_action::{CodeActionEngine, ListConversion};

/// Applies a conversion's single text edit to `text`, returning the result.
fn apply(text: &str, conversion: &ListConversion) -> String {
    let edit = &conversion.edit;
    let start_line = edit.range.start.line as usize;
    let start_char = edit.range.start.character as usize;
    let end_line = edit.range.end.line as usize;
    let end_char = edit.range.end.character as usize;

    let lines = text.split('\n').collect::<Vec<&str>>();
    let mut out = String::new();
    for line in &lines[..start_line] {
        out.push_str(line);
        out.push('\n');
    }
    out.extend(lines[start_line].chars().take(start_char));
    out.push_str(&edit.new_text);
    out.extend(lines[end_line].chars().skip(end_char));
    for line in &lines[end_line + 1..] {
        out.push('\n');
        out.push_str(line);
    }
    out
}

#[test]
fn flow_list_converts_to_block_list() {
    let text = "thing_dependencies:\n  edge_a:\n    things: [t_a, t_b]";
    let conversions = CodeActionEngine::list_conversions(text, 2);

    assert_eq!(1, conversions.len());
    assert_eq!("Convert `things` to a block list", conversions[0].title);
    assert_eq!(
        "thing_dependencies:\n  edge_a:\n    things:\n      - t_a\n      - t_b",
        apply(text, &conversions[0])
    );
}

#[test]
fn block_list_converts_to_inline_list_from_key_line() {
    let text = "thing_dependencies:\n  edge_a:\n    things:\n      - t_a\n      - t_b";
    let conversions = CodeActionEngine::list_conversions(text, 2);

    assert_eq!(1, conversions.len());
    assert_eq!("Convert `things` to an inline list", conversions[0].title);
    assert_eq!(
        "thing_dependencies:\n  edge_a:\n    things: [t_a, t_b]",
        apply(text, &conversions[0])
    );
}

#[test]
fn block_list_converts_to_inline_list_from_item_line() {
    let text = "thing_dependencies:\n  edge_a:\n    things:\n      - t_a\n      - t_b";
    // Cursor on the second `- ` item line.
    let conversions = CodeActionEngine::list_conversions(text, 4);

    assert_eq!(1, conversions.len());
    assert_eq!(
        "thing_dependencies:\n  edge_a:\n    things: [t_a, t_b]",
        apply(text, &conversions[0])
    );
}

#[test]
fn same_indent_block_list_converts_to_inline_list() {
    // `- ` items at the same indent as `things:`.
    let text = "thing_dependencies:\n  edge_a:\n    things:\n    - t_a\n    - t_b";
    let conversions = CodeActionEngine::list_conversions(text, 3);

    assert_eq!(1, conversions.len());
    assert_eq!(
        "thing_dependencies:\n  edge_a:\n    things: [t_a, t_b]",
        apply(text, &conversions[0])
    );
}

#[test]
fn no_conversion_offered_for_scalar_value() {
    let text = "thing_dependencies:\n  edge_a:\n    kind: cyclic";
    let conversions = CodeActionEngine::list_conversions(text, 2);

    assert!(conversions.is_empty());
}
