//! Thing name row component.
//!
//! A single editable row for a thing name (`ThingId` -> display label).

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::input_model::InputDiagram;

use crate::components::editor::{
    common::{ID_INPUT_CLASS, INPUT_CLASS, REMOVE_BTN, ROW_CLASS},
    datalists::list_ids,
};

use super::{
    drag_handle::DragHandle, drag_row_border_class::drag_row_border_class,
    things_page_ops::ThingsPageOps,
};

/// A single editable row for a thing name (`ThingId` -> display label).
#[component]
pub fn ThingNameRow(
    input_diagram: Signal<InputDiagram<'static>>,
    thing_id: String,
    thing_name: String,
    index: usize,
    drag_index: Signal<Option<usize>>,
    drop_target: Signal<Option<usize>>,
) -> Element {
    let border_class = drag_row_border_class(drag_index, drop_target, index);

    rsx! {
        div {
            class: "{ROW_CLASS} {border_class}",
            draggable: "true",
            ondragstart: move |_| {
                drag_index.set(Some(index));
            },
            ondragover: move |evt| {
                evt.prevent_default();
                drop_target.set(Some(index));
            },
            ondrop: move |evt| {
                evt.prevent_default();
                if let Some(from) = *drag_index.read() {
                    if from != index {
                        ThingsPageOps::thing_move(input_diagram, from, index);
                    }
                }
                drag_index.set(None);
                drop_target.set(None);
            },
            ondragend: move |_| {
                drag_index.set(None);
                drop_target.set(None);
            },

            DragHandle {}

            // ThingId input
            input {
                class: ID_INPUT_CLASS,
                style: "max-width:14rem",
                list: list_ids::THING_IDS,
                placeholder: "thing_id",
                value: "{thing_id}",
                pattern: "^[a-zA-Z_][a-zA-Z0-9_]*$",
                onchange: {
                    let thing_id_old = thing_id.clone();
                    move |evt: dioxus::events::FormEvent| {
                        let thing_id_new = evt.value();
                        ThingsPageOps::thing_rename(input_diagram, &thing_id_old, &thing_id_new);
                    }
                },
            }

            // Display name input
            input {
                class: INPUT_CLASS,
                placeholder: "Display name",
                value: "{thing_name}",
                oninput: {
                    let thing_id = thing_id.clone();
                    move |evt: dioxus::events::FormEvent| {
                        let name = evt.value();
                        ThingsPageOps::thing_name_update(input_diagram, &thing_id, &name);
                    }
                },
            }

            // Remove button
            span {
                class: REMOVE_BTN,
                onclick: {
                    let thing_id = thing_id.clone();
                    move |_| {
                        ThingsPageOps::thing_remove(input_diagram, &thing_id);
                    }
                },
                "✕"
            }
        }
    }
}
