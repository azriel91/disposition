//! Mutation operations for the Tags editor page.
//!
//! This module is a thin Signal-aware wrapper around
//! [`disposition_input_rt::tags_page_ops::TagsPageOps`]. Each method
//! acquires a read or write guard on the [`Signal`] and delegates to the
//! framework-agnostic implementation.

use dioxus::signals::{Signal, WritableExt};
use disposition::input_model::InputDiagram;

/// Mutation operations for the Tags editor page.
pub(crate) struct TagsPageOps;

impl TagsPageOps {
    // === Tag name helpers === //

    /// Adds a new tag with a unique placeholder TagId.
    pub(crate) fn tag_add(mut input_diagram: Signal<InputDiagram<'static>>) {
        disposition_input_rt::tags_page_ops::TagsPageOps::tag_add(&mut input_diagram.write());
    }

    /// Removes a tag from the `tags` map.
    pub(crate) fn tag_remove(mut input_diagram: Signal<InputDiagram<'static>>, tag_id_str: &str) {
        disposition_input_rt::tags_page_ops::TagsPageOps::tag_remove(
            &mut input_diagram.write(),
            tag_id_str,
        );
    }

    /// Renames a tag across all maps in the [`InputDiagram`].
    pub(crate) fn tag_rename(
        mut input_diagram: Signal<InputDiagram<'static>>,
        tag_id_old_str: &str,
        tag_id_new_str: &str,
    ) {
        disposition_input_rt::tags_page_ops::TagsPageOps::tag_rename(
            &mut input_diagram.write(),
            tag_id_old_str,
            tag_id_new_str,
        );
    }

    /// Updates the display name for an existing tag.
    pub(crate) fn tag_name_update(
        mut input_diagram: Signal<InputDiagram<'static>>,
        tag_id_str: &str,
        name: &str,
    ) {
        disposition_input_rt::tags_page_ops::TagsPageOps::tag_name_update(
            &mut input_diagram.write(),
            tag_id_str,
            name,
        );
    }

    /// Moves a tag entry from one index to another in the `tags` map.
    pub(crate) fn tag_move(
        mut input_diagram: Signal<InputDiagram<'static>>,
        from: usize,
        to: usize,
    ) {
        disposition_input_rt::tags_page_ops::TagsPageOps::tag_move(
            &mut input_diagram.write(),
            from,
            to,
        );
    }

    // === Tag things helpers === //

    /// Moves a tag->things entry from one index to another in the
    /// `tag_things` map.
    pub(crate) fn tag_things_entry_move(
        mut input_diagram: Signal<InputDiagram<'static>>,
        from: usize,
        to: usize,
    ) {
        disposition_input_rt::tags_page_ops::TagsPageOps::tag_things_entry_move(
            &mut input_diagram.write(),
            from,
            to,
        );
    }

    /// Adds a new tag->things entry, picking an unmapped tag or generating a
    /// placeholder.
    pub(crate) fn tag_things_entry_add(mut input_diagram: Signal<InputDiagram<'static>>) {
        disposition_input_rt::tags_page_ops::TagsPageOps::tag_things_entry_add(
            &mut input_diagram.write(),
        );
    }

    /// Removes a tag->things entry.
    pub(crate) fn tag_things_entry_remove(
        mut input_diagram: Signal<InputDiagram<'static>>,
        tag_id_str: &str,
    ) {
        disposition_input_rt::tags_page_ops::TagsPageOps::tag_things_entry_remove(
            &mut input_diagram.write(),
            tag_id_str,
        );
    }

    /// Renames the key of a tag->things entry.
    pub(crate) fn tag_things_entry_rename(
        mut input_diagram: Signal<InputDiagram<'static>>,
        tag_id_old_str: &str,
        tag_id_new_str: &str,
        current_things: &[String],
    ) {
        disposition_input_rt::tags_page_ops::TagsPageOps::tag_things_entry_rename(
            &mut input_diagram.write(),
            tag_id_old_str,
            tag_id_new_str,
            current_things,
        );
    }

    /// Updates a single thing within a tag's thing set at the given index.
    pub(crate) fn tag_things_thing_update(
        mut input_diagram: Signal<InputDiagram<'static>>,
        tag_id_str: &str,
        idx: usize,
        thing_id_new_str: &str,
    ) {
        disposition_input_rt::tags_page_ops::TagsPageOps::tag_things_thing_update(
            &mut input_diagram.write(),
            tag_id_str,
            idx,
            thing_id_new_str,
        );
    }

    /// Removes a thing from a tag's thing set by index.
    pub(crate) fn tag_things_thing_remove(
        mut input_diagram: Signal<InputDiagram<'static>>,
        tag_id_str: &str,
        idx: usize,
    ) {
        disposition_input_rt::tags_page_ops::TagsPageOps::tag_things_thing_remove(
            &mut input_diagram.write(),
            tag_id_str,
            idx,
        );
    }

    /// Adds a thing to a tag's thing set.
    pub(crate) fn tag_things_thing_add(
        mut input_diagram: Signal<InputDiagram<'static>>,
        tag_id_str: &str,
    ) {
        disposition_input_rt::tags_page_ops::TagsPageOps::tag_things_thing_add(
            &mut input_diagram.write(),
            tag_id_str,
        );
    }
}
