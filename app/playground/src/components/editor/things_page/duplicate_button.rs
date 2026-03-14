use dioxus::{
    core::Callback,
    html::MouseEvent,
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
};

use crate::components::editor::common::FieldNav;

/// CSS classes for the duplicate button.
///
/// Styled similarly to the remove button but with a neutral colour instead of
/// red so it is visually distinct.
const DUPLICATE_BTN: &str = "\
    bg-transparent \
    border-none \
    cursor-pointer \
    outline-none \
    rounded \
    p-0 \
    px-1 \
    text-xs \
    text-gray-400 \
    hover:text-gray-200 \
    focus:border \
    focus:border-solid \
    focus:border-blue-400 \
";

/// A button that duplicates a thing (including attributes) when clicked.
#[component]
pub fn DuplicateButton(data_attr: &'static str, onclick: Callback<MouseEvent>) -> Element {
    rsx! {
        button {
            class: DUPLICATE_BTN,
            tabindex: "-1",
            "data-action": "duplicate",
            title: "Duplicate thing (including attributes)",
            onclick,
            onkeydown: FieldNav::value_onkeydown(data_attr),
            DoubleSquareFilled {}
        },
    }
}

/// Two squares with rounded corners, with the front square filled.
#[component]
pub fn DoubleSquareFilled() -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            class: "\
                h-4 \
                w-4 \
                [&>*]:stroke-2 \
                [&>*]:stroke-gray-300 \
                [&>rect]:fill-gray-300 \
                hover:[&>*]:stroke-gray-100 \
                hover:[&>rect]:fill-gray-100 \
            ",
            width: "24",
            height: "24",
            view_box: "0 0 24 24",
            rect {
                x: "2",
                y: "9",
                width: "13",
                height: "13",
                rx: "2",
                ry: "2",
                fill: "currentColor",
                stroke: "currentColor",
            },
            path {
                d: "\
                    M 10 5 \
                    V 4 \
                    a 2 2 0 0 1 2 -2 \
                    H 20 \
                    a 2 2 0 0 1 2 2 \
                    V 13 \
                    a 2 2 0 0 1 -2 2 \
                    h -1\
                ",
                fill: "none",
                stroke: "currentColor",
            },
        }
    }
}
