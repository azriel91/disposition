//! Collapse bar component.
//!
//! Provides a toggleable bar that collapses or expands a section of rows.
//!
//! Supports keyboard navigation:
//!
//! - **Enter / Space**: toggle the collapse state (native button behaviour).
//! - **Up arrow**: focus the last row in the preceding `KeyValueRowContainer`
//!   (if any).
//! - **Down arrow**: focus the first row in the following
//!   `KeyValueRowContainer` (if any).

use dioxus::{
    document,
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Key, Props},
};

use crate::components::editor::common::COLLAPSE_BAR;

/// CSS classes appended to the collapse bar button for keyboard focus
/// visibility.
const COLLAPSE_BAR_FOCUS: &str = "\
    focus:outline-none \
    focus:ring-1 \
    focus:ring-blue-400\
";

/// A clickable bar that toggles between collapsed and expanded states.
///
/// When collapsed the text is on top with a wide V-shaped down chevron below.
/// When expanded the up chevron is on top with the text below.
///
/// The bar is a `<button>`, so it is natively focusable and activatable via
/// Enter / Space. Arrow Up focuses the last child of the previous sibling
/// container; Arrow Down focuses the first child of the next sibling
/// container.
#[component]
pub fn CollapseBar(
    collapsed: bool,
    total: usize,
    visible: usize,
    on_toggle: dioxus::prelude::EventHandler<dioxus::events::MouseEvent>,
) -> Element {
    let hidden = total.saturating_sub(visible);
    let label = if collapsed {
        format!("... {hidden} more")
    } else {
        String::from("...")
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

    let bar_class = format!("{COLLAPSE_BAR} {COLLAPSE_BAR_FOCUS}");

    rsx! {
        button {
            class: "{bar_class}",
            onclick: move |evt| on_toggle.call(evt),

            onkeydown: move |evt| {
                match evt.key() {
                    Key::ArrowUp => {
                        evt.prevent_default();
                        evt.stop_propagation();
                        // Focus the last focusable child of the previous
                        // sibling element (typically a KeyValueRowContainer).
                        document::eval(
                            "(() => {\
                                let bar = document.activeElement;\
                                if (!bar) return;\
                                let prev = bar.previousElementSibling;\
                                while (prev) {\
                                    let children = prev.querySelectorAll('[tabindex=\"0\"]');\
                                    if (children.length > 0) {\
                                        children[children.length - 1].focus();\
                                        return;\
                                    }\
                                    prev = prev.previousElementSibling;\
                                }\
                            })()"
                        );
                    }
                    Key::ArrowDown => {
                        evt.prevent_default();
                        evt.stop_propagation();
                        // Focus the first focusable child of the next sibling
                        // element (typically a KeyValueRowContainer).
                        document::eval(
                            "(() => {\
                                let bar = document.activeElement;\
                                if (!bar) return;\
                                let next = bar.nextElementSibling;\
                                while (next) {\
                                    let child = next.querySelector('[tabindex=\"0\"]');\
                                    if (child) {\
                                        child.focus();\
                                        return;\
                                    }\
                                    next = next.nextElementSibling;\
                                }\
                            })()"
                        );
                    }
                    _ => {}
                }
            },

            if collapsed {
                // Text on top, V arrow below.
                span {
                    class: "text-xs tracking-widest",
                    "{label}"
                }
                span {
                    class: "{chevron_down_class}",
                }
            } else {
                // ^ arrow on top, text below.
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
