//! Mutation operations for the Tags editor page.
//!
//! Grouped here so that related functions are discoverable when sorted by
//! name, per the project's `noun_verb` naming convention.
//!
//! All methods operate on `&mut InputDiagram<'static>` instead of a
//! framework-specific signal type, making them testable without a UI runtime.

use disposition_input_model::{theme::TagIdOrDefaults, thing::ThingId, InputDiagram};
use disposition_model_common::Set;

use crate::{
    id_parse::{parse_tag_id, parse_thing_id},
    id_rename::id_rename_in_input_diagram,
};

/// Mutation operations for the Tags editor page.
pub struct TagsPageOps;

impl TagsPageOps {
    // === Tag name helpers === //

    /// Adds a new tag with a unique placeholder TagId.
    pub fn tag_add(input_diagram: &mut InputDiagram<'static>) {
        let mut n = input_diagram.tags.len();
        loop {
            let candidate = format!("tag_{n}");
            if let Some(tag_id) = parse_tag_id(&candidate)
                && !input_diagram.tags.contains_key(&tag_id)
            {
                input_diagram.tags.insert(tag_id, String::new());
                break;
            }
            n += 1;
        }
    }

    /// Removes a tag from the `tags` map.
    pub fn tag_remove(input_diagram: &mut InputDiagram<'static>, tag_id_str: &str) {
        if let Some(tag_id) = parse_tag_id(tag_id_str) {
            input_diagram.tags.remove(&tag_id);
        }
    }

    /// Renames a tag across all maps in the [`InputDiagram`].
    pub fn tag_rename(
        input_diagram: &mut InputDiagram<'static>,
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

        // tags: rename TagId key.
        if let Some(index) = input_diagram.tags.get_index_of(&tag_id_old) {
            let _result = input_diagram.tags.replace_index(index, tag_id_new.clone());
        }

        // tag_things: rename TagId key.
        if let Some(index) = input_diagram.tag_things.get_index_of(&tag_id_old) {
            let _result = input_diagram
                .tag_things
                .replace_index(index, tag_id_new.clone());
        }

        // theme_tag_things_focus: rename TagIdOrDefaults::Custom key.
        let tag_key_old = TagIdOrDefaults::Custom(tag_id_old.clone());
        if let Some(index) = input_diagram
            .theme_tag_things_focus
            .get_index_of(&tag_key_old)
        {
            let tag_key_new = TagIdOrDefaults::Custom(tag_id_new.clone());
            let _result = input_diagram
                .theme_tag_things_focus
                .replace_index(index, tag_key_new);
        }

        // Shared rename across entity_descs, entity_tooltips, entity_types,
        // and all theme style maps.
        let id_old = tag_id_old.into_inner();
        let id_new = tag_id_new.into_inner();
        id_rename_in_input_diagram(input_diagram, &id_old, &id_new);
    }

    /// Updates the display name for an existing tag.
    pub fn tag_name_update(
        input_diagram: &mut InputDiagram<'static>,
        tag_id_str: &str,
        name: &str,
    ) {
        if let Some(tag_id) = parse_tag_id(tag_id_str)
            && let Some(entry) = input_diagram.tags.get_mut(&tag_id)
        {
            *entry = name.to_owned();
        }
    }

    /// Moves a tag entry from one index to another in the `tags` map.
    pub fn tag_move(input_diagram: &mut InputDiagram<'static>, from: usize, to: usize) {
        input_diagram.tags.move_index(from, to);
    }

    // === Tag things helpers === //

    /// Moves a tag->things entry from one index to another in the
    /// `tag_things` map.
    pub fn tag_things_entry_move(
        input_diagram: &mut InputDiagram<'static>,
        from: usize,
        to: usize,
    ) {
        input_diagram.tag_things.move_index(from, to);
    }

    /// Adds a new tag->things entry, picking an unmapped tag or generating a
    /// placeholder.
    pub fn tag_things_entry_add(input_diagram: &mut InputDiagram<'static>) {
        let tag_id = input_diagram
            .tags
            .keys()
            .find(|tag_id| !input_diagram.tag_things.contains_key(*tag_id))
            .cloned();

        match tag_id {
            Some(tag_id) => {
                input_diagram.tag_things.insert(tag_id, Set::new());
            }
            None => {
                let mut n = input_diagram.tag_things.len();
                loop {
                    let candidate = format!("tag_{n}");
                    if let Some(tag_id) = parse_tag_id(&candidate)
                        && !input_diagram.tag_things.contains_key(&tag_id)
                    {
                        input_diagram.tag_things.insert(tag_id, Set::new());
                        break;
                    }
                    n += 1;
                }
            }
        }
    }

    /// Removes a tag->things entry.
    pub fn tag_things_entry_remove(input_diagram: &mut InputDiagram<'static>, tag_id_str: &str) {
        if let Some(tag_id) = parse_tag_id(tag_id_str) {
            input_diagram.tag_things.remove(&tag_id);
        }
    }

    /// Renames the key of a tag->things entry.
    pub fn tag_things_entry_rename(
        input_diagram: &mut InputDiagram<'static>,
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
        input_diagram.tag_things.insert(tag_id_new, things);
        input_diagram.tag_things.swap_remove(&tag_id_old);
    }

    /// Updates a single thing within a tag's thing set at the given index.
    pub fn tag_things_thing_update(
        input_diagram: &mut InputDiagram<'static>,
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
    pub fn tag_things_thing_remove(
        input_diagram: &mut InputDiagram<'static>,
        tag_id_str: &str,
        idx: usize,
    ) {
        let tag_id = match parse_tag_id(tag_id_str) {
            Some(tag_id) => tag_id,
            None => return,
        };

        if let Some(things) = input_diagram.tag_things.get_mut(&tag_id)
            && idx < things.len()
        {
            things.remove_index(idx);
        }
    }

    /// Adds a thing to a tag's thing set.
    pub fn tag_things_thing_add(input_diagram: &mut InputDiagram<'static>, tag_id_str: &str) {
        let tag_id = match parse_tag_id(tag_id_str) {
            Some(tag_id) => tag_id,
            None => return,
        };

        // Pick the first thing ID as a placeholder.
        let placeholder = input_diagram
            .things
            .keys()
            .next()
            .map(|thing_id| thing_id.as_str().to_owned())
            .unwrap_or_else(|| "thing_0".to_owned());
        let thing_id_new = match parse_thing_id(&placeholder) {
            Some(thing_id) => thing_id,
            None => return,
        };

        if let Some(things) = input_diagram.tag_things.get_mut(&tag_id) {
            things.insert(thing_id_new);
        }
    }
}
