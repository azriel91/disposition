//! Collects IDs already defined in the document, for value completion.
//!
//! The JSON schema describes *which* ID type a value position expects (e.g. a
//! `ThingId`), but not the concrete IDs the user has defined -- those live in
//! the document. This scans the buffer for the IDs declared under each
//! ID-defining block so they can be offered as value suggestions.

use std::collections::BTreeSet;

use crate::completion::{
    id_category::IdCategory,
    yaml_lines::{indent, is_blank_or_comment, line_map_key},
};

/// IDs defined in the document, grouped by category.
#[derive(Clone, Debug, Default)]
pub struct DynamicCompletions {
    /// `ThingId`s declared under `things` and `thing_hierarchy`.
    thing_ids: BTreeSet<String>,
    /// `TagId`s declared under `tags`.
    tag_ids: BTreeSet<String>,
    /// `ProcessStepId`s declared under any process's `steps`.
    step_ids: BTreeSet<String>,
    /// `EdgeGroupId`s declared under `thing_dependencies` / `thing_interactions`.
    edge_group_ids: BTreeSet<String>,
}

impl DynamicCompletions {
    /// Scans `text` for IDs defined under each ID-declaring block.
    pub fn from_text(text: &str) -> DynamicCompletions {
        let lines = text.split('\n').collect::<Vec<&str>>();

        let mut thing_ids = collect_block_keys(&lines, "things", true, true);
        thing_ids.extend(collect_block_keys(&lines, "thing_hierarchy", true, false));
        thing_ids.extend(collect_block_keys(&lines, "thing_descs", true, true));

        let tag_ids = collect_block_keys(&lines, "tags", true, true);

        let step_ids = collect_block_keys(&lines, "steps", false, true);

        let mut edge_group_ids = collect_block_keys(&lines, "thing_dependencies", true, true);
        edge_group_ids.extend(collect_block_keys(&lines, "thing_interactions", true, true));

        DynamicCompletions {
            thing_ids,
            tag_ids,
            step_ids,
            edge_group_ids,
        }
    }

    /// Returns the defined IDs for `category`, sorted.
    pub fn ids_for(&self, category: IdCategory) -> Vec<&str> {
        match category {
            IdCategory::Thing => self.thing_ids.iter().map(String::as_str).collect(),
            IdCategory::Tag => self.tag_ids.iter().map(String::as_str).collect(),
            IdCategory::ProcessStep => self.step_ids.iter().map(String::as_str).collect(),
            IdCategory::EdgeGroup => self.edge_group_ids.iter().map(String::as_str).collect(),
            IdCategory::Any => self
                .thing_ids
                .iter()
                .chain(&self.tag_ids)
                .chain(&self.step_ids)
                .chain(&self.edge_group_ids)
                .map(String::as_str)
                .collect(),
        }
    }
}

/// Collects the map keys declared inside every `name:` block in `lines`.
///
/// `top_level_only`: only match `name:` blocks at indent 0 (so a nested key that
/// happens to share the name is ignored). `first_level_only`: collect only the
/// block's direct children (the shallowest child indent) rather than every
/// nested key.
fn collect_block_keys(
    lines: &[&str],
    name: &str,
    top_level_only: bool,
    first_level_only: bool,
) -> BTreeSet<String> {
    let mut keys = BTreeSet::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let is_block_start = !is_blank_or_comment(line)
            && (!top_level_only || indent(line) == 0)
            && line_map_key(line).as_deref() == Some(name);

        if !is_block_start {
            i += 1;
            continue;
        }

        let block_indent = indent(line);
        let body_start = i + 1;

        // Find where the block body ends and the shallowest child indent.
        let mut body_end = body_start;
        let mut min_child_indent: Option<usize> = None;
        while body_end < lines.len() {
            let body_line = lines[body_end];
            if is_blank_or_comment(body_line) {
                body_end += 1;
                continue;
            }
            let body_indent = indent(body_line);
            if body_indent <= block_indent {
                break;
            }
            min_child_indent = Some(min_child_indent.map_or(body_indent, |m| m.min(body_indent)));
            body_end += 1;
        }

        for body_line in &lines[body_start..body_end] {
            if is_blank_or_comment(body_line) {
                continue;
            }
            if first_level_only && Some(indent(body_line)) != min_child_indent {
                continue;
            }
            if let Some(key) = line_map_key(body_line) {
                keys.insert(key);
            }
        }

        i = body_end;
    }

    keys
}
