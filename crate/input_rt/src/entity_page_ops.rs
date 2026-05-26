//! Entity page mutation helpers.
//!
//! Also contains `IdDuplicateParts`, a small helper used by
//! [`EntityPageOps::thing_duplicate`] to derive unique copy IDs.
//!
//! Provides [`EntityPageOps`] which groups all mutation operations for the
//! Entity editor page so that related functions are discoverable when sorted
//! by name, per the project's `noun_verb` naming convention.
//!
//! All methods operate on `&mut InputDiagram<'static>` instead of a
//! framework-specific signal type, making them testable without a UI runtime.

use disposition_input_model::InputDiagram;

use crate::{
    id_parse::{parse_id, parse_thing_id},
    on_change_target::OnChangeTarget,
};

/// Mutation operations for the Entity editor page.
///
/// Grouped here so that related functions are discoverable when sorted by
/// name, per the project's `noun_verb` naming convention.
pub struct EntityPageOps;

impl EntityPageOps {
    /// Adds a new thing description row with a unique placeholder Id.
    pub fn thing_desc_add(input_diagram: &mut InputDiagram<'static>) {
        let mut n = input_diagram.thing_descs.len();
        loop {
            let candidate = format!("thing_{n}");
            if let Some(id) = parse_id(&candidate)
                && !input_diagram.thing_descs.contains_key(&id)
            {
                input_diagram.thing_descs.insert(id, String::new());
                break;
            }
            n += 1;
        }
    }

    /// Adds a new edge description row with a unique placeholder Id.
    pub fn edge_desc_add(input_diagram: &mut InputDiagram<'static>) {
        let mut n = input_diagram.edge_descs.len();
        loop {
            let candidate = format!("edge_{n}");
            if let Some(id) = parse_id(&candidate)
                && !input_diagram.edge_descs.contains_key(&id)
            {
                input_diagram.edge_descs.insert(id, String::new());
                break;
            }
            n += 1;
        }
    }

    /// Adds a new entity tooltip row with a unique placeholder Id.
    pub fn entity_tooltip_add(input_diagram: &mut InputDiagram<'static>) {
        let mut n = input_diagram.entity_tooltips.len();
        loop {
            let candidate = format!("entity_{n}");
            if let Some(id) = parse_id(&candidate)
                && !input_diagram.entity_tooltips.contains_key(&id)
            {
                input_diagram.entity_tooltips.insert(id, String::new());
                break;
            }
            n += 1;
        }
    }

    // === Key-value (copy-text / desc / tooltip) mutation helpers === //

    /// Renames the key of a key-value entry in the target map.
    pub fn kv_entry_rename(
        input_diagram: &mut InputDiagram<'static>,
        target: OnChangeTarget,
        id_old_str: &str,
        id_new_str: &str,
        current_value: &str,
    ) {
        if id_old_str == id_new_str {
            return;
        }
        match target {
            OnChangeTarget::CopyText => {
                let entity_id_old = match parse_thing_id(id_old_str) {
                    Some(id) => id,
                    None => return,
                };
                let entity_id_new = match parse_thing_id(id_new_str) {
                    Some(id) => id,
                    None => return,
                };
                input_diagram
                    .thing_copy_text
                    .insert(entity_id_new, current_value.to_owned());
                input_diagram.thing_copy_text.swap_remove(&entity_id_old);
            }
            OnChangeTarget::ThingDesc => {
                let id_old = match parse_id(id_old_str) {
                    Some(id) => id,
                    None => return,
                };
                let id_new = match parse_id(id_new_str) {
                    Some(id) => id,
                    None => return,
                };
                input_diagram
                    .thing_descs
                    .insert(id_new, current_value.to_owned());
                input_diagram.thing_descs.swap_remove(&id_old);
            }
            OnChangeTarget::EdgeDesc => {
                let id_old = match parse_id(id_old_str) {
                    Some(id) => id,
                    None => return,
                };
                let id_new = match parse_id(id_new_str) {
                    Some(id) => id,
                    None => return,
                };
                input_diagram
                    .edge_descs
                    .insert(id_new, current_value.to_owned());
                input_diagram.edge_descs.swap_remove(&id_old);
            }
            OnChangeTarget::EntityTooltip => {
                let id_old = match parse_id(id_old_str) {
                    Some(id) => id,
                    None => return,
                };
                let id_new = match parse_id(id_new_str) {
                    Some(id) => id,
                    None => return,
                };
                input_diagram
                    .entity_tooltips
                    .insert(id_new, current_value.to_owned());
                input_diagram.entity_tooltips.swap_remove(&id_old);
            }
        }
    }

    /// Updates the value of a key-value entry in the target map.
    pub fn kv_entry_update(
        input_diagram: &mut InputDiagram<'static>,
        target: OnChangeTarget,
        id_str: &str,
        value: &str,
    ) {
        match target {
            OnChangeTarget::CopyText => {
                if let Some(entity_id) = parse_thing_id(id_str)
                    && let Some(entry) = input_diagram.thing_copy_text.get_mut(&entity_id)
                {
                    *entry = value.to_owned();
                }
            }
            OnChangeTarget::ThingDesc => {
                if let Some(entity_id) = parse_id(id_str)
                    && let Some(entry) = input_diagram.thing_descs.get_mut(&entity_id)
                {
                    *entry = value.to_owned();
                }
            }
            OnChangeTarget::EdgeDesc => {
                if let Some(entity_id) = parse_id(id_str)
                    && let Some(entry) = input_diagram.edge_descs.get_mut(&entity_id)
                {
                    *entry = value.to_owned();
                }
            }
            OnChangeTarget::EntityTooltip => {
                if let Some(entity_id) = parse_id(id_str)
                    && let Some(entry) = input_diagram.entity_tooltips.get_mut(&entity_id)
                {
                    *entry = value.to_owned();
                }
            }
        }
    }

    /// Removes a key-value entry from the target map.
    pub fn kv_entry_remove(
        input_diagram: &mut InputDiagram<'static>,
        target: OnChangeTarget,
        id_str: &str,
    ) {
        match target {
            OnChangeTarget::CopyText => {
                if let Some(entity_id) = parse_thing_id(id_str) {
                    input_diagram.thing_copy_text.remove(&entity_id);
                }
            }
            OnChangeTarget::ThingDesc => {
                if let Some(entity_id) = parse_id(id_str) {
                    input_diagram.thing_descs.remove(&entity_id);
                }
            }
            OnChangeTarget::EdgeDesc => {
                if let Some(entity_id) = parse_id(id_str) {
                    input_diagram.edge_descs.remove(&entity_id);
                }
            }
            OnChangeTarget::EntityTooltip => {
                if let Some(entity_id) = parse_id(id_str) {
                    input_diagram.entity_tooltips.remove(&entity_id);
                }
            }
        }
    }

    /// Moves a key-value entry from one index to another in the target map.
    pub fn kv_entry_move(
        input_diagram: &mut InputDiagram<'static>,
        target: OnChangeTarget,
        from: usize,
        to: usize,
    ) {
        match target {
            OnChangeTarget::CopyText => input_diagram.thing_copy_text.move_index(from, to),
            OnChangeTarget::ThingDesc => input_diagram.thing_descs.move_index(from, to),
            OnChangeTarget::EdgeDesc => input_diagram.edge_descs.move_index(from, to),
            OnChangeTarget::EntityTooltip => input_diagram.entity_tooltips.move_index(from, to),
        }
    }
}
