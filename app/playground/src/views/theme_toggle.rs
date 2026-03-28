//! Theme toggle button component.
//!
//! Renders a button with a sun icon (light mode) or a moon icon (dark mode).
//! Clicking the button toggles between the two colour schemes.

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element},
    signals::{ReadableExt, Signal},
};

use crate::hooks::{dark_mode_toggle, use_dark_mode};

/// CSS classes for the toggle button.
const TOGGLE_BTN: &str = "\
    flex \
    items-center \
    justify-center \
    w-8 \
    h-8 \
    rounded-lg \
    border \
    border-gray-600 \
    cursor-pointer \
    select-none \
    transition-colors \
    duration-150 \
    hover:bg-gray-700 \
    focus:outline-none \
    focus:ring-1 \
    focus:ring-blue-400\
";

/// A button that toggles between light and dark colour schemes.
///
/// Displays a sun icon when the current theme is light, and a moon icon when
/// the current theme is dark. Clicking the button switches to the other
/// scheme and persists the choice to `localStorage`.
#[component]
pub fn ThemeToggle() -> Element {
    let is_dark: Signal<bool> = use_dark_mode();
    let dark = *is_dark.read();

    rsx! {
        button {
            class: TOGGLE_BTN,
            title: if dark { "Switch to light mode" } else { "Switch to dark mode" },
            "aria-label": if dark { "Switch to light mode" } else { "Switch to dark mode" },
            onclick: move |_| {
                dark_mode_toggle(is_dark);
            },

            if dark {
                MoonIcon {}
            } else {
                SunIcon {}
            }
        }
    }
}

/// Sun SVG icon -- displayed when light mode is active.
///
/// Based on Heroicons "sun" (24x24, stroke-only).
#[component]
fn SunIcon() -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            class: "w-5 h-5",
            width: "20",
            height: "20",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",

            // Centre circle (the sun body).
            circle { cx: "12", cy: "12", r: "5" }

            // Rays.
            line { x1: "12", y1: "1",  x2: "12", y2: "3"  }
            line { x1: "12", y1: "21", x2: "12", y2: "23" }
            line { x1: "4.22",  y1: "4.22",  x2: "5.64",  y2: "5.64"  }
            line { x1: "18.36", y1: "18.36", x2: "19.78", y2: "19.78" }
            line { x1: "1",  y1: "12", x2: "3",  y2: "12" }
            line { x1: "21", y1: "12", x2: "23", y2: "12" }
            line { x1: "4.22",  y1: "19.78", x2: "5.64",  y2: "18.36" }
            line { x1: "18.36", y1: "5.64",  x2: "19.78", y2: "4.22"  }
        }
    }
}

/// Moon SVG icon -- displayed when dark mode is active.
///
/// Based on Heroicons "moon" (24x24, stroke-only).
#[component]
fn MoonIcon() -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            class: "w-5 h-5",
            width: "20",
            height: "20",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",

            // Crescent moon path.
            path {
                d: "M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z",
            }
        }
    }
}
