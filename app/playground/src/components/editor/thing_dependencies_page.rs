//! Thing Dependencies editor page.
//!
//! Allows editing `thing_dependencies` -- a map from `EdgeGroupId` to
//! `EdgeGroup`, where each group has an `EdgeKind` (Cyclic / Sequence /
//! Symmetric) and a list of `ThingId`s.
//!
//! Also provides `ThingInteractionsPage` which shares the same card component
//! and mutation helpers via [`MapTarget`].

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::{
    input_model::{
        edge::{EdgeGroup, EdgeKind},
        thing::ThingId,
        InputDiagram,
    },
    model_common::edge::EdgeGroupId,
};

use crate::components::editor::{
    common::{
        id_rename_in_input_diagram, parse_edge_group_id, parse_thing_id, ADD_BTN, CARD_CLASS,
        INPUT_CLASS, LABEL_CLASS, REMOVE_BTN, ROW_CLASS_SIMPLE, SECTION_HEADING, SELECT_CLASS,
    },
    datalists::list_ids,
};

/// Serialised snapshot of one edge group entry for rendering.
#[derive(Clone, PartialEq)]
struct EdgeGroupEntry {
    edge_group_id: String,
    edge_kind: EdgeKind,
    things: Vec<ThingId<'static>>,
}

/// The **Thing Dependencies** editor page.
#[component]
pub fn ThingDependenciesPage(input_diagram: Signal<InputDiagram<'static>>) -> Element {
    let diagram = input_diagram.read();

    let entries: Vec<EdgeGroupEntry> = diagram
        .thing_dependencies
        .iter()
        .map(|(edge_group_id, edge_group)| EdgeGroupEntry {
            edge_group_id: edge_group_id.as_str().to_owned(),
            edge_kind: edge_group.kind,
            things: edge_group
                .things
                .iter()
                .map(|thing_id| ThingId::from(thing_id.clone().into_inner().into_static()))
                .collect(),
        })
        .collect();

    drop(diagram);

    rsx! {
        div {
            class: "flex flex-col gap-2",

            h3 { class: SECTION_HEADING, "Thing Dependencies" }
            p {
                class: "text-xs text-gray-500 mb-1",
                "Static relationships between things. Each edge group has an ID, a kind (cyclic / sequence / symmetric), and a list of things."
            }

            for entry in entries.iter() {
                {
                    let entry = entry.clone();
                    rsx! {
                        EdgeGroupCard {
                            key: "{entry.edge_group_id}",
                            input_diagram,
                            entry,
                            target: MapTarget::Dependencies,
                        }
                    }
                }
            }

            div {
                class: ADD_BTN,
                onclick: move |_| {
                    EdgeGroupCardOps::edge_group_add(input_diagram, MapTarget::Dependencies);
                },
                "+ Add dependency edge group"
            }
        }
    }
}

/// The **Thing Interactions** editor page.
///
/// Structurally identical to dependencies but operates on
/// `thing_interactions`.
#[component]
pub fn ThingInteractionsPage(input_diagram: Signal<InputDiagram<'static>>) -> Element {
    let diagram = input_diagram.read();

    let entries: Vec<EdgeGroupEntry> = diagram
        .thing_interactions
        .iter()
        .map(|(edge_group_id, edge_group)| EdgeGroupEntry {
            edge_group_id: edge_group_id.as_str().to_owned(),
            edge_kind: edge_group.kind,
            things: edge_group
                .things
                .iter()
                .map(|thing_id| ThingId::from(thing_id.clone().into_inner().into_static()))
                .collect(),
        })
        .collect();

    drop(diagram);

    rsx! {
        div {
            class: "flex flex-col gap-2",

            h3 { class: SECTION_HEADING, "Thing Interactions" }
            p {
                class: "text-xs text-gray-500 mb-1",
                "Runtime communication between things. Same structure as dependencies but represents runtime interactions."
            }

            for entry in entries.iter() {
                {
                    let entry = entry.clone();
                    rsx! {
                        EdgeGroupCard {
                            key: "{entry.edge_group_id}",
                            input_diagram,
                            entry,
                            target: MapTarget::Interactions,
                        }
                    }
                }
            }

            div {
                class: ADD_BTN,
                onclick: move |_| {
                    EdgeGroupCardOps::edge_group_add(input_diagram, MapTarget::Interactions);
                },
                "+ Add interaction edge group"
            }
        }
    }
}

// === Shared types === //

/// Which map inside `InputDiagram` we are editing.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum MapTarget {
    Dependencies,
    Interactions,
}

// === Edge group card component === //

/// A card for a single edge group (used by both dependencies and interactions).
#[component]
fn EdgeGroupCard(
    input_diagram: Signal<InputDiagram<'static>>,
    entry: EdgeGroupEntry,
    target: MapTarget,
) -> Element {
    let edge_group_id = entry.edge_group_id.clone();
    let edge_kind = entry.edge_kind;
    let things = entry.things.clone();

    rsx! {
        div {
            class: CARD_CLASS,

            // === EdgeGroupId + Remove === //
            div {
                class: ROW_CLASS_SIMPLE,

                input {
                    class: INPUT_CLASS,
                    style: "max-width:16rem",
                    list: list_ids::EDGE_GROUP_IDS,
                    placeholder: "edge_group_id",
                    value: "{edge_group_id}",
                    onchange: {
                        let edge_group_id_old = edge_group_id.clone();
                        move |evt: dioxus::events::FormEvent| {
                            let edge_group_id_new = evt.value();
                            EdgeGroupCardOps::edge_group_rename(input_diagram, &edge_group_id_old, &edge_group_id_new);
                        }
                    },
                }

                span {
                    class: REMOVE_BTN,
                    onclick: {
                        let edge_group_id = edge_group_id.clone();
                        move |_| {
                            EdgeGroupCardOps::edge_group_remove(input_diagram, target, &edge_group_id);
                        }
                    },
                    "✕ Remove"
                }
            }

            // === kind === //
            div {
                class: "flex flex-col items-start gap-1 pl-4",

                label { class: LABEL_CLASS, "kind" }

                select {
                    class: SELECT_CLASS,
                    value: "{edge_kind}",
                    onchange: {
                        let edge_group_id = edge_group_id.clone();
                        let current_things = things.clone();
                        move |evt: dioxus::events::FormEvent| {
                            let kind_str = evt.value();
                            if let Ok(edge_kind_new) = kind_str.parse::<EdgeKind>() {
                                EdgeGroupCardOps::edge_kind_change(
                                    input_diagram,
                                    target,
                                    &edge_group_id,
                                    edge_kind_new,
                                    &current_things,
                                );
                            }
                        }
                    },
                    option { value: "cyclic", "Cyclic" }
                    option { value: "sequence", "Sequence" }
                    option { value: "symmetric", "Symmetric" }
                }
            }

            // === things === //
            div {
                class: "flex flex-col gap-1 pl-4",

                label { class: LABEL_CLASS, "things" }

                for (idx, thing_id) in things.iter().enumerate() {
                    {
                        let thing_id = thing_id.clone();
                        let edge_group_id = edge_group_id.clone();
                        rsx! {
                            div {
                                key: "{edge_group_id}_{idx}",
                                class: ROW_CLASS_SIMPLE,

                                span {
                                    class: "text-xs text-gray-500 w-6 text-right",
                                    "{idx}."
                                }

                                input {
                                    class: INPUT_CLASS,
                                    style: "max-width:14rem",
                                    list: list_ids::THING_IDS,
                                    placeholder: "thing_id",
                                    value: "{thing_id}",
                                    onchange: {
                                        let edge_group_id = edge_group_id.clone();
                                        move |evt: dioxus::events::FormEvent| {
                                            let thing_id_new = evt.value();
                                            EdgeGroupCardOps::edge_thing_update(
                                                input_diagram,
                                                target,
                                                &edge_group_id,
                                                idx,
                                                &thing_id_new,
                                            );
                                        }
                                    },
                                }

                                span {
                                    class: REMOVE_BTN,
                                    onclick: {
                                        let edge_group_id = edge_group_id.clone();
                                        move |_| {
                                            EdgeGroupCardOps::edge_thing_remove(
                                                input_diagram,
                                                target,
                                                &edge_group_id,
                                                idx,
                                            );
                                        }
                                    },
                                    "✕"
                                }
                            }
                        }
                    }
                }

                div {
                    class: ADD_BTN,
                    onclick: {
                        let edge_group_id = edge_group_id.clone();
                        move |_| {
                            EdgeGroupCardOps::edge_thing_add(input_diagram, target, &edge_group_id);
                        }
                    },
                    "+ Add thing"
                }
            }
        }
    }
}

// === EdgeGroupCard helpers and mutation methods === //

/// Mutation operations for the edge group card component.
///
/// Grouped here so that related functions are discoverable when sorted by
/// name, per the project's `noun_verb` naming convention.
struct EdgeGroupCardOps;

impl EdgeGroupCardOps {
    // === Map-target helpers === //

    /// Sets the [`EdgeGroup`] for a given `EdgeGroupId` in the target map.
    fn edge_group_set(
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
    fn edge_group_remove_by_id(
        input_diagram: &mut InputDiagram<'static>,
        target: MapTarget,
        edge_group_id: &EdgeGroupId<'static>,
    ) {
        match target {
            MapTarget::Dependencies => {
                input_diagram.thing_dependencies.swap_remove(edge_group_id);
            }
            MapTarget::Interactions => {
                input_diagram.thing_interactions.swap_remove(edge_group_id);
            }
        }
    }

    /// Returns the number of edge groups in the target map.
    fn edge_group_count(input_diagram: &InputDiagram<'static>, target: MapTarget) -> usize {
        match target {
            MapTarget::Dependencies => input_diagram.thing_dependencies.len(),
            MapTarget::Interactions => input_diagram.thing_interactions.len(),
        }
    }

    /// Returns whether the target map contains the given edge group ID.
    fn edge_group_contains(
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
    fn edge_group_add(mut input_diagram: Signal<InputDiagram<'static>>, target: MapTarget) {
        let mut n = Self::edge_group_count(&input_diagram.read(), target);
        loop {
            let candidate = format!("edge_{n}");
            if let Some(edge_group_id) = parse_edge_group_id(&candidate)
                && !Self::edge_group_contains(&input_diagram.read(), target, &edge_group_id)
            {
                Self::edge_group_set(
                    &mut input_diagram.write(),
                    target,
                    &edge_group_id,
                    EdgeGroup::new(EdgeKind::Sequence, Vec::new()),
                );
                break;
            }
            n += 1;
        }
    }

    /// Removes an edge group from the target map.
    fn edge_group_remove(
        mut input_diagram: Signal<InputDiagram<'static>>,
        target: MapTarget,
        edge_group_id_str: &str,
    ) {
        if let Some(edge_group_id) = parse_edge_group_id(edge_group_id_str) {
            Self::edge_group_remove_by_id(&mut input_diagram.write(), target, &edge_group_id);
        }
    }

    /// Renames an edge group across all maps in the [`InputDiagram`].
    fn edge_group_rename(
        mut input_diagram: Signal<InputDiagram<'static>>,
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

        let mut input_diagram_ref = input_diagram.write();

        // thing_dependencies: rename EdgeGroupId key.
        if let Some(index) = input_diagram_ref
            .thing_dependencies
            .get_index_of(&edge_group_id_old)
        {
            let _result = input_diagram_ref
                .thing_dependencies
                .replace_index(index, edge_group_id_new.clone());
        }

        // thing_interactions: rename EdgeGroupId key.
        if let Some(index) = input_diagram_ref
            .thing_interactions
            .get_index_of(&edge_group_id_old)
        {
            let _result = input_diagram_ref
                .thing_interactions
                .replace_index(index, edge_group_id_new.clone());
        }

        // processes: rename EdgeGroupId in step_thing_interactions values.
        input_diagram_ref
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
        id_rename_in_input_diagram(&mut input_diagram_ref, &id_old, &id_new);
    }

    /// Changes the edge kind (cyclic / sequence / symmetric) for an edge
    /// group, preserving the current thing list.
    fn edge_kind_change(
        mut input_diagram: Signal<InputDiagram<'static>>,
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
        Self::edge_group_set(
            &mut input_diagram.write(),
            target,
            &edge_group_id,
            edge_group,
        );
    }

    /// Updates a single thing within an edge group at the given index.
    fn edge_thing_update(
        mut input_diagram: Signal<InputDiagram<'static>>,
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

        let mut input_diagram = input_diagram.write();
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
    fn edge_thing_remove(
        mut input_diagram: Signal<InputDiagram<'static>>,
        target: MapTarget,
        edge_group_id_str: &str,
        idx: usize,
    ) {
        let edge_group_id = match parse_edge_group_id(edge_group_id_str) {
            Some(edge_group_id) => edge_group_id,
            None => return,
        };

        let mut input_diagram = input_diagram.write();
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

    /// Adds a thing to an edge group, using the first existing thing ID as a
    /// placeholder.
    fn edge_thing_add(
        mut input_diagram: Signal<InputDiagram<'static>>,
        target: MapTarget,
        edge_group_id_str: &str,
    ) {
        let edge_group_id = match parse_edge_group_id(edge_group_id_str) {
            Some(edge_group_id) => edge_group_id,
            None => return,
        };

        // Find any existing thing ID as a placeholder.
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
        let edge_group = match target {
            MapTarget::Dependencies => input_diagram.thing_dependencies.get_mut(&edge_group_id),
            MapTarget::Interactions => input_diagram.thing_interactions.get_mut(&edge_group_id),
        };
        if let Some(edge_group) = edge_group {
            edge_group.things.push(thing_id_new);
        }
    }
}
