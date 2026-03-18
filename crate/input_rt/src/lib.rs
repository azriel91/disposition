//! Runtime mutation operations for `disposition` input diagrams.
//!
//! This crate provides pure-logic helpers that operate on
//! `&mut InputDiagram<'static>` (and `&InputDiagram<'static>` for read-only
//! queries). By taking plain references instead of framework-specific signal
//! types the helpers are testable without a UI runtime and can be reused
//! across different frontends.

pub use crate::{
    edge_group_card_ops::EdgeGroupCardOps, entity_types_page_ops::EntityTypesPageOps,
    flat_entry::FlatEntry, map_target::MapTarget, on_change_target::OnChangeTarget,
    process_card_ops::ProcessCardOps, processes_page_ops::ProcessesPageOps,
    step_interaction_card_ops::StepInteractionCardOps,
    style_aliases_section_ops::StyleAliasesSectionOps, tags_page_ops::TagsPageOps,
    thing_layout_ops::ThingLayoutOps, things_page_ops::ThingsPageOps,
};

pub mod flat_entry;
pub mod id_parse;
pub mod id_rename;

mod edge_group_card_ops;
mod entity_types_page_ops;
mod map_target;
mod on_change_target;
mod process_card_ops;
mod processes_page_ops;
mod step_interaction_card_ops;
mod style_aliases_section_ops;
mod tags_page_ops;
mod thing_layout_ops;
mod things_page_ops;
