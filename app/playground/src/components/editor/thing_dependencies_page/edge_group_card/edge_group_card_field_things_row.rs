//! A single thing-ID row inside the things field of an edge group card.
//!
//! Extracted from [`EdgeGroupCard`] to keep the parent component concise.
//!
//! Keyboard shortcuts (on the input):
//!
//! - **Alt+Up / Alt+Down**: move the thing up or down in the list.
//! - All other keys fall through to the standard field navigation
//!   (`field_keydown` with the card-level data attribute).
//!
//! The row also supports drag-and-drop reordering via a [`DragHandle`]
//! grip indicator, with drop-target border highlighting provided by
//! [`drag_border_class`].
//!
//! [`EdgeGroupCard`]: super::EdgeGroupCard

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::input_model::InputDiagram;
use disposition_input_rt::{EdgeGroupCardOps, MapTarget};

use crate::components::editor::{
    common::{CardComponent, FieldNav, REMOVE_BTN, ROW_CLASS},
    datalists::list_ids,
    reorderable::{drag_border_class, DragHandle},
    thing_dependencies_page::{DATA_ATTR, FIELD_INPUT_CLASS},
};

/// A single thing-ID row inside the things field of an edge group card.
///
/// Displays a drag handle, row index, a thing-ID input with Alt+Up/Down
/// reordering, and a remove button. Supports drag-and-drop reordering.
#[component]
pub(crate) fn EdgeGroupCardFieldThingsRow(
    input_diagram: Signal<InputDiagram<'static>>,
    target: MapTarget,
    edge_group_id: String,
    thing_id: String,
    index: usize,
    thing_count: usize,
    mut thing_focus_idx: Signal<Option<usize>>,
    drag_index: Signal<Option<usize>>,
    drop_target: Signal<Option<usize>>,
) -> Element {
    let can_move_up = index > 0;
    let can_move_down = index + 1 < thing_count;
    let border_class = drag_border_class(drag_index, drop_target, index);

    rsx! {
        div {
            class: "{ROW_CLASS} {border_class}",
            draggable: "true",
            "data-edge-thing-row": "",

            // === Drag-and-drop === //
            ondragstart: move |_| {
                drag_index.set(Some(index));
            },
            ondragover: move |evt| {
                evt.prevent_default();
                drop_target.set(Some(index));
            },
            ondrop: {
                let edge_group_id = edge_group_id.clone();
                move |evt| {
                    evt.prevent_default();
                    if let Some(from) = *drag_index.read()
                        && from != index
                    {
                        EdgeGroupCardOps::edge_thing_move(
                            &mut input_diagram.write(),
                            target,
                            &edge_group_id,
                            from,
                            index,
                        );
                    }
                    drag_index.set(None);
                    drop_target.set(None);
                }
            },
            ondragend: move |_| {
                drag_index.set(None);
                drop_target.set(None);
            },

            DragHandle {}

            span {
                class: "text-xs text-gray-500 w-6 text-right",
                "{index}."
            }

            input {
                class: FIELD_INPUT_CLASS,
                style: "max-width:14rem",
                tabindex: "-1",
                list: list_ids::THING_IDS,
                placeholder: "thing_id",
                value: "{thing_id}",
                onchange: {
                    let edge_group_id = edge_group_id.clone();
                    move |evt: dioxus::events::FormEvent| {
                        let thing_id_new = evt.value();
                        EdgeGroupCardOps::edge_thing_update(
                            &mut input_diagram.write(),
                            target,
                            &edge_group_id,
                            index,
                            &thing_id_new,
                        );
                    }
                },
                onkeydown: {
                    let edge_group_id = edge_group_id.clone();
                    let edge_group_id_down = edge_group_id.clone();
                    CardComponent::field_onkeydown(
                        DATA_ATTR,
                        can_move_up,
                        can_move_down,
                        move || {
                            EdgeGroupCardOps::edge_thing_move(
                                &mut input_diagram.write(),
                                target,
                                &edge_group_id,
                                index,
                                index - 1,
                            );
                            thing_focus_idx.set(Some(index - 1));
                        },
                        move || {
                            EdgeGroupCardOps::edge_thing_move(
                                &mut input_diagram.write(),
                                target,
                                &edge_group_id_down,
                                index,
                                index + 1,
                            );
                            thing_focus_idx.set(Some(index + 1));
                        },
                    )
                },
            }

            button {
                class: REMOVE_BTN,
                tabindex: "-1",
                "data-action": "remove",
                onclick: {
                    let edge_group_id = edge_group_id.clone();
                    move |_| {
                        EdgeGroupCardOps::edge_thing_remove(
                            &mut input_diagram.write(),
                            target,
                            &edge_group_id,
                            index,
                        );
                    }
                },
                onkeydown: FieldNav::value_onkeydown(DATA_ATTR),
                "x"
            }
        }
    }
}
