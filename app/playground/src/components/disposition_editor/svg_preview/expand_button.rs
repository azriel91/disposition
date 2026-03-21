use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{Signal, WritableExt},
};

use crate::components::disposition_editor::svg_preview::BTN_CLASS;

#[component]
pub(crate) fn ExpandButton(svg_preview_expanded: Signal<bool>) -> Element {
    rsx! {
        button {
            class: BTN_CLASS,
            tabindex: "0",
            title: "Expand SVG preview (f)",
            onclick: move |_| {
                svg_preview_expanded.set(true);
            },
            ExpandIcon {}
        }
    }
}

/// Expand icon: arrows pointing outward to the four corners.
#[component]
pub(crate) fn ExpandIcon() -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            class: "h-5 w-5",
            width: "24",
            height: "24",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",

            // Top-left corner.
            polyline { points: "8 3 3 3 3 8" }
            line { x1: "3", y1: "3", x2: "10", y2: "10" }

            // Top-right corner.
            polyline { points: "16 3 21 3 21 8" }
            line { x1: "21", y1: "3", x2: "14", y2: "10" }

            // Bottom-left corner.
            polyline { points: "8 21 3 21 3 16" }
            line { x1: "3", y1: "21", x2: "10", y2: "14" }

            // Bottom-right corner.
            polyline { points: "16 21 21 21 21 16" }
            line { x1: "21", y1: "21", x2: "14", y2: "14" }
        }
    }
}
