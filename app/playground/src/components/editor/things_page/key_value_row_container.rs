//! Container component for draggable key-value rows.

use dioxus::prelude::{
    component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props,
};

/// A container for multiple draggable key-value rows (or [`ThingNameRow`]s).
///
/// Uses `group/key-value-rows` so that child rows can react to an active drag
/// via `group-active/key-value-rows:_` utilities. Does **not** use `gap` on
/// the flex container -- each row carries its own padding instead, so there are
/// no dead-zones between rows where a drop would be missed.
///
/// [`ThingNameRow`]: super::thing_name_row::ThingNameRow
#[component]
pub fn KeyValueRowContainer(children: Element) -> Element {
    rsx! {
        div {
            class: "flex flex-col group/key-value-rows",
            {children}
        }
    }
}
