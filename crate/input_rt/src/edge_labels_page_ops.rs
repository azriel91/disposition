//! Edge labels page mutation helpers.
//!
//! Provides [`EdgeLabelsPageOps`] which groups all mutation operations for the
//! Edge Labels editor page.
//!
//! All methods operate on `&mut InputDiagram<'static>` instead of a
//! framework-specific signal type, making them testable without a UI runtime.

use disposition_input_model::InputDiagram;
use disposition_model_common::edge::EdgeLabel;

use crate::id_parse::{parse_edge_id, parse_id};

/// Mutation operations for the Edge Labels editor page.
///
/// Grouped here so that related functions are discoverable when sorted by
/// name, per the project's `noun_verb` naming convention.
pub struct EdgeLabelsPageOps;

impl EdgeLabelsPageOps {
    // === Page-level data helpers === //

    /// Returns the page entries in display order as
    /// `(edge_id_str, from, to, edge_desc)` tuples.
    ///
    /// The primary iteration order is `edge_labels`. For each edge ID the
    /// corresponding `edge_descs` entry (if any) is looked up and included
    /// as `edge_desc`.
    pub fn edge_label_entries(
        input_diagram: &InputDiagram<'static>,
    ) -> Vec<(String, String, String, String)> {
        input_diagram
            .edge_labels
            .iter()
            .map(|(edge_id, edge_label)| {
                let id_str = edge_id.as_str().to_owned();
                let edge_desc = parse_id(edge_id.as_str())
                    .and_then(|id| input_diagram.edge_descs.get(&id))
                    .cloned()
                    .unwrap_or_default();
                (
                    id_str,
                    edge_label.from.clone(),
                    edge_label.to.clone(),
                    edge_desc,
                )
            })
            .collect()
    }

    // === Entry-level mutation helpers === //

    /// Adds a new edge label row with a unique placeholder `EdgeId`.
    pub fn edge_label_add(input_diagram: &mut InputDiagram<'static>) {
        let mut n = input_diagram.edge_labels.len();
        loop {
            let candidate = format!("edge_{n}");
            if let Some(edge_id) = parse_edge_id(&candidate)
                && !input_diagram.edge_labels.contains_key(&edge_id)
            {
                input_diagram
                    .edge_labels
                    .insert(edge_id, EdgeLabel::default());
                break;
            }
            n += 1;
        }
    }

    /// Renames an edge label entry's key in both `edge_labels` and
    /// `edge_descs`.
    ///
    /// If `edge_id_old_str` or `edge_id_new_str` is not a valid identifier the
    /// rename is silently skipped. When the old key has a corresponding
    /// `edge_descs` entry, that entry is also renamed to `edge_id_new_str`.
    pub fn edge_label_rename(
        input_diagram: &mut InputDiagram<'static>,
        edge_id_old_str: &str,
        edge_id_new_str: &str,
        current_edge_label: EdgeLabel,
        current_entity_desc: &str,
    ) {
        if edge_id_old_str == edge_id_new_str {
            return;
        }
        let edge_id_old = match parse_edge_id(edge_id_old_str) {
            Some(id) => id,
            None => return,
        };
        let edge_id_new = match parse_edge_id(edge_id_new_str) {
            Some(id) => id,
            None => return,
        };
        input_diagram
            .edge_labels
            .insert(edge_id_new, current_edge_label);
        input_diagram.edge_labels.swap_remove(&edge_id_old);

        // Also rename the corresponding edge_descs entry when present.
        if let Some(entity_id_old) = parse_id(edge_id_old_str)
            && input_diagram.edge_descs.contains_key(&entity_id_old)
            && let Some(entity_id_new) = parse_id(edge_id_new_str)
        {
            input_diagram
                .edge_descs
                .insert(entity_id_new, current_entity_desc.to_owned());
            input_diagram.edge_descs.swap_remove(&entity_id_old);
        }
    }

    /// Updates the `from` label for the given edge in `edge_labels`.
    pub fn edge_label_from_update(
        input_diagram: &mut InputDiagram<'static>,
        edge_id_str: &str,
        from: &str,
    ) {
        if let Some(edge_id) = parse_edge_id(edge_id_str)
            && let Some(entry) = input_diagram.edge_labels.get_mut(&edge_id)
        {
            entry.from = from.to_owned();
        }
    }

    /// Updates the `to` label for the given edge in `edge_labels`.
    pub fn edge_label_to_update(
        input_diagram: &mut InputDiagram<'static>,
        edge_id_str: &str,
        to: &str,
    ) {
        if let Some(edge_id) = parse_edge_id(edge_id_str)
            && let Some(entry) = input_diagram.edge_labels.get_mut(&edge_id)
        {
            entry.to = to.to_owned();
        }
    }

    /// Upserts the edge description for the given edge in `edge_descs`.
    ///
    /// Creates a new entry if none exists, updates it if one is present.
    pub fn edge_label_entity_desc_update(
        input_diagram: &mut InputDiagram<'static>,
        edge_id_str: &str,
        entity_desc: &str,
    ) {
        if let Some(entity_id) = parse_id(edge_id_str) {
            input_diagram
                .edge_descs
                .insert(entity_id, entity_desc.to_owned());
        }
    }

    /// Removes an edge label entry from both `edge_labels` and `edge_descs`.
    pub fn edge_label_remove(input_diagram: &mut InputDiagram<'static>, edge_id_str: &str) {
        if let Some(edge_id) = parse_edge_id(edge_id_str) {
            input_diagram.edge_labels.swap_remove(&edge_id);
        }
        if let Some(entity_id) = parse_id(edge_id_str) {
            input_diagram.edge_descs.swap_remove(&entity_id);
        }
    }

    /// Moves an edge label entry from one index to another in `edge_labels`.
    pub fn edge_label_move(input_diagram: &mut InputDiagram<'static>, from: usize, to: usize) {
        input_diagram.edge_labels.move_index(from, to);
    }

    /// Returns the total number of edge label entries.
    pub fn edge_label_count(input_diagram: &InputDiagram<'static>) -> usize {
        input_diagram.edge_labels.len()
    }
}
