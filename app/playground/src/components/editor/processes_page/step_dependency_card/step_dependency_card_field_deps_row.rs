//! A single dependency row within the deps list of a [`StepDependencyCard`].
//!
//! Extracted from [`StepDependencyCard`] to keep the parent component concise.
//!
//! Keyboard shortcuts (on the row wrapper):
//!
//! - **Up / Down**: move focus to the previous / next sibling row.
//! - **Ctrl+Up / Ctrl+Down**: jump to the first / last row.
//! - **Alt+Up / Alt+Down**: move the dependency up or down in the list.
//! - **Alt+Shift+Up / Alt+Shift+Down**: insert a new dependency above / below
//!   the current row.
//! - **Ctrl+Shift+K**: remove the dependency.
//! - **Enter**: focus the first input inside the row for editing.
//! - **Escape**: focus the parent card wrapper.
//!
//! Keyboard shortcuts (on inputs inside the row):
//!
//! - **Alt+Up / Alt+Down**: move the dependency up or down in the list.
//! - **Alt+Shift+Up / Alt+Shift+Down**: insert a new dependency above / below.
//! - **Ctrl+Shift+K**: remove the dependency.
//! - **Tab / Shift+Tab**: cycle through focusable fields within the row.
//! - **Escape**: return focus to the row wrapper.
//!
//! The row also supports drag-and-drop reordering via a [`DragHandle`]
//! grip indicator, with drop-target border highlighting provided by
//! [`drag_border_class`].
//!
//! [`StepDependencyCard`]: super::StepDependencyCard

use dioxus::{
    prelude::{
        component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Callback, Element, Props,
    },
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::input_model::InputDiagram;
use disposition_input_rt::StepDependencyCardOps;

use crate::components::editor::{
    common::{RowComponent, REMOVE_BTN, ROW_CLASS},
    datalists::list_ids,
    processes_page::{DATA_ATTR, FIELD_INPUT_CLASS},
    reorderable::{drag_border_class, DragHandle},
};

/// Data attribute placed on each step-dependency row wrapper.
///
/// Used by [`ReorderableContainer`] and keyboard navigation helpers
/// to locate sibling rows within the deps list.
///
/// [`ReorderableContainer`]: crate::components::editor::reorderable::ReorderableContainer
const ROW_DATA_ATTR: &str = "data-step-dep-row";

/// A single dependency row within the deps section of a step-dependency card.
///
/// Displays a drag handle, row index, a process step ID input (with a datalist
/// scoped to the process's steps), and a remove button for one entry in the
/// step's dependency set. Supports full keyboard navigation (Up/Down focus
/// cycling, Alt reorder, Alt+Shift insert, Ctrl+Shift+K remove, Enter to edit,
/// Escape to card) and drag-and-drop reordering.
#[component]
pub(crate) fn StepDependencyCardFieldDepsRow(
    input_diagram: Signal<InputDiagram<'static>>,
    process_id: String,
    step_id: String,
    dep_id: String,
    index: usize,
    dep_count: usize,
    dep_focus_idx: Signal<Option<usize>>,
    drag_index: Signal<Option<usize>>,
    drop_target: Signal<Option<usize>>,
    on_move: Callback<(usize, usize)>,
    on_add: Callback<usize>,
    on_remove: Callback<usize>,
) -> Element {
    let border_class = drag_border_class(drag_index, drop_target, index);
    let step_ids_list = list_ids::process_step_ids_for(&process_id);

    rsx! {
        div {
            class: "{ROW_CLASS} {border_class} rounded focus:border-blue-400 focus:bg-gray-800 focus:outline-none",
            tabindex: "0",
            draggable: "true",
            "data-step-dep-row": "",
            "data-input-diagram-field": "{process_id}_{step_id}_dep_{index}",

            // === Row-level keyboard shortcuts === //
            onkeydown: RowComponent::row_onkeydown(
                ROW_DATA_ATTR,
                DATA_ATTR,
                index,
                dep_count,
                dep_focus_idx,
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
                let process_id = process_id.clone();
                let step_id = step_id.clone();
                move |evt| {
                    evt.prevent_default();
                    if let Some(from) = *drag_index.read()
                        && from != index
                    {
                        StepDependencyCardOps::step_dependency_dep_move(
                            &mut input_diagram.write(),
                            &process_id,
                            &step_id,
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
                list: "{step_ids_list}",
                placeholder: "step_id",
                value: "{dep_id}",
                onchange: {
                    let process_id = process_id.clone();
                    let step_id = step_id.clone();
                    move |evt: dioxus::events::FormEvent| {
                        StepDependencyCardOps::step_dependency_dep_update(
                            &mut input_diagram.write(),
                            &process_id,
                            &step_id,
                            index,
                            &evt.value(),
                        );
                    }
                },
                onkeydown: RowComponent::row_field_onkeydown(
                    ROW_DATA_ATTR,
                    DATA_ATTR,
                    index,
                    dep_count,
                    dep_focus_idx,
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
                    let process_id = process_id.clone();
                    let step_id = step_id.clone();
                    move |_| {
                        StepDependencyCardOps::step_dependency_dep_remove(
                            &mut input_diagram.write(),
                            &process_id,
                            &step_id,
                            index,
                        );
                    }
                },
                onkeydown: RowComponent::row_field_onkeydown(
                    ROW_DATA_ATTR,
                    DATA_ATTR,
                    index,
                    dep_count,
                    dep_focus_idx,
                    on_move,
                    on_add,
                    on_remove,
                ),
                "\u{2715}"
            }
        }
    }
}
