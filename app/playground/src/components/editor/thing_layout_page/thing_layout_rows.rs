//! Container component for thing layout direction rows.
//!
//! Wraps the list of [`ThingLayoutRow`]s in a styled container with a heading.
//! This section lets users override the flex direction for container things
//! (things that have children in the hierarchy).
//!
//! [`ThingLayoutRow`]: super::thing_layout_row::ThingLayoutRow

use dioxus::prelude::{
    component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props,
};

use crate::components::editor::common::SECTION_HEADING;

/// A container for [`ThingLayoutRow`]s with a section heading and an add
/// button.
///
/// # Props
///
/// * `children`: the `ThingLayoutRow` elements rendered inside the container.
///
/// [`ThingLayoutRow`]: super::thing_layout_row::ThingLayoutRow
#[component]
pub fn ThingLayoutRows(children: Element) -> Element {
    rsx! {
        div {
            class: "flex flex-col gap-1 mt-2",

            h3 { class: SECTION_HEADING, "Thing Layout Directions" }

            p {
                class: "text-xs text-gray-500 mb-1",
                "Override the flex direction for container things. \
                 By default, direction alternates between column and row at each nesting level."
            }

            div {
                class: "\
                    flex \
                    flex-col \
                    rounded-lg \
                    border \
                    border-gray-700 \
                    bg-gray-900 \
                    p-2 \
                    gap-1\
                ",

                {children}
            }
        }
    }
}
