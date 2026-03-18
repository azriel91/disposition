//! A single thing-ID row inside the things field of an edge group card.
//!
//! Extracted from [`EdgeGroupCard`] to keep the parent component concise.
//!
//! Keyboard shortcuts (on the row wrapper):
//!
//! - **Up / Down**: move focus to the previous / next sibling row.
//! - **Ctrl+Up / Ctrl+Down**: jump to the first / last row.
//! - **Alt+Up / Alt+Down**: move the thing up or down in the list.
//! - **Alt+Shift+Up / Alt+Shift+Down**: insert a new thing above / below the
//!   current row.
//! - **Ctrl+Shift+K**: remove the thing.
//! - **Enter**: focus the first input inside the row for editing.
//! - **Escape**: focus the parent card wrapper.
//!
//! Keyboard shortcuts (on inputs inside the row):
//!
//! - **Alt+Up / Alt+Down**: move the thing up or down in the list.
//! - **Alt+Shift+Up / Alt+Shift+Down**: insert a new thing above / below.
//! - **Ctrl+Shift+K**: remove the thing.
//! - **Tab / Shift+Tab**: cycle through focusable fields within the row.
//! - **Escape**: return focus to the row wrapper.
//!
//! The row also supports drag-and-drop reordering via a [`DragHandle`]
//! grip indicator, with drop-target border highlighting provided by
//! [`drag_border_class`].
//!
//! [`EdgeGroupCard`]: super::EdgeGroupCard

use dioxus::{
    prelude::{
        component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Callback, Element, Props,
    },
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::input_model::InputDiagram;
use disposition_input_rt::{EdgeGroupCardOps, MapTarget};

use crate::components::editor::{
    common::{RowComponent, REMOVE_BTN, ROW_CLASS},
    datalists::list_ids,
    reorderable::{drag_border_class, DragHandle},
    thing_dependencies_page::{DATA_ATTR, FIELD_INPUT_CLASS},
};

/// Data attribute placed on each edge-thing row wrapper.
///
/// Used by [`ReorderableContainer`] and keyboard navigation helpers
/// to locate sibling rows within the things list.
///
/// [`ReorderableContainer`]: crate::components::editor::reorderable::ReorderableContainer
const ROW_DATA_ATTR: &str = "data-edge-thing-row";

/// A single thing-ID row inside the things field of an edge group card.
///
/// Displays a drag handle, row index, a thing-ID input with full keyboard
/// navigation (Up/Down focus cycling, Alt reorder, Alt+Shift insert,
/// Ctrl+Shift+K remove, Enter to edit, Escape to card), and a remove
/// button. Supports drag-and-drop reordering.
#[component]
pub(crate) fn EdgeGroupCardFieldThingsRow(
    input_diagram: Signal<InputDiagram<'static>>,
    target: MapTarget,
    edge_group_id: String,
    thing_id: String,
    index: usize,
    thing_count: usize,
    thing_focus_idx: Signal<Option<usize>>,
    drag_index: Signal<Option<usize>>,
    drop_target: Signal<Option<usize>>,
    on_move: Callback<(usize, usize)>,
    on_add: Callback<usize>,
    on_remove: Callback<usize>,
) -> Element {
    let border_class = drag_border_class(drag_index, drop_target, index);

    rsx! {
        div {
            class: "{ROW_CLASS} {border_class} rounded focus:border-blue-400 focus:bg-gray-800 focus:outline-none",
            tabindex: "0",
            draggable: "true",
            "data-edge-thing-row": "",
            "data-input-diagram-field": "{edge_group_id}_thing_{index}",

            // === Row-level keyboard shortcuts === //
            onkeydown: RowComponent::row_onkeydown(
                ROW_DATA_ATTR,
                DATA_ATTR,
                index,
                thing_count,
                thing_focus_idx,
                on_move,
                on_add,
                on_remove,
            ),

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
                onkeydown: RowComponent::row_field_onkeydown(
                    ROW_DATA_ATTR,
                    DATA_ATTR,
                    index,
                    thing_count,
                    thing_focus_idx,
                    on_move,
                    on_add,
                    on_remove,
                ),
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
                onkeydown: RowComponent::row_field_onkeydown(
                    ROW_DATA_ATTR,
                    DATA_ATTR,
                    index,
                    thing_count,
                    thing_focus_idx,
                    on_move,
                    on_add,
                    on_remove,
                ),
                "x"
            }
        }
    }
}
