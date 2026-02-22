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
    common::{parse_tag_id, parse_thing_id, rename_id_in_theme_styles},
    datalists::list_ids,
};

/// CSS classes shared by section headings.
const SECTION_HEADING: &str = "text-sm font-bold text-gray-300 mt-4 mb-1";

/// CSS classes for a card-like container.
const CARD_CLASS: &str = "\
    rounded-lg \
    border \
    border-gray-700 \
    bg-gray-900 \
    p-3 \
    mb-2 \
    flex \
    flex-col \
    gap-2\
";

/// CSS classes for text inputs.
const INPUT_CLASS: &str = "\
    flex-1 \
    rounded \
    border \
    border-gray-600 \
    bg-gray-800 \
    text-gray-200 \
    px-2 py-1 \
    text-sm \
    font-mono \
    focus:border-blue-400 \
    focus:outline-none\
";

/// CSS classes for the small "remove" button.
const REMOVE_BTN: &str = "\
    text-red-400 \
    hover:text-red-300 \
    text-xs \
    cursor-pointer \
    px-1\
";

/// CSS classes for the "add" button.
const ADD_BTN: &str = "\
    mt-1 \
    text-sm \
    text-blue-400 \
    hover:text-blue-300 \
    cursor-pointer \
    select-none\
";

/// Row-level flex layout.
const ROW_CLASS: &str = "flex flex-row gap-2 items-center";

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
        .map(|(id, name)| (id.as_str().to_owned(), name.clone()))
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

            for (id, name) in tag_entries.iter() {
                {
                    let id = id.clone();
                    let name = name.clone();
                    rsx! {
                        TagNameRow {
                            key: "{id}",
                            input_diagram,
                            tag_id: id,
                            tag_name: name,
                        }
                    }
                }
            }

            div {
                class: ADD_BTN,
                onclick: move |_| {
                    add_tag(input_diagram);
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
                    add_tag_things_entry(input_diagram);
                },
                "+ Add tag → things mapping"
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tag name row
// ---------------------------------------------------------------------------

/// A single editable row for a tag name (TagId -> display label).
#[component]
fn TagNameRow(
    input_diagram: Signal<InputDiagram<'static>>,
    tag_id: String,
    tag_name: String,
) -> Element {
    rsx! {
        div {
            class: ROW_CLASS,

            // TagId input
            input {
                class: INPUT_CLASS,
                style: "max-width:14rem",
                list: list_ids::TAG_IDS,
                placeholder: "tag_id",
                value: "{tag_id}",
                onchange: {
                    let old_id = tag_id.clone();
                    move |evt: dioxus::events::FormEvent| {
                        let new_id_str = evt.value();
                        rename_tag(input_diagram, &old_id, &new_id_str);
                    }
                },
            }

            // Display name input
            input {
                class: INPUT_CLASS,
                placeholder: "Display name",
                value: "{tag_name}",
                oninput: {
                    let id = tag_id.clone();
                    move |evt: dioxus::events::FormEvent| {
                        let new_name = evt.value();
                        update_tag_name(input_diagram, &id, &new_name);
                    }
                },
            }

            // Remove button
            span {
                class: REMOVE_BTN,
                onclick: {
                    let id = tag_id.clone();
                    move |_| {
                        remove_tag(input_diagram, &id);
                    }
                },
                "✕"
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tag things card
// ---------------------------------------------------------------------------

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
                class: ROW_CLASS,

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
                        let old_id = tag_id.clone();
                        let current_things = things.clone();
                        move |evt: dioxus::events::FormEvent| {
                            rename_tag_things_entry(input_diagram, &old_id, &evt.value(), &current_things);
                        }
                    },
                }

                span {
                    class: REMOVE_BTN,
                    onclick: {
                        let id = tag_id.clone();
                        move |_| {
                            remove_tag_things_entry(input_diagram, &id);
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
                        let tid = tag_id.clone();
                        rsx! {
                            div {
                                key: "{tid}_{idx}",
                                class: ROW_CLASS,

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
                                        let tid2 = tid.clone();
                                        move |evt: dioxus::events::FormEvent| {
                                            update_thing_in_tag(input_diagram, &tid2, idx, &evt.value());
                                        }
                                    },
                                }

                                span {
                                    class: REMOVE_BTN,
                                    onclick: {
                                        let tid2 = tid.clone();
                                        move |_| {
                                            remove_thing_from_tag(input_diagram, &tid2, idx);
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
                        let tid = tag_id.clone();
                        move |_| {
                            add_thing_to_tag(input_diagram, &tid);
                        }
                    },
                    "+ Add thing"
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Mutation helpers: tag names
// ---------------------------------------------------------------------------

fn add_tag(mut diag: Signal<InputDiagram<'static>>) {
    let mut n = diag.read().tags.len();
    loop {
        let candidate = format!("tag_{n}");
        if let Some(tid) = parse_tag_id(&candidate) {
            if !diag.read().tags.contains_key(&tid) {
                diag.write().tags.insert(tid, String::new());
                break;
            }
        }
        n += 1;
    }
}

fn remove_tag(mut diag: Signal<InputDiagram<'static>>, id: &str) {
    if let Some(tid) = parse_tag_id(id) {
        diag.write().tags.swap_remove(&tid);
    }
}

fn rename_tag(mut diag: Signal<InputDiagram<'static>>, old_id: &str, new_id: &str) {
    if old_id == new_id {
        return;
    }
    let old = match parse_tag_id(old_id) {
        Some(id) => id,
        None => return,
    };
    let new = match parse_tag_id(new_id) {
        Some(id) => id,
        None => return,
    };

    let mut input_diagram = diag.write();
    let InputDiagram {
        things: _,
        thing_copy_text: _,
        thing_hierarchy: _,
        thing_dependencies: _,
        thing_interactions: _,
        processes: _,
        tags,
        tag_things,
        entity_descs,
        entity_tooltips,
        entity_types,
        theme_default,
        theme_types_styles,
        theme_thing_dependencies_styles,
        theme_tag_things_focus,
        css: _,
    } = &mut *input_diagram;

    // tags: rename TagId key.
    if let Some(index) = tags.get_index_of(&old) {
        let _result = tags.replace_index(index, new.clone());
    }

    // tag_things: rename TagId key.
    if let Some(index) = tag_things.get_index_of(&old) {
        let _result = tag_things.replace_index(index, new.clone());
    }

    // entity_descs / entity_tooltips / entity_types: keys are Id, which
    // may refer to a TagId.
    let id_old = old.clone().into_inner();
    let id_new = new.clone().into_inner();
    if let Some(index) = entity_descs.get_index_of(&id_old) {
        let _result = entity_descs.replace_index(index, id_new.clone());
    }
    if let Some(index) = entity_tooltips.get_index_of(&id_old) {
        let _result = entity_tooltips.replace_index(index, id_new.clone());
    }
    if let Some(index) = entity_types.get_index_of(&id_old) {
        let _result = entity_types.replace_index(index, id_new.clone());
    }

    // theme_default: rename in base_styles and process_step_selected_styles.
    rename_id_in_theme_styles(&mut theme_default.base_styles, &id_old, &id_new);
    rename_id_in_theme_styles(
        &mut theme_default.process_step_selected_styles,
        &id_old,
        &id_new,
    );

    // theme_types_styles: rename in each ThemeStyles value.
    theme_types_styles.values_mut().for_each(|theme_styles| {
        rename_id_in_theme_styles(theme_styles, &id_old, &id_new);
    });

    // theme_thing_dependencies_styles: rename in both ThemeStyles fields.
    rename_id_in_theme_styles(
        &mut theme_thing_dependencies_styles.things_included_styles,
        &id_old,
        &id_new,
    );
    rename_id_in_theme_styles(
        &mut theme_thing_dependencies_styles.things_excluded_styles,
        &id_old,
        &id_new,
    );

    // theme_tag_things_focus: rename TagIdOrDefaults::Custom key and
    // rename in each ThemeStyles value.
    let tag_key_old = TagIdOrDefaults::Custom(old.clone());
    if let Some(index) = theme_tag_things_focus.get_index_of(&tag_key_old) {
        let tag_key_new = TagIdOrDefaults::Custom(new.clone());
        let _result = theme_tag_things_focus.replace_index(index, tag_key_new);
    }
    theme_tag_things_focus
        .values_mut()
        .for_each(|theme_styles| {
            rename_id_in_theme_styles(theme_styles, &id_old, &id_new);
        });
}

fn update_tag_name(mut diag: Signal<InputDiagram<'static>>, id: &str, name: &str) {
    if let Some(tid) = parse_tag_id(id) {
        if let Some(entry) = diag.write().tags.get_mut(&tid) {
            *entry = name.to_owned();
        }
    }
}

// ---------------------------------------------------------------------------
// Mutation helpers: tag things
// ---------------------------------------------------------------------------

fn add_tag_things_entry(mut diag: Signal<InputDiagram<'static>>) {
    // Pick a tag that doesn't already have an entry, or generate a placeholder.
    let d = diag.read();
    let tag_id = d
        .tags
        .keys()
        .find(|tid| !d.tag_things.contains_key(*tid))
        .cloned();

    match tag_id {
        Some(tid) => {
            drop(d);
            diag.write().tag_things.insert(tid, Set::new());
        }
        None => {
            let mut n = d.tag_things.len();
            loop {
                let candidate = format!("tag_{n}");
                if let Some(tid) = parse_tag_id(&candidate) {
                    if !d.tag_things.contains_key(&tid) {
                        drop(d);
                        diag.write().tag_things.insert(tid, Set::new());
                        break;
                    }
                }
                n += 1;
            }
        }
    }
}

fn remove_tag_things_entry(mut diag: Signal<InputDiagram<'static>>, tag_id: &str) {
    if let Some(tid) = parse_tag_id(tag_id) {
        diag.write().tag_things.swap_remove(&tid);
    }
}

fn rename_tag_things_entry(
    mut diag: Signal<InputDiagram<'static>>,
    old_id: &str,
    new_id: &str,
    current_things: &[String],
) {
    if old_id == new_id {
        return;
    }
    let old = match parse_tag_id(old_id) {
        Some(id) => id,
        None => return,
    };
    let new = match parse_tag_id(new_id) {
        Some(id) => id,
        None => return,
    };
    let things: Set<ThingId<'static>> = current_things
        .iter()
        .filter_map(|s| parse_thing_id(s))
        .collect();
    let mut d = diag.write();
    d.tag_things.swap_remove(&old);
    d.tag_things.insert(new, things);
}

fn update_thing_in_tag(
    mut diag: Signal<InputDiagram<'static>>,
    tag_id: &str,
    idx: usize,
    new_thing_str: &str,
) {
    let tid = match parse_tag_id(tag_id) {
        Some(id) => id,
        None => return,
    };
    let new_thing = match parse_thing_id(new_thing_str) {
        Some(t) => t,
        None => return,
    };

    let mut d = diag.write();
    if let Some(things) = d.tag_things.get_mut(&tid) {
        // `Set` (IndexSet) does not support indexed mutation directly.
        // Rebuild the set with the replacement at the given position.
        let mut new_set = Set::with_capacity(things.len());
        for (i, existing) in things.iter().enumerate() {
            if i == idx {
                new_set.insert(new_thing.clone());
            } else {
                new_set.insert(existing.clone());
            }
        }
        *things = new_set;
    }
}

fn remove_thing_from_tag(mut diag: Signal<InputDiagram<'static>>, tag_id: &str, idx: usize) {
    let tid = match parse_tag_id(tag_id) {
        Some(id) => id,
        None => return,
    };

    let mut d = diag.write();
    if let Some(things) = d.tag_things.get_mut(&tid) {
        // IndexSet supports `swap_remove_index`.
        if idx < things.len() {
            things.swap_remove_index(idx);
        }
    }
}

fn add_thing_to_tag(mut diag: Signal<InputDiagram<'static>>, tag_id: &str) {
    let tid = match parse_tag_id(tag_id) {
        Some(id) => id,
        None => return,
    };

    // Pick the first thing ID as a placeholder.
    let placeholder = {
        let d = diag.read();
        d.things
            .keys()
            .next()
            .map(|t| t.as_str().to_owned())
            .unwrap_or_else(|| "thing_0".to_owned())
    };
    let new_thing = match parse_thing_id(&placeholder) {
        Some(t) => t,
        None => return,
    };

    let mut d = diag.write();
    if let Some(things) = d.tag_things.get_mut(&tid) {
        things.insert(new_thing);
    }
}
