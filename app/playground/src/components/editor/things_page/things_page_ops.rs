//! Things page mutation helpers.
//!
//! Provides [`ThingsPageOps`] which groups all mutation operations for the
//! Things editor page so that related functions are discoverable when sorted
//! by name, per the project's `noun_verb` naming convention.

use dioxus::signals::{ReadableExt, Signal, WritableExt};
use disposition::{
    input_model::{
        edge::EdgeGroup,
        thing::{ThingHierarchy, ThingId},
        InputDiagram,
    },
    model_common::Id,
};

use crate::components::editor::common::{id_rename_in_input_diagram, parse_id, parse_thing_id};

use super::on_change_target::OnChangeTarget;

/// Mutation operations for the Things editor page.
///
/// Grouped here so that related functions are discoverable when sorted by
/// name, per the project's `noun_verb` naming convention.
pub struct ThingsPageOps;

impl ThingsPageOps {
    // === Thing helpers === //

    /// Adds a new thing row with a unique placeholder ID.
    pub fn thing_add(mut input_diagram: Signal<InputDiagram<'static>>) {
        let mut n = input_diagram.read().things.len();
        loop {
            let candidate = format!("thing_{n}");
            if let Some(thing_id) = parse_thing_id(&candidate)
                && !input_diagram.read().things.contains_key(&thing_id)
            {
                input_diagram.write().things.insert(thing_id, String::new());
                break;
            }
            n += 1;
        }
    }

    /// Updates the display name for an existing thing.
    pub fn thing_name_update(
        mut input_diagram: Signal<InputDiagram<'static>>,
        thing_id_str: &str,
        name: &str,
    ) {
        if let Some(thing_id) = parse_thing_id(thing_id_str)
            && let Some(entry) = input_diagram.write().things.get_mut(&thing_id)
        {
            *entry = name.to_owned();
        }
    }

    /// Renames a thing across all maps in the [`InputDiagram`].
    pub fn thing_rename(
        mut input_diagram: Signal<InputDiagram<'static>>,
        thing_id_old_str: &str,
        thing_id_new_str: &str,
    ) {
        if thing_id_old_str == thing_id_new_str {
            return;
        }
        let mut input_diagram_ref = input_diagram.write();

        if let Ok(thing_id_old) = Id::new(thing_id_old_str)
            .map(Id::into_static)
            .map(ThingId::from)
            && let Ok(thing_id_new) = Id::new(thing_id_new_str)
                .map(Id::into_static)
                .map(ThingId::from)
        {
            // things: rename ThingId key.
            if let Some(thing_index) = input_diagram_ref.things.get_index_of(&thing_id_old) {
                let _result = input_diagram_ref
                    .things
                    .replace_index(thing_index, thing_id_new.clone());
            }

            // thing_copy_text: rename ThingId key.
            if let Some(thing_index) = input_diagram_ref
                .thing_copy_text
                .get_index_of(&thing_id_old)
            {
                let _result = input_diagram_ref
                    .thing_copy_text
                    .replace_index(thing_index, thing_id_new.clone());
            }

            // thing_hierarchy: recursive rename.
            if let Some((thing_hierarchy_with_id, thing_index)) = Self::thing_rename_in_hierarchy(
                &mut input_diagram_ref.thing_hierarchy,
                &thing_id_old,
            ) {
                let _result =
                    thing_hierarchy_with_id.replace_index(thing_index, thing_id_new.clone());
            }

            // thing_dependencies: rename ThingIds inside EdgeGroup values.
            input_diagram_ref
                .thing_dependencies
                .values_mut()
                .for_each(|edge_group| {
                    Self::thing_rename_in_edge_group(edge_group, &thing_id_old, &thing_id_new);
                });

            // thing_interactions: same structure as thing_dependencies.
            input_diagram_ref
                .thing_interactions
                .values_mut()
                .for_each(|edge_group| {
                    Self::thing_rename_in_edge_group(edge_group, &thing_id_old, &thing_id_new);
                });

            // tag_things: rename ThingIds in each Set<ThingId> value.
            input_diagram_ref.tag_things.values_mut().for_each(
                |thing_ids: &mut disposition::model_common::Set<ThingId<'static>>| {
                    if let Some(index) = thing_ids.get_index_of(&thing_id_old) {
                        let _result = thing_ids.replace_index(index, thing_id_new.clone());
                    }
                },
            );

            // Shared rename across entity_descs, entity_tooltips, entity_types,
            // and all theme style maps.
            let id_old = thing_id_old.into_inner();
            let id_new = thing_id_new.into_inner();
            id_rename_in_input_diagram(&mut input_diagram_ref, &id_old, &id_new);
        }
    }

    /// Replaces occurrences of `thing_id_old` with `thing_id_new` inside an
    /// [`EdgeGroup`] (which contains a `Vec<ThingId>`).
    fn thing_rename_in_edge_group(
        edge_group: &mut EdgeGroup<'static>,
        thing_id_old: &ThingId<'static>,
        thing_id_new: &ThingId<'static>,
    ) {
        edge_group.things.iter_mut().for_each(|thing_id| {
            if thing_id == thing_id_old {
                *thing_id = thing_id_new.clone();
            }
        });
    }

    /// Searches recursively through a [`ThingHierarchy`] for a given
    /// [`ThingId`] key, returning a mutable reference to the containing map
    /// and the index within it.
    fn thing_rename_in_hierarchy<'f, 'id>(
        thing_hierarchy: &'f mut ThingHierarchy<'id>,
        thing_id: &'f ThingId<'id>,
    ) -> Option<(&'f mut ThingHierarchy<'id>, usize)> {
        if let Some(thing_index) = thing_hierarchy.get_index_of(thing_id) {
            Some((thing_hierarchy, thing_index))
        } else {
            thing_hierarchy
                .values_mut()
                .find_map(|thing_hierarchy_child| {
                    Self::thing_rename_in_hierarchy(thing_hierarchy_child, thing_id)
                })
        }
    }

    /// Removes a thing from the `things` map.
    pub fn thing_remove(mut input_diagram: Signal<InputDiagram<'static>>, thing_id_str: &str) {
        if let Some(thing_id) = parse_thing_id(thing_id_str) {
            input_diagram.write().things.swap_remove(&thing_id);
        }
    }

    /// Moves a thing entry from one index to another in the `things` map.
    pub fn thing_move(mut input_diagram: Signal<InputDiagram<'static>>, from: usize, to: usize) {
        input_diagram.write().things.move_index(from, to);
    }

    // === Copy text helpers === //

    /// Adds a new copy-text row with a unique placeholder ThingId.
    pub fn copy_text_add(mut input_diagram: Signal<InputDiagram<'static>>) {
        let mut n = input_diagram.read().thing_copy_text.len();
        loop {
            let candidate = format!("thing_{n}");
            if let Some(thing_id) = parse_thing_id(&candidate)
                && !input_diagram.read().thing_copy_text.contains_key(&thing_id)
            {
                input_diagram
                    .write()
                    .thing_copy_text
                    .insert(thing_id, String::new());
                break;
            }
            n += 1;
        }
    }

    /// Adds a new entity description row with a unique placeholder Id.
    pub fn entity_desc_add(mut input_diagram: Signal<InputDiagram<'static>>) {
        let mut n = input_diagram.read().entity_descs.len();
        loop {
            let candidate = format!("entity_{n}");
            if let Some(id) = parse_id(&candidate)
                && !input_diagram.read().entity_descs.contains_key(&id)
            {
                input_diagram.write().entity_descs.insert(id, String::new());
                break;
            }
            n += 1;
        }
    }

    /// Adds a new entity tooltip row with a unique placeholder Id.
    pub fn entity_tooltip_add(mut input_diagram: Signal<InputDiagram<'static>>) {
        let mut n = input_diagram.read().entity_tooltips.len();
        loop {
            let candidate = format!("entity_{n}");
            if let Some(id) = parse_id(&candidate)
                && !input_diagram.read().entity_tooltips.contains_key(&id)
            {
                input_diagram
                    .write()
                    .entity_tooltips
                    .insert(id, String::new());
                break;
            }
            n += 1;
        }
    }

    // === Key-value (copy-text / desc / tooltip) mutation helpers === //

    /// Renames the key of a key-value entry in the target map.
    pub fn kv_entry_rename(
        mut input_diagram: Signal<InputDiagram<'static>>,
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
                let thing_id_old = match parse_thing_id(id_old_str) {
                    Some(id) => id,
                    None => return,
                };
                let thing_id_new = match parse_thing_id(id_new_str) {
                    Some(id) => id,
                    None => return,
                };
                let mut input_diagram = input_diagram.write();
                input_diagram.thing_copy_text.swap_remove(&thing_id_old);
                input_diagram
                    .thing_copy_text
                    .insert(thing_id_new, current_value.to_owned());
            }
            OnChangeTarget::EntityDesc => {
                let id_old = match parse_id(id_old_str) {
                    Some(id) => id,
                    None => return,
                };
                let id_new = match parse_id(id_new_str) {
                    Some(id) => id,
                    None => return,
                };
                let mut input_diagram = input_diagram.write();
                input_diagram.entity_descs.swap_remove(&id_old);
                input_diagram
                    .entity_descs
                    .insert(id_new, current_value.to_owned());
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
                let mut input_diagram = input_diagram.write();
                input_diagram.entity_tooltips.swap_remove(&id_old);
                input_diagram
                    .entity_tooltips
                    .insert(id_new, current_value.to_owned());
            }
        }
    }

    /// Updates the value of a key-value entry in the target map.
    pub fn kv_entry_update(
        mut input_diagram: Signal<InputDiagram<'static>>,
        target: OnChangeTarget,
        id_str: &str,
        value: &str,
    ) {
        match target {
            OnChangeTarget::CopyText => {
                if let Some(thing_id) = parse_thing_id(id_str)
                    && let Some(entry) = input_diagram.write().thing_copy_text.get_mut(&thing_id)
                {
                    *entry = value.to_owned();
                }
            }
            OnChangeTarget::EntityDesc => {
                if let Some(entity_id) = parse_id(id_str)
                    && let Some(entry) = input_diagram.write().entity_descs.get_mut(&entity_id)
                {
                    *entry = value.to_owned();
                }
            }
            OnChangeTarget::EntityTooltip => {
                if let Some(entity_id) = parse_id(id_str)
                    && let Some(entry) = input_diagram.write().entity_tooltips.get_mut(&entity_id)
                {
                    *entry = value.to_owned();
                }
            }
        }
    }

    /// Removes a key-value entry from the target map.
    pub fn kv_entry_remove(
        mut input_diagram: Signal<InputDiagram<'static>>,
        target: OnChangeTarget,
        id_str: &str,
    ) {
        match target {
            OnChangeTarget::CopyText => {
                if let Some(thing_id) = parse_thing_id(id_str) {
                    input_diagram.write().thing_copy_text.swap_remove(&thing_id);
                }
            }
            OnChangeTarget::EntityDesc => {
                if let Some(entity_id) = parse_id(id_str) {
                    input_diagram.write().entity_descs.swap_remove(&entity_id);
                }
            }
            OnChangeTarget::EntityTooltip => {
                if let Some(entity_id) = parse_id(id_str) {
                    input_diagram
                        .write()
                        .entity_tooltips
                        .swap_remove(&entity_id);
                }
            }
        }
    }

    /// Moves a key-value entry from one index to another in the target map.
    pub fn kv_entry_move(
        mut input_diagram: Signal<InputDiagram<'static>>,
        target: OnChangeTarget,
        from: usize,
        to: usize,
    ) {
        let mut input_diagram = input_diagram.write();
        match target {
            OnChangeTarget::CopyText => input_diagram.thing_copy_text.move_index(from, to),
            OnChangeTarget::EntityDesc => input_diagram.entity_descs.move_index(from, to),
            OnChangeTarget::EntityTooltip => input_diagram.entity_tooltips.move_index(from, to),
        }
    }
}
