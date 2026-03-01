//! Container component for draggable key-value rows.
//!
//! Manages post-render focus via a `focus_index` signal so that rows
//! retain focus after keyboard-driven reorder operations (Alt+Up/Down).

use dioxus::{
    document,
    hooks::use_effect,
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{Signal, WritableExt},
};

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
/// * `children`: the row elements rendered inside the container.
///
/// [`IdValueRow`]: crate::components::editor::id_value_row::IdValueRow
#[component]
pub fn KeyValueRowContainer(
    section_id: &'static str,
    mut focus_index: Signal<Option<usize>>,
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

    rsx! {
        div {
            class: "flex flex-col group/key-value-rows",
            "data-section-id": section_id,
            {children}
        }
    }
}
