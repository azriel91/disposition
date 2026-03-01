//! Mutation operations for the Tags editor page.
//!
//! Grouped here so that related functions are discoverable when sorted by
//! name, per the project's `noun_verb` naming convention.

use dioxus::signals::{ReadableExt, Signal, WritableExt};
use disposition::{
    input_model::{theme::TagIdOrDefaults, thing::ThingId, InputDiagram},
    model_common::Set,
};

use crate::components::editor::common::{id_rename_in_input_diagram, parse_tag_id, parse_thing_id};

/// Mutation operations for the Tags editor page.
pub(crate) struct TagsPageOps;

impl TagsPageOps {
    // === Tag name helpers === //

    /// Adds a new tag with a unique placeholder TagId.
    pub(crate) fn tag_add(mut input_diagram: Signal<InputDiagram<'static>>) {
        let mut n = input_diagram.read().tags.len();
        loop {
            let candidate = format!("tag_{n}");
            if let Some(tag_id) = parse_tag_id(&candidate)
                && !input_diagram.read().tags.contains_key(&tag_id)
            {
                input_diagram.write().tags.insert(tag_id, String::new());
                break;
            }
            n += 1;
        }
    }

    /// Removes a tag from the `tags` map.
    pub(crate) fn tag_remove(mut input_diagram: Signal<InputDiagram<'static>>, tag_id_str: &str) {
        if let Some(tag_id) = parse_tag_id(tag_id_str) {
            input_diagram.write().tags.shift_remove(&tag_id);
        }
    }

    /// Renames a tag across all maps in the [`InputDiagram`].
    pub(crate) fn tag_rename(
        mut input_diagram: Signal<InputDiagram<'static>>,
        tag_id_old_str: &str,
        tag_id_new_str: &str,
    ) {
        if tag_id_old_str == tag_id_new_str {
            return;
        }
        let tag_id_old = match parse_tag_id(tag_id_old_str) {
            Some(tag_id) => tag_id,
            None => return,
        };
        let tag_id_new = match parse_tag_id(tag_id_new_str) {
            Some(tag_id) => tag_id,
            None => return,
        };

        let mut input_diagram_ref = input_diagram.write();

        // tags: rename TagId key.
        if let Some(index) = input_diagram_ref.tags.get_index_of(&tag_id_old) {
            let _result = input_diagram_ref
                .tags
                .replace_index(index, tag_id_new.clone());
        }

        // tag_things: rename TagId key.
        if let Some(index) = input_diagram_ref.tag_things.get_index_of(&tag_id_old) {
            let _result = input_diagram_ref
                .tag_things
                .replace_index(index, tag_id_new.clone());
        }

        // theme_tag_things_focus: rename TagIdOrDefaults::Custom key.
        let tag_key_old = TagIdOrDefaults::Custom(tag_id_old.clone());
        if let Some(index) = input_diagram_ref
            .theme_tag_things_focus
            .get_index_of(&tag_key_old)
        {
            let tag_key_new = TagIdOrDefaults::Custom(tag_id_new.clone());
            let _result = input_diagram_ref
                .theme_tag_things_focus
                .replace_index(index, tag_key_new);
        }

        // Shared rename across entity_descs, entity_tooltips, entity_types,
        // and all theme style maps.
        let id_old = tag_id_old.into_inner();
        let id_new = tag_id_new.into_inner();
        id_rename_in_input_diagram(&mut input_diagram_ref, &id_old, &id_new);
    }

    /// Updates the display name for an existing tag.
    pub(crate) fn tag_name_update(
        mut input_diagram: Signal<InputDiagram<'static>>,
        tag_id_str: &str,
        name: &str,
    ) {
        if let Some(tag_id) = parse_tag_id(tag_id_str)
            && let Some(entry) = input_diagram.write().tags.get_mut(&tag_id)
        {
            *entry = name.to_owned();
        }
    }

    /// Moves a tag entry from one index to another in the `tags` map.
    pub(crate) fn tag_move(
        mut input_diagram: Signal<InputDiagram<'static>>,
        from: usize,
        to: usize,
    ) {
        input_diagram.write().tags.move_index(from, to);
    }

    // === Tag things helpers === //

    /// Adds a new tag->things entry, picking an unmapped tag or generating a
    /// placeholder.
    pub(crate) fn tag_things_entry_add(mut input_diagram: Signal<InputDiagram<'static>>) {
        let input_diagram_read = input_diagram.read();
        let tag_id = input_diagram_read
            .tags
            .keys()
            .find(|tag_id| !input_diagram_read.tag_things.contains_key(*tag_id))
            .cloned();

        match tag_id {
            Some(tag_id) => {
                drop(input_diagram_read);
                input_diagram.write().tag_things.insert(tag_id, Set::new());
            }
            None => {
                let mut n = input_diagram_read.tag_things.len();
                loop {
                    let candidate = format!("tag_{n}");
                    if let Some(tag_id) = parse_tag_id(&candidate)
                        && !input_diagram_read.tag_things.contains_key(&tag_id)
                    {
                        drop(input_diagram_read);
                        input_diagram.write().tag_things.insert(tag_id, Set::new());
                        break;
                    }
                    n += 1;
                }
            }
        }
    }

    /// Removes a tag->things entry.
    pub(crate) fn tag_things_entry_remove(
        mut input_diagram: Signal<InputDiagram<'static>>,
        tag_id_str: &str,
    ) {
        if let Some(tag_id) = parse_tag_id(tag_id_str) {
            input_diagram.write().tag_things.shift_remove(&tag_id);
        }
    }

    /// Renames the key of a tag->things entry.
    pub(crate) fn tag_things_entry_rename(
        mut input_diagram: Signal<InputDiagram<'static>>,
        tag_id_old_str: &str,
        tag_id_new_str: &str,
        current_things: &[String],
    ) {
        if tag_id_old_str == tag_id_new_str {
            return;
        }
        let tag_id_old = match parse_tag_id(tag_id_old_str) {
            Some(tag_id) => tag_id,
            None => return,
        };
        let tag_id_new = match parse_tag_id(tag_id_new_str) {
            Some(tag_id) => tag_id,
            None => return,
        };
        let things: Set<ThingId<'static>> = current_things
            .iter()
            .filter_map(|s| parse_thing_id(s))
            .collect();
        let mut input_diagram = input_diagram.write();
        input_diagram.tag_things.insert(tag_id_new, things);
        input_diagram.tag_things.swap_remove(&tag_id_old);
    }

    /// Updates a single thing within a tag's thing set at the given index.
    pub(crate) fn tag_things_thing_update(
        mut input_diagram: Signal<InputDiagram<'static>>,
        tag_id_str: &str,
        idx: usize,
        thing_id_new_str: &str,
    ) {
        let tag_id = match parse_tag_id(tag_id_str) {
            Some(tag_id) => tag_id,
            None => return,
        };
        let thing_id_new = match parse_thing_id(thing_id_new_str) {
            Some(thing_id) => thing_id,
            None => return,
        };

        let mut input_diagram = input_diagram.write();
        if let Some(things) = input_diagram.tag_things.get_mut(&tag_id) {
            // `Set` (IndexSet) does not support indexed mutation directly.
            // Rebuild the set with the replacement at the given position.
            let mut things_new = Set::with_capacity(things.len());
            for (i, existing) in things.iter().enumerate() {
                if i == idx {
                    things_new.insert(thing_id_new.clone());
                } else {
                    things_new.insert(existing.clone());
                }
            }
            *things = things_new;
        }
    }

    /// Removes a thing from a tag's thing set by index.
    pub(crate) fn tag_things_thing_remove(
        mut input_diagram: Signal<InputDiagram<'static>>,
        tag_id_str: &str,
        idx: usize,
    ) {
        let tag_id = match parse_tag_id(tag_id_str) {
            Some(tag_id) => tag_id,
            None => return,
        };

        let mut input_diagram = input_diagram.write();
        if let Some(things) = input_diagram.tag_things.get_mut(&tag_id)
            && idx < things.len()
        {
            things.shift_remove_index(idx);
        }
    }

    /// Adds a thing to a tag's thing set.
    pub(crate) fn tag_things_thing_add(
        mut input_diagram: Signal<InputDiagram<'static>>,
        tag_id_str: &str,
    ) {
        let tag_id = match parse_tag_id(tag_id_str) {
            Some(tag_id) => tag_id,
            None => return,
        };

        // Pick the first thing ID as a placeholder.
        let placeholder = {
            let input_diagram = input_diagram.read();
            input_diagram
                .things
                .keys()
                .next()
                .map(|thing_id| thing_id.as_str().to_owned())
                .unwrap_or_else(|| "thing_0".to_owned())
        };
        let thing_id_new = match parse_thing_id(&placeholder) {
            Some(thing_id) => thing_id,
            None => return,
        };

        let mut input_diagram = input_diagram.write();
        if let Some(things) = input_diagram.tag_things.get_mut(&tag_id) {
            things.insert(thing_id_new);
        }
    }
}
