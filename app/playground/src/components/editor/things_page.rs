//! Things editor page.
//!
//! Allows editing:
//! - `things`: `ThingId` -> display name
//! - `thing_copy_text`: `ThingId` -> clipboard text
//! - `entity_descs`: `Id` -> description (filtered to thing IDs here)
//! - `entity_tooltips`: `Id` -> tooltip
//! - `thing_hierarchy`: recursive nesting of things

use dioxus::{
    hooks::use_signal,
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::{
    input_model::{edge::EdgeKind, thing::ThingHierarchy, InputDiagram},
    model_common::Id,
};

use crate::{
    components::editor::{
        common::{
            id_rename_in_input_diagram, parse_id, parse_thing_id, ADD_BTN, COLLAPSE_BAR,
            DRAG_HANDLE, ID_INPUT_CLASS, INPUT_CLASS, REMOVE_BTN, ROW_CLASS, SECTION_HEADING,
            TEXTAREA_CLASS,
        },
        datalists::list_ids,
    },
    editor_state::ThingsPageUiState,
};

use disposition::input_model::thing::ThingId;

/// Number of rows shown when a section is collapsed.
const COLLAPSE_THRESHOLD: usize = 4;

/// A container for multiple draggable [`KeyValueRow`]s (or [`ThingNameRow`]s).
///
/// Uses `group/key-value-rows` so that child rows can react to an active drag
/// via `group-active/key-value-rows:_` utilities. Does **not** use `gap` on
/// the flex container -- each row carries its own padding instead, so there are
/// no dead-zones between rows where a drop would be missed.
#[component]
fn KeyValueRowContainer(children: Element) -> Element {
    rsx! {
        div {
            class: "flex flex-col group/key-value-rows",
            {children}
        }
    }
}

/// The **Things** editor page.
///
/// Renders editable rows for each `ThingId` in the diagram's `things` map, as
/// well as associated copy-text, descriptions, tooltips, and hierarchy.
#[component]
pub fn ThingsPage(
    input_diagram: Signal<InputDiagram<'static>>,
    things_ui_state: Signal<ThingsPageUiState>,
) -> Element {
    // Drag-and-drop state: tracks the index currently being dragged per section.
    let thing_drag_idx: Signal<Option<usize>> = use_signal(|| None);
    let copy_text_drag_idx: Signal<Option<usize>> = use_signal(|| None);
    let desc_drag_idx: Signal<Option<usize>> = use_signal(|| None);
    let tooltip_drag_idx: Signal<Option<usize>> = use_signal(|| None);

    // Drop-target state: tracks which row is being hovered over per section.
    let thing_drop_target: Signal<Option<usize>> = use_signal(|| None);
    let copy_text_drop_target: Signal<Option<usize>> = use_signal(|| None);
    let desc_drop_target: Signal<Option<usize>> = use_signal(|| None);
    let tooltip_drop_target: Signal<Option<usize>> = use_signal(|| None);

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

    // Read collapsed states.
    let ui = things_ui_state.read();
    let thing_names_collapsed = ui.thing_names_collapsed;
    let copy_text_collapsed = ui.copy_text_collapsed;
    let entity_descs_collapsed = ui.entity_descs_collapsed;
    let entity_tooltips_collapsed = ui.entity_tooltips_collapsed;
    drop(ui);

    // Determine which entries are visible for each section.
    let thing_names_collapsible = thing_entries.len() > COLLAPSE_THRESHOLD;
    let copy_text_collapsible = copy_text_entries.len() > COLLAPSE_THRESHOLD;
    let entity_descs_collapsible = desc_entries.len() > COLLAPSE_THRESHOLD;
    let entity_tooltips_collapsible = tooltip_entries.len() > COLLAPSE_THRESHOLD;

    let visible_things: Vec<(usize, &(String, String))> =
        if thing_names_collapsible && thing_names_collapsed {
            thing_entries
                .iter()
                .enumerate()
                .take(COLLAPSE_THRESHOLD)
                .collect()
        } else {
            thing_entries.iter().enumerate().collect()
        };

    let visible_copy_text: Vec<(usize, &(String, String))> =
        if copy_text_collapsible && copy_text_collapsed {
            copy_text_entries
                .iter()
                .enumerate()
                .take(COLLAPSE_THRESHOLD)
                .collect()
        } else {
            copy_text_entries.iter().enumerate().collect()
        };

    let visible_descs: Vec<(usize, &(String, String))> =
        if entity_descs_collapsible && entity_descs_collapsed {
            desc_entries
                .iter()
                .enumerate()
                .take(COLLAPSE_THRESHOLD)
                .collect()
        } else {
            desc_entries.iter().enumerate().collect()
        };

    let visible_tooltips: Vec<(usize, &(String, String))> =
        if entity_tooltips_collapsible && entity_tooltips_collapsed {
            tooltip_entries
                .iter()
                .enumerate()
                .take(COLLAPSE_THRESHOLD)
                .collect()
        } else {
            tooltip_entries.iter().enumerate().collect()
        };

    let thing_count = thing_entries.len();
    let copy_text_count = copy_text_entries.len();
    let desc_count = desc_entries.len();
    let tooltip_count = tooltip_entries.len();

    rsx! {
        div {
            class: "flex flex-col gap-2",

            // ── Thing Names ──────────────────────────────────────────
            h3 { class: SECTION_HEADING, "Thing Names" }
            p {
                class: "text-xs text-gray-500 mb-1",
                "Map of ThingId -> display label."
            }

            KeyValueRowContainer {
                for (idx, (id, name)) in visible_things.iter() {
                    {
                        let id = id.clone();
                        let name = name.clone();
                        let idx = *idx;
                        rsx! {
                            ThingNameRow {
                                    key: "{id}",
                                    input_diagram,
                                    thing_id: id,
                                    thing_name: name,
                                    index: idx,
                                    drag_index: thing_drag_idx,
                                    drop_target: thing_drop_target,
                                }
                        }
                    }
                }
            }

            // Collapse bar for Thing Names
            if thing_names_collapsible {
                {
                    rsx! {
                        CollapseBar {
                            collapsed: thing_names_collapsed,
                            total: thing_count,
                            visible: if thing_names_collapsed { COLLAPSE_THRESHOLD } else { thing_count },
                            on_toggle: move |_| {
                                things_ui_state.write().thing_names_collapsed = !thing_names_collapsed;
                            },
                        }
                    }
                }
            }

            // Add new thing
            div {
                class: ADD_BTN,
                onclick: move |_| {
                    ThingsPageOps::thing_add(input_diagram);
                },
                "+ Add thing"
            }

            // ── Copy Text ────────────────────────────────────────────
            h3 { class: SECTION_HEADING, "Thing Copy Text" }
            p {
                class: "text-xs text-gray-500 mb-1",
                "Optional clipboard text per ThingId (defaults to display label)."
            }

            KeyValueRowContainer {
                for (idx, (id, text)) in visible_copy_text.iter() {
                    {
                        let id = id.clone();
                        let text = text.clone();
                        let idx = *idx;
                        rsx! {
                            KeyValueRow {
                                key: "ct_{id}",
                                input_diagram,
                                entry_id: id,
                                entry_value: text,
                                id_list: list_ids::THING_IDS,
                                on_change: OnChangeTarget::CopyText,
                                index: idx,
                                drag_index: copy_text_drag_idx,
                                drop_target: copy_text_drop_target,
                            }
                        }
                    }
                }
            }

            // Collapse bar for Copy Text
            if copy_text_collapsible {
                {
                    rsx! {
                        CollapseBar {
                            collapsed: copy_text_collapsed,
                            total: copy_text_count,
                            visible: if copy_text_collapsed { COLLAPSE_THRESHOLD } else { copy_text_count },
                            on_toggle: move |_| {
                                things_ui_state.write().copy_text_collapsed = !copy_text_collapsed;
                            },
                        }
                    }
                }
            }

            div {
                class: ADD_BTN,
                onclick: move |_| {
                    ThingsPageOps::copy_text_add(input_diagram);
                },
                "+ Add copy text"
            }

            // ── Entity Descriptions ──────────────────────────────────
            h3 { class: SECTION_HEADING, "Entity Descriptions" }
            p {
                class: "text-xs text-gray-500 mb-1",
                "Descriptions rendered next to entities in the diagram."
            }

            KeyValueRowContainer {
                for (idx, (id, desc)) in visible_descs.iter() {
                    {
                        let id = id.clone();
                        let desc = desc.clone();
                        let idx = *idx;
                        rsx! {
                            KeyValueRow {
                                key: "desc_{id}",
                                input_diagram,
                                entry_id: id,
                                entry_value: desc,
                                id_list: list_ids::ENTITY_IDS,
                                on_change: OnChangeTarget::EntityDesc,
                                index: idx,
                                drag_index: desc_drag_idx,
                                drop_target: desc_drop_target,
                            }
                        }
                    }
                }
            }

            // Collapse bar for Entity Descriptions
            if entity_descs_collapsible {
                {
                    rsx! {
                        CollapseBar {
                            collapsed: entity_descs_collapsed,
                            total: desc_count,
                            visible: if entity_descs_collapsed { COLLAPSE_THRESHOLD } else { desc_count },
                            on_toggle: move |_| {
                                things_ui_state.write().entity_descs_collapsed = !entity_descs_collapsed;
                            },
                        }
                    }
                }
            }

            div {
                class: ADD_BTN,
                onclick: move |_| {
                    ThingsPageOps::entity_desc_add(input_diagram);
                },
                "+ Add description"
            }

            // ── Entity Tooltips ──────────────────────────────────────
            h3 { class: SECTION_HEADING, "Entity Tooltips" }
            p {
                class: "text-xs text-gray-500 mb-1",
                "Tooltip text (markdown) shown on hover."
            }

            KeyValueRowContainer {
                for (idx, (id, tip)) in visible_tooltips.iter() {
                    {
                        let id = id.clone();
                        let tip = tip.clone();
                        let idx = *idx;
                        rsx! {
                            KeyValueRow {
                                key: "tip_{id}",
                                input_diagram,
                                entry_id: id,
                                entry_value: tip,
                                id_list: list_ids::ENTITY_IDS,
                                on_change: OnChangeTarget::EntityTooltip,
                                index: idx,
                                drag_index: tooltip_drag_idx,
                                drop_target: tooltip_drop_target,
                            }
                        }
                    }
                }
            }

            // Collapse bar for Entity Tooltips
            if entity_tooltips_collapsible {
                {
                    rsx! {
                        CollapseBar {
                            collapsed: entity_tooltips_collapsed,
                            total: tooltip_count,
                            visible: if entity_tooltips_collapsed { COLLAPSE_THRESHOLD } else { tooltip_count },
                            on_toggle: move |_| {
                                things_ui_state.write().entity_tooltips_collapsed = !entity_tooltips_collapsed;
                            },
                        }
                    }
                }
            }

            div {
                class: ADD_BTN,
                onclick: move |_| {
                    ThingsPageOps::entity_tooltip_add(input_diagram);
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
                class: TEXTAREA_CLASS,
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

// ===========================================================================
// ThingsPage mutation helpers
// ===========================================================================

/// Mutation operations for the Things editor page.
///
/// Grouped here so that related functions are discoverable when sorted by
/// name, per the project's `noun_verb` naming convention.
struct ThingsPageOps;

impl ThingsPageOps {
    /// Adds a new thing row with a unique placeholder ID.
    fn thing_add(mut input_diagram: Signal<InputDiagram<'static>>) {
        let mut n = input_diagram.read().things.len();
        loop {
            let candidate = format!("thing_{n}");
            if let Some(thing_id) = parse_thing_id(&candidate) {
                if !input_diagram.read().things.contains_key(&thing_id) {
                    input_diagram.write().things.insert(thing_id, String::new());
                    break;
                }
            }
            n += 1;
        }
    }

    /// Updates the display name for an existing thing.
    fn thing_name_update(
        mut input_diagram: Signal<InputDiagram<'static>>,
        thing_id_str: &str,
        name: &str,
    ) {
        if let Some(thing_id) = parse_thing_id(thing_id_str) {
            if let Some(entry) = input_diagram.write().things.get_mut(&thing_id) {
                *entry = name.to_owned();
            }
        }
    }

    /// Renames a thing across all maps in the [`InputDiagram`].
    fn thing_rename(
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
            if let Some((thing_hierarchy_with_id, thing_index)) =
                Self::thing_hierarchy_recursive_search(
                    &mut input_diagram_ref.thing_hierarchy,
                    &thing_id_old,
                )
            {
                let _result =
                    thing_hierarchy_with_id.replace_index(thing_index, thing_id_new.clone());
            }

            // thing_dependencies: rename ThingIds inside EdgeKind values.
            input_diagram_ref
                .thing_dependencies
                .values_mut()
                .for_each(|edge_kind| {
                    Self::thing_rename_in_edge_kind(edge_kind, &thing_id_old, &thing_id_new);
                });

            // thing_interactions: same structure as thing_dependencies.
            input_diagram_ref
                .thing_interactions
                .values_mut()
                .for_each(|edge_kind| {
                    Self::thing_rename_in_edge_kind(edge_kind, &thing_id_old, &thing_id_new);
                });

            // tag_things: rename ThingIds in each Set<ThingId> value.
            input_diagram_ref
                .tag_things
                .values_mut()
                .for_each(|thing_ids| {
                    if let Some(index) = thing_ids.get_index_of(&thing_id_old) {
                        let _result = thing_ids.replace_index(index, thing_id_new.clone());
                    }
                });

            // Shared rename across entity_descs, entity_tooltips, entity_types,
            // and all theme style maps.
            let id_old = thing_id_old.into_inner();
            let id_new = thing_id_new.into_inner();
            id_rename_in_input_diagram(&mut input_diagram_ref, &id_old, &id_new);
        }
    }

    /// Replaces occurrences of `thing_id_old` with `thing_id_new` inside an
    /// [`EdgeKind`] (which wraps a `Vec<ThingId>`).
    fn thing_rename_in_edge_kind(
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

    /// Searches recursively through a [`ThingHierarchy`] for a given
    /// [`ThingId`] key, returning a mutable reference to the containing map
    /// and the index within it.
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
                    Self::thing_hierarchy_recursive_search(thing_hierarchy_child, thing_id)
                })
        }
    }

    /// Removes a thing from the `things` map.
    fn thing_remove(mut input_diagram: Signal<InputDiagram<'static>>, thing_id_str: &str) {
        if let Some(thing_id) = parse_thing_id(thing_id_str) {
            input_diagram.write().things.swap_remove(&thing_id);
        }
    }

    /// Moves a thing entry from one index to another in the `things` map.
    fn thing_move(mut input_diagram: Signal<InputDiagram<'static>>, from: usize, to: usize) {
        input_diagram.write().things.move_index(from, to);
    }

    // ── Copy text helpers ────────────────────────────────────────────

    /// Adds a new copy-text row with a unique placeholder ThingId.
    fn copy_text_add(mut input_diagram: Signal<InputDiagram<'static>>) {
        let mut n = input_diagram.read().thing_copy_text.len();
        loop {
            let candidate = format!("thing_{n}");
            if let Some(thing_id) = parse_thing_id(&candidate) {
                if !input_diagram.read().thing_copy_text.contains_key(&thing_id) {
                    input_diagram
                        .write()
                        .thing_copy_text
                        .insert(thing_id, String::new());
                    break;
                }
            }
            n += 1;
        }
    }

    /// Adds a new entity description row with a unique placeholder Id.
    fn entity_desc_add(mut input_diagram: Signal<InputDiagram<'static>>) {
        let mut n = input_diagram.read().entity_descs.len();
        loop {
            let candidate = format!("entity_{n}");
            if let Some(id) = parse_id(&candidate) {
                if !input_diagram.read().entity_descs.contains_key(&id) {
                    input_diagram.write().entity_descs.insert(id, String::new());
                    break;
                }
            }
            n += 1;
        }
    }

    /// Adds a new entity tooltip row with a unique placeholder Id.
    fn entity_tooltip_add(mut input_diagram: Signal<InputDiagram<'static>>) {
        let mut n = input_diagram.read().entity_tooltips.len();
        loop {
            let candidate = format!("entity_{n}");
            if let Some(id) = parse_id(&candidate) {
                if !input_diagram.read().entity_tooltips.contains_key(&id) {
                    input_diagram
                        .write()
                        .entity_tooltips
                        .insert(id, String::new());
                    break;
                }
            }
            n += 1;
        }
    }

    // ── Key-value (copy-text / desc / tooltip) mutation helpers ──────

    /// Renames the key of a key-value entry in the target map.
    fn kv_entry_rename(
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
    fn kv_entry_update(
        mut input_diagram: Signal<InputDiagram<'static>>,
        target: OnChangeTarget,
        id_str: &str,
        value: &str,
    ) {
        match target {
            OnChangeTarget::CopyText => {
                if let Some(thing_id) = parse_thing_id(id_str) {
                    if let Some(entry) = input_diagram.write().thing_copy_text.get_mut(&thing_id) {
                        *entry = value.to_owned();
                    }
                }
            }
            OnChangeTarget::EntityDesc => {
                if let Some(entity_id) = parse_id(id_str) {
                    if let Some(entry) = input_diagram.write().entity_descs.get_mut(&entity_id) {
                        *entry = value.to_owned();
                    }
                }
            }
            OnChangeTarget::EntityTooltip => {
                if let Some(entity_id) = parse_id(id_str) {
                    if let Some(entry) = input_diagram.write().entity_tooltips.get_mut(&entity_id) {
                        *entry = value.to_owned();
                    }
                }
            }
        }
    }

    /// Removes a key-value entry from the target map.
    fn kv_entry_remove(
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
    fn kv_entry_move(
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

// ===========================================================================
// Collapse bar component
// ===========================================================================

/// A clickable bar that toggles between collapsed and expanded states.
///
/// When collapsed the text is on top with a wide V-shaped down chevron below.
/// When expanded the up chevron is on top with the text below.
#[component]
fn CollapseBar(
    collapsed: bool,
    total: usize,
    visible: usize,
    on_toggle: dioxus::prelude::EventHandler<dioxus::events::MouseEvent>,
) -> Element {
    let hidden = total.saturating_sub(visible);
    let label = if collapsed {
        format!("··· {hidden} more")
    } else {
        String::from("···")
    };

    // Wide V chevron: a small rotated square border using Tailwind classes.
    // Collapsed = points down (below text), Expanded = points up (above text).
    let chevron_down_class = "\
        inline-block \
        w-2.5 \
        h-2.5 \
        border-l-2 \
        border-b-2 \
        border-current \
        -rotate-45 \
        mb-1\
    ";
    let chevron_up_class = "\
        inline-block \
        w-2.5 \
        h-2.5 \
        border-l-2 \
        border-b-2 \
        border-current \
        rotate-135 \
        mt-1\
    ";

    rsx! {
        button {
            class: COLLAPSE_BAR,
            onclick: move |evt| on_toggle.call(evt),

            if collapsed {
                // Text on top, V arrow below
                span {
                    class: "text-xs tracking-widest",
                    "{label}"
                }
                span {
                    class: "{chevron_down_class}",
                }
            } else {
                // ^ arrow on top, text below
                span {
                    class: "{chevron_up_class}",
                }
                span {
                    class: "text-xs tracking-widest",
                    "{label}"
                }
            }
        }
    }
}

// ===========================================================================
// Helper components
// ===========================================================================

/// A single editable row for a thing name (ThingId -> display label).
#[component]
fn ThingNameRow(
    input_diagram: Signal<InputDiagram<'static>>,
    thing_id: String,
    thing_name: String,
    index: usize,
    drag_index: Signal<Option<usize>>,
    drop_target: Signal<Option<usize>>,
) -> Element {
    let border_class = drag_row_border_class(drag_index, drop_target, index);

    rsx! {
        div {
            class: "{ROW_CLASS} {border_class}",
            draggable: "true",
            ondragstart: move |_| {
                drag_index.set(Some(index));
            },
            ondragover: move |evt| {
                evt.prevent_default();
                drop_target.set(Some(index));
            },
            ondrop: move |evt| {
                evt.prevent_default();
                if let Some(from) = *drag_index.read() {
                    if from != index {
                        ThingsPageOps::thing_move(input_diagram, from, index);
                    }
                }
                drag_index.set(None);
                drop_target.set(None);
            },
            ondragend: move |_| {
                drag_index.set(None);
                drop_target.set(None);
            },

            // Drag handle
            DragHandle {}

            // ThingId input
            input {
                class: ID_INPUT_CLASS,
                style: "max-width:14rem",
                list: list_ids::THING_IDS,
                placeholder: "thing_id",
                value: "{thing_id}",
                pattern: "^[a-zA-Z_][a-zA-Z0-9_]*$",
                onchange: {
                    let thing_id_old = thing_id.clone();
                    move |evt: dioxus::events::FormEvent| {
                        let thing_id_new = evt.value();
                        ThingsPageOps::thing_rename(input_diagram, &thing_id_old, &thing_id_new);
                    }
                },
            }

            // Display name input
            input {
                class: INPUT_CLASS,
                placeholder: "Display name",
                value: "{thing_name}",
                oninput: {
                    let thing_id = thing_id.clone();
                    move |evt: dioxus::events::FormEvent| {
                        let name = evt.value();
                        ThingsPageOps::thing_name_update(input_diagram, &thing_id, &name);
                    }
                },
            }

            // Remove button
            span {
                class: REMOVE_BTN,
                onclick: {
                    let thing_id = thing_id.clone();
                    move |_| {
                        ThingsPageOps::thing_remove(input_diagram, &thing_id);
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
    index: usize,
    drag_index: Signal<Option<usize>>,
    drop_target: Signal<Option<usize>>,
) -> Element {
    let border_class = drag_row_border_class(drag_index, drop_target, index);

    rsx! {
        div {
            class: "{ROW_CLASS} {border_class}",
            draggable: "true",
            ondragstart: move |_| {
                drag_index.set(Some(index));
            },
            ondragover: move |evt| {
                evt.prevent_default();
                drop_target.set(Some(index));
            },
            ondrop: move |evt| {
                evt.prevent_default();
                if let Some(from) = *drag_index.read() {
                    if from != index {
                        ThingsPageOps::kv_entry_move(input_diagram, on_change, from, index);
                    }
                }
                drag_index.set(None);
                drop_target.set(None);
            },
            ondragend: move |_| {
                drag_index.set(None);
                drop_target.set(None);
            },

            // Drag handle
            DragHandle {}

            input {
                class: ID_INPUT_CLASS,
                style: "max-width:14rem",
                list: "{id_list}",
                placeholder: "id",
                value: "{entry_id}",
                onchange: {
                    let id_old = entry_id.clone();
                    let value = entry_value.clone();
                    move |evt: dioxus::events::FormEvent| {
                        let id_new = evt.value();
                        ThingsPageOps::kv_entry_rename(
                            input_diagram,
                            on_change,
                            &id_old,
                            &id_new,
                            &value,
                        );
                    }
                },
            }

            input {
                class: INPUT_CLASS,
                placeholder: "value",
                value: "{entry_value}",
                oninput: {
                    let entry_id = entry_id.clone();
                    move |evt: dioxus::events::FormEvent| {
                        let new_value = evt.value();
                        ThingsPageOps::kv_entry_update(
                            input_diagram,
                            on_change,
                            &entry_id,
                            &new_value,
                        );
                    }
                },
            }

            span {
                class: REMOVE_BTN,
                onclick: {
                    let entry_id = entry_id.clone();
                    move |_| {
                        ThingsPageOps::kv_entry_remove(input_diagram, on_change, &entry_id);
                    }
                },
                "✕"
            }
        }
    }
}

/// Returns Tailwind border-color classes for the drop-target indicator.
///
/// Always returns **both** `border-t-*` and `border-b-*` colour classes so
/// that there is never a cascade conflict with a competing colour class on
/// the same element (Tailwind v4 orders utilities by property, not by the
/// order they appear in the `class` attribute).
///
/// - When this row is the drop target and the drag source is above, the bottom
///   border turns blue (`border-b-blue-400`) and the top stays transparent.
/// - When the drag source is below, the top border turns blue
///   (`border-t-blue-400`) and the bottom stays transparent.
/// - Otherwise both borders are transparent.
fn drag_row_border_class(
    drag_index: Signal<Option<usize>>,
    drop_target: Signal<Option<usize>>,
    index: usize,
) -> &'static str {
    let drag_src = *drag_index.read();
    let is_target = drop_target.read().map_or(false, |i| i == index);

    if is_target {
        if let Some(from) = drag_src {
            if from != index {
                if from < index {
                    return "border-t-transparent border-b-blue-400";
                } else {
                    return "border-t-blue-400 border-b-transparent";
                }
            }
        }
    }
    "border-t-transparent border-b-transparent"
}

/// A grip-dots drag handle (⠿) that visually indicates a row is draggable.
///
/// The actual drag-and-drop behaviour is handled by the parent row's
/// `draggable` / `ondragstart` / `ondragover` / `ondrop` / `ondragend`
/// attributes; this component is purely visual.
#[component]
fn DragHandle() -> Element {
    rsx! {
        span {
            class: DRAG_HANDLE,
            title: "Drag to reorder",
            "⠿"
        }
    }
}
