//! Thing layout row component.
//!
//! A single row in the thing layout direction editor. Each row displays a
//! drag handle, a thing ID label, a `<select>` dropdown for choosing the
//! flex direction, and a remove button to clear the override.
//!
//! Keyboard shortcuts:
//!
//! - **Up / Down** (on row): move focus to the previous / next row.
//! - **Alt+Up / Alt+Down**: move the entry up or down in the list.
//! - **Enter** (on row): focus the first interactive element (select) inside
//!   the row for editing.
//! - **Escape** (on row): focus the parent section / tab.
//! - **Tab / Shift+Tab** (inside select or remove button): cycle through
//!   interactive elements within the same row. Wraps from last to first / first
//!   to last.
//! - **Esc** (inside select or remove button): return focus to the parent row.
//! - **Ctrl+Shift+K** (on row): remove the current entry.

use dioxus::{
    prelude::{
        component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Callback, Element, Props,
    },
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::model_common::{layout::FlexDirection, Id};

use crate::components::editor::{
    common::{FieldNav, REMOVE_BTN, ROW_CLASS, SELECT_CLASS},
    reorderable::{drag_border_class, DragHandle},
};

/// The `data-*` attribute placed on each `ThingLayoutRow` wrapper.
///
/// Used by keyboard nav helpers to locate the nearest ancestor row.
pub(super) const DATA_ATTR: &str = "data-thing-layout-id";

/// A single row in the thing layout direction editor.
///
/// Shows a drag handle, the thing ID, a `<select>` for picking the flex
/// direction, and a remove button that deletes the override (reverting to
/// the depth-based default).
///
/// # Props
///
/// * `input_diagram`: not used directly; mutation callbacks are provided by the
///   parent.
/// * `node_inbuilt_or_thing_id`: the `Id` whose layout direction is being
///   configured.
/// * `direction`: the currently selected `FlexDirection`.
/// * `index`: zero-based position of this row in the layout entries list.
/// * `entry_count`: total number of layout entries.
/// * `drag_index` / `drop_target`: shared drag-and-drop signals.
/// * `focus_index`: shared focus-after-move signal.
/// * `on_move`: callback to reorder `(from_index, to_index)`.
/// * `on_direction_change`: callback to change the flex direction for this
///   entry.
/// * `on_remove`: callback to remove this entry by its ID string.
#[component]
pub fn ThingLayoutRow(
    node_inbuilt_or_thing_id: Id<'static>,
    direction: FlexDirection,
    index: usize,
    entry_count: usize,
    drag_index: Signal<Option<usize>>,
    drop_target: Signal<Option<usize>>,
    focus_index: Signal<Option<usize>>,
    on_move: Callback<(usize, usize)>,
    on_direction_change: Callback<(String, FlexDirection)>,
    on_remove: Callback<String>,
) -> Element {
    let thing_id_display = node_inbuilt_or_thing_id.to_string();
    let border_class = drag_border_class(drag_index, drop_target, index);

    // Map direction to select value.
    let selected_value = match direction {
        FlexDirection::Row => "row",
        FlexDirection::RowReverse => "row_reverse",
        FlexDirection::Column => "column",
        FlexDirection::ColumnReverse => "column_reverse",
    };

    rsx! {
        div {
            class: "{ROW_CLASS} {border_class} rounded focus:border-blue-400 focus:bg-gray-800 focus:outline-none",
            tabindex: "0",
            draggable: "true",
            "data-thing-layout-id": "{thing_id_display}",
            "data-input-diagram-field": "thing_layout_{thing_id_display}",

            // === Keyboard shortcuts (row-level) === //
            onkeydown: FieldNav::div_onkeydown(
                DATA_ATTR,
                index,
                entry_count,
                thing_id_display.clone(),
                focus_index,
                on_move,
                // on_add: no-op (adding is handled by the input below the rows)
                Callback::new(|_: usize| {}),
                on_remove,
                None,
            ),

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
                    && from != index
                {
                    on_move.call((from, index));
                }
                drag_index.set(None);
                drop_target.set(None);
            },
            ondragend: move |_| {
                drag_index.set(None);
                drop_target.set(None);
            },

            DragHandle {}

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
                tabindex: "-1",
                value: selected_value,
                "aria-label": "Flex direction for {thing_id_display}",
                onchange: {
                    let id_str = thing_id_display.clone();
                    move |evt: dioxus::events::FormEvent| {
                        let new_direction = match evt.value().as_str() {
                            "row" => FlexDirection::Row,
                            "row_reverse" => FlexDirection::RowReverse,
                            "column" => FlexDirection::Column,
                            "column_reverse" => FlexDirection::ColumnReverse,
                            _ => return,
                        };
                        on_direction_change.call((id_str.clone(), new_direction));
                    }
                },
                onkeydown: FieldNav::value_onkeydown(DATA_ATTR),

                option { value: "row", "Row" }
                option { value: "row_reverse", "Row Reverse" }
                option { value: "column", "Column" }
                option { value: "column_reverse", "Column Reverse" }
            }

            // === Remove button === //
            button {
                class: REMOVE_BTN,
                tabindex: "-1",
                "data-action": "remove",
                title: "Remove layout override",
                onclick: {
                    let id_str = thing_id_display.clone();
                    move |_| {
                        on_remove.call(id_str.clone());
                    }
                },
                onkeydown: FieldNav::value_onkeydown(DATA_ATTR),
                "\u{2715}"
            }
        }
    }
}
