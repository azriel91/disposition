//! Shared reorderable component helpers.
//!
//! Provides the visual and behavioural building blocks that are common to
//! all reorderable entries in the editor: `IdValueRow`, `EdgeGroupCard`,
//! `TagThingsCard`, `ProcessCard`, and `CssClassPartialsCard`.
//!
//! ## Contents
//!
//! - [`DragHandle`]: a purely-visual grip indicator for draggable entries.
//! - [`drag_border_class`]: computes Tailwind border classes for the
//!   drop-target indicator during drag-and-drop.
//! - [`ReorderableContainer`]: a wrapper component that manages post-reorder
//!   focus via a `focus_index` signal, analogous to `IdValueRowContainer` but
//!   usable by any reorderable entry list.

use dioxus::{
    document,
    hooks::use_effect,
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal, WritableExt},
};

use crate::components::editor::common::DRAG_HANDLE;

// === DragHandle === //

/// A grip-dots drag handle that visually indicates an entry is draggable.
///
/// The actual drag-and-drop behaviour is handled by the parent element's
/// `draggable` / `ondragstart` / `ondragover` / `ondrop` / `ondragend`
/// attributes; this component is purely visual.
///
/// Used by both `IdValueRow` rows and `*Card` components.
#[component]
pub fn DragHandle() -> Element {
    rsx! {
        span {
            class: DRAG_HANDLE,
            title: "Drag to reorder",
            "\u{283F}"
        }
    }
}

// === drag_border_class === //

/// Returns Tailwind border-color classes for the drop-target indicator.
///
/// Always returns **both** `border-t-*` and `border-b-*` colour classes so
/// that there is never a cascade conflict with a competing colour class on
/// the same element (Tailwind v4 orders utilities by property, not by the
/// order they appear in the `class` attribute).
///
/// - When this entry is the drop target and the drag source is above, the
///   bottom border turns blue (`border-b-blue-400`) and the top stays
///   transparent.
/// - When the drag source is below, the top border turns blue
///   (`border-t-blue-400`) and the bottom stays transparent.
/// - Otherwise both borders are transparent.
pub fn drag_border_class(
    drag_index: Signal<Option<usize>>,
    drop_target: Signal<Option<usize>>,
    index: usize,
) -> &'static str {
    let drag_src = *drag_index.read();
    let is_target = drop_target.read().is_some_and(|i| i == index);

    if is_target
        && let Some(from) = drag_src
        && from != index
    {
        if from < index {
            return "border-t-transparent border-b-blue-400";
        } else {
            return "border-t-blue-400 border-b-transparent";
        }
    }
    "border-t-transparent border-b-transparent"
}

// === ReorderableContainer === //

/// A container for a list of reorderable entries (cards or rows).
///
/// After a keyboard-driven reorder (Alt+Up / Alt+Down) the entry at the
/// new position should receive focus. This component watches the
/// `focus_index` signal and, when set, focuses the child element at that
/// index after the DOM has updated.
///
/// # Props
///
/// * `data_attr`: the `data-*` attribute name used by the child entries, e.g.
///   `"data-edge-group-card"`, `"data-entry-id"`. Used to select only the
///   direct reorderable children when computing focus targets.
/// * `section_id`: a unique identifier for this section, used as a
///   `data-reorderable-section` attribute so focus JS can locate the correct
///   container. e.g. `"edge_groups_deps"`, `"tag_things"`.
/// * `focus_index`: when set to `Some(idx)`, the entry at child index `idx`
///   (counting only children that have `data_attr`) receives focus after the
///   next DOM update.
/// * `children`: the entry elements rendered inside the container.
#[component]
pub fn ReorderableContainer(
    data_attr: String,
    section_id: String,
    mut focus_index: Signal<Option<usize>>,
    children: Element,
) -> Element {
    let section_id_effect = section_id.clone();
    let data_attr_effect = data_attr.clone();

    use_effect(move || {
        if let Some(idx) = focus_index() {
            focus_index.set(None);
            document::eval(&format!(
                "setTimeout(() => {{\
                    let container = document.querySelector(\
                        '[data-reorderable-section=\"{section_id_effect}\"]'\
                    );\
                    if (!container) return;\
                    let entries = Array.from(\
                        container.querySelectorAll('[{data_attr_effect}]')\
                    );\
                    if (entries[{idx}]) entries[{idx}].focus();\
                }}, 0)"
            ));
        }
    });

    rsx! {
        div {
            class: "flex flex-col",
            "data-reorderable-section": "{section_id}",
            {children}
        }
    }
}
