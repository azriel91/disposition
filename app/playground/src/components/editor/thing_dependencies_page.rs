//! Thing Dependencies editor page.
//!
//! Allows editing `thing_dependencies` -- a map from `EdgeGroupId` to
//! `EdgeGroup`, where each group has an `EdgeKind` (Cyclic / Sequence /
//! Symmetric) and a list of `ThingId`s.
//!
//! Also provides `ThingInteractionsPage` which shares the same card component
//! and mutation helpers via [`MapTarget`].
//!
//! The heavy lifting is delegated to submodules:
//! - [`edge_group_card`]: collapsible card for a single edge group.
//! - [`edge_group_card_ops`]: mutation helpers for edge group entries.

mod edge_group_card;
mod edge_group_card_ops;

use dioxus::{
    hooks::use_signal,
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal},
};
use disposition::input_model::{edge::EdgeKind, thing::ThingId, InputDiagram};

use crate::components::editor::common::{RenameRefocus, ADD_BTN, INPUT_CLASS, SECTION_HEADING};

use self::{edge_group_card::EdgeGroupCard, edge_group_card_ops::EdgeGroupCardOps};

/// Serialised snapshot of one edge group entry for rendering.
#[derive(Clone, PartialEq)]
pub(crate) struct EdgeGroupEntry {
    pub(crate) edge_group_id: String,
    pub(crate) edge_kind: EdgeKind,
    pub(crate) things: Vec<ThingId<'static>>,
}

// === Shared types === //

/// Which map inside `InputDiagram` we are editing.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum MapTarget {
    Dependencies,
    Interactions,
}

// === Edge group card constants === //

/// The `data-*` attribute placed on each `EdgeGroupCard` wrapper.
///
/// Used by [`keyboard_nav`](crate::components::editor::keyboard_nav) helpers
/// to locate the nearest ancestor card.
pub(crate) const DATA_ATTR: &str = "data-edge-group-card";

/// The `data-*` attribute that holds the card's ID value (for post-rename
/// focus).
pub(crate) const DATA_ID_ATTR: &str = "data-edge-group-card-id";

/// CSS classes for the focusable edge group card wrapper.
///
/// Extends the standard card styling with focus ring and transitions.
pub(crate) const EDGE_GROUP_CARD_CLASS: &str = "\
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
pub(crate) const COLLAPSED_HEADER_CLASS: &str = "\
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
pub(crate) const FIELD_INPUT_CLASS: &str = INPUT_CLASS;

// === ThingDependenciesPage component === //

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
                tabindex: -1,
                onclick: move |_| {
                    EdgeGroupCardOps::edge_group_add(input_diagram, MapTarget::Dependencies);
                },
                "+ Add dependency edge group"
            }
        }
    }
}

// === ThingInteractionsPage component === //

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
                tabindex: -1,
                onclick: move |_| {
                    EdgeGroupCardOps::edge_group_add(input_diagram, MapTarget::Interactions);
                },
                "+ Add interaction edge group"
            }
        }
    }
}
