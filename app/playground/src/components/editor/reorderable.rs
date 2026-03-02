//! Shared reorderable component helpers.
//!
//! Provides the visual and behavioural building blocks that are common to
//! all reorderable entries in the editor: [`IdValueRow`], [`EdgeGroupCard`],
//! [`TagThingsCard`], [`ProcessCard`], [`CssClassPartialsCard`], and
//! [`StyleAliasesSection`].
//!
//! ## Contents
//!
//! - [`DragHandle`]: a purely-visual grip indicator for draggable entries.
//! - [`drag_border_class`]: computes Tailwind border classes for the
//!   drop-target indicator during drag-and-drop.
//! - [`is_rename_target`]: checks whether a `rename_refocus` signal targets a
//!   specific entry ID. Used by collapsible cards to initialise their
//!   `collapsed` state to `false` when the card was just recreated via a
//!   rename.
//! - [`ReorderableContainer`]: a wrapper component that manages post-reorder
//!   focus via a `focus_index` signal and optional post-rename focus via a
//!   `rename_refocus` signal. Subsumes the former `IdValueRowContainer` and is
//!   usable by any reorderable entry list.
//!
//! [`IdValueRow`]: crate::components::editor::id_value_row::IdValueRow
//! [`EdgeGroupCard`]: crate::components::editor::thing_dependencies_page::edge_group_card::EdgeGroupCard
//! [`TagThingsCard`]: crate::components::editor::tags_page::tag_things_card::TagThingsCard
//! [`ProcessCard`]: crate::components::editor::processes_page::process_card::ProcessCard
//! [`CssClassPartialsCard`]: crate::components::editor::theme_styles_editor::css_class_partials_card::CssClassPartialsCard
//! [`StyleAliasesSection`]: crate::components::editor::theme_page::style_aliases_section::StyleAliasesSection

use dioxus::{
    document,
    hooks::use_effect,
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal, WritableExt},
};

use crate::components::editor::common::{RenameRefocus, RenameRefocusTarget, DRAG_HANDLE};

// === DragHandle === //

/// A grip-dots drag handle that visually indicates an entry is draggable.
///
/// The actual drag-and-drop behaviour is handled by the parent element's
/// `draggable` / `ondragstart` / `ondragover` / `ondrop` / `ondragend`
/// attributes; this component is purely visual.
///
/// Used by both [`IdValueRow`] rows and `*Card` components.
///
/// [`IdValueRow`]: crate::components::editor::id_value_row::IdValueRow
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

// === is_rename_target === //

/// Returns `true` if the `rename_refocus` signal currently targets `entry_id`.
///
/// Collapsible card components call this during render (before effects run)
/// to decide their initial `collapsed` state. When a card is recreated
/// after an ID rename, the signal still holds the rename info at render
/// time; the [`ReorderableContainer`]'s `use_effect` clears it and handles
/// DOM focus afterwards.
///
/// # Example
///
/// ```rust,ignore
/// let mut collapsed = use_signal(|| {
///     !is_rename_target(rename_refocus, &my_id)
/// });
/// ```
pub fn is_rename_target(rename_refocus: Signal<Option<RenameRefocus>>, entry_id: &str) -> bool {
    rename_refocus
        .read()
        .as_ref()
        .is_some_and(|r| r.new_id == entry_id)
}

// === ReorderableContainer === //

/// A container for a list of reorderable entries (cards or rows).
///
/// After a keyboard-driven reorder (Alt+Up / Alt+Down) the entry at the
/// new position should receive focus. This component watches the
/// `focus_index` signal and, when set, focuses the child element at that
/// index after the DOM has updated.
///
/// Optionally handles post-rename focus via `rename_refocus` and
/// `data_id_attr`. When an ID rename destroys and recreates a child
/// element, the container locates the new element by its `data_id_attr`
/// value and focuses the appropriate sub-element (ID input, next field,
/// or the entry wrapper itself) based on the [`RenameRefocusTarget`].
///
/// # Props
///
/// * `data_attr`: the `data-*` attribute name used by the child entries, e.g.
///   `"data-edge-group-card"`, `"data-entry-id"`. Used to select only the
///   direct reorderable children when computing focus targets.
/// * `section_id`: a unique identifier for this section, used as a
///   `data-reorderable-section` attribute so focus JS can locate the correct
///   container. e.g. `"edge_groups_deps"`, `"tag_things"`, `"thing_names"`.
/// * `focus_index`: when set to `Some(idx)`, the entry at child index `idx`
///   (counting only children that have `data_attr`) receives focus after the
///   next DOM update.
/// * `data_id_attr`: optional `data-*` attribute name that holds the entry's ID
///   value, e.g. `"data-entry-id"`. Required when `rename_refocus` is provided.
///   Used to locate the newly created entry after an ID rename.
/// * `rename_refocus`: optional signal for post-rename focus. When set to
///   `Some(refocus)`, the entry whose `data_id_attr` matches `refocus.new_id`
///   receives focus after the next DOM update, with the correct sub-element
///   focused based on `refocus.target`.
/// * `children`: the entry elements rendered inside the container.
///
/// [`IdValueRow`]: crate::components::editor::id_value_row::IdValueRow
#[component]
pub fn ReorderableContainer(
    data_attr: String,
    section_id: String,
    mut focus_index: Signal<Option<usize>>,
    #[props(default)] data_id_attr: Option<String>,
    #[props(default)] mut rename_refocus: Option<Signal<Option<RenameRefocus>>>,
    children: Element,
) -> Element {
    // === Post-reorder focus effect === //
    let section_id_focus = section_id.clone();
    let data_attr_focus = data_attr.clone();

    use_effect(move || {
        if let Some(idx) = focus_index() {
            focus_index.set(None);
            document::eval(&format!(
                "setTimeout(() => {{\
                    let container = document.querySelector(\
                        '[data-reorderable-section=\"{section_id_focus}\"]'\
                    );\
                    if (!container) return;\
                    let entries = Array.from(\
                        container.querySelectorAll('[{data_attr_focus}]')\
                    );\
                    if (entries[{idx}]) entries[{idx}].focus();\
                }}, 0)"
            ));
        }
    });

    // === Post-rename focus effect === //
    let section_id_rename = section_id.clone();
    let data_id_attr_rename = data_id_attr.clone();

    use_effect(move || {
        let Some(ref mut rename_signal) = rename_refocus else {
            return;
        };
        let Some(data_id_attr) = data_id_attr_rename.as_deref() else {
            return;
        };

        if let Some(RenameRefocus { new_id, target }) = rename_signal() {
            rename_signal.set(None);

            let section_sel = format!("[data-reorderable-section=\"{section_id_rename}\"]");
            let entry_sel = format!("[{data_id_attr}=\"{new_id}\"]");

            // Selector for focusable fields inside the entry.
            let focusable_sel = "input, select, textarea, button, [data-action=\"remove\"]";

            let js = match target {
                RenameRefocusTarget::NextField => {
                    format!(
                        "setTimeout(() => {{\
                            let container = document.querySelector('{section_sel}');\
                            if (!container) return;\
                            let entry = container.querySelector('{entry_sel}');\
                            if (!entry) return;\
                            let items = Array.from(\
                                entry.querySelectorAll('{focusable_sel}')\
                            );\
                            if (items.length > 1) {{\
                                items[1].focus();\
                            }} else if (items.length === 1) {{\
                                items[0].focus();\
                            }} else {{\
                                entry.focus();\
                            }}\
                        }}, 0)"
                    )
                }
                RenameRefocusTarget::IdInput => {
                    format!(
                        "setTimeout(() => {{\
                            let container = document.querySelector('{section_sel}');\
                            if (!container) return;\
                            let entry = container.querySelector('{entry_sel}');\
                            if (!entry) return;\
                            let input = entry.querySelector('input');\
                            if (input) {{\
                                input.focus();\
                            }} else {{\
                                entry.focus();\
                            }}\
                        }}, 0)"
                    )
                }
                RenameRefocusTarget::FocusParent => {
                    format!(
                        "setTimeout(() => {{\
                            let container = document.querySelector('{section_sel}');\
                            if (!container) return;\
                            let entry = container.querySelector('{entry_sel}');\
                            if (!entry) return;\
                            entry.focus();\
                        }}, 0)"
                    )
                }
            };

            document::eval(&js);
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
