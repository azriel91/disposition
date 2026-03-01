//! Thing Dependencies editor page.
//!
//! Allows editing `thing_dependencies` -- a map from `EdgeGroupId` to
//! `EdgeGroup`, where each group has an `EdgeKind` (Cyclic / Sequence /
//! Symmetric) and a list of `ThingId`s.
//!
//! Also provides `ThingInteractionsPage` which shares the same card component
//! and mutation helpers via [`MapTarget`].

use dioxus::{
    document,
    hooks::{use_effect, use_signal},
    prelude::{
        component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Key,
        ModifiersInteraction, Props,
    },
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
        id_rename_in_input_diagram, parse_edge_group_id, parse_thing_id, RenameRefocus, ADD_BTN,
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
    // Post-rename focus state for edge group cards.
    let edge_group_rename_refocus: Signal<Option<RenameRefocus>> = use_signal(|| None);

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
                            rename_refocus: edge_group_rename_refocus,
                        }
                    }
                }
            }

            button {
                class: ADD_BTN,
                tabindex: 0,
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
    // Post-rename focus state for edge group cards.
    let edge_group_rename_refocus: Signal<Option<RenameRefocus>> = use_signal(|| None);

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
                            rename_refocus: edge_group_rename_refocus,
                        }
                    }
                }
            }

            button {
                class: ADD_BTN,
                tabindex: 0,
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

// === EdgeGroupCard JS helpers === //

/// JavaScript snippet: focus the parent `[data-edge-group-card]` ancestor.
const JS_FOCUS_PARENT_CARD: &str = "\
    document.activeElement\
        ?.closest('[data-edge-group-card]')\
        ?.focus()";

/// JavaScript snippet: Tab to the next focusable element (input, select, or
/// `[data-action="remove"]`) within the same `[data-edge-group-card]`.
const JS_TAB_NEXT_FIELD: &str = "\
    (() => {\
        let el = document.activeElement;\
        if (!el) return;\
        let card = el.closest('[data-edge-group-card]');\
        if (!card) return;\
        let items = Array.from(card.querySelectorAll(\
            'input, select, button, [data-action=\"remove\"]'\
        ));\
        let idx = items.indexOf(el);\
        if (idx >= 0 && idx + 1 < items.length) {\
            items[idx + 1].focus();\
        } else {\
            card.focus();\
        }\
    })()";

/// JavaScript snippet: Shift+Tab to the previous focusable element within the
/// same `[data-edge-group-card]`.
const JS_TAB_PREV_FIELD: &str = "\
    (() => {\
        let el = document.activeElement;\
        if (!el) return;\
        let card = el.closest('[data-edge-group-card]');\
        if (!card) return;\
        let items = Array.from(card.querySelectorAll(\
            'input, select, button, [data-action=\"remove\"]'\
        ));\
        let idx = items.indexOf(el);\
        if (idx > 0) {\
            items[idx - 1].focus();\
        } else {\
            card.focus();\
        }\
    })()";

// === Edge group card CSS === //

/// JavaScript snippet: focus the previous sibling `[data-edge-group-card]`.
const JS_FOCUS_PREV_CARD: &str = "\
    (() => {\
        let el = document.activeElement;\
        if (!el) return;\
        let card = el.closest('[data-edge-group-card]') || el;\
        let prev = card.previousElementSibling;\
        while (prev) {\
            if (prev.hasAttribute && prev.hasAttribute('data-edge-group-card')) {\
                prev.focus();\
                return;\
            }\
            prev = prev.previousElementSibling;\
        }\
    })()";

/// JavaScript snippet: focus the next sibling `[data-edge-group-card]`.
const JS_FOCUS_NEXT_CARD: &str = "\
    (() => {\
        let el = document.activeElement;\
        if (!el) return;\
        let card = el.closest('[data-edge-group-card]') || el;\
        let next = card.nextElementSibling;\
        while (next) {\
            if (next.hasAttribute && next.hasAttribute('data-edge-group-card')) {\
                next.focus();\
                return;\
            }\
            next = next.nextElementSibling;\
        }\
    })()";

/// CSS classes for the focusable edge group card wrapper.
///
/// Extends the standard card styling with focus ring and transitions.
const EDGE_GROUP_CARD_CLASS: &str = "\
    rounded-lg \
    border \
    border-gray-700 \
    bg-gray-900 \
    p-3 \
    mb-2 \
    flex \
    flex-col \
    gap-2 \
    focus:outline-none \
    focus:ring-1 \
    focus:ring-blue-400 \
    transition-all \
    duration-150\
";

/// CSS classes for the collapsed summary header.
const COLLAPSED_HEADER_CLASS: &str = "\
    flex \
    flex-row \
    items-center \
    gap-3 \
    cursor-pointer \
    select-none\
";

/// CSS classes for an input inside an edge group card.
///
/// These elements use `tabindex="-1"` so they are skipped by the normal tab
/// order; the user enters edit mode by pressing Enter on the focused card.
const FIELD_INPUT_CLASS: &str = INPUT_CLASS;

// === Edge group card component === //

/// A card for a single edge group (used by both dependencies and interactions).
///
/// Supports keyboard shortcuts:
///
/// - **ArrowRight**: expand the card (when collapsed).
/// - **ArrowLeft**: collapse the card (when expanded).
/// - **Enter**: expand + focus the first input inside the card.
/// - **Tab / Shift+Tab** (inside a field): cycle through focusable fields
///   within the card.
/// - **Esc** (inside a field): return focus to the card wrapper.
///
/// When collapsed, shows the edge group ID, kind, and number of things.
#[component]
fn EdgeGroupCard(
    input_diagram: Signal<InputDiagram<'static>>,
    entry: EdgeGroupEntry,
    target: MapTarget,
    mut rename_refocus: Signal<Option<RenameRefocus>>,
) -> Element {
    let edge_group_id = entry.edge_group_id.clone();
    let edge_kind = entry.edge_kind;
    let things = entry.things.clone();
    let mut collapsed = use_signal(|| true);
    // Tracks whether Tab (true) or Enter/blur (false) triggered the last ID
    // input change, so we know which field to focus after re-render.
    let mut id_input_tab_pressed = use_signal(|| false);

    // Clone before moving into the closure so `edge_group_id` remains
    // available for the `rsx!` block below.
    let edge_group_id_for_effect = edge_group_id.clone();

    // After an ID rename this card is destroyed and recreated under the new
    // key. If the rename_refocus signal carries our new ID, focus the correct
    // sub-element once the DOM has settled.
    use_effect(move || {
        let refocus = rename_refocus.read().clone();
        if let Some(RenameRefocus {
            new_id,
            tab_pressed,
        }) = refocus
        {
            if new_id == edge_group_id_for_effect {
                rename_refocus.set(None);
                let js = if tab_pressed {
                    format!(
                        "setTimeout(() => {{\
                            let card = document.querySelector(\
                                '[data-edge-group-card-id=\"{new_id}\"]'\
                            );\
                            if (!card) return;\
                            let items = Array.from(\
                                card.querySelectorAll(\
                                    'input, select, button, [data-action=\"remove\"]'\
                                )\
                            );\
                            if (items.length > 1) {{\
                                items[1].focus();\
                            }} else if (items.length === 1) {{\
                                items[0].focus();\
                            }} else {{\
                                card.focus();\
                            }}\
                        }}, 0)"
                    )
                } else {
                    format!(
                        "setTimeout(() => {{\
                            let card = document.querySelector(\
                                '[data-edge-group-card-id=\"{new_id}\"]'\
                            );\
                            if (!card) return;\
                            let input = card.querySelector('input');\
                            if (input) {{\
                                input.focus();\
                            }} else {{\
                                card.focus();\
                            }}\
                        }}, 0)"
                    )
                };
                document::eval(&js);
            }
        }
    });

    let thing_count = things.len();
    let thing_suffix = if thing_count != 1 { "s" } else { "" };
    let kind_label = match edge_kind {
        EdgeKind::Cyclic => "cyclic",
        EdgeKind::Sequence => "sequence",
        EdgeKind::Symmetric => "symmetric",
    };

    rsx! {
        div {
            class: EDGE_GROUP_CARD_CLASS,
            tabindex: "0",
            "data-edge-group-card": "true",

            // === Card identity for post-rename focus === //
            "data-edge-group-card-id": "{edge_group_id}",

            // === Card-level keyboard shortcuts === //
            onkeydown: move |evt| {
                match evt.key() {
                    Key::ArrowUp => {
                        evt.prevent_default();
                        document::eval(JS_FOCUS_PREV_CARD);
                    }
                    Key::ArrowDown => {
                        evt.prevent_default();
                        document::eval(JS_FOCUS_NEXT_CARD);
                    }
                    Key::ArrowLeft => {
                        evt.prevent_default();
                        collapsed.set(true);
                    }
                    Key::ArrowRight => {
                        evt.prevent_default();
                        collapsed.set(false);
                    }
                    Key::Character(ref c) if c == " " => {
                        evt.prevent_default();
                        let is_collapsed = *collapsed.read();
                        collapsed.set(!is_collapsed);
                    }
                    Key::Enter => {
                        evt.prevent_default();
                        collapsed.set(false);
                        document::eval(
                            "setTimeout(() => {\
                                document.activeElement\
                                    ?.querySelector('input, select')\
                                    ?.focus();\
                            }, 0)"
                        );
                    }
                    _ => {}
                }
            },

            if *collapsed.read() {
                // === Collapsed summary === //
                div {
                    class: COLLAPSED_HEADER_CLASS,
                    onclick: move |_| collapsed.set(false),

                    // Expand chevron
                    span {
                        class: "text-gray-500 text-xs",
                        ">"
                    }

                    span {
                        class: "text-sm font-mono text-blue-400",
                        "{edge_group_id}"
                    }

                    span {
                        class: "text-xs text-gray-500 italic",
                        "{kind_label}"
                    }

                    span {
                        class: "text-xs text-gray-500",
                        "({thing_count} thing{thing_suffix})"
                    }
                }
            } else {
                // === Expanded content === //

                // Collapse toggle
                div {
                    class: "flex flex-row items-center gap-1 cursor-pointer select-none mb-1",
                    onclick: move |_| collapsed.set(true),

                    span {
                        class: "text-gray-500 text-xs rotate-90 inline-block",
                        ">"
                    }
                    span {
                        class: "text-xs text-gray-500",
                        "Collapse"
                    }
                }

                // === EdgeGroupId + Remove === //
                div {
                    class: ROW_CLASS_SIMPLE,

                    input {
                        class: FIELD_INPUT_CLASS,
                        style: "max-width:16rem",
                        tabindex: "-1",
                        list: list_ids::EDGE_GROUP_IDS,
                        placeholder: "edge_group_id",
                        value: "{edge_group_id}",
                        onchange: {
                            let edge_group_id_old = edge_group_id.clone();
                            move |evt: dioxus::events::FormEvent| {
                                let id_new = evt.value();
                                let tab_pressed = *id_input_tab_pressed.read();
                                EdgeGroupCardOps::edge_group_rename(
                                    input_diagram,
                                    &edge_group_id_old,
                                    &id_new,
                                );
                                rename_refocus.set(Some(RenameRefocus {
                                    new_id: id_new,
                                    tab_pressed,
                                }));
                            }
                        },
                        onkeydown: move |evt| {
                            match evt.key() {
                                Key::Tab => id_input_tab_pressed.set(!evt.modifiers().shift()),
                                Key::Enter => id_input_tab_pressed.set(false),
                                _ => {}
                            }
                            edge_group_card_field_keydown(evt);
                        },
                    }

                    button {
                        class: REMOVE_BTN,
                        tabindex: "-1",
                        "data-action": "remove",
                        onclick: {
                            let edge_group_id = edge_group_id.clone();
                            move |_| {
                                EdgeGroupCardOps::edge_group_remove(input_diagram, target, &edge_group_id);
                            }
                        },
                        onkeydown: move |evt| {
                            edge_group_card_field_keydown(evt);
                        },
                        "x Remove"
                    }
                }

                // === kind === //
                div {
                    class: "flex flex-col items-start gap-1 pl-4",

                    label { class: LABEL_CLASS, "kind" }

                    select {
                        class: SELECT_CLASS,
                        tabindex: "-1",
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
                        onkeydown: move |evt| {
                            edge_group_card_field_keydown(evt);
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
                                        class: FIELD_INPUT_CLASS,
                                        style: "max-width:14rem",
                                        tabindex: "-1",
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
                                        onkeydown: move |evt| {
                                            edge_group_card_field_keydown(evt);
                                        },
                                    }

                                    button {
                                        class: REMOVE_BTN,
                                        tabindex: "-1",
                                        "data-action": "remove",
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
                                        onkeydown: move |evt| {
                                            edge_group_card_field_keydown(evt);
                                        },
                                        "x"
                                    }
                                }
                            }
                        }
                    }

                    button {
                        class: ADD_BTN,
                        tabindex: -1,
                        onclick: {
                            let edge_group_id = edge_group_id.clone();
                            move |_| {
                                EdgeGroupCardOps::edge_thing_add(input_diagram, target, &edge_group_id);
                            }
                        },
                        onkeydown: move |evt| {
                            edge_group_card_field_keydown(evt);
                        },
                        "+ Add thing"
                    }
                }
            }
        }
    }
}

/// Shared `onkeydown` handler for inputs, selects, and remove buttons inside
/// an `EdgeGroupCard`.
///
/// - **Esc**: return focus to the parent `EdgeGroupCard`.
/// - **Tab / Shift+Tab**: cycle through focusable fields within the card.
/// - **ArrowUp / ArrowDown / ArrowLeft / ArrowRight**: stop propagation so the
///   card-level handler does not fire (allows cursor movement in text inputs
///   and select navigation).
fn edge_group_card_field_keydown(evt: dioxus::events::KeyboardEvent) {
    let shift = evt.modifiers().shift();
    match evt.key() {
        Key::Escape => {
            evt.prevent_default();
            evt.stop_propagation();
            document::eval(JS_FOCUS_PARENT_CARD);
        }
        Key::Tab => {
            evt.prevent_default();
            evt.stop_propagation();
            if shift {
                document::eval(JS_TAB_PREV_FIELD);
            } else {
                document::eval(JS_TAB_NEXT_FIELD);
            }
        }
        Key::ArrowUp | Key::ArrowDown | Key::ArrowLeft | Key::ArrowRight => {
            evt.stop_propagation();
        }
        Key::Character(ref c) if c == " " => {
            // Prevents the parent card from collapsing.
            evt.stop_propagation();
        }
        _ => {}
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
                input_diagram.thing_dependencies.shift_remove(edge_group_id);
            }
            MapTarget::Interactions => {
                input_diagram.thing_interactions.shift_remove(edge_group_id);
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
