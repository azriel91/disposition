//! Collects IDs already defined in the document, for value completion.
//!
//! The JSON schema describes *which* ID type a value position expects (e.g. a
//! `ThingId`), but not the concrete IDs the user has defined -- those live in
//! the document. This scans the buffer for the IDs declared under each
//! ID-defining block so they can be offered as value suggestions.

use std::collections::BTreeSet;

use crate::completion::{
    id_category::IdCategory,
    key_category::KeyCategory,
    yaml_lines::{indent, is_blank_or_comment, is_list_item, line_map_key, split_flow_items},
};

/// IDs defined in the document, grouped by category.
#[derive(Clone, Debug, Default)]
pub struct DynamicCompletions {
    /// `ThingId`s declared under the `things` hierarchy and `thing_names`.
    thing_ids: BTreeSet<String>,
    /// `TagId`s declared under `tags`.
    tag_ids: BTreeSet<String>,
    /// `ProcessId`s declared under `processes`.
    process_ids: BTreeSet<String>,
    /// `ProcessStepId`s declared under any process's `steps`.
    step_ids: BTreeSet<String>,
    /// `EdgeGroupId`s declared under `thing_dependencies` /
    /// `thing_interactions`.
    edge_group_ids: BTreeSet<String>,
    /// Custom `type_*` ids declared as list items under any `entity_types`
    /// entry (e.g. `type_organisation` from `entity_types.t_aws:
    /// [type_organisation]`).
    entity_type_ids: BTreeSet<String>,
}

impl DynamicCompletions {
    /// Scans `text` for IDs defined under each ID-declaring block.
    pub fn from_text(text: &str) -> DynamicCompletions {
        let lines = text.split('\n').collect::<Vec<&str>>();

        // `things` is the recursive hierarchy (collect all nested keys);
        // `thing_names` and `thing_descs` are flat maps (direct children only).
        let mut thing_ids = collect_block_keys(&lines, "things", true, false);
        thing_ids.extend(collect_block_keys(&lines, "thing_names", true, true));
        thing_ids.extend(collect_block_keys(&lines, "thing_descs", true, true));

        let tag_ids = collect_block_keys(&lines, "tags", true, true);

        let process_ids = collect_block_keys(&lines, "processes", true, true);

        let step_ids = collect_block_keys(&lines, "steps", false, true);

        let mut edge_group_ids = collect_block_keys(&lines, "thing_dependencies", true, true);
        edge_group_ids.extend(collect_block_keys(&lines, "thing_interactions", true, true));

        let entity_type_ids = collect_entity_types_values(&lines);

        DynamicCompletions {
            thing_ids,
            tag_ids,
            process_ids,
            step_ids,
            edge_group_ids,
            entity_type_ids,
        }
    }

    /// Returns the defined IDs for `category`, sorted.
    pub fn ids_for(&self, category: IdCategory) -> Vec<&str> {
        match category {
            IdCategory::Thing => self.thing_ids.iter().map(String::as_str).collect(),
            IdCategory::Tag => self.tag_ids.iter().map(String::as_str).collect(),
            IdCategory::ProcessStep => self.step_ids.iter().map(String::as_str).collect(),
            IdCategory::EdgeGroup => self.edge_group_ids.iter().map(String::as_str).collect(),
            IdCategory::EntityType => self.entity_type_ids.iter().map(String::as_str).collect(),
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

    /// Returns the suggested map *key* labels for `key_category`.
    ///
    /// These are the document-derived IDs, templated IDs, and known literal
    /// keys. Schema-derived suggestions (the built-in `StyleAlias` /
    /// `EntityType` keys) are *not* included here -- the completion engine adds
    /// those from the schema, which this type does not have access to. Custom
    /// `EntityType`s declared in `entity_types`, however, *are*
    /// document-derived and so are included here.
    pub fn key_suggestions(&self, key_category: KeyCategory) -> Vec<String> {
        let owned = |ids: Vec<&str>| ids.into_iter().map(str::to_string).collect::<Vec<String>>();

        match key_category {
            KeyCategory::ThingId => owned(self.ids_for(IdCategory::Thing)),
            KeyCategory::EdgeGroupDep => vec![self.edge_group_suggestion("edge_dep")],
            KeyCategory::EdgeGroupInteraction => vec![self.edge_group_suggestion("edge_ix")],
            KeyCategory::TagName => vec![String::from("tag_example")],
            KeyCategory::TagId => owned(self.ids_for(IdCategory::Tag)),
            KeyCategory::EdgeId => self.edge_ids(),
            KeyCategory::Entity => {
                let mut suggestions = owned(self.ids_for(IdCategory::Thing));
                suggestions.extend(self.process_ids.iter().cloned());
                suggestions.extend(owned(self.ids_for(IdCategory::ProcessStep)));
                suggestions.extend(self.edge_ids());
                suggestions
            }
            KeyCategory::StyleAlias => vec![String::from("style_alias_custom")],
            KeyCategory::ThemeStyles => {
                let mut suggestions =
                    vec![String::from("node_defaults"), String::from("edge_defaults")];
                suggestions.extend(owned(self.ids_for(IdCategory::Thing)));
                suggestions.extend(owned(self.ids_for(IdCategory::EdgeGroup)));
                suggestions.extend(self.edge_ids());
                suggestions
            }
            KeyCategory::TagFocus => {
                let mut suggestions = vec![String::from("tag_defaults")];
                suggestions.extend(owned(self.ids_for(IdCategory::Tag)));
                suggestions
            }
            // Custom entity types declared in `entity_types`; built-in
            // entity types come from the schema, added by the engine.
            KeyCategory::EntityType => owned(self.ids_for(IdCategory::EntityType)),
        }
    }

    /// Builds an edge group ID suggestion from the first two defined things.
    ///
    /// e.g. `edge_dep__t_a_t_b` when `t_a` and `t_b` are the first two
    /// `thing_ids`, `edge_dep__t_a` when only one thing is defined, and the
    /// bare `edge_dep_` fallback when there are no things to choose from.
    fn edge_group_suggestion(&self, prefix: &str) -> String {
        let mut thing_ids = self.thing_ids.iter();
        match (thing_ids.next(), thing_ids.next()) {
            (Some(thing_id_0), Some(thing_id_1)) => {
                format!("{prefix}__{thing_id_0}_{thing_id_1}")
            }
            (Some(thing_id_0), None) => format!("{prefix}__{thing_id_0}"),
            _ => format!("{prefix}_"),
        }
    }

    /// Returns `<edge_group_id>__0` for each edge group defined in the
    /// document.
    fn edge_ids(&self) -> Vec<String> {
        self.edge_group_ids
            .iter()
            .map(|edge_group_id| format!("{edge_group_id}__0"))
            .collect()
    }
}

/// Collects the map keys declared inside every `name:` block in `lines`.
///
/// `top_level_only`: only match `name:` blocks at indent 0 (so a nested key
/// that happens to share the name is ignored). `first_level_only`: collect only
/// the block's direct children (the shallowest child indent) rather than every
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

/// Collects the custom entity type ids (`type_*` list items) declared under
/// each entry of the top-level `entity_types` block, e.g. `type_organisation`
/// from `entity_types.t_aws: [type_organisation]` or a block-sequence
/// `- type_organisation` item.
fn collect_entity_types_values(lines: &[&str]) -> BTreeSet<String> {
    let mut values = BTreeSet::new();

    let Some(block_start) = lines.iter().position(|line| {
        !is_blank_or_comment(line)
            && indent(line) == 0
            && line_map_key(line).as_deref() == Some("entity_types")
    }) else {
        return values;
    };

    for line in &lines[block_start + 1..] {
        if is_blank_or_comment(line) {
            continue;
        }
        if indent(line) == 0 {
            break;
        }

        if is_list_item(line) {
            let value = line.trim_start().trim_start_matches('-').trim();
            if !value.is_empty() {
                values.insert(value.to_string());
            }
        } else if line_map_key(line).is_some()
            && let Some(colon_idx) = line.find(':')
        {
            let value = line[colon_idx + 1..].trim();
            if let Some(inner) = value.strip_prefix('[').and_then(|v| v.strip_suffix(']')) {
                values.extend(split_flow_items(inner).into_iter().map(str::to_string));
            }
        }
    }

    values
}
