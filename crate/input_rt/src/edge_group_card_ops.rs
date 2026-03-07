//! Mutation operations for edge group entries.
//!
//! Grouped here so that related functions are discoverable when sorted by
//! name, per the project's `noun_verb` naming convention.
//!
//! All methods operate on `&mut InputDiagram<'static>` (or
//! `&InputDiagram<'static>` for read-only queries) instead of a
//! framework-specific signal type, making them testable without a UI runtime.

use disposition_input_model::{
    edge::{EdgeGroup, EdgeKind},
    thing::ThingId,
    InputDiagram,
};
use disposition_model_common::edge::EdgeGroupId;

use crate::{
    id_parse::{parse_edge_group_id, parse_thing_id},
    id_rename::id_rename_in_input_diagram,
    map_target::MapTarget,
};

/// Mutation operations for edge group entries.
pub struct EdgeGroupCardOps;

impl EdgeGroupCardOps {
    // === Map-target helpers === //

    /// Sets the [`EdgeGroup`] for a given `EdgeGroupId` in the target map.
    pub fn edge_group_set(
        input_diagram: &mut InputDiagram<'static>,
        target: MapTarget,
        edge_group_id: &EdgeGroupId<'static>,
        edge_group: EdgeGroup<'static>,
    ) {
        match target {
            MapTarget::Dependencies => {
                input_diagram
                    .thing_dependencies
                    .insert(edge_group_id.clone(), edge_group);
            }
            MapTarget::Interactions => {
                input_diagram
                    .thing_interactions
                    .insert(edge_group_id.clone(), edge_group);
            }
        }
    }

    /// Removes an edge group by ID from the target map.
    pub fn edge_group_remove_by_id(
        input_diagram: &mut InputDiagram<'static>,
        target: MapTarget,
        edge_group_id: &EdgeGroupId<'static>,
    ) {
        match target {
            MapTarget::Dependencies => {
                input_diagram.thing_dependencies.remove(edge_group_id);
            }
            MapTarget::Interactions => {
                input_diagram.thing_interactions.remove(edge_group_id);
            }
        }
    }

    /// Returns the number of edge groups in the target map.
    pub fn edge_group_count(input_diagram: &InputDiagram<'static>, target: MapTarget) -> usize {
        match target {
            MapTarget::Dependencies => input_diagram.thing_dependencies.len(),
            MapTarget::Interactions => input_diagram.thing_interactions.len(),
        }
    }

    /// Returns whether the target map contains the given edge group ID.
    pub fn edge_group_contains(
        input_diagram: &InputDiagram<'static>,
        target: MapTarget,
        edge_group_id: &EdgeGroupId<'static>,
    ) -> bool {
        match target {
            MapTarget::Dependencies => input_diagram.thing_dependencies.contains_key(edge_group_id),
            MapTarget::Interactions => input_diagram.thing_interactions.contains_key(edge_group_id),
        }
    }

    // === Mutation helpers === //

    /// Adds a new edge group with a unique placeholder ID.
    pub fn edge_group_add(input_diagram: &mut InputDiagram<'static>, target: MapTarget) {
        let mut n = Self::edge_group_count(input_diagram, target);
        loop {
            let candidate = format!("edge_{n}");
            if let Some(edge_group_id) = parse_edge_group_id(&candidate)
                && !Self::edge_group_contains(input_diagram, target, &edge_group_id)
            {
                Self::edge_group_set(
                    input_diagram,
                    target,
                    &edge_group_id,
                    EdgeGroup::new(EdgeKind::Sequence, Vec::new()),
                );
                break;
            }
            n += 1;
        }
    }

    /// Moves an edge group from one index to another in the target map.
    pub fn edge_group_move(
        input_diagram: &mut InputDiagram<'static>,
        target: MapTarget,
        from: usize,
        to: usize,
    ) {
        match target {
            MapTarget::Dependencies => {
                input_diagram.thing_dependencies.move_index(from, to);
            }
            MapTarget::Interactions => {
                input_diagram.thing_interactions.move_index(from, to);
            }
        }
    }

    /// Removes an edge group from the target map.
    pub fn edge_group_remove(
        input_diagram: &mut InputDiagram<'static>,
        target: MapTarget,
        edge_group_id_str: &str,
    ) {
        if let Some(edge_group_id) = parse_edge_group_id(edge_group_id_str) {
            Self::edge_group_remove_by_id(input_diagram, target, &edge_group_id);
        }
    }

    /// Renames an edge group across all maps in the [`InputDiagram`].
    pub fn edge_group_rename(
        input_diagram: &mut InputDiagram<'static>,
        edge_group_id_old_str: &str,
        edge_group_id_new_str: &str,
    ) {
        if edge_group_id_old_str == edge_group_id_new_str {
            return;
        }
        let edge_group_id_old = match parse_edge_group_id(edge_group_id_old_str) {
            Some(edge_group_id) => edge_group_id,
            None => return,
        };
        let edge_group_id_new = match parse_edge_group_id(edge_group_id_new_str) {
            Some(edge_group_id) => edge_group_id,
            None => return,
        };

        // thing_dependencies: rename EdgeGroupId key.
        if let Some(index) = input_diagram
            .thing_dependencies
            .get_index_of(&edge_group_id_old)
        {
            let _result = input_diagram
                .thing_dependencies
                .replace_index(index, edge_group_id_new.clone());
        }

        // thing_interactions: rename EdgeGroupId key.
        if let Some(index) = input_diagram
            .thing_interactions
            .get_index_of(&edge_group_id_old)
        {
            let _result = input_diagram
                .thing_interactions
                .replace_index(index, edge_group_id_new.clone());
        }

        // processes: rename EdgeGroupId in step_thing_interactions values.
        input_diagram
            .processes
            .values_mut()
            .for_each(|process_diagram| {
                process_diagram
                    .step_thing_interactions
                    .values_mut()
                    .for_each(|edge_group_ids| {
                        for edge_group_id in edge_group_ids.iter_mut() {
                            if edge_group_id == &edge_group_id_old {
                                *edge_group_id = edge_group_id_new.clone();
                            }
                        }
                    });
            });

        // Shared rename across entity_descs, entity_tooltips, entity_types,
        // and all theme style maps.
        let id_old = edge_group_id_old.into_inner();
        let id_new = edge_group_id_new.into_inner();
        id_rename_in_input_diagram(input_diagram, &id_old, &id_new);
    }

    /// Changes the edge kind (cyclic / sequence / symmetric) for an edge
    /// group, preserving the current thing list.
    pub fn edge_kind_change(
        input_diagram: &mut InputDiagram<'static>,
        target: MapTarget,
        edge_group_id_str: &str,
        edge_kind_new: EdgeKind,
        current_things: &[ThingId<'static>],
    ) {
        let edge_group_id = match parse_edge_group_id(edge_group_id_str) {
            Some(edge_group_id) => edge_group_id,
            None => return,
        };
        let things: Vec<ThingId<'static>> = current_things
            .iter()
            .filter_map(|s| parse_thing_id(s))
            .collect();
        let edge_group = EdgeGroup::new(edge_kind_new, things);
        Self::edge_group_set(input_diagram, target, &edge_group_id, edge_group);
    }

    /// Updates a single thing within an edge group at the given index.
    pub fn edge_thing_update(
        input_diagram: &mut InputDiagram<'static>,
        target: MapTarget,
        edge_group_id_str: &str,
        idx: usize,
        thing_id_new_str: &str,
    ) {
        let edge_group_id = match parse_edge_group_id(edge_group_id_str) {
            Some(edge_group_id) => edge_group_id,
            None => return,
        };
        let thing_id_new = match parse_thing_id(thing_id_new_str) {
            Some(thing_id) => thing_id,
            None => return,
        };

        let edge_group = match target {
            MapTarget::Dependencies => input_diagram.thing_dependencies.get_mut(&edge_group_id),
            MapTarget::Interactions => input_diagram.thing_interactions.get_mut(&edge_group_id),
        };
        if let Some(edge_group) = edge_group
            && idx < edge_group.things.len()
        {
            edge_group.things[idx] = thing_id_new;
        }
    }

    /// Removes a thing from an edge group by index.
    pub fn edge_thing_remove(
        input_diagram: &mut InputDiagram<'static>,
        target: MapTarget,
        edge_group_id_str: &str,
        idx: usize,
    ) {
        let edge_group_id = match parse_edge_group_id(edge_group_id_str) {
            Some(edge_group_id) => edge_group_id,
            None => return,
        };

        let edge_group = match target {
            MapTarget::Dependencies => input_diagram.thing_dependencies.get_mut(&edge_group_id),
            MapTarget::Interactions => input_diagram.thing_interactions.get_mut(&edge_group_id),
        };
        if let Some(edge_group) = edge_group
            && idx < edge_group.things.len()
        {
            edge_group.things.remove(idx);
        }
    }

    /// Moves a thing within an edge group from one index to another.
    pub fn edge_thing_move(
        input_diagram: &mut InputDiagram<'static>,
        target: MapTarget,
        edge_group_id_str: &str,
        from: usize,
        to: usize,
    ) {
        let edge_group_id = match parse_edge_group_id(edge_group_id_str) {
            Some(edge_group_id) => edge_group_id,
            None => return,
        };

        let edge_group = match target {
            MapTarget::Dependencies => input_diagram.thing_dependencies.get_mut(&edge_group_id),
            MapTarget::Interactions => input_diagram.thing_interactions.get_mut(&edge_group_id),
        };
        if let Some(edge_group) = edge_group {
            let len = edge_group.things.len();
            if from < len && to < len && from != to {
                let thing = edge_group.things.remove(from);
                edge_group.things.insert(to, thing);
            }
        }
    }

    /// Adds a thing to an edge group, using the first existing thing ID as a
    /// placeholder.
    pub fn edge_thing_add(
        input_diagram: &mut InputDiagram<'static>,
        target: MapTarget,
        edge_group_id_str: &str,
    ) {
        let edge_group_id = match parse_edge_group_id(edge_group_id_str) {
            Some(edge_group_id) => edge_group_id,
            None => return,
        };

        // Find any existing thing ID as a placeholder.
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

        let edge_group = match target {
            MapTarget::Dependencies => input_diagram.thing_dependencies.get_mut(&edge_group_id),
            MapTarget::Interactions => input_diagram.thing_interactions.get_mut(&edge_group_id),
        };
        if let Some(edge_group) = edge_group {
            edge_group.things.push(thing_id_new);
        }
    }
}
