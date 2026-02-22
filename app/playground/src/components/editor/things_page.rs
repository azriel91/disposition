//! Things editor page.
//!
//! Allows editing:
//! - `things`: `ThingId` -> display name
//! - `thing_copy_text`: `ThingId` -> clipboard text
//! - `entity_descs`: `Id` -> description (filtered to thing IDs here)
//! - `entity_tooltips`: `Id` -> tooltip
//! - `thing_hierarchy`: recursive nesting of things

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::input_model::{edge::EdgeKind, thing::ThingHierarchy, InputDiagram};

use crate::components::editor::{
    common::{parse_id, parse_thing_id, rename_id_in_theme_styles},
    datalists::list_ids,
};

/// CSS classes shared by all section headings inside editor pages.
const SECTION_HEADING: &str = "text-sm font-bold text-gray-300 mt-4 mb-1";

/// CSS classes shared by the outer wrapper of a key-value row.
const ROW_CLASS: &str = "flex flex-row gap-2 items-center mb-1";

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

/// The **Things** editor page.
///
/// Renders editable rows for each `ThingId` in the diagram's `things` map, as
/// well as associated copy-text, descriptions, tooltips, and hierarchy.
#[component]
pub fn ThingsPage(input_diagram: Signal<InputDiagram<'static>>) -> Element {
    let diagram = input_diagram.read();

    // Snapshot current thing keys + values so we can iterate without holding
    // the borrow across the event handlers.
    let thing_entries: Vec<(String, String)> = diagram
        .things
        .iter()
        .map(|(id, name)| (id.as_str().to_owned(), name.clone()))
        .collect();

    let copy_text_entries: Vec<(String, String)> = diagram
        .thing_copy_text
        .iter()
        .map(|(id, text)| (id.as_str().to_owned(), text.clone()))
        .collect();

    let desc_entries: Vec<(String, String)> = diagram
        .entity_descs
        .iter()
        .map(|(id, desc)| (id.as_str().to_owned(), desc.clone()))
        .collect();

    let tooltip_entries: Vec<(String, String)> = diagram
        .entity_tooltips
        .iter()
        .map(|(id, tip)| (id.as_str().to_owned(), tip.clone()))
        .collect();

    // Serialize the current hierarchy to a YAML snippet for a simple textarea
    // editor (hierarchy is recursive and hard to represent with flat inputs).
    let hierarchy_yaml = serde_saphyr::to_string(&diagram.thing_hierarchy)
        .unwrap_or_default()
        .trim()
        .to_owned();

    // Drop the immutable borrow before rendering (we need `input_diagram` for
    // event handlers).
    drop(diagram);

    rsx! {
        div {
            class: "flex flex-col gap-2",

            // ── Thing Names ──────────────────────────────────────────
            h3 { class: SECTION_HEADING, "Thing Names" }
            p {
                class: "text-xs text-gray-500 mb-1",
                "Map of ThingId -> display label."
            }

            for (id, name) in thing_entries.iter() {
                {
                    let id = id.clone();
                    let name = name.clone();
                    rsx! {
                        ThingNameRow {
                            key: "{id}",
                            input_diagram,
                            thing_id: id,
                            thing_name: name,
                        }
                    }
                }
            }

            // Add new thing
            div {
                class: ADD_BTN,
                onclick: move |_| {
                    add_thing_row(input_diagram);
                },
                "+ Add thing"
            }

            // ── Copy Text ────────────────────────────────────────────
            h3 { class: SECTION_HEADING, "Thing Copy Text" }
            p {
                class: "text-xs text-gray-500 mb-1",
                "Optional clipboard text per ThingId (defaults to display label)."
            }

            for (id, text) in copy_text_entries.iter() {
                {
                    let id = id.clone();
                    let text = text.clone();
                    rsx! {
                        KeyValueRow {
                            key: "ct_{id}",
                            input_diagram,
                            entry_id: id,
                            entry_value: text,
                            id_list: list_ids::THING_IDS,
                            on_change: OnChangeTarget::CopyText,
                        }
                    }
                }
            }

            div {
                class: ADD_BTN,
                onclick: move |_| {
                    add_copy_text_row(input_diagram);
                },
                "+ Add copy text"
            }

            // ── Entity Descriptions ──────────────────────────────────
            h3 { class: SECTION_HEADING, "Entity Descriptions" }
            p {
                class: "text-xs text-gray-500 mb-1",
                "Descriptions rendered next to entities in the diagram."
            }

            for (id, desc) in desc_entries.iter() {
                {
                    let id = id.clone();
                    let desc = desc.clone();
                    rsx! {
                        KeyValueRow {
                            key: "desc_{id}",
                            input_diagram,
                            entry_id: id,
                            entry_value: desc,
                            id_list: list_ids::ENTITY_IDS,
                            on_change: OnChangeTarget::EntityDesc,
                        }
                    }
                }
            }

            div {
                class: ADD_BTN,
                onclick: move |_| {
                    add_entity_desc_row(input_diagram);
                },
                "+ Add description"
            }

            // ── Entity Tooltips ──────────────────────────────────────
            h3 { class: SECTION_HEADING, "Entity Tooltips" }
            p {
                class: "text-xs text-gray-500 mb-1",
                "Tooltip text (markdown) shown on hover."
            }

            for (id, tip) in tooltip_entries.iter() {
                {
                    let id = id.clone();
                    let tip = tip.clone();
                    rsx! {
                        KeyValueRow {
                            key: "tip_{id}",
                            input_diagram,
                            entry_id: id,
                            entry_value: tip,
                            id_list: list_ids::ENTITY_IDS,
                            on_change: OnChangeTarget::EntityTooltip,
                        }
                    }
                }
            }

            div {
                class: ADD_BTN,
                onclick: move |_| {
                    add_entity_tooltip_row(input_diagram);
                },
                "+ Add tooltip"
            }

            // ── Thing Hierarchy ──────────────────────────────────────
            h3 { class: SECTION_HEADING, "Thing Hierarchy (YAML)" }
            p {
                class: "text-xs text-gray-500 mb-1",
                "Recursive nesting of things. Edit as YAML."
            }
            textarea {
                class: "\
                    w-full \
                    min-h-24 \
                    rounded \
                    border \
                    border-gray-600 \
                    bg-gray-800 \
                    text-gray-200 \
                    p-2 \
                    font-mono \
                    text-sm \
                    focus:border-blue-400 \
                    focus:outline-none\
                ",
                value: "{hierarchy_yaml}",
                oninput: move |evt| {
                    let text = evt.value();
                    if let Ok(hierarchy) = serde_saphyr::from_str(&text) {
                        input_diagram.write().thing_hierarchy = hierarchy;
                    }
                },
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helper components
// ---------------------------------------------------------------------------

/// A single editable row for a thing name (ThingId -> display label).
#[component]
fn ThingNameRow(
    input_diagram: Signal<InputDiagram<'static>>,
    thing_id: String,
    thing_name: String,
) -> Element {
    rsx! {
        div {
            class: ROW_CLASS,

            // ThingId input
            input {
                class: INPUT_CLASS,
                style: "max-width:14rem",
                list: list_ids::THING_IDS,
                placeholder: "thing_id",
                value: "{thing_id}",
                onchange: {
                    let old_id = thing_id.clone();
                    move |evt: dioxus::events::FormEvent| {
                        let new_id_str = evt.value();
                        rename_thing(input_diagram, &old_id, &new_id_str);
                    }
                },
            }

            // Display name input
            input {
                class: INPUT_CLASS,
                placeholder: "Display name",
                value: "{thing_name}",
                oninput: {
                    let id = thing_id.clone();
                    move |evt: dioxus::events::FormEvent| {
                        let new_name = evt.value();
                        update_thing_name(input_diagram, &id, &new_name);
                    }
                },
            }

            // Remove button
            span {
                class: REMOVE_BTN,
                onclick: {
                    let id = thing_id.clone();
                    move |_| {
                        remove_thing(input_diagram, &id);
                    }
                },
                "✕"
            }
        }
    }
}

/// Which field a generic key-value row targets.
#[derive(Clone, Copy, PartialEq, Eq)]
enum OnChangeTarget {
    CopyText,
    EntityDesc,
    EntityTooltip,
}

/// A reusable key-value row for maps keyed by an ID string.
#[component]
fn KeyValueRow(
    input_diagram: Signal<InputDiagram<'static>>,
    entry_id: String,
    entry_value: String,
    id_list: &'static str,
    on_change: OnChangeTarget,
) -> Element {
    rsx! {
        div {
            class: ROW_CLASS,

            input {
                class: INPUT_CLASS,
                style: "max-width:14rem",
                list: "{id_list}",
                placeholder: "id",
                value: "{entry_id}",
                onchange: {
                    let old_id = entry_id.clone();
                    let value = entry_value.clone();
                    move |evt: dioxus::events::FormEvent| {
                        let new_id = evt.value();
                        rename_kv_entry(input_diagram, on_change, &old_id, &new_id, &value);
                    }
                },
            }

            input {
                class: INPUT_CLASS,
                placeholder: "value",
                value: "{entry_value}",
                oninput: {
                    let id = entry_id.clone();
                    move |evt: dioxus::events::FormEvent| {
                        let new_value = evt.value();
                        update_kv_value(input_diagram, on_change, &id, &new_value);
                    }
                },
            }

            span {
                class: REMOVE_BTN,
                onclick: {
                    let id = entry_id.clone();
                    move |_| {
                        remove_kv_entry(input_diagram, on_change, &id);
                    }
                },
                "✕"
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Mutation helpers
// ---------------------------------------------------------------------------

use disposition::{input_model::thing::ThingId, model_common::Id};

fn add_thing_row(mut input_diagram: Signal<InputDiagram<'static>>) {
    // Find a unique placeholder ID.
    let mut n = input_diagram.read().things.len();
    loop {
        let candidate = format!("thing_{n}");
        if let Some(tid) = parse_thing_id(&candidate) {
            if !input_diagram.read().things.contains_key(&tid) {
                input_diagram.write().things.insert(tid, String::new());
                break;
            }
        }
        n += 1;
    }
}

fn update_thing_name(mut input_diagram: Signal<InputDiagram<'static>>, id: &str, name: &str) {
    if let Some(tid) = parse_thing_id(id) {
        if let Some(entry) = input_diagram.write().things.get_mut(&tid) {
            *entry = name.to_owned();
        }
    }
}

fn rename_thing(
    mut input_diagram: Signal<InputDiagram<'static>>,
    thing_id_old: &str,
    thing_id_new: &str,
) {
    let mut input_diagram = input_diagram.write();
    if thing_id_old == thing_id_new {
        return;
    }
    let InputDiagram {
        things,
        thing_copy_text,
        thing_hierarchy,
        thing_dependencies,
        thing_interactions,
        processes,
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
    if let Ok(thing_id_old) = Id::new(thing_id_old)
        .map(Id::into_static)
        .map(ThingId::from)
        && let Ok(thing_id_new) = Id::new(thing_id_new)
            .map(Id::into_static)
            .map(ThingId::from)
    {
        // Note: Results here are ignored -- we may want to be stricter here, e.g. try
        // replacing in all fields, and if any fail, revert.
        if let Some(thing_index) = things.get_index_of(&thing_id_old) {
            let _thing_names_replace_result =
                things.replace_index(thing_index, thing_id_new.clone());
        }
        if let Some(thing_index) = thing_copy_text.get_index_of(&thing_id_old) {
            let _thing_copy_text_replace_result =
                thing_copy_text.replace_index(thing_index, thing_id_new.clone());
        }
        if let Some((thing_hierarchy_with_id, thing_index)) =
            thing_hierarchy_recursive_search(thing_hierarchy, &thing_id_old)
        {
            let _thing_hierarchy_replace_result =
                thing_hierarchy_with_id.replace_index(thing_index, thing_id_new.clone());
        }

        // thing_dependencies: rename ThingIds inside EdgeKind values.
        thing_dependencies.values_mut().for_each(|edge_kind| {
            rename_thing_in_edge_kind(edge_kind, &thing_id_old, &thing_id_new);
        });

        // thing_interactions: same structure as thing_dependencies.
        thing_interactions.values_mut().for_each(|edge_kind| {
            rename_thing_in_edge_kind(edge_kind, &thing_id_old, &thing_id_new);
        });

        // processes: ProcessDiagram fields do not contain ThingId — skip.
        let _ = processes;

        // tags: TagNames keys are TagId, not ThingId — skip.
        let _ = tags;

        // tag_things: rename ThingIds in each Set<ThingId> value.
        tag_things.values_mut().for_each(|thing_ids| {
            if let Some(index) = thing_ids.get_index_of(&thing_id_old) {
                let _result = thing_ids.replace_index(index, thing_id_new.clone());
            }
        });

        // entity_descs / entity_tooltips / entity_types: keys are Id, which
        // may refer to a ThingId.
        let id_old = thing_id_old.clone().into_inner();
        let id_new = thing_id_new.clone().into_inner();
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

        // theme_tag_things_focus: rename in each ThemeStyles value.
        theme_tag_things_focus
            .values_mut()
            .for_each(|theme_styles| {
                rename_id_in_theme_styles(theme_styles, &id_old, &id_new);
            });
    }
}

/// Replaces occurrences of `thing_id_old` with `thing_id_new` inside an
/// [`EdgeKind`] (which wraps a `Vec<ThingId>`).
fn rename_thing_in_edge_kind(
    edge_kind: &mut EdgeKind<'static>,
    thing_id_old: &ThingId<'static>,
    thing_id_new: &ThingId<'static>,
) {
    let things = match edge_kind {
        EdgeKind::Cyclic(things) | EdgeKind::Sequence(things) | EdgeKind::Symmetric(things) => {
            things
        }
    };
    things.iter_mut().for_each(|thing_id| {
        if thing_id == thing_id_old {
            *thing_id = thing_id_new.clone();
        }
    });
}

fn thing_hierarchy_recursive_search<'f, 'id>(
    thing_hierarchy: &'f mut ThingHierarchy<'id>,
    thing_id: &'f ThingId<'id>,
) -> Option<(&'f mut ThingHierarchy<'id>, usize)> {
    if let Some(thing_index) = thing_hierarchy.get_index_of(thing_id) {
        Some((thing_hierarchy, thing_index))
    } else {
        thing_hierarchy
            .values_mut()
            .find_map(|thing_hierarchy_child| {
                thing_hierarchy_recursive_search(thing_hierarchy_child, thing_id)
            })
    }
}

fn remove_thing(mut input_diagram: Signal<InputDiagram<'static>>, id: &str) {
    if let Some(thing_id) = parse_thing_id(id) {
        input_diagram.write().things.swap_remove(&thing_id);
    }
}

// ── Generic key-value helpers for copy-text / descs / tooltips ───────────

fn add_copy_text_row(mut input_diagram: Signal<InputDiagram<'static>>) {
    let mut n = input_diagram.read().thing_copy_text.len();
    loop {
        let candidate = format!("thing_{n}");
        if let Some(tid) = parse_thing_id(&candidate) {
            if !input_diagram.read().thing_copy_text.contains_key(&tid) {
                input_diagram
                    .write()
                    .thing_copy_text
                    .insert(tid, String::new());
                break;
            }
        }
        n += 1;
    }
}

fn add_entity_desc_row(mut diag: Signal<InputDiagram<'static>>) {
    let mut n = diag.read().entity_descs.len();
    loop {
        let candidate = format!("entity_{n}");
        if let Some(id) = parse_id(&candidate) {
            if !diag.read().entity_descs.contains_key(&id) {
                diag.write().entity_descs.insert(id, String::new());
                break;
            }
        }
        n += 1;
    }
}

fn add_entity_tooltip_row(mut diag: Signal<InputDiagram<'static>>) {
    let mut n = diag.read().entity_tooltips.len();
    loop {
        let candidate = format!("entity_{n}");
        if let Some(id) = parse_id(&candidate) {
            if !diag.read().entity_tooltips.contains_key(&id) {
                diag.write().entity_tooltips.insert(id, String::new());
                break;
            }
        }
        n += 1;
    }
}

fn rename_kv_entry(
    mut diag: Signal<InputDiagram<'static>>,
    target: OnChangeTarget,
    old_id: &str,
    new_id: &str,
    current_value: &str,
) {
    if old_id == new_id {
        return;
    }
    match target {
        OnChangeTarget::CopyText => {
            let old = match parse_thing_id(old_id) {
                Some(id) => id,
                None => return,
            };
            let new = match parse_thing_id(new_id) {
                Some(id) => id,
                None => return,
            };
            let mut d = diag.write();
            d.thing_copy_text.swap_remove(&old);
            d.thing_copy_text.insert(new, current_value.to_owned());
        }
        OnChangeTarget::EntityDesc => {
            let old = match parse_id(old_id) {
                Some(id) => id,
                None => return,
            };
            let new = match parse_id(new_id) {
                Some(id) => id,
                None => return,
            };
            let mut d = diag.write();
            d.entity_descs.swap_remove(&old);
            d.entity_descs.insert(new, current_value.to_owned());
        }
        OnChangeTarget::EntityTooltip => {
            let old = match parse_id(old_id) {
                Some(id) => id,
                None => return,
            };
            let new = match parse_id(new_id) {
                Some(id) => id,
                None => return,
            };
            let mut d = diag.write();
            d.entity_tooltips.swap_remove(&old);
            d.entity_tooltips.insert(new, current_value.to_owned());
        }
    }
}

fn update_kv_value(
    mut input_diagram: Signal<InputDiagram<'static>>,
    target: OnChangeTarget,
    id: &str,
    value: &str,
) {
    match target {
        OnChangeTarget::CopyText => {
            if let Some(thing_id) = parse_thing_id(id) {
                if let Some(entry) = input_diagram.write().thing_copy_text.get_mut(&thing_id) {
                    *entry = value.to_owned();
                }
            }
        }
        OnChangeTarget::EntityDesc => {
            if let Some(entity_id) = parse_id(id) {
                if let Some(entry) = input_diagram.write().entity_descs.get_mut(&entity_id) {
                    *entry = value.to_owned();
                }
            }
        }
        OnChangeTarget::EntityTooltip => {
            if let Some(entity_id) = parse_id(id) {
                if let Some(entry) = input_diagram.write().entity_tooltips.get_mut(&entity_id) {
                    *entry = value.to_owned();
                }
            }
        }
    }
}

fn remove_kv_entry(
    mut input_diagram: Signal<InputDiagram<'static>>,
    target: OnChangeTarget,
    id: &str,
) {
    match target {
        OnChangeTarget::CopyText => {
            if let Some(thing_id) = parse_thing_id(id) {
                input_diagram.write().thing_copy_text.swap_remove(&thing_id);
            }
        }
        OnChangeTarget::EntityDesc => {
            if let Some(entity_id) = parse_id(id) {
                input_diagram.write().entity_descs.swap_remove(&entity_id);
            }
        }
        OnChangeTarget::EntityTooltip => {
            if let Some(entity_id) = parse_id(id) {
                input_diagram
                    .write()
                    .entity_tooltips
                    .swap_remove(&entity_id);
            }
        }
    }
}
