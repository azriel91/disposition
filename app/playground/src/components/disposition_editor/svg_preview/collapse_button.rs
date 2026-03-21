use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{Signal, WritableExt},
};

use crate::components::disposition_editor::svg_preview::BTN_CLASS;

#[component]
pub(crate) fn CollapseButton(svg_preview_expanded: Signal<bool>) -> Element {
    rsx! {
        button {
            class: BTN_CLASS,
            tabindex: "0",
            title: "Restore editor (Escape / f)",
            onclick: move |_| {
                svg_preview_expanded.set(false);
            },
            CollapseIcon {}
        }
    }
}

/// Collapse icon: arrows pointing inward from the four corners.
#[component]
pub(crate) fn CollapseIcon() -> Element {
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

            // From top-left corner inward.
            polyline { points: "4 14 10 14 10 20" }
            line { x1: "3", y1: "21", x2: "10", y2: "14" }

            // From top-right corner inward.
            polyline { points: "20 14 14 14 14 20" }
            line { x1: "21", y1: "21", x2: "14", y2: "14" }

            // From bottom-left corner inward.
            polyline { points: "4 10 10 10 10 4" }
            line { x1: "3", y1: "3", x2: "10", y2: "10" }

            // From bottom-right corner inward.
            polyline { points: "20 10 14 10 14 4" }
            line { x1: "21", y1: "3", x2: "14", y2: "10" }
        }
    }
}
