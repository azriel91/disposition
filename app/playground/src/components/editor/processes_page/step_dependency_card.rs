//! Card component for a single step's dependency list.
//!
//! Displays the step ID, a remove button, and a list of process step IDs
//! (the step's prerequisites) that can be individually edited, removed, or
//! added to.

mod step_dependency_card_field_deps;
mod step_dependency_card_field_deps_row;
mod step_dependency_card_field_id;

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::Signal,
};
use disposition::input_model::InputDiagram;

use crate::components::editor::common::INNER_CARD_CLASS;

use self::{
    step_dependency_card_field_deps::StepDependencyCardFieldDeps,
    step_dependency_card_field_deps_row::StepDependencyCardFieldDepsRow,
    step_dependency_card_field_id::StepDependencyCardFieldId,
};

/// A card for one step's dependency list.
#[component]
pub(crate) fn StepDependencyCard(
    input_diagram: Signal<InputDiagram<'static>>,
    process_id: String,
    step_id: String,
    dep_ids: Vec<String>,
) -> Element {
    rsx! {
        div {
            class: INNER_CARD_CLASS,

            // Step ID + remove
            StepDependencyCardFieldId {
                input_diagram,
                process_id: process_id.clone(),
                step_id: step_id.clone(),
                dep_ids: dep_ids.clone(),
            }

            // Dependency step IDs
            StepDependencyCardFieldDeps {
                input_diagram,
                process_id,
                step_id,
                dep_ids,
            }
        }
    }
}
