//! Key-value row component.
//!
//! A reusable editable row for maps keyed by an ID string.
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

use crate::components::editor::common::{ID_INPUT_CLASS, INPUT_CLASS, REMOVE_BTN, ROW_CLASS};

use super::{
    drag_handle::DragHandle, drag_row_border_class::drag_row_border_class,
    on_change_target::OnChangeTarget, things_page_ops::ThingsPageOps,
};

/// A reusable key-value row for maps keyed by an ID string.
#[component]
pub fn KeyValueRow(
    input_diagram: Signal<InputDiagram<'static>>,
    entry_id: String,
    entry_value: String,
    id_list: &'static str,
    on_change: OnChangeTarget,
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
                            ThingsPageOps::kv_entry_move(input_diagram, on_change, index, index - 1);
                            focus_index.set(Some(index - 1));
                        }
                    }
                    Key::ArrowDown if alt => {
                        evt.prevent_default();
                        if can_move_down {
                            ThingsPageOps::kv_entry_move(input_diagram, on_change, index, index + 1);
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
                        ThingsPageOps::kv_entry_move(input_diagram, on_change, from, index);
                    }
                drag_index.set(None);
                drop_target.set(None);
            },
            ondragend: move |_| {
                drag_index.set(None);
                drop_target.set(None);
            },

            DragHandle {}

            input {
                class: ID_INPUT_CLASS,
                style: "max-width:14rem",
                list: "{id_list}",
                placeholder: "id",
                value: "{entry_id}",
                onchange: {
                    let id_old = entry_id.clone();
                    let value = entry_value.clone();
                    move |evt: dioxus::events::FormEvent| {
                        let id_new = evt.value();
                        ThingsPageOps::kv_entry_rename(
                            input_diagram,
                            on_change,
                            &id_old,
                            &id_new,
                            &value,
                        );
                    }
                },
            }

            input {
                class: INPUT_CLASS,
                placeholder: "value",
                value: "{entry_value}",
                oninput: {
                    let entry_id = entry_id.clone();
                    move |evt: dioxus::events::FormEvent| {
                        let new_value = evt.value();
                        ThingsPageOps::kv_entry_update(
                            input_diagram,
                            on_change,
                            &entry_id,
                            &new_value,
                        );
                    }
                },
            }

            span {
                class: REMOVE_BTN,
                onclick: {
                    let entry_id = entry_id.clone();
                    move |_| {
                        ThingsPageOps::kv_entry_remove(input_diagram, on_change, &entry_id);
                    }
                },
                "✕"
            }
        }
    }
}
