//! Container component for draggable ID-value rows.
//!
//! Manages post-render focus via a `focus_index` signal so that rows
//! retain focus after keyboard-driven reorder operations (Alt+Up/Down).
//!
//! Also handles post-rename focus via a `rename_refocus` signal so that
//! after an ID rename destroys and recreates a row element, the correct
//! sub-element (ID input on Enter, next field on Tab) receives focus.

use dioxus::{
    document,
    hooks::use_effect,
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{Signal, WritableExt},
};

use crate::components::editor::common::{RenameRefocus, RenameRefocusTarget};

/// A container for multiple draggable [`IdValueRow`]s.
///
/// Uses `group/key-value-rows` so that child rows can react to an active drag
/// via `group-active/key-value-rows:_` utilities. Does **not** use `gap` on
/// the flex container -- each row carries its own padding instead, so there are
/// no dead-zones between rows where a drop would be missed.
///
/// # Props
///
/// * `section_id`: a unique identifier for this section, used as a
///   `data-section-id` attribute so that the focus JS can locate the correct
///   container. e.g. `"thing_names"`, `"copy_text"`.
/// * `focus_index`: when set to `Some(idx)`, the row at that child index
///   receives focus after the next DOM update.
/// * `rename_refocus`: when set to `Some(refocus)`, the row whose
///   `data-entry-id` matches `refocus.new_id` receives focus after the next DOM
///   update, with the correct sub-element focused based on `refocus.target`.
/// * `children`: the row elements rendered inside the container.
///
/// [`IdValueRow`]: crate::components::editor::id_value_row::IdValueRow
#[component]
pub fn IdValueRowContainer(
    section_id: &'static str,
    mut focus_index: Signal<Option<usize>>,
    mut rename_refocus: Signal<Option<RenameRefocus>>,
    children: Element,
) -> Element {
    // After the DOM re-renders, focus the row identified by `focus_index`.
    use_effect(move || {
        if let Some(idx) = focus_index() {
            focus_index.set(None);
            document::eval(&format!(
                "setTimeout(() => {{\
                    let container = document.querySelector(\
                        '[data-section-id=\"{section_id}\"]'\
                    );\
                    if (container) {{\
                        let row = container.children[{idx}];\
                        if (row) row.focus();\
                    }}\
                }}, 0)"
            ));
        }
    });

    // After an ID rename the old row is destroyed and a new one is created
    // under the new key. Focus the correct sub-element inside that new row.
    use_effect(move || {
        if let Some(RenameRefocus { new_id, target }) = rename_refocus() {
            rename_refocus.set(None);
            // JS: find the row by data-entry-id, then focus:
            // - NextField: the second focusable element (first after the ID input).
            // - IdInput: the first input (the ID input).
            // - FocusParent: the row wrapper itself.
            let js = match target {
                RenameRefocusTarget::NextField => {
                    format!(
                        "setTimeout(() => {{\
                            let container = document.querySelector(\
                                '[data-section-id=\"{section_id}\"]'\
                            );\
                            if (!container) return;\
                            let row = container.querySelector(\
                                '[data-entry-id=\"{new_id}\"]'\
                            );\
                            if (!row) return;\
                            let items = Array.from(\
                                row.querySelectorAll('input, [data-action=\"remove\"]')\
                            );\
                            if (items.length > 1) {{\
                                items[1].focus();\
                            }} else if (items.length === 1) {{\
                                items[0].focus();\
                            }} else {{\
                                row.focus();\
                            }}\
                        }}, 0)"
                    )
                }
                RenameRefocusTarget::IdInput => {
                    format!(
                        "setTimeout(() => {{\
                            let container = document.querySelector(\
                                '[data-section-id=\"{section_id}\"]'\
                            );\
                            if (!container) return;\
                            let row = container.querySelector(\
                                '[data-entry-id=\"{new_id}\"]'\
                            );\
                            if (!row) return;\
                            let input = row.querySelector('input');\
                            if (input) {{\
                                input.focus();\
                            }} else {{\
                                row.focus();\
                            }}\
                        }}, 0)"
                    )
                }
                RenameRefocusTarget::FocusParent => {
                    format!(
                        "setTimeout(() => {{\
                            let container = document.querySelector(\
                                '[data-section-id=\"{section_id}\"]'\
                            );\
                            if (!container) return;\
                            let row = container.querySelector(\
                                '[data-entry-id=\"{new_id}\"]'\
                            );\
                            if (!row) return;\
                            row.focus();\
                        }}, 0)"
                    )
                }
            };
            document::eval(&js);
        }
    });

    rsx! {
        div {
            class: "flex flex-col group/key-value-rows",
            "data-section-id": section_id,
            {children}
        }
    }
}
