//! Resolves a cursor position in a YAML buffer to a schema path + target.
//!
//! This is a lightweight, indentation-based resolver rather than a full YAML
//! parse: the buffer is usually mid-edit (and therefore invalid YAML), so we
//! reconstruct the key path to the cursor and whether a key or a value is being
//! typed from line indentation alone. This tolerates incomplete input that a
//! strict parser would reject.

use std::collections::BTreeSet;

use crate::completion::{
    completion_target::CompletionTarget,
    yaml_lines::{indent, is_blank_or_comment, is_list_item, line_map_key},
};

/// The schema location the cursor is at: the map-key `path` from the document
/// root to the cursor's container, plus what is being completed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CursorContext {
    /// Keys from the root to the container the cursor is inside, e.g.
    /// `["render_options"]` when completing a child of `render_options`.
    pub path: Vec<String>,
    /// Whether a key or a value (and which key's value) is being completed.
    pub target: CompletionTarget,
    /// Map keys already declared as siblings of the cursor (the other direct
    /// children of the cursor's container). Duplicate map keys are invalid, so
    /// these are filtered out of key completions.
    pub sibling_keys: BTreeSet<String>,
}

impl CursorContext {
    /// Resolves the cursor at `line` / `character` (zero-based, as in LSP
    /// [`Position`](async_lsp::lsp_types::Position)) within `text`.
    pub fn at(text: &str, line: u32, character: u32) -> CursorContext {
        let lines = text.split('\n').collect::<Vec<&str>>();
        let line_idx = (line as usize).min(lines.len().saturating_sub(1));
        let current_line = lines.get(line_idx).copied().unwrap_or("");

        let current_indent = indent(current_line);

        // Text on the current line up to the cursor column.
        let prefix = {
            let chars = current_line.chars().collect::<Vec<char>>();
            let col = (character as usize).min(chars.len());
            chars[..col].iter().collect::<String>()
        };

        let trimmed = prefix.trim_start();
        // `list_needs_space` is set for a bare `-` with no following space, so a
        // selected value is inserted as `- value` rather than `-value`.
        let (is_list, list_needs_space, after_dash) = match trimmed.strip_prefix("- ") {
            Some(rest) => (true, false, rest),
            None if trimmed == "-" => (true, true, ""),
            None => (false, false, trimmed),
        };

        if let Some(colon_idx) = after_dash.find(':') {
            // `key: <cursor>` -- completing a value. (`key: <cursor>` completes
            // the value *of* `key`, which is not itself a list element even if
            // the line is a `- key: ..` item.)
            let key = after_dash[..colon_idx].trim().to_string();
            let value_part = &after_dash[colon_idx + 1..];
            // Inside `[ .. ]` flow brackets when more `[` than `]` precede the
            // cursor; a separator space is needed when nothing follows `key:`.
            let in_flow_list =
                value_part.matches('[').count() > value_part.matches(']').count();
            let needs_space = value_part.is_empty();
            let path = Self::ancestor_chain(&lines, line_idx, current_indent);
            return CursorContext {
                path,
                target: CompletionTarget::Value {
                    key,
                    in_sequence: in_flow_list,
                    needs_space,
                },
                sibling_keys: BTreeSet::new(),
            };
        }

        if is_list {
            // `- <cursor>` -- a value within the enclosing sequence. The owning
            // key may sit at the same indent as the `- ` items (a block sequence
            // that is not indented under its key), or shallower.
            if let Some((path, key)) =
                Self::list_value_target(&lines, line_idx, current_indent)
            {
                return CursorContext {
                    path,
                    target: CompletionTarget::Value {
                        key,
                        in_sequence: true,
                        needs_space: list_needs_space,
                    },
                    sibling_keys: BTreeSet::new(),
                };
            }
        }

        // Typing a key -- offer the container's fields, minus siblings already
        // declared.
        let path = Self::ancestor_chain(&lines, line_idx, current_indent);
        let sibling_keys = Self::sibling_keys(&lines, line_idx, current_indent);
        CursorContext {
            path,
            target: CompletionTarget::Key,
            sibling_keys,
        }
    }

    /// Collects the map keys already declared as direct siblings of the cursor.
    ///
    /// Scans up and down from the cursor line, gathering map keys at exactly
    /// `base_indent` until a shallower (dedented) line bounds the enclosing
    /// block in each direction. Lines deeper than `base_indent` are descendants
    /// of a sibling and are skipped; the cursor's own line is excluded.
    fn sibling_keys(lines: &[&str], from_line_idx: usize, base_indent: usize) -> BTreeSet<String> {
        let mut sibling_keys = BTreeSet::new();

        let mut collect = |line: &str| -> bool {
            if is_blank_or_comment(line) {
                return true;
            }
            let line_indent = indent(line);
            if line_indent < base_indent {
                return false;
            }
            if line_indent == base_indent
                && let Some(key) = line_map_key(line)
            {
                sibling_keys.insert(key);
            }
            true
        };

        for line in lines[..from_line_idx].iter().rev() {
            if !collect(line) {
                break;
            }
        }
        for line in lines.iter().skip(from_line_idx + 1) {
            if !collect(line) {
                break;
            }
        }

        sibling_keys
    }

    /// Resolves the `(path, key)` whose value is the sequence containing the
    /// `- ` item at `from_line_idx`.
    ///
    /// The owning key is the nearest line above that is a map key at indent
    /// `<= base_indent`, skipping sibling `- ` items at `base_indent` and any
    /// deeper-indented content belonging to them. This handles block sequences
    /// at the *same* indent as their key (`things:\n- a`), not just those
    /// indented under it. Returns `None` when no owning key is found (e.g. a
    /// top-level or nested-in-sequence list).
    fn list_value_target(
        lines: &[&str],
        from_line_idx: usize,
        base_indent: usize,
    ) -> Option<(Vec<String>, String)> {
        let owner_idx = lines[..from_line_idx]
            .iter()
            .enumerate()
            .rev()
            .find_map(|(idx, line)| {
                if is_blank_or_comment(line) {
                    return None;
                }
                let line_indent = indent(line);
                // Deeper content belongs to a sibling item; a sibling item at
                // the same indent is part of the same sequence -- skip both.
                if line_indent > base_indent
                    || (line_indent == base_indent && is_list_item(line))
                {
                    return None;
                }
                Some(idx)
            })?;

        let owner_key = line_map_key(lines[owner_idx])?;
        let owner_indent = indent(lines[owner_idx]);
        let path = Self::ancestor_chain(lines, owner_idx, owner_indent);
        Some((path, owner_key))
    }

    /// Walks upward from `from_line_idx`, collecting the map keys of containers
    /// enclosing the cursor -- lines with strictly decreasing indentation.
    ///
    /// Returns the keys in root-to-closest order, e.g. `["processes",
    /// "proc_app_dev", "steps"]`.
    fn ancestor_chain(lines: &[&str], from_line_idx: usize, base_indent: usize) -> Vec<String> {
        let mut path = Vec::new();
        let mut needed_indent = base_indent;

        for line in lines[..from_line_idx].iter().rev() {
            if is_blank_or_comment(line) {
                continue;
            }

            let line_indent = indent(line);
            if line_indent >= needed_indent {
                continue;
            }

            // A shallower line: either a parent map key, or a list-item marker
            // whose own owner key sits even shallower. Either way, descend the
            // search to this indent; only push actual map keys.
            if let Some(key) = line_map_key(line) {
                path.push(key);
            }
            needed_indent = line_indent;

            if needed_indent == 0 {
                break;
            }
        }

        path.reverse();
        path
    }
}
