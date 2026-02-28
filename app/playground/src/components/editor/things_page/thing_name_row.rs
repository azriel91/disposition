//! Thing name row component.
//!
//! A single editable row for a thing name (`ThingId` -> display label).
//! Supports keyboard shortcuts:
//!
//! - **Up / Down**: move focus to the previous / next row.
//! - **Alt+Up / Alt+Down**: move the entry up or down in the list.

use dioxus::{
    document,
    prelude::{
        component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Key,
        ModifiersInteraction, Props,
    },
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
    entry_count: usize,
    drag_index: Signal<Option<usize>>,
    drop_target: Signal<Option<usize>>,
    mut focus_index: Signal<Option<usize>>,
) -> Element {
    let border_class = drag_row_border_class(drag_index, drop_target, index);

    let can_move_up = index > 0;
    let can_move_down = index + 1 < entry_count;

    rsx! {
        div {
            class: "{ROW_CLASS} {border_class} rounded focus:border-blue-400 focus:bg-gray-800 focus:outline-none",
            tabindex: "0",
            draggable: "true",

            // === Keyboard shortcuts === //
            onkeydown: move |evt| {
                let alt = evt.modifiers().alt();

                match evt.key() {
                    Key::ArrowUp if alt => {
                        evt.prevent_default();
                        if can_move_up {
                            ThingsPageOps::thing_move(input_diagram, index, index - 1);
                            focus_index.set(Some(index - 1));
                        }
                    }
                    Key::ArrowDown if alt => {
                        evt.prevent_default();
                        if can_move_down {
                            ThingsPageOps::thing_move(input_diagram, index, index + 1);
                            focus_index.set(Some(index + 1));
                        }
                    }
                    Key::ArrowUp => {
                        evt.prevent_default();
                        document::eval(
                            "document.activeElement\
                                ?.previousElementSibling\
                                ?.focus()",
                        );
                    }
                    Key::ArrowDown => {
                        evt.prevent_default();
                        document::eval(
                            "document.activeElement\
                                ?.nextElementSibling\
                                ?.focus()",
                        );
                    }
                    _ => {}
                }
            },

            // === Drag-and-drop === //
            ondragstart: move |_| {
                drag_index.set(Some(index));
            },
            ondragover: move |evt| {
                evt.prevent_default();
                drop_target.set(Some(index));
            },
            ondrop: move |evt| {
                evt.prevent_default();
                if let Some(from) = *drag_index.read()
                    && from != index {
                        ThingsPageOps::thing_move(input_diagram, from, index);
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
