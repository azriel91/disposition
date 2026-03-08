//! Mutation operations for the Entity Types editor page.
//!
//! Grouped here so that related functions are discoverable when sorted by
//! name, per the project's `noun_verb` naming convention.
//!
//! All methods operate on `&mut InputDiagram<'static>` instead of a
//! framework-specific signal type, making them testable without a UI runtime.

use disposition_input_model::InputDiagram;
use disposition_model_common::{entity::EntityType, Set};

use crate::{id_parse::parse_id, id_rename::id_rename_in_input_diagram};

/// Mutation operations for the Entity Types editor page.
pub struct EntityTypesPageOps;

impl EntityTypesPageOps {
    // === Entry-level helpers (keyed by entity Id) === //

    /// Adds a new entity types entry with a unique placeholder Id.
    ///
    /// The new entry starts with an empty set of entity types.
    pub fn entry_add(input_diagram: &mut InputDiagram<'static>) {
        let mut n = input_diagram.entity_types.len();
        loop {
            let candidate = format!("entity_{n}");
            if let Some(id) = parse_id(&candidate)
                && !input_diagram.entity_types.contains_key(&id)
            {
                input_diagram.entity_types.insert(id, Set::new());
                break;
            }
            n += 1;
        }
    }

    /// Removes an entity types entry by its Id string.
    pub fn entry_remove(input_diagram: &mut InputDiagram<'static>, entity_id_str: &str) {
        if let Some(id) = parse_id(entity_id_str) {
            input_diagram.entity_types.remove(&id);
        }
    }

    /// Moves an entity types entry from one index to another.
    pub fn entry_move(input_diagram: &mut InputDiagram<'static>, from: usize, to: usize) {
        input_diagram.entity_types.move_index(from, to);
    }

    /// Renames an entity types entry key across all maps in the
    /// [`InputDiagram`].
    ///
    /// The current set of entity types is preserved under the new key.
    pub fn entry_rename(
        input_diagram: &mut InputDiagram<'static>,
        entity_id_old_str: &str,
        entity_id_new_str: &str,
        current_types: &[String],
    ) {
        if entity_id_old_str == entity_id_new_str {
            return;
        }
        let id_old = match parse_id(entity_id_old_str) {
            Some(id) => id,
            None => return,
        };
        let id_new = match parse_id(entity_id_new_str) {
            Some(id) => id,
            None => return,
        };

        // Rebuild the types set from the current UI state.
        let types: Set<EntityType> = current_types
            .iter()
            .filter_map(|s| parse_id(s).map(EntityType::from))
            .collect();

        // Insert the new key and remove the old one, preserving position.
        input_diagram.entity_types.insert(id_new.clone(), types);
        input_diagram.entity_types.swap_remove(&id_old);

        // Shared rename across entity_descs, entity_tooltips, entity_types,
        // and all theme style maps.
        id_rename_in_input_diagram(input_diagram, &id_old, &id_new);
    }

    // === Type-level helpers (within a single entity's type set) === //

    /// Adds a new entity type to an entity's type set.
    ///
    /// Uses `"type_custom_0"` (incrementing) as a placeholder if no existing
    /// custom types are defined.
    pub fn type_add(input_diagram: &mut InputDiagram<'static>, entity_id_str: &str) {
        let id = match parse_id(entity_id_str) {
            Some(id) => id,
            None => return,
        };

        if let Some(types) = input_diagram.entity_types.get_mut(&id) {
            let mut n = 0;
            loop {
                let candidate = format!("type_custom_{n}");
                if let Some(type_id) = parse_id(&candidate) {
                    let entity_type = EntityType::from(type_id);
                    if !types.contains(&entity_type) {
                        types.insert(entity_type);
                        break;
                    }
                }
                n += 1;
            }
        }
    }

    /// Removes an entity type from an entity's type set by index.
    pub fn type_remove(input_diagram: &mut InputDiagram<'static>, entity_id_str: &str, idx: usize) {
        let id = match parse_id(entity_id_str) {
            Some(id) => id,
            None => return,
        };

        if let Some(types) = input_diagram.entity_types.get_mut(&id)
            && idx < types.len()
        {
            types.remove_index(idx);
        }
    }

    /// Updates an entity type at a given index within an entity's type set.
    pub fn type_update(
        input_diagram: &mut InputDiagram<'static>,
        entity_id_str: &str,
        idx: usize,
        type_new_str: &str,
    ) {
        let id = match parse_id(entity_id_str) {
            Some(id) => id,
            None => return,
        };
        let type_id_new = match parse_id(type_new_str) {
            Some(id) => id,
            None => return,
        };
        let entity_type_new = EntityType::from(type_id_new);

        if let Some(types) = input_diagram.entity_types.get_mut(&id) {
            // `Set` (IndexSet/OrderSet) does not support indexed mutation
            // directly. Rebuild the set with the replacement at the given
            // position.
            let mut types_new = Set::with_capacity(types.len());
            for (i, existing) in types.iter().enumerate() {
                if i == idx {
                    types_new.insert(entity_type_new.clone());
                } else {
                    types_new.insert(existing.clone());
                }
            }
            *types = types_new;
        }
    }

    /// Moves an entity type within an entity's type set from one index to
    /// another.
    pub fn type_move(
        input_diagram: &mut InputDiagram<'static>,
        entity_id_str: &str,
        from: usize,
        to: usize,
    ) {
        let id = match parse_id(entity_id_str) {
            Some(id) => id,
            None => return,
        };

        if let Some(types) = input_diagram.entity_types.get_mut(&id) {
            types.move_index(from, to);
        }
    }
}
