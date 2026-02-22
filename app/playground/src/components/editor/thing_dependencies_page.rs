//! Thing Dependencies editor page.
//!
//! Allows editing `thing_dependencies` -- a map from `EdgeGroupId` to
//! `EdgeKind` (Cyclic / Sequence / Symmetric), where each variant contains a
//! list of `ThingId`s.

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::{
    input_model::{
        edge::EdgeKind,
        theme::{IdOrDefaults, ThemeStyles},
        thing::ThingId,
        InputDiagram,
    },
    model_common::{edge::EdgeGroupId, Id},
};

use super::datalists::list_ids;

/// CSS classes shared by section headings.
const SECTION_HEADING: &str = "text-sm font-bold text-gray-300 mt-4 mb-1";

/// CSS classes for a card-like container for each edge group.
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

/// CSS classes for a select / dropdown.
const SELECT_CLASS: &str = "\
    rounded \
    border \
    border-gray-600 \
    bg-gray-800 \
    text-gray-200 \
    px-2 py-1 \
    text-sm \
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

/// Serialised snapshot of one edge group entry for rendering.
#[derive(Clone, PartialEq)]
struct EdgeGroupEntry {
    edge_group_id: String,
    kind_label: String,
    things: Vec<String>,
}

/// The **Thing Dependencies** editor page.
#[component]
pub fn ThingDependenciesPage(input_diagram: Signal<InputDiagram<'static>>) -> Element {
    let diagram = input_diagram.read();

    let entries: Vec<EdgeGroupEntry> = diagram
        .thing_dependencies
        .iter()
        .map(|(id, kind)| {
            let (kind_label, things) = edge_kind_to_parts(kind);
            EdgeGroupEntry {
                edge_group_id: id.as_str().to_owned(),
                kind_label: kind_label.to_owned(),
                things,
            }
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
                    add_edge_group(input_diagram, MapTarget::Dependencies);
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
        .map(|(id, kind)| {
            let (kind_label, things) = edge_kind_to_parts(kind);
            EdgeGroupEntry {
                edge_group_id: id.as_str().to_owned(),
                kind_label: kind_label.to_owned(),
                things,
            }
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
                    add_edge_group(input_diagram, MapTarget::Interactions);
                },
                "+ Add interaction edge group"
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Shared card component
// ---------------------------------------------------------------------------

/// Which map inside `InputDiagram` we are editing.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum MapTarget {
    Dependencies,
    Interactions,
}

/// A card for a single edge group (used by both dependencies and interactions).
#[component]
fn EdgeGroupCard(
    input_diagram: Signal<InputDiagram<'static>>,
    entry: EdgeGroupEntry,
    target: MapTarget,
) -> Element {
    let edge_id = entry.edge_group_id.clone();
    let kind_label = entry.kind_label.clone();
    let things = entry.things.clone();

    rsx! {
        div {
            class: CARD_CLASS,

            // ── Header row: EdgeGroupId + Kind selector + Remove ─────
            div {
                class: ROW_CLASS,

                // Edge group ID
                input {
                    class: INPUT_CLASS,
                    style: "max-width:16rem",
                    list: list_ids::EDGE_GROUP_IDS,
                    placeholder: "edge_group_id",
                    value: "{edge_id}",
                    onchange: {
                        let old_id = edge_id.clone();
                        move |evt: dioxus::events::FormEvent| {
                            let new_id = evt.value();
                            rename_edge_group(input_diagram, &old_id, &new_id);
                        }
                    },
                }

                // EdgeKind selector
                select {
                    class: SELECT_CLASS,
                    value: "{kind_label}",
                    onchange: {
                        let eid = edge_id.clone();
                        let current_things = things.clone();
                        move |evt: dioxus::events::FormEvent| {
                            let new_kind = evt.value();
                            change_edge_kind(input_diagram, target, &eid, &new_kind, &current_things);
                        }
                    },
                    option { value: "cyclic", "Cyclic" }
                    option { value: "sequence", "Sequence" }
                    option { value: "symmetric", "Symmetric" }
                }

                // Remove edge group
                span {
                    class: REMOVE_BTN,
                    onclick: {
                        let eid = edge_id.clone();
                        move |_| {
                            remove_edge_group(input_diagram, target, &eid);
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
                        let eid = edge_id.clone();
                        rsx! {
                            div {
                                key: "{eid}_{idx}",
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
                                        let eid2 = eid.clone();
                                        move |evt: dioxus::events::FormEvent| {
                                            let new_val = evt.value();
                                            update_thing_in_edge(input_diagram, target, &eid2, idx, &new_val);
                                        }
                                    },
                                }

                                span {
                                    class: REMOVE_BTN,
                                    onclick: {
                                        let eid2 = eid.clone();
                                        move |_| {
                                            remove_thing_from_edge(input_diagram, target, &eid2, idx);
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
                        let eid = edge_id.clone();
                        move |_| {
                            add_thing_to_edge(input_diagram, target, &eid);
                        }
                    },
                    "+ Add thing"
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Decompose an `EdgeKind` into a label string and a list of thing ID strings.
fn edge_kind_to_parts(kind: &EdgeKind<'_>) -> (&'static str, Vec<String>) {
    match kind {
        EdgeKind::Cyclic(things) => (
            "cyclic",
            things.iter().map(|t| t.as_str().to_owned()).collect(),
        ),
        EdgeKind::Sequence(things) => (
            "sequence",
            things.iter().map(|t| t.as_str().to_owned()).collect(),
        ),
        EdgeKind::Symmetric(things) => (
            "symmetric",
            things.iter().map(|t| t.as_str().to_owned()).collect(),
        ),
    }
}

/// Reconstruct an `EdgeKind<'static>` from a label and a list of thing ID
/// strings.
fn parts_to_edge_kind(label: &str, thing_strs: &[String]) -> Option<EdgeKind<'static>> {
    let things: Vec<ThingId<'static>> = thing_strs
        .iter()
        .filter_map(|s| parse_thing_id(s))
        .collect();

    match label {
        "cyclic" => Some(EdgeKind::Cyclic(things)),
        "sequence" => Some(EdgeKind::Sequence(things)),
        "symmetric" => Some(EdgeKind::Symmetric(things)),
        _ => None,
    }
}

fn parse_thing_id(s: &str) -> Option<ThingId<'static>> {
    Id::new(s).ok().map(|id| ThingId::from(id.into_static()))
}

fn parse_edge_group_id(s: &str) -> Option<EdgeGroupId<'static>> {
    EdgeGroupId::new(s).ok().map(|id| id.into_static())
}

// ---------------------------------------------------------------------------
// Mutation helpers
// ---------------------------------------------------------------------------

fn set_edge_kind(
    diag: &mut InputDiagram<'static>,
    target: MapTarget,
    edge_id: &EdgeGroupId<'static>,
    kind: EdgeKind<'static>,
) {
    match target {
        MapTarget::Dependencies => {
            diag.thing_dependencies.insert(edge_id.clone(), kind);
        }
        MapTarget::Interactions => {
            diag.thing_interactions.insert(edge_id.clone(), kind);
        }
    }
}

fn remove_edge_group_by_id(
    diag: &mut InputDiagram<'static>,
    target: MapTarget,
    edge_id: &EdgeGroupId<'static>,
) {
    match target {
        MapTarget::Dependencies => {
            diag.thing_dependencies.swap_remove(edge_id);
        }
        MapTarget::Interactions => {
            diag.thing_interactions.swap_remove(edge_id);
        }
    }
}

fn edge_group_count(diag: &InputDiagram<'static>, target: MapTarget) -> usize {
    match target {
        MapTarget::Dependencies => diag.thing_dependencies.len(),
        MapTarget::Interactions => diag.thing_interactions.len(),
    }
}

fn edge_group_contains(
    diag: &InputDiagram<'static>,
    target: MapTarget,
    edge_id: &EdgeGroupId<'static>,
) -> bool {
    match target {
        MapTarget::Dependencies => diag.thing_dependencies.contains_key(edge_id),
        MapTarget::Interactions => diag.thing_interactions.contains_key(edge_id),
    }
}

fn add_edge_group(mut diag: Signal<InputDiagram<'static>>, target: MapTarget) {
    let mut n = edge_group_count(&diag.read(), target);
    loop {
        let candidate = format!("edge_{n}");
        if let Some(eid) = parse_edge_group_id(&candidate) {
            if !edge_group_contains(&diag.read(), target, &eid) {
                set_edge_kind(
                    &mut diag.write(),
                    target,
                    &eid,
                    EdgeKind::Sequence(Vec::new()),
                );
                break;
            }
        }
        n += 1;
    }
}

fn remove_edge_group(mut diag: Signal<InputDiagram<'static>>, target: MapTarget, id: &str) {
    if let Some(eid) = parse_edge_group_id(id) {
        remove_edge_group_by_id(&mut diag.write(), target, &eid);
    }
}

fn rename_edge_group(mut diag: Signal<InputDiagram<'static>>, old_id: &str, new_id: &str) {
    if old_id == new_id {
        return;
    }
    let edge_id_old = match parse_edge_group_id(old_id) {
        Some(id) => id,
        None => return,
    };
    let edge_id_new = match parse_edge_group_id(new_id) {
        Some(id) => id,
        None => return,
    };

    let mut input_diagram = diag.write();
    let InputDiagram {
        things: _,
        thing_copy_text: _,
        thing_hierarchy: _,
        thing_dependencies,
        thing_interactions,
        processes,
        tags: _,
        tag_things: _,
        entity_descs,
        entity_tooltips,
        entity_types,
        theme_default,
        theme_types_styles,
        theme_thing_dependencies_styles,
        theme_tag_things_focus,
        css: _,
    } = &mut *input_diagram;

    // thing_dependencies: rename EdgeGroupId key.
    if let Some(index) = thing_dependencies.get_index_of(&edge_id_old) {
        let _result = thing_dependencies.replace_index(index, edge_id_new.clone());
    }

    // thing_interactions: rename EdgeGroupId key.
    if let Some(index) = thing_interactions.get_index_of(&edge_id_old) {
        let _result = thing_interactions.replace_index(index, edge_id_new.clone());
    }

    // processes: rename EdgeGroupId in step_thing_interactions values.
    processes.values_mut().for_each(|process_diagram| {
        process_diagram
            .step_thing_interactions
            .values_mut()
            .for_each(|edge_group_ids| {
                for edge_group_id in edge_group_ids.iter_mut() {
                    if edge_group_id == &edge_id_old {
                        *edge_group_id = edge_id_new.clone();
                    }
                }
            });
    });

    // entity_descs / entity_tooltips / entity_types: keys are Id, which
    // may refer to an EdgeGroupId.
    let id_old = edge_id_old.clone().into_inner();
    let id_new = edge_id_new.clone().into_inner();
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

/// Replaces an [`IdOrDefaults::Id`] key that matches `id_old` with `id_new`
/// inside a [`ThemeStyles`] map.
fn rename_id_in_theme_styles(
    theme_styles: &mut ThemeStyles<'static>,
    id_old: &Id<'static>,
    id_new: &Id<'static>,
) {
    let key_old = IdOrDefaults::Id(id_old.clone());
    if let Some(index) = theme_styles.get_index_of(&key_old) {
        let key_new = IdOrDefaults::Id(id_new.clone());
        let _result = theme_styles.replace_index(index, key_new);
    }
}

fn change_edge_kind(
    mut diag: Signal<InputDiagram<'static>>,
    target: MapTarget,
    edge_id: &str,
    new_kind_label: &str,
    current_things: &[String],
) {
    let eid = match parse_edge_group_id(edge_id) {
        Some(id) => id,
        None => return,
    };
    if let Some(kind) = parts_to_edge_kind(new_kind_label, current_things) {
        set_edge_kind(&mut diag.write(), target, &eid, kind);
    }
}

fn update_thing_in_edge(
    mut diag: Signal<InputDiagram<'static>>,
    target: MapTarget,
    edge_id: &str,
    idx: usize,
    new_thing_str: &str,
) {
    let eid = match parse_edge_group_id(edge_id) {
        Some(id) => id,
        None => return,
    };
    let new_thing = match parse_thing_id(new_thing_str) {
        Some(t) => t,
        None => return,
    };

    let mut d = diag.write();
    let kind = match target {
        MapTarget::Dependencies => d.thing_dependencies.get_mut(&eid),
        MapTarget::Interactions => d.thing_interactions.get_mut(&eid),
    };
    if let Some(kind) = kind {
        let things = match kind {
            EdgeKind::Cyclic(t) | EdgeKind::Sequence(t) | EdgeKind::Symmetric(t) => t,
        };
        if idx < things.len() {
            things[idx] = new_thing;
        }
    }
}

fn remove_thing_from_edge(
    mut diag: Signal<InputDiagram<'static>>,
    target: MapTarget,
    edge_id: &str,
    idx: usize,
) {
    let eid = match parse_edge_group_id(edge_id) {
        Some(id) => id,
        None => return,
    };

    let mut d = diag.write();
    let kind = match target {
        MapTarget::Dependencies => d.thing_dependencies.get_mut(&eid),
        MapTarget::Interactions => d.thing_interactions.get_mut(&eid),
    };
    if let Some(kind) = kind {
        let things = match kind {
            EdgeKind::Cyclic(t) | EdgeKind::Sequence(t) | EdgeKind::Symmetric(t) => t,
        };
        if idx < things.len() {
            things.remove(idx);
        }
    }
}

fn add_thing_to_edge(mut diag: Signal<InputDiagram<'static>>, target: MapTarget, edge_id: &str) {
    let eid = match parse_edge_group_id(edge_id) {
        Some(id) => id,
        None => return,
    };

    // Find any existing thing ID as a placeholder, or use an empty-ish
    // fallback.
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
    let kind = match target {
        MapTarget::Dependencies => d.thing_dependencies.get_mut(&eid),
        MapTarget::Interactions => d.thing_interactions.get_mut(&eid),
    };
    if let Some(kind) = kind {
        let things = match kind {
            EdgeKind::Cyclic(t) | EdgeKind::Sequence(t) | EdgeKind::Symmetric(t) => t,
        };
        things.push(new_thing);
    }
}
