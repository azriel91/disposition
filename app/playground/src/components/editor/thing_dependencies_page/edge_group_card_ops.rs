//! Mutation operations for the edge group card component.
//!
//! This module is a thin Signal-aware wrapper around
//! [`disposition_input_rt::edge_group_card_ops::EdgeGroupCardOps`]. Each method
//! acquires a read or write guard on the [`Signal`] and delegates to the
//! framework-agnostic implementation.

use dioxus::signals::{Signal, WritableExt};
use disposition::input_model::{edge::EdgeKind, thing::ThingId, InputDiagram};

use super::MapTarget;

/// Mutation operations for the edge group card component.
pub(crate) struct EdgeGroupCardOps;

impl EdgeGroupCardOps {
    /// Adds a new edge group with a unique placeholder ID.
    pub(crate) fn edge_group_add(
        mut input_diagram: Signal<InputDiagram<'static>>,
        target: MapTarget,
    ) {
        disposition_input_rt::edge_group_card_ops::EdgeGroupCardOps::edge_group_add(
            &mut input_diagram.write(),
            target.into_rt(),
        );
    }

    /// Moves an edge group from one index to another in the target map.
    pub(crate) fn edge_group_move(
        mut input_diagram: Signal<InputDiagram<'static>>,
        target: MapTarget,
        from: usize,
        to: usize,
    ) {
        disposition_input_rt::edge_group_card_ops::EdgeGroupCardOps::edge_group_move(
            &mut input_diagram.write(),
            target.into_rt(),
            from,
            to,
        );
    }

    /// Removes an edge group from the target map.
    pub(crate) fn edge_group_remove(
        mut input_diagram: Signal<InputDiagram<'static>>,
        target: MapTarget,
        edge_group_id_str: &str,
    ) {
        disposition_input_rt::edge_group_card_ops::EdgeGroupCardOps::edge_group_remove(
            &mut input_diagram.write(),
            target.into_rt(),
            edge_group_id_str,
        );
    }

    /// Renames an edge group across all maps in the [`InputDiagram`].
    pub(crate) fn edge_group_rename(
        mut input_diagram: Signal<InputDiagram<'static>>,
        edge_group_id_old_str: &str,
        edge_group_id_new_str: &str,
    ) {
        disposition_input_rt::edge_group_card_ops::EdgeGroupCardOps::edge_group_rename(
            &mut input_diagram.write(),
            edge_group_id_old_str,
            edge_group_id_new_str,
        );
    }

    /// Changes the edge kind (cyclic / sequence / symmetric) for an edge
    /// group, preserving the current thing list.
    pub(crate) fn edge_kind_change(
        mut input_diagram: Signal<InputDiagram<'static>>,
        target: MapTarget,
        edge_group_id_str: &str,
        edge_kind_new: EdgeKind,
        current_things: &[ThingId<'static>],
    ) {
        disposition_input_rt::edge_group_card_ops::EdgeGroupCardOps::edge_kind_change(
            &mut input_diagram.write(),
            target.into_rt(),
            edge_group_id_str,
            edge_kind_new,
            current_things,
        );
    }

    /// Updates a single thing within an edge group at the given index.
    pub(crate) fn edge_thing_update(
        mut input_diagram: Signal<InputDiagram<'static>>,
        target: MapTarget,
        edge_group_id_str: &str,
        idx: usize,
        thing_id_new_str: &str,
    ) {
        disposition_input_rt::edge_group_card_ops::EdgeGroupCardOps::edge_thing_update(
            &mut input_diagram.write(),
            target.into_rt(),
            edge_group_id_str,
            idx,
            thing_id_new_str,
        );
    }

    /// Removes a thing from an edge group by index.
    pub(crate) fn edge_thing_remove(
        mut input_diagram: Signal<InputDiagram<'static>>,
        target: MapTarget,
        edge_group_id_str: &str,
        idx: usize,
    ) {
        disposition_input_rt::edge_group_card_ops::EdgeGroupCardOps::edge_thing_remove(
            &mut input_diagram.write(),
            target.into_rt(),
            edge_group_id_str,
            idx,
        );
    }

    /// Moves a thing within an edge group from one index to another.
    pub(crate) fn edge_thing_move(
        mut input_diagram: Signal<InputDiagram<'static>>,
        target: MapTarget,
        edge_group_id_str: &str,
        from: usize,
        to: usize,
    ) {
        disposition_input_rt::edge_group_card_ops::EdgeGroupCardOps::edge_thing_move(
            &mut input_diagram.write(),
            target.into_rt(),
            edge_group_id_str,
            from,
            to,
        );
    }

    /// Adds a thing to an edge group, using the first existing thing ID as a
    /// placeholder.
    pub(crate) fn edge_thing_add(
        mut input_diagram: Signal<InputDiagram<'static>>,
        target: MapTarget,
        edge_group_id_str: &str,
    ) {
        disposition_input_rt::edge_group_card_ops::EdgeGroupCardOps::edge_thing_add(
            &mut input_diagram.write(),
            target.into_rt(),
            edge_group_id_str,
        );
    }
}
