//! Tags editor page.
//!
//! Allows editing:
//! - `tags`: `TagId` -> display name
//! - `tag_things`: `TagId` -> `Set<ThingId>`

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::{
    input_model::{theme::TagIdOrDefaults, thing::ThingId, InputDiagram},
    model_common::Set,
};

use crate::components::editor::{
    common::{
        id_rename_in_input_diagram, parse_tag_id, parse_thing_id, ADD_BTN, CARD_CLASS, INPUT_CLASS,
        REMOVE_BTN, ROW_CLASS_SIMPLE, SECTION_HEADING,
    },
    datalists::list_ids,
};

/// The **Tags** editor page.
///
/// Provides two sections:
/// 1. Tag Names: map of `TagId` to display label.
/// 2. Tag Things: map of `TagId` to set of `ThingId`s associated with the tag.
#[component]
pub fn TagsPage(input_diagram: Signal<InputDiagram<'static>>) -> Element {
    let diagram = input_diagram.read();

    // Snapshot tag names.
    let tag_entries: Vec<(String, String)> = diagram
        .tags
        .iter()
        .map(|(tag_id, name)| (tag_id.as_str().to_owned(), name.clone()))
        .collect();

    // Snapshot tag -> things associations.
    let tag_things_entries: Vec<(String, Vec<String>)> = diagram
        .tag_things
        .iter()
        .map(|(tag_id, thing_ids)| {
            let things: Vec<String> = thing_ids.iter().map(|t| t.as_str().to_owned()).collect();
            (tag_id.as_str().to_owned(), things)
        })
        .collect();

    drop(diagram);

    rsx! {
        div {
            class: "flex flex-col gap-2",

            // ── Tag Names ────────────────────────────────────────────
            h3 { class: SECTION_HEADING, "Tag Names" }
            p {
                class: "text-xs text-gray-500 mb-1",
                "Map of TagId to display label."
            }

            for (tag_id, tag_name) in tag_entries.iter() {
                {
                    let tag_id = tag_id.clone();
                    let tag_name = tag_name.clone();
                    rsx! {
                        TagNameRow {
                            key: "{tag_id}",
                            input_diagram,
                            tag_id,
                            tag_name,
                        }
                    }
                }
            }

            div {
                class: ADD_BTN,
                onclick: move |_| {
                    TagsPageOps::tag_add(input_diagram);
                },
                "+ Add tag"
            }

            // ── Tag Things ───────────────────────────────────────────
            h3 { class: SECTION_HEADING, "Tag → Things" }
            p {
                class: "text-xs text-gray-500 mb-1",
                "Things highlighted when each tag is focused."
            }

            for (tag_id, things) in tag_things_entries.iter() {
                {
                    let tag_id = tag_id.clone();
                    let things = things.clone();
                    rsx! {
                        TagThingsCard {
                            key: "tt_{tag_id}",
                            input_diagram,
                            tag_id,
                            things,
                        }
                    }
                }
            }

            div {
                class: ADD_BTN,
                onclick: move |_| {
                    TagsPageOps::tag_things_entry_add(input_diagram);
                },
                "+ Add tag → things mapping"
            }
        }
    }
}

// ===========================================================================
// TagsPage mutation helpers
// ===========================================================================

/// Mutation operations for the Tags editor page.
///
/// Grouped here so that related functions are discoverable when sorted by
/// name, per the project's `noun_verb` naming convention.
struct TagsPageOps;

impl TagsPageOps {
    // ── Tag name helpers ─────────────────────────────────────────────

    /// Adds a new tag with a unique placeholder TagId.
    fn tag_add(mut input_diagram: Signal<InputDiagram<'static>>) {
        let mut n = input_diagram.read().tags.len();
        loop {
            let candidate = format!("tag_{n}");
            if let Some(tag_id) = parse_tag_id(&candidate)
                && !input_diagram.read().tags.contains_key(&tag_id) {
                    input_diagram.write().tags.insert(tag_id, String::new());
                    break;
                }
            n += 1;
        }
    }

    /// Removes a tag from the `tags` map.
    fn tag_remove(mut input_diagram: Signal<InputDiagram<'static>>, tag_id_str: &str) {
        if let Some(tag_id) = parse_tag_id(tag_id_str) {
            input_diagram.write().tags.swap_remove(&tag_id);
        }
    }

    /// Renames a tag across all maps in the [`InputDiagram`].
    fn tag_rename(
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
    fn tag_name_update(
        mut input_diagram: Signal<InputDiagram<'static>>,
        tag_id_str: &str,
        name: &str,
    ) {
        if let Some(tag_id) = parse_tag_id(tag_id_str)
            && let Some(entry) = input_diagram.write().tags.get_mut(&tag_id) {
                *entry = name.to_owned();
            }
    }

    // ── Tag things helpers ───────────────────────────────────────────

    /// Adds a new tag→things entry, picking an unmapped tag or generating a
    /// placeholder.
    fn tag_things_entry_add(mut input_diagram: Signal<InputDiagram<'static>>) {
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
                        && !input_diagram_read.tag_things.contains_key(&tag_id) {
                            drop(input_diagram_read);
                            input_diagram.write().tag_things.insert(tag_id, Set::new());
                            break;
                        }
                    n += 1;
                }
            }
        }
    }

    /// Removes a tag→things entry.
    fn tag_things_entry_remove(mut input_diagram: Signal<InputDiagram<'static>>, tag_id_str: &str) {
        if let Some(tag_id) = parse_tag_id(tag_id_str) {
            input_diagram.write().tag_things.swap_remove(&tag_id);
        }
    }

    /// Renames the key of a tag→things entry.
    fn tag_things_entry_rename(
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
        input_diagram.tag_things.swap_remove(&tag_id_old);
        input_diagram.tag_things.insert(tag_id_new, things);
    }

    /// Updates a single thing within a tag's thing set at the given index.
    fn tag_things_thing_update(
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
    fn tag_things_thing_remove(
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
            && idx < things.len() {
                things.swap_remove_index(idx);
            }
    }

    /// Adds a thing to a tag's thing set.
    fn tag_things_thing_add(mut input_diagram: Signal<InputDiagram<'static>>, tag_id_str: &str) {
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

// ===========================================================================
// Helper components
// ===========================================================================

/// A single editable row for a tag name (TagId -> display label).
#[component]
fn TagNameRow(
    input_diagram: Signal<InputDiagram<'static>>,
    tag_id: String,
    tag_name: String,
) -> Element {
    rsx! {
        div {
            class: ROW_CLASS_SIMPLE,

            // TagId input
            input {
                class: INPUT_CLASS,
                style: "max-width:14rem",
                list: list_ids::TAG_IDS,
                placeholder: "tag_id",
                value: "{tag_id}",
                onchange: {
                    let tag_id_old = tag_id.clone();
                    move |evt: dioxus::events::FormEvent| {
                        let tag_id_new = evt.value();
                        TagsPageOps::tag_rename(input_diagram, &tag_id_old, &tag_id_new);
                    }
                },
            }

            // Display name input
            input {
                class: INPUT_CLASS,
                placeholder: "Display name",
                value: "{tag_name}",
                oninput: {
                    let tag_id = tag_id.clone();
                    move |evt: dioxus::events::FormEvent| {
                        let name = evt.value();
                        TagsPageOps::tag_name_update(input_diagram, &tag_id, &name);
                    }
                },
            }

            // Remove button
            span {
                class: REMOVE_BTN,
                onclick: {
                    let tag_id = tag_id.clone();
                    move |_| {
                        TagsPageOps::tag_remove(input_diagram, &tag_id);
                    }
                },
                "✕"
            }
        }
    }
}

/// A card for one tag's associated things.
#[component]
fn TagThingsCard(
    input_diagram: Signal<InputDiagram<'static>>,
    tag_id: String,
    things: Vec<String>,
) -> Element {
    rsx! {
        div {
            class: CARD_CLASS,

            // ── Header: TagId + Remove ───────────────────────────────
            div {
                class: ROW_CLASS_SIMPLE,

                label {
                    class: "text-xs text-gray-500 w-12",
                    "Tag"
                }

                input {
                    class: INPUT_CLASS,
                    style: "max-width:14rem",
                    list: list_ids::TAG_IDS,
                    placeholder: "tag_id",
                    value: "{tag_id}",
                    onchange: {
                        let tag_id_old = tag_id.clone();
                        let current_things = things.clone();
                        move |evt: dioxus::events::FormEvent| {
                            TagsPageOps::tag_things_entry_rename(
                                input_diagram,
                                &tag_id_old,
                                &evt.value(),
                                &current_things,
                            );
                        }
                    },
                }

                span {
                    class: REMOVE_BTN,
                    onclick: {
                        let tag_id = tag_id.clone();
                        move |_| {
                            TagsPageOps::tag_things_entry_remove(input_diagram, &tag_id);
                        }
                    },
                    "✕ Remove"
                }
            }

            // ── Thing list ───────────────────────────────────────────
            div {
                class: "flex flex-col gap-1 pl-4",

                for (idx, thing_id) in things.iter().enumerate() {
                    {
                        let thing_id = thing_id.clone();
                        let tag_id = tag_id.clone();
                        rsx! {
                            div {
                                key: "{tag_id}_{idx}",
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
                                        let tag_id = tag_id.clone();
                                        move |evt: dioxus::events::FormEvent| {
                                            TagsPageOps::tag_things_thing_update(
                                                input_diagram,
                                                &tag_id,
                                                idx,
                                                &evt.value(),
                                            );
                                        }
                                    },
                                }

                                span {
                                    class: REMOVE_BTN,
                                    onclick: {
                                        let tag_id = tag_id.clone();
                                        move |_| {
                                            TagsPageOps::tag_things_thing_remove(
                                                input_diagram,
                                                &tag_id,
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
                        let tag_id = tag_id.clone();
                        move |_| {
                            TagsPageOps::tag_things_thing_add(input_diagram, &tag_id);
                        }
                    },
                    "+ Add thing"
                }
            }
        }
    }
}
