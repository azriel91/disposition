//! Code actions for the `disposition` LSP language server.
//!
//! Currently offers conversion of a YAML sequence value between its inline
//! (flow) form `key: [a, b, c]` and its block form
//! (`key:` followed by `- a` / `- b` / `- c` lines), in either direction.
//!
//! Like the completion engine, this works structurally on the buffer's lines
//! (indentation + simple shape checks) rather than via a full YAML parse, so it
//! tolerates mid-edit input. It targets flat sequences of scalars (e.g. lists
//! of `ThingId`s), which is what the `InputDiagram` schema uses.

use async_lsp::lsp_types::{Position, Range, TextEdit};

use crate::completion::yaml_lines::{
    indent, is_blank_or_comment, is_list_item, line_map_key, split_flow_items,
};

pub use self::list_conversion::ListConversion;

mod list_conversion;

/// Computes sequence flow/block conversion code actions for a cursor position.
pub struct CodeActionEngine;

impl CodeActionEngine {
    /// Returns the list conversions applicable at `line` (zero-based) in
    /// `text`.
    ///
    /// At most one applies at a time: a `key: [..]` flow value offers a
    /// conversion to a block list, while a `key:` with following `- ` items
    /// (the cursor on the key or on any item) offers a conversion to an inline
    /// list.
    pub fn list_conversions(text: &str, line: u32) -> Vec<ListConversion> {
        let lines = text.split('\n').collect::<Vec<&str>>();
        let line_idx = (line as usize).min(lines.len().saturating_sub(1));

        [
            Self::flow_to_block(&lines, line_idx),
            Self::block_to_flow(&lines, line_idx),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    /// Converts a `key: [a, b, c]` flow value on `line_idx` into a block list.
    fn flow_to_block(lines: &[&str], line_idx: usize) -> Option<ListConversion> {
        let line = *lines.get(line_idx)?;
        let key = line_map_key(line)?;
        let colon_idx = line.find(':')?;
        let value = line[colon_idx + 1..].trim();
        let inner = value.strip_prefix('[')?.strip_suffix(']')?;

        let items = split_flow_items(inner);
        if items.is_empty() {
            return None;
        }

        let child_indent = " ".repeat(indent(line) + 2);
        let mut new_text = line[..=colon_idx].to_string();
        for item in &items {
            new_text.push('\n');
            new_text.push_str(&child_indent);
            new_text.push_str("- ");
            new_text.push_str(item);
        }

        Some(ListConversion {
            title: format!("Convert `{key}` to a block list"),
            edit: TextEdit {
                range: whole_line_range(line_idx, line),
                new_text,
            },
        })
    }

    /// Converts a `key:` with following `- ` block items into a `key: [..]`
    /// flow value. The cursor may be on the key line or on any item line.
    fn block_to_flow(lines: &[&str], line_idx: usize) -> Option<ListConversion> {
        let key_line_idx = Self::sequence_key_line(lines, line_idx)?;
        let key_line = lines[key_line_idx];
        let key = line_map_key(key_line)?;

        // The key must have no inline value -- a block sequence follows it.
        let colon_idx = key_line.find(':')?;
        if !key_line[colon_idx + 1..].trim().is_empty() {
            return None;
        }

        let key_indent = indent(key_line);
        let mut items = Vec::new();
        let mut last_idx = key_line_idx;
        for (idx, line) in lines.iter().enumerate().skip(key_line_idx + 1) {
            if is_blank_or_comment(line) {
                break;
            }
            if indent(line) < key_indent || !is_list_item(line) {
                break;
            }
            items.push(line.trim_start().trim_start_matches('-').trim().to_string());
            last_idx = idx;
        }
        if items.is_empty() {
            return None;
        }

        let new_text = format!(
            "{indent}{key}: [{items}]",
            indent = " ".repeat(key_indent),
            items = items.join(", "),
        );

        Some(ListConversion {
            title: format!("Convert `{key}` to an inline list"),
            edit: TextEdit {
                range: Range {
                    start: Position {
                        line: key_line_idx as u32,
                        character: 0,
                    },
                    end: Position {
                        line: last_idx as u32,
                        character: lines[last_idx].chars().count() as u32,
                    },
                },
                new_text,
            },
        })
    }

    /// Resolves the key line whose value is the block sequence at `line_idx`.
    ///
    /// `line_idx` may be the `key:` line itself, or one of the `- ` item lines
    /// (in which case the owning key above is returned).
    fn sequence_key_line(lines: &[&str], line_idx: usize) -> Option<usize> {
        let line = *lines.get(line_idx)?;

        if line_map_key(line).is_some() {
            return Some(line_idx);
        }

        if !is_list_item(line) {
            return None;
        }

        // Walk up past sibling items / their nested content to the owning key.
        let item_indent = indent(line);
        lines[..line_idx]
            .iter()
            .enumerate()
            .rev()
            .find_map(|(idx, candidate)| {
                if is_blank_or_comment(candidate) {
                    return None;
                }
                let candidate_indent = indent(candidate);
                if candidate_indent > item_indent
                    || (candidate_indent == item_indent && is_list_item(candidate))
                {
                    return None;
                }
                line_map_key(candidate).map(|_key| idx)
            })
    }
}

/// The range covering the whole of `line` at `line_idx` (column 0 to its end).
fn whole_line_range(line_idx: usize, line: &str) -> Range {
    Range {
        start: Position {
            line: line_idx as u32,
            character: 0,
        },
        end: Position {
            line: line_idx as u32,
            character: line.chars().count() as u32,
        },
    }
}
