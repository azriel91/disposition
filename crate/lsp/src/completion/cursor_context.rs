//! Resolves a cursor position in a YAML buffer to a schema path + target.
//!
//! This is a lightweight, indentation-based resolver rather than a full YAML
//! parse: the buffer is usually mid-edit (and therefore invalid YAML), so we
//! reconstruct the key path to the cursor and whether a key or a value is being
//! typed from line indentation alone. This tolerates incomplete input that a
//! strict parser would reject.

use crate::completion::{
    completion_target::CompletionTarget,
    yaml_lines::{indent, is_blank_or_comment, line_map_key},
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
        let (is_list, after_dash) = match trimmed.strip_prefix("- ") {
            Some(rest) => (true, rest),
            None if trimmed == "-" => (true, ""),
            None => (false, trimmed),
        };

        if let Some(colon_idx) = after_dash.find(':') {
            // `key: <cursor>` -- completing a value.
            let key = after_dash[..colon_idx].trim().to_string();
            let path = Self::ancestor_chain(&lines, line_idx, current_indent);
            return CursorContext {
                path,
                target: CompletionTarget::Value { key },
            };
        }

        if is_list {
            // `- <cursor>` -- a value within the enclosing array. The owning key
            // is the closest ancestor; its parents form the path.
            let chain = Self::ancestor_chain(&lines, line_idx, current_indent);
            if let Some((key, parents)) = chain.split_last() {
                return CursorContext {
                    path: parents.to_vec(),
                    target: CompletionTarget::Value { key: key.clone() },
                };
            }
        }

        // Typing a key -- offer the container's fields.
        let path = Self::ancestor_chain(&lines, line_idx, current_indent);
        CursorContext {
            path,
            target: CompletionTarget::Key,
        }
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
