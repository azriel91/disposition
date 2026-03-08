//! Entity Types editor page.
//!
//! Allows editing `entity_types`: a map from `Id` to `Set<EntityType>`,
//! where each entry associates an entity (thing, edge, tag, process, etc.)
//! with a set of custom types for common styling.
//!
//! The heavy lifting is delegated to submodules:
//!
//! - [`entity_type_card`]: collapsible card for a single entity's type set.

pub(crate) mod entity_type_card;

use dioxus::{
    hooks::use_signal,
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::input_model::InputDiagram;
use disposition_input_rt::EntityTypesPageOps;

use crate::components::editor::{
    common::{RenameRefocus, ADD_BTN, SECTION_HEADING},
    reorderable::ReorderableContainer,
};

use self::entity_type_card::EntityTypeCard;

/// Snapshot of a single entity types entry for rendering.
#[derive(Clone, PartialEq)]
pub(crate) struct EntityTypeEntry {
    /// The entity Id, e.g. `"t_aws"`, `"tag_app_development"`.
    pub(crate) entity_id: String,
    /// The entity type strings, e.g. `["type_organisation", "type_server"]`.
    pub(crate) types: Vec<String>,
}

// === EntityTypeCard constants === //

/// The `data-*` attribute placed on each `EntityTypeCard` wrapper.
///
/// Used by [`keyboard_nav`](crate::components::editor::keyboard_nav) helpers
/// to locate the nearest ancestor card.
pub(crate) const DATA_ATTR: &str = "data-entity-type-card";

/// CSS classes for the focusable entity type card wrapper.
///
/// Extends `CARD_CLASS` with focus ring styling and transitions.
pub(crate) const ENTITY_TYPE_CARD_CLASS: &str = "\
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

/// CSS classes for an input inside an entity type card.
///
/// These elements use `tabindex="-1"` so they are skipped by the normal tab
/// order; the user enters edit mode by pressing Enter on the focused card.
pub(crate) const FIELD_INPUT_CLASS: &str = crate::components::editor::common::INPUT_CLASS;

// === EntityTypesPage component === //

/// The **Entity Types** editor page.
///
/// Provides a reorderable list of entity type cards, each mapping an entity
/// Id to a set of entity types for common styling.
#[component]
pub fn EntityTypesPage(input_diagram: Signal<InputDiagram<'static>>) -> Element {
    // Post-rename focus state for entity type cards.
    let rename_refocus: Signal<Option<RenameRefocus>> = use_signal(|| None);

    // Drag-and-drop state for entity type cards.
    let drag_idx: Signal<Option<usize>> = use_signal(|| None);
    let drop_target: Signal<Option<usize>> = use_signal(|| None);
    // Focus-after-move state for entity type card reorder.
    let focus_idx: Signal<Option<usize>> = use_signal(|| None);

    let diagram = input_diagram.read();

    let entries: Vec<EntityTypeEntry> = diagram
        .entity_types
        .iter()
        .map(|(entity_id, types_set)| {
            let types: Vec<String> = types_set
                .iter()
                .map(|entity_type| entity_type.as_str().to_owned())
                .collect();

            EntityTypeEntry {
                entity_id: entity_id.as_str().to_owned(),
                types,
            }
        })
        .collect();

    drop(diagram);

    let entry_count = entries.len();

    rsx! {
        div {
            class: "flex flex-col gap-2",

            h3 { class: SECTION_HEADING, "Entity Types" }
            p {
                class: "text-xs text-gray-500 mb-1",
                "Assign entity types to things, edges, tags, processes, and process steps for common styling."
            }

            ReorderableContainer {
                data_attr: DATA_ATTR.to_owned(),
                section_id: "entity_types".to_owned(),
                focus_index: focus_idx,
                rename_refocus: Some(rename_refocus),

                for (idx, entry) in entries.iter().enumerate() {
                    {
                        let entry = entry.clone();
                        rsx! {
                            EntityTypeCard {
                                key: "{entry.entity_id}",
                                input_diagram,
                                entry,
                                index: idx,
                                entry_count,
                                drag_index: drag_idx,
                                drop_target,
                                focus_index: focus_idx,
                                rename_refocus,
                            }
                        }
                    }
                }
            }

            button {
                class: ADD_BTN,
                tabindex: 0,
                onclick: move |_| {
                    EntityTypesPageOps::entry_add(&mut input_diagram.write());
                },
                "+ Add entity type mapping"
            }
        }
    }
}
