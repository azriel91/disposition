//! Commonizes row-level state and keyboard handling for reorderable rows
//! nested inside card components.
//!
//! Row components across the editor (e.g. `EntityTypeCardFieldTypesRow`,
//! `EdgeGroupCardFieldThingsRow`, `ProcessCardFieldStepsRow`,
//! `TagThingsCardFieldThingsRow`, `StepInteractionCardFieldEdgesRow`) share
//! the same keyboard interaction pattern:
//!
//! 1. **Up / Down** (on row wrapper): move focus to the previous / next sibling
//!    row.
//! 2. **Ctrl+Up / Ctrl+Down**: jump to the first / last row.
//! 3. **Alt+Up / Alt+Down**: reorder the row within its nesting level.
//! 4. **Alt+Shift+Up / Alt+Shift+Down**: insert a new row above / below the
//!    current row.
//! 5. **Ctrl+Shift+K**: remove the current row.
//! 6. **Enter**: focus the first input inside the row for editing.
//! 7. **Escape**: focus the parent card wrapper.
//!
//! This module extracts that boilerplate into [`RowComponent`] with reusable
//! `row_onkeydown` and `row_field_onkeydown` helpers.

use dioxus::{
    core::Event,
    document,
    html::KeyboardData,
    prelude::{Callback, Key, ModifiersInteraction},
    signals::{Signal, WritableExt},
};

use crate::components::editor::keyboard_nav::KeyboardNav;

/// Groups common row-level logic: keyboard handling for reorderable rows
/// inside card components.
///
/// Mirrors the pattern used by [`CardComponent`](super::CardComponent) for
/// card-level helpers and [`FieldNav`](super::FieldNav) for field-level
/// helpers.
pub struct RowComponent;

impl RowComponent {
    /// Returns an `onkeydown` handler for the wrapper `div` of a reorderable
    /// row inside a card.
    ///
    /// Handles:
    ///
    /// - **Up / Down**: navigate to the previous / next sibling row.
    /// - **Ctrl+Up / Ctrl+Down**: jump to the first / last row.
    /// - **Alt+Up / Alt+Down**: reorder the row up / down.
    /// - **Alt+Shift+Up / Alt+Shift+Down**: insert a new row before / after the
    ///   current row.
    /// - **Ctrl+Shift+K**: remove the current row.
    /// - **Tab / Shift+Tab**: cycle through all focusable fields within the
    ///   parent card (so that focus moves to card-level fields such as the
    ///   edge-kind `<select>` rather than jumping to the card wrapper).
    /// - **Enter**: focus the first input inside the row for editing.
    /// - **Escape**: focus the parent card wrapper.
    ///
    /// # Parameters
    ///
    /// * `row_data_attr`: the `data-*` attribute on the row wrapper, e.g.
    ///   `"data-entity-type-row"`, `"data-edge-thing-row"`.
    /// * `card_data_attr`: the `data-*` attribute on the parent card wrapper,
    ///   e.g. `"data-entity-type-card"`, `"data-edge-group-card"`. Used by
    ///   Escape to focus the card.
    /// * `index`: zero-based position of this row in its list.
    /// * `entry_count`: total number of rows in the list.
    /// * `focus_index`: signal set after a move/add so the container can
    ///   re-focus the row at its new position.
    /// * `on_move`: callback to reorder the row. Receives `(from, to)`.
    /// * `on_add`: callback to insert a new row at a given index.
    /// * `on_remove`: callback to remove the current row by index.
    #[allow(clippy::too_many_arguments)]
    pub fn row_onkeydown(
        row_data_attr: &'static str,
        card_data_attr: &'static str,
        index: usize,
        entry_count: usize,
        mut focus_index: Signal<Option<usize>>,
        on_move: Callback<(usize, usize)>,
        on_add: Callback<usize>,
        on_remove: Callback<usize>,
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
                // === Alt+Shift: insert new row === //
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

                // === Ctrl+Shift+K: remove row === //
                Key::Character(ref c) if ctrl && shift && c.eq_ignore_ascii_case("k") => {
                    evt.prevent_default();
                    evt.stop_propagation();
                    // Schedule focus on the next/prev sibling row
                    // *before* the DOM element is removed.
                    document::eval(&KeyboardNav::js_focus_after_field_remove());
                    on_remove.call(index);
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
                        document::eval(&KeyboardNav::js_focus_first_entry(row_data_attr));
                    }
                }
                Key::ArrowDown if ctrl => {
                    evt.prevent_default();
                    evt.stop_propagation();
                    if !is_last {
                        document::eval(&KeyboardNav::js_focus_last_entry(row_data_attr));
                    }
                }

                // === Plain arrows: prev / next row === //
                Key::ArrowUp => {
                    evt.prevent_default();
                    evt.stop_propagation();
                    if !is_first {
                        document::eval(&KeyboardNav::js_focus_prev_entry(row_data_attr));
                    }
                }
                Key::ArrowDown => {
                    evt.prevent_default();
                    evt.stop_propagation();
                    if !is_last {
                        document::eval(&KeyboardNav::js_focus_next_entry(row_data_attr));
                    }
                }

                // === Enter: focus first input inside row === //
                Key::Enter => {
                    evt.prevent_default();
                    evt.stop_propagation();
                    document::eval(&KeyboardNav::js_focus_first_field());
                }

                // === Escape: focus parent card === //
                Key::Escape => {
                    evt.prevent_default();
                    evt.stop_propagation();
                    document::eval(&KeyboardNav::js_focus_parent_entry(card_data_attr));
                }

                // === Tab / Shift+Tab === //
                // Within the row list, Tab/Shift+Tab moves between sibling
                // rows. At the boundaries (first row Shift+Tab, last row
                // Tab), focus breaks out to the next/prev card-level field.
                Key::Tab if shift && is_first => {
                    evt.prevent_default();
                    evt.stop_propagation();
                    document::eval(&KeyboardNav::js_tab_prev_field_from(
                        card_data_attr,
                        row_data_attr,
                    ));
                }
                Key::Tab if !shift && is_last => {
                    evt.prevent_default();
                    evt.stop_propagation();
                    document::eval(&KeyboardNav::js_tab_next_field_from(
                        card_data_attr,
                        row_data_attr,
                    ));
                }
                Key::Tab if shift => {
                    evt.prevent_default();
                    evt.stop_propagation();
                    document::eval(&KeyboardNav::js_focus_prev_entry(row_data_attr));
                }
                Key::Tab => {
                    evt.prevent_default();
                    evt.stop_propagation();
                    document::eval(&KeyboardNav::js_focus_next_entry(row_data_attr));
                }

                // === Space: prevent toggle on parent card === //
                Key::Character(ref c) if c == " " => {
                    evt.prevent_default();
                    evt.stop_propagation();
                }

                _ => {}
            }
        }
    }

    /// Returns an `onkeydown` handler for a field (input, select, button)
    /// inside a reorderable row that is nested within a card.
    ///
    /// The returned closure intercepts **Alt+Up / Alt+Down** for reordering,
    /// **Alt+Shift+Up / Alt+Shift+Down** for insertion, and
    /// **Ctrl+Shift+K** for removal.
    ///
    /// **Tab / Shift+Tab** cycle through all focusable fields within the
    /// parent *card* (identified by `card_data_attr`), so that focus
    /// correctly moves between row fields and card-level fields such as
    /// the edge-kind `<select>` in `EdgeGroupCardFieldKind`.
    ///
    /// **Escape** returns focus to the parent *row* wrapper (identified by
    /// `row_data_attr`), keeping the two-level hierarchy (field -> row ->
    /// card) intact.
    ///
    /// All other keys (arrow keys, Enter, Space) have their propagation
    /// stopped so that row- and card-level handlers do not interfere with
    /// in-field editing (cursor movement, native select navigation, etc.).
    ///
    /// # Parameters
    ///
    /// * `row_data_attr`: the `data-*` attribute on the row wrapper, e.g.
    ///   `"data-entity-type-row"`. Used by Escape to focus the row.
    /// * `card_data_attr`: the `data-*` attribute on the parent card wrapper,
    ///   e.g. `"data-edge-group-card"`. Used by Tab / Shift+Tab to cycle
    ///   through all focusable fields in the card.
    /// * `index`: zero-based position of this row in its list.
    /// * `entry_count`: total number of rows in the list.
    /// * `focus_index`: signal set after a move/add so the container can
    ///   re-focus the row at its new position.
    /// * `on_move`: callback to reorder the row. Receives `(from, to)`.
    /// * `on_add`: callback to insert a new row at a given index.
    /// * `on_remove`: callback to remove the current row by index.
    #[allow(clippy::too_many_arguments)]
    pub fn row_field_onkeydown(
        row_data_attr: &'static str,
        card_data_attr: &'static str,
        index: usize,
        entry_count: usize,
        mut focus_index: Signal<Option<usize>>,
        on_move: Callback<(usize, usize)>,
        on_add: Callback<usize>,
        on_remove: Callback<usize>,
    ) -> impl FnMut(Event<KeyboardData>) {
        let can_move_up = index > 0;
        let can_move_down = index + 1 < entry_count;

        move |evt: Event<KeyboardData>| {
            let alt = evt.modifiers().alt();
            let ctrl = evt.modifiers().ctrl();
            let shift = evt.modifiers().shift();

            match evt.key() {
                // === Alt+Shift: insert new row === //
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

                // === Ctrl+Shift+K: remove row === //
                Key::Character(ref c) if ctrl && shift && c.eq_ignore_ascii_case("k") => {
                    evt.prevent_default();
                    evt.stop_propagation();
                    document::eval(&KeyboardNav::js_focus_after_field_remove());
                    on_remove.call(index);
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

                // === Tab / Shift+Tab: cycle within the card === //
                Key::Tab => {
                    evt.prevent_default();
                    evt.stop_propagation();
                    if shift {
                        document::eval(&KeyboardNav::js_tab_prev_field(card_data_attr));
                    } else {
                        document::eval(&KeyboardNav::js_tab_next_field(card_data_attr));
                    }
                }

                // === Escape: focus parent row wrapper === //
                Key::Escape => {
                    evt.prevent_default();
                    evt.stop_propagation();
                    document::eval(&KeyboardNav::js_focus_parent_entry(row_data_attr));
                }

                // === Enter: stop propagation (allow native behaviour) === //
                Key::Enter => {
                    evt.stop_propagation();
                }

                // === Arrow keys: stop propagation (cursor / select nav) === //
                Key::ArrowUp | Key::ArrowDown | Key::ArrowLeft | Key::ArrowRight => {
                    evt.stop_propagation();
                }

                // === Space: stop propagation (prevent card toggle) === //
                Key::Character(ref c) if c == " " => {
                    evt.stop_propagation();
                }

                _ => {}
            }
        }
    }
}
