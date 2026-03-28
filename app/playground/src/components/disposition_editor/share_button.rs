//! Share button component for the disposition editor.
//!
//! Renders a small button with a share icon (arrow leaving a box) that, when
//! clicked, opens the share modal.

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{Signal, WritableExt},
};

/// CSS classes for the share button -- mirrors `CopyButton` styling.
const SHARE_BTN_CLASS: &str = "\
    flex-none \
    flex \
    justify-center \
    items-center \
    p-2 \
    text-gray-200 \
    rounded-lg \
    bg-gray-800 \
    border-gray-600 \
    hover:bg-gray-700 \
    active:bg-gray-800 \
    focus:outline-none \
    focus:ring-2 \
    focus:ring-blue-600 \
    focus:ring-offset-2 \
    focus:ring-offset-gray-800\
";

/// A button that opens the share modal.
///
/// # Props
///
/// * `show_share_modal`: signal to set to `true` when clicked.
#[component]
pub fn ShareButton(mut show_share_modal: Signal<bool>) -> Element {
    rsx! {
        button {
            class: SHARE_BTN_CLASS,
            tabindex: "0",
            title: "Share link (Ctrl + Shift + S)",
            onclick: move |_| {
                show_share_modal.set(true);
            },

            ShareIcon {}
        }
    }
}

/// Share icon: an arrow pointing up-right leaving a rounded box.
#[component]
fn ShareIcon() -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            class: "\
                h-5 \
                w-5\
            ",
            width: "24",
            height: "24",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",

            // Box (open at top-right corner).
            path {
                d: "\
                    M 4 12 \
                    V 20 \
                    a 2 2 0 0 0 2 2 \
                    H 18 \
                    a 2 2 0 0 0 2 -2 \
                    V 12\
                ",
            }

            // Arrow shaft going up from center.
            line {
                x1: "12",
                y1: "2",
                x2: "12",
                y2: "15",
            }

            // Arrow head.
            polyline {
                points: "8 6 12 2 16 6",
            }
        }
    }
}
