//! Things page mutation helpers.
//!
//! This module is a thin Signal-aware wrapper around
//! [`disposition_input_rt::things_page_ops::ThingsPageOps`]. Each method
//! acquires a read or write guard on the [`Signal`] and delegates to the
//! framework-agnostic implementation.

use dioxus::signals::{Signal, WritableExt};
use disposition::input_model::InputDiagram;
use disposition_input_rt::on_change_target::OnChangeTarget;

/// Mutation operations for the Things editor page.
///
/// Grouped here so that related functions are discoverable when sorted by
/// name, per the project's `noun_verb` naming convention.
pub struct ThingsPageOps;

impl ThingsPageOps {
    // === Thing helpers === //

    /// Adds a new thing row with a unique placeholder ID.
    ///
    /// Also inserts a corresponding entry into `thing_hierarchy` with an
    /// empty hierarchy so the thing appears in the layout editor.
    pub fn thing_add(mut input_diagram: Signal<InputDiagram<'static>>) {
        disposition_input_rt::things_page_ops::ThingsPageOps::thing_add(&mut input_diagram.write());
    }

    /// Duplicates a thing and all its entries across the [`InputDiagram`].
    ///
    /// The duplicate ID is derived from the original:
    ///
    /// * If the original ends with `_<number>` (e.g. `t_something_1`), the
    ///   number is incremented (`t_something_2`). Leading zeroes are preserved
    ///   when the incremented value fits in the same width (e.g.
    ///   `t_something_001` becomes `t_something_002`), but an extra digit is
    ///   added when it overflows (e.g. `_999` becomes `_1000`).
    /// * Otherwise `_copy_1` is appended (e.g. `t_foo` becomes `t_foo_copy_1`).
    ///   A subsequent duplicate of `t_foo_copy_1` will increment the trailing
    ///   number to `t_foo_copy_2`.
    ///
    /// If the candidate ID already exists, the number keeps incrementing
    /// until a unique ID is found.
    ///
    /// For `thing_hierarchy`, the duplicate is inserted as a sibling
    /// immediately after the original but with an empty child hierarchy
    /// (children can only have one parent).
    pub fn thing_duplicate(mut input_diagram: Signal<InputDiagram<'static>>, thing_id_str: &str) {
        disposition_input_rt::things_page_ops::ThingsPageOps::thing_duplicate(
            &mut input_diagram.write(),
            thing_id_str,
        );
    }

    /// Updates the display name for an existing thing.
    pub fn thing_name_update(
        mut input_diagram: Signal<InputDiagram<'static>>,
        thing_id_str: &str,
        name: &str,
    ) {
        disposition_input_rt::things_page_ops::ThingsPageOps::thing_name_update(
            &mut input_diagram.write(),
            thing_id_str,
            name,
        );
    }

    /// Renames a thing across all maps in the [`InputDiagram`].
    pub fn thing_rename(
        mut input_diagram: Signal<InputDiagram<'static>>,
        thing_id_old_str: &str,
        thing_id_new_str: &str,
    ) {
        disposition_input_rt::things_page_ops::ThingsPageOps::thing_rename(
            &mut input_diagram.write(),
            thing_id_old_str,
            thing_id_new_str,
        );
    }

    /// Removes a thing and all references to it from the [`InputDiagram`].
    ///
    /// Uses `shift_remove` to preserve ordering of remaining entries.
    pub fn thing_remove(mut input_diagram: Signal<InputDiagram<'static>>, thing_id_str: &str) {
        disposition_input_rt::things_page_ops::ThingsPageOps::thing_remove(
            &mut input_diagram.write(),
            thing_id_str,
        );
    }

    /// Moves a thing entry from one index to another in the `things` map.
    pub fn thing_move(mut input_diagram: Signal<InputDiagram<'static>>, from: usize, to: usize) {
        disposition_input_rt::things_page_ops::ThingsPageOps::thing_move(
            &mut input_diagram.write(),
            from,
            to,
        );
    }

    // === Copy text helpers === //

    /// Adds a new copy-text row with a unique placeholder ThingId.
    pub fn copy_text_add(mut input_diagram: Signal<InputDiagram<'static>>) {
        disposition_input_rt::things_page_ops::ThingsPageOps::copy_text_add(
            &mut input_diagram.write(),
        );
    }

    /// Adds a new entity description row with a unique placeholder Id.
    pub fn entity_desc_add(mut input_diagram: Signal<InputDiagram<'static>>) {
        disposition_input_rt::things_page_ops::ThingsPageOps::entity_desc_add(
            &mut input_diagram.write(),
        );
    }

    /// Adds a new entity tooltip row with a unique placeholder Id.
    pub fn entity_tooltip_add(mut input_diagram: Signal<InputDiagram<'static>>) {
        disposition_input_rt::things_page_ops::ThingsPageOps::entity_tooltip_add(
            &mut input_diagram.write(),
        );
    }

    // === Key-value (copy-text / desc / tooltip) mutation helpers === //

    /// Renames the key of a key-value entry in the target map.
    pub fn kv_entry_rename(
        mut input_diagram: Signal<InputDiagram<'static>>,
        target: OnChangeTarget,
        id_old_str: &str,
        id_new_str: &str,
        current_value: &str,
    ) {
        disposition_input_rt::things_page_ops::ThingsPageOps::kv_entry_rename(
            &mut input_diagram.write(),
            target,
            id_old_str,
            id_new_str,
            current_value,
        );
    }

    /// Updates the value of a key-value entry in the target map.
    pub fn kv_entry_update(
        mut input_diagram: Signal<InputDiagram<'static>>,
        target: OnChangeTarget,
        id_str: &str,
        value: &str,
    ) {
        disposition_input_rt::things_page_ops::ThingsPageOps::kv_entry_update(
            &mut input_diagram.write(),
            target,
            id_str,
            value,
        );
    }

    /// Removes a key-value entry from the target map.
    pub fn kv_entry_remove(
        mut input_diagram: Signal<InputDiagram<'static>>,
        target: OnChangeTarget,
        id_str: &str,
    ) {
        disposition_input_rt::things_page_ops::ThingsPageOps::kv_entry_remove(
            &mut input_diagram.write(),
            target,
            id_str,
        );
    }

    /// Moves a key-value entry from one index to another in the target map.
    pub fn kv_entry_move(
        mut input_diagram: Signal<InputDiagram<'static>>,
        target: OnChangeTarget,
        from: usize,
        to: usize,
    ) {
        disposition_input_rt::things_page_ops::ThingsPageOps::kv_entry_move(
            &mut input_diagram.write(),
            target,
            from,
            to,
        );
    }
}
