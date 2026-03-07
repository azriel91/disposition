//! Runtime mutation operations for `disposition` input diagrams.
//!
//! This crate provides pure-logic helpers that operate on
//! `&mut InputDiagram<'static>` (and `&InputDiagram<'static>` for read-only
//! queries). By taking plain references instead of framework-specific signal
//! types the helpers are testable without a UI runtime and can be reused
//! across different frontends.
//!
//! # Modules
//!
//! * [`id_parse`] -- strongly-typed ID parsers (`parse_thing_id`,
//!   `parse_edge_group_id`, etc.).
//! * [`id_rename`] -- shared rename-across-diagram helpers.
//! * [`flat_entry`] -- flat representation of the recursive `ThingHierarchy`
//!   tree.
//! * [`on_change_target`] -- target map selector for generic key-value row
//!   mutations.
//! * [`map_target`] -- selector for `thing_dependencies` vs
//!   `thing_interactions`.
//! * [`edge_group_card_ops`] -- mutation helpers for edge group entries.
//! * [`process_card_ops`] -- mutation helpers for process card entries.
//! * [`processes_page_ops`] -- mutation helpers for the processes page.
//! * [`step_interaction_card_ops`] -- mutation helpers for step interaction
//!   cards.
//! * [`style_aliases_section_ops`] -- mutation helpers for style alias renames.
//! * [`tags_page_ops`] -- mutation helpers for the tags page.
//! * [`thing_layout_ops`] -- mutation helpers for the thing layout page.
//! * [`things_page_ops`] -- mutation helpers for the things page.

pub use crate::{
    edge_group_card_ops::EdgeGroupCardOps, flat_entry::FlatEntry, map_target::MapTarget,
    on_change_target::OnChangeTarget, process_card_ops::ProcessCardOps,
    processes_page_ops::ProcessesPageOps, step_interaction_card_ops::StepInteractionCardOps,
    style_aliases_section_ops::StyleAliasesSectionOps, tags_page_ops::TagsPageOps,
    thing_layout_ops::ThingLayoutOps, things_page_ops::ThingsPageOps,
};

pub mod flat_entry;
pub mod id_parse;
pub mod id_rename;

mod edge_group_card_ops;
mod map_target;
mod on_change_target;
mod process_card_ops;
mod processes_page_ops;
mod step_interaction_card_ops;
mod style_aliases_section_ops;
mod tags_page_ops;
mod thing_layout_ops;
mod things_page_ops;
