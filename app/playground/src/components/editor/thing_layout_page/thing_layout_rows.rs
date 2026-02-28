//! Container component for thing layout rows.
//!
//! Wraps the list of [`ThingLayoutRow`]s in a focusable container that
//! supports keyboard navigation:
//!
//! - **Tab**: focus the container itself (via the normal tab order).
//! - **Enter** (on container): focus the first `ThingLayoutRow` inside.
//! - **Esc** (on a `ThingLayoutRow`): return focus to the container.
//!
//! The container also manages post-render focus via a `focus_index` signal
//! so that rows retain focus after keyboard-driven reorder operations
//! (Alt+Up/Down, indent/outdent).
//!
//! [`ThingLayoutRow`]: super::thing_layout_row::ThingLayoutRow

use dioxus::{
    document,
    hooks::use_effect,
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Key, Props},
    signals::{Signal, WritableExt},
};

/// JavaScript snippet: focus the first `[tabindex="-1"]` child inside the
/// currently focused element.
const JS_FOCUS_FIRST_ROW: &str = "\
    document.activeElement\
        ?.querySelector('[tabindex=\"-1\"]')\
        ?.focus()";

/// A focusable container for [`ThingLayoutRow`]s.
///
/// # Props
///
/// * `focus_index`: when set to `Some(idx)`, the row at that child index
///   receives focus after the next DOM update. The signal is cleared after the
///   focus is applied.
/// * `children`: the `ThingLayoutRow` elements rendered inside the container.
///
/// [`ThingLayoutRow`]: super::thing_layout_row::ThingLayoutRow
#[component]
pub fn ThingLayoutRows(mut focus_index: Signal<Option<usize>>, children: Element) -> Element {
    // After the DOM re-renders, focus the row identified by `focus_index`.
    use_effect(move || {
        if let Some(idx) = focus_index() {
            focus_index.set(None);
            document::eval(&format!(
                "setTimeout(() => {{\
                    let container = document.querySelector(\
                        '[data-thing-layout-rows]'\
                    );\
                    if (container) {{\
                        let rows = container.querySelectorAll('[tabindex=\"-1\"]');\
                        if (rows[{idx}]) rows[{idx}].focus();\
                    }}\
                }}, 0)"
            ));
        }
    });

    rsx! {
        div {
            class: "\
                flex \
                flex-col \
                rounded-lg \
                border \
                border-gray-700 \
                bg-gray-900 \
                p-2 \
                gap-0 \
                focus:outline-none \
                focus:ring-1 \
                focus:ring-blue-400\
            ",

            tabindex: "0",
            "data-thing-layout-rows": "true",

            // === Keyboard shortcuts (container-level) === //
            onkeydown: move |evt| {
                if evt.key() == Key::Enter {
                    evt.prevent_default();
                    document::eval(JS_FOCUS_FIRST_ROW);
                }
            },

            {children}
        }
    }
}
