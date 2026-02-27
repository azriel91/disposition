//! Collapse bar component.
//!
//! Provides a toggleable bar that collapses or expands a section of rows.

use dioxus::prelude::{
    component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props,
};

use crate::components::editor::common::COLLAPSE_BAR;

/// A clickable bar that toggles between collapsed and expanded states.
///
/// When collapsed the text is on top with a wide V-shaped down chevron below.
/// When expanded the up chevron is on top with the text below.
#[component]
pub fn CollapseBar(
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
