//! Card component for a single step's thing-interaction list.
//!
//! Displays the step ID, a remove button, and a list of edge group IDs
//! that can be individually edited, removed, or added to.

mod step_interaction_card_field_edges;
mod step_interaction_card_field_edges_row;
mod step_interaction_card_field_id;

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::Signal,
};
use disposition::input_model::InputDiagram;

use crate::components::editor::common::INNER_CARD_CLASS;

use self::{
    step_interaction_card_field_edges::StepInteractionCardFieldEdges,
    step_interaction_card_field_edges_row::StepInteractionCardFieldEdgesRow,
    step_interaction_card_field_id::StepInteractionCardFieldId,
};

/// A card for one step's thing-interaction list.
#[component]
pub(crate) fn StepInteractionCard(
    input_diagram: Signal<InputDiagram<'static>>,
    process_id: String,
    step_id: String,
    edge_ids: Vec<String>,
) -> Element {
    rsx! {
        div {
            class: INNER_CARD_CLASS,

            // Step ID + remove
            StepInteractionCardFieldId {
                input_diagram,
                process_id: process_id.clone(),
                step_id: step_id.clone(),
                edge_ids: edge_ids.clone(),
            }

            // Edge group IDs
            StepInteractionCardFieldEdges {
                input_diagram,
                process_id,
                step_id,
                edge_ids,
            }
        }
    }
}
