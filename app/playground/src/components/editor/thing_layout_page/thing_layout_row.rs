//! Thing layout row component.
//!
//! A single row in the thing layout direction editor. Each row displays a
//! thing ID alongside a `<select>` dropdown for choosing the flex direction
//! and a remove button to clear the override.

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{Signal, WritableExt},
};
use disposition::{
    input_model::{thing::ThingId, InputDiagram},
    model_common::layout::FlexDirection,
};

/// CSS classes for the layout row container.
const ROW_CLASS: &str = "\
    flex flex-row gap-2 items-center \
    py-1 \
    border-b \
    border-gray-800\
";

/// CSS classes for the flex-direction `<select>`.
const SELECT_CLASS: &str = "\
    rounded \
    px-2 py-0.5 \
    text-sm \
    bg-gray-700 \
    hover:bg-gray-600 \
    text-gray-200 \
    border \
    border-gray-600 \
    focus:outline-none \
    focus:border-blue-400 \
    cursor-pointer\
";

/// CSS classes for the remove button.
const REMOVE_BTN: &str = "\
    text-gray-500 \
    hover:text-red-400 \
    text-xs \
    cursor-pointer \
    px-1 \
    select-none \
    leading-none\
";

/// A single row in the thing layout direction editor.
///
/// Shows the thing ID, a `<select>` for picking the flex direction, and a
/// remove button that deletes the override (reverting to the depth-based
/// default).
///
/// # Props
///
/// * `input_diagram`: the shared diagram signal.
/// * `thing_id`: the `ThingId` whose layout direction is being configured.
/// * `direction`: the currently selected `FlexDirection`.
#[component]
pub fn ThingLayoutRow(
    input_diagram: Signal<InputDiagram<'static>>,
    thing_id: ThingId<'static>,
    direction: FlexDirection,
) -> Element {
    let thing_id_display = thing_id.to_string();

    // Map direction to select index.
    let selected_value = match direction {
        FlexDirection::Row => "row",
        FlexDirection::RowReverse => "row_reverse",
        FlexDirection::Column => "column",
        FlexDirection::ColumnReverse => "column_reverse",
    };

    let thing_id_for_change = thing_id.clone();
    let thing_id_for_remove = thing_id.clone();

    rsx! {
        div {
            class: ROW_CLASS,
            "data-input-diagram-field": "thing_layout_{thing_id_display}",

            // === Thing ID label === //
            span {
                class: "\
                    flex-1 \
                    text-sm \
                    font-mono \
                    text-gray-200 \
                    truncate\
                ",
                title: "{thing_id_display}",
                "{thing_id_display}"
            }

            // === Direction select === //
            select {
                class: SELECT_CLASS,
                value: selected_value,
                "aria-label": "Flex direction for {thing_id_display}",
                onchange: move |evt: dioxus::events::FormEvent| {
                    let new_direction = match evt.value().as_str() {
                        "row" => FlexDirection::Row,
                        "row_reverse" => FlexDirection::RowReverse,
                        "column" => FlexDirection::Column,
                        "column_reverse" => FlexDirection::ColumnReverse,
                        _ => return,
                    };
                    input_diagram
                        .write()
                        .thing_layouts
                        .insert(thing_id_for_change.clone(), new_direction);
                },

                option { value: "row", "Row" }
                option { value: "row_reverse", "Row Reverse" }
                option { value: "column", "Column" }
                option { value: "column_reverse", "Column Reverse" }
            }

            // === Remove button === //
            span {
                class: REMOVE_BTN,
                title: "Remove layout override",
                onclick: move |_| {
                    input_diagram
                        .write()
                        .thing_layouts
                        .remove(&thing_id_for_remove);
                },
                "✕"
            }
        }
    }
}
