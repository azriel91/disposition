//! Shared editable row component for ID-value maps.
//!
//! Provides [`IdValueRow`] -- a reusable row with:
//! - A drag handle for reordering.
//! - An ID input (with optional datalist and pattern).
//! - A value input.
//! - A remove button.
//!
//! Keyboard shortcuts:
//!
//! - **Up / Down** (on row): move focus to the previous / next row.
//! - **Ctrl+Up / Ctrl+Down** (on row): jump to the first / last row.
//! - **Alt+Up / Alt+Down**: move the entry up or down in the list.
//! - **Alt+Shift+Up / Alt+Shift+Down**: insert a new entry before / after the
//!   current row.
//! - **Ctrl+Shift+K** (on row): remove the current entry.
//! - **Enter** (on row): focus the first input inside the row for editing.
//! - **Escape** (on row): focus the parent section / tab.
//! - **Tab** (inside an input or remove button): cycle to the next interactive
//!   element within the same row. Wraps from last to first.
//! - **Shift+Tab** (inside an input or remove button): cycle to the previous
//!   interactive element within the same row. Wraps from first to last.
//! - **Esc** (inside an input or remove button): return focus to the parent
//!   row.
//! - **Space** (inside an input or remove button): stop propagation.
//!
//! Arrow keys are **not** intercepted when an `<input>` has focus, so the
//! cursor can still be moved within the text field.
//!
//! After an ID rename the row element is destroyed and recreated under the new
//! key. The row signals its stable parent container via `rename_refocus` so
//! that the container can re-focus the correct field in the new element after
//! the DOM update.

use dioxus::{
    hooks::use_signal,
    prelude::{
        component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Callback, Element, Props,
    },
    signals::{ReadableExt, Signal, WritableExt},
};

use crate::components::editor::{
    common::{
        FieldNav, RenameRefocus, RenameRefocusTarget, ID_INPUT_CLASS, INPUT_CLASS, REMOVE_BTN,
        ROW_CLASS,
    },
    reorderable::{drag_border_class, DragHandle},
};

// === Data attribute for the row wrapper === //

/// The `data-*` attribute placed on each `IdValueRow` wrapper.
///
/// Used by [`keyboard_nav`](crate::components::editor::keyboard_nav) helpers
/// to locate the nearest ancestor row.
const DATA_ATTR: &str = "data-entry-id";

// === IdValueRow component === //

/// A reusable editable row for ID-value maps.
///
/// The row renders a drag handle, an ID input, a value input, and a remove
/// button with unified keyboard and drag-and-drop behaviour. Callers
/// supply callbacks for the five mutation operations that differ between
/// pages.
///
/// # Callbacks
///
/// * `on_move(from, to)`: reorder the entry from index `from` to index `to`.
/// * `on_rename(id_old, id_new)`: change the entry key.
/// * `on_update(id, value)`: change the entry value.
/// * `on_remove(id)`: delete the entry.
/// * `on_add(index)`: insert a new entry at `index`.
///
/// # Props
///
/// * `entry_id`: the current ID string, e.g. `"thing_0"`.
/// * `entry_value`: the current value string.
/// * `id_list`: datalist id for the ID input (e.g. `list_ids::THING_IDS`).
/// * `id_placeholder`: placeholder text for the ID input, e.g. `"thing_id"`.
/// * `value_placeholder`: placeholder text for the value input, e.g. `"Display
///   name"`.
/// * `index`: position of this entry in its list.
/// * `entry_count`: total number of entries.
/// * `drag_index` / `drop_target`: shared drag-and-drop signals.
/// * `focus_index`: shared focus-after-move signal.
/// * `rename_refocus`: when an ID rename completes, this signal is set so that
///   the stable parent container can re-focus the correct field inside the
///   newly created row element.
#[component]
pub fn IdValueRow(
    entry_id: String,
    entry_value: String,
    id_list: String,
    id_placeholder: String,
    value_placeholder: String,
    index: usize,
    entry_count: usize,
    drag_index: Signal<Option<usize>>,
    drop_target: Signal<Option<usize>>,
    focus_index: Signal<Option<usize>>,
    mut rename_refocus: Signal<Option<RenameRefocus>>,
    on_move: Callback<(usize, usize)>,
    on_rename: Callback<(String, String)>,
    on_update: Callback<(String, String)>,
    on_remove: Callback<String>,
    on_add: Callback<usize>,
) -> Element {
    let border_class = drag_border_class(drag_index, drop_target, index);

    // Tracks which refocus target the next ID rename should use.
    // - `IdInput`: Enter or blur triggered the rename.
    // - `NextField`: forward Tab triggered the rename.
    // - `FocusParent`: Shift+Tab or Esc triggered the rename.
    let rename_target = use_signal(|| RenameRefocusTarget::IdInput);

    rsx! {
        div {
            class: "{ROW_CLASS} {border_class} rounded focus:border-blue-400 focus:bg-gray-800 focus:outline-none",
            tabindex: "0",
            draggable: "true",
            "data-entry-id": "{entry_id}",

            // === Keyboard shortcuts (row-level) === //
            onkeydown: FieldNav::div_onkeydown(
                DATA_ATTR,
                index,
                entry_count,
                entry_id.clone(),
                on_move,
                focus_index,
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

            // === ID input === //
            input {
                class: ID_INPUT_CLASS,
                style: "max-width:14rem",
                tabindex: "-1",
                list: "{id_list}",
                placeholder: "{id_placeholder}",
                value: "{entry_id}",
                pattern: "^[a-zA-Z_][a-zA-Z0-9_]*$",
                onchange: {
                    let id_old = entry_id.clone();
                    move |evt: dioxus::events::FormEvent| {
                        let id_new = evt.value();
                        let target = *rename_target.read();
                        on_rename.call((id_old.clone(), id_new.clone()));
                        rename_refocus.set(Some(RenameRefocus {
                            new_id: id_new,
                            target,
                        }));
                    }
                },
                onkeydown: FieldNav::id_onkeydown(DATA_ATTR, rename_target),
            }

            // === Value input === //
            input {
                class: INPUT_CLASS,
                tabindex: "-1",
                placeholder: "{value_placeholder}",
                value: "{entry_value}",
                oninput: {
                    let entry_id = entry_id.clone();
                    move |evt: dioxus::events::FormEvent| {
                        let new_value = evt.value();
                        on_update.call((entry_id.clone(), new_value));
                    }
                },
                onkeydown: FieldNav::value_onkeydown(DATA_ATTR),
            }

            // === Remove button === //
            button {
                class: REMOVE_BTN,
                tabindex: "-1",
                "data-action": "remove",
                onclick: {
                    let entry_id = entry_id.clone();
                    move |_| {
                        on_remove.call(entry_id.clone());
                    }
                },
                onkeydown: FieldNav::value_onkeydown(DATA_ATTR),
                "\u{2715}"
            }
        }
    }
}
