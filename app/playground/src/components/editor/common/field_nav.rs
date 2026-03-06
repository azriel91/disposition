//! Field-level keyboard navigation helpers.
//!
//! [`FieldNav`] provides reusable `onkeydown` handler factories for the
//! interactive elements inside editor rows and cards:
//!
//! - [`FieldNav::id_onkeydown`]: ID input fields (tracks rename refocus
//!   target).
//! - [`FieldNav::value_onkeydown`]: value inputs and remove buttons.
//! - [`FieldNav::div_onkeydown`]: the outermost `div` wrapper of an
//!   [`IdValueRow`](crate::components::editor::id_value_row::IdValueRow).
//!
//! ## `div_onkeydown` keyboard shortcuts
//!
//! - **Up / Down**: move focus to the previous / next row.
//! - **Ctrl+Up / Ctrl+Down**: jump to the first / last row.
//! - **Alt+Up / Alt+Down**: move the entry up or down in the list.
//! - **Alt+Shift+Up / Alt+Shift+Down**: insert a new entry before / after the
//!   current row.
//! - **Enter**: focus the first input inside the row for editing.
//! - **Escape**: focus the parent section / tab.

use dioxus::{
    core::Event,
    document,
    html::KeyboardData,
    prelude::{Callback, Key, ModifiersInteraction},
    signals::{Signal, WritableExt},
};

use crate::components::editor::{common::RenameRefocusTarget, keyboard_nav};

/// JS snippet: move focus to the previous sibling row.
const JS_FOCUS_PREV_ROW: &str = "\
    document.activeElement\
        ?.previousElementSibling\
        ?.focus()";

/// JS snippet: from the current row, walk forwards through the container's
/// following siblings to find a focusable element. First checks for a next
/// sibling row, then walks to the container's next siblings.
const JS_FOCUS_AFTER_CONTAINER: &str = "\
    (() => {\
        let row = document.activeElement;\
        if (!row) return;\
        let nextRow = row.nextElementSibling;\
        if (nextRow) { nextRow.focus(); return; }\
        let container = row.parentElement;\
        if (!container) return;\
        let next = container.nextElementSibling;\
        while (next) {\
            if (next.tabIndex >= 0) {\
                next.focus();\
                return;\
            }\
            next = next.nextElementSibling;\
        }\
    })()";

/// Commonizes logic for navigating fields in a form, including rename refocus
/// targets.
pub struct FieldNav;

impl FieldNav {
    /// Returns an `onkeydown` handler for the outermost `div` of an
    /// [`IdValueRow`](crate::components::editor::id_value_row::IdValueRow).
    ///
    /// Handles:
    ///
    /// - **Up / Down**: navigate to the previous / next row.
    /// - **Ctrl+Up / Ctrl+Down**: jump to the first / last row.
    /// - **Alt+Up / Alt+Down**: reorder the entry up / down.
    /// - **Alt+Shift+Up / Alt+Shift+Down**: insert a new entry before / after
    ///   the current row.
    /// - **Enter**: focus the first input inside the row.
    /// - **Escape**: focus the parent section / tab.
    ///
    /// # Parameters
    ///
    /// * `data_attr`: the `data-*` attribute on the row wrapper, e.g.
    ///   `"data-entry-id"`.
    /// * `index`: zero-based position of this row in its list.
    /// * `entry_count`: total number of entries in the list.
    /// * `on_move`: callback to reorder `(from_index, to_index)`.
    /// * `focus_index`: signal set after a move so the container can re-focus
    ///   the row at its new position.
    /// * `on_add`: callback to insert a new entry at a given index.
    pub fn div_onkeydown(
        data_attr: &'static str,
        index: usize,
        entry_count: usize,
        on_move: Callback<(usize, usize)>,
        mut focus_index: Signal<Option<usize>>,
        on_add: Callback<usize>,
    ) -> impl FnMut(Event<KeyboardData>) {
        let can_move_up = index > 0;
        let can_move_down = index + 1 < entry_count;
        let is_first = index == 0;
        let is_last = index + 1 >= entry_count;

        move |evt: Event<KeyboardData>| {
            let alt = evt.modifiers().alt();
            let ctrl = evt.modifiers().ctrl();
            let shift = evt.modifiers().shift();

            match evt.key() {
                // === Alt+Shift: insert new entry === //
                Key::ArrowUp if alt && shift => {
                    evt.prevent_default();
                    evt.stop_propagation();
                    on_add.call(index);
                    focus_index.set(Some(index));
                }
                Key::ArrowDown if alt && shift => {
                    evt.prevent_default();
                    evt.stop_propagation();
                    let insert_at = index + 1;
                    on_add.call(insert_at);
                    focus_index.set(Some(insert_at));
                }

                // === Alt: reorder === //
                Key::ArrowUp if alt => {
                    evt.prevent_default();
                    evt.stop_propagation();
                    if can_move_up {
                        on_move.call((index, index - 1));
                        focus_index.set(Some(index - 1));
                    }
                }
                Key::ArrowDown if alt => {
                    evt.prevent_default();
                    evt.stop_propagation();
                    if can_move_down {
                        on_move.call((index, index + 1));
                        focus_index.set(Some(index + 1));
                    }
                }

                // === Ctrl: jump to first / last === //
                Key::ArrowUp if ctrl => {
                    evt.prevent_default();
                    evt.stop_propagation();
                    if !is_first {
                        document::eval(&keyboard_nav::js_focus_first_entry(data_attr));
                    }
                }
                Key::ArrowDown if ctrl => {
                    evt.prevent_default();
                    evt.stop_propagation();
                    if !is_last {
                        document::eval(&keyboard_nav::js_focus_last_entry(data_attr));
                    }
                }

                // === Plain arrows: prev / next row === //
                Key::ArrowUp => {
                    evt.prevent_default();
                    evt.stop_propagation();
                    if !is_first {
                        document::eval(JS_FOCUS_PREV_ROW);
                    }
                }
                Key::ArrowDown => {
                    evt.prevent_default();
                    evt.stop_propagation();
                    if !is_last {
                        document::eval(JS_FOCUS_AFTER_CONTAINER);
                    }
                }

                // === Escape / Enter === //
                Key::Escape => {
                    evt.prevent_default();
                    evt.stop_propagation();
                    document::eval(keyboard_nav::JS_FOCUS_PARENT_SECTION);
                }
                Key::Enter => {
                    evt.prevent_default();
                    evt.stop_propagation();
                    document::eval(&keyboard_nav::js_focus_first_field());
                }

                _ => {}
            }
        }
    }

    /// Returns a handler for `keydown` events on ID fields, updating the rename
    /// target signal based on the key pressed.
    ///
    /// # Parameters
    ///
    /// * `data_attr`: The data attribute used to identify the field, e.g.
    ///   `"data-entry-id"`, `"data-edge-group-card"`, etc.
    /// * `rename_target`: The signal to update with the rename refocus target.
    pub fn id_onkeydown(
        data_attr: &'static str,
        mut rename_target: Signal<RenameRefocusTarget>,
    ) -> impl FnMut(Event<KeyboardData>) {
        move |evt: Event<KeyboardData>| {
            // Record which refocus target the upcoming onchange should
            // use before field_keydown handles the event (which
            // prevents default and may move focus).
            match evt.key() {
                Key::Tab if evt.modifiers().shift() => {
                    rename_target.set(RenameRefocusTarget::FocusParent);
                }
                Key::Tab => {
                    rename_target.set(RenameRefocusTarget::NextField);
                }
                Key::Escape => {
                    rename_target.set(RenameRefocusTarget::FocusParent);
                }
                Key::Enter => {
                    rename_target.set(RenameRefocusTarget::IdInput);
                }
                _ => {}
            }
            keyboard_nav::field_keydown(evt, data_attr);
        }
    }

    /// Returns a handler for `keydown` events on value fields, updating the
    /// rename target signal based on the key pressed.
    ///
    /// # Parameters
    ///
    /// * `data_attr`: The data attribute used to identify the field, e.g.
    ///   `"data-entry-id"`, `"data-edge-group-card"`, etc.
    pub fn value_onkeydown(data_attr: &'static str) -> impl FnMut(Event<KeyboardData>) {
        move |evt: Event<KeyboardData>| {
            keyboard_nav::field_keydown(evt, data_attr);
        }
    }
}
