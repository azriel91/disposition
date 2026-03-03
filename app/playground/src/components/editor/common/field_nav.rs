use dioxus::{
    core::Event,
    html::KeyboardData,
    prelude::{Key, ModifiersInteraction},
    signals::{Signal, WritableExt},
};

use crate::components::editor::{common::RenameRefocusTarget, keyboard_nav};

/// Commonizes logic for navigating fields in a form, including rename refocus
/// targets.
pub struct FieldNav;

impl FieldNav {
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
