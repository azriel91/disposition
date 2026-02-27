//! `<datalist>` elements for autocomplete on ID fields.
//!
//! Each datalist is populated from the current [`InputDiagram`] so that
//! `<input list="...">` fields get browser-native autocomplete suggestions.

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{Memo, ReadableExt},
};
use disposition::{
    input_model::{theme::StyleAlias, InputDiagram},
    model_common::Set,
};

/// Well-known datalist element IDs that editor pages can reference via
/// `<input list="...">`.
pub mod list_ids {
    /// All `ThingId`s currently defined in `things`.
    pub const THING_IDS: &str = "thing_ids";
    /// All `EdgeGroupId`s from `thing_dependencies` and `thing_interactions`.
    pub const EDGE_GROUP_IDS: &str = "edge_group_ids";
    /// All `TagId`s from `tags`.
    pub const TAG_IDS: &str = "tag_ids";
    /// All `ProcessId`s from `processes`.
    pub const PROCESS_IDS: &str = "process_ids";
    /// All `ProcessStepId`s from every process's `steps`.
    pub const PROCESS_STEP_IDS: &str = "process_step_ids";
    /// Union of thing, edge-group, tag, process, and process-step IDs.
    pub const ENTITY_IDS: &str = "entity_ids";
    /// Built-in + user-defined `StyleAlias` values.
    pub const STYLE_ALIASES: &str = "style_aliases";
}

/// Renders all `<datalist>` elements derived from the current
/// [`InputDiagram`].
///
/// Place this component once near the root of the editor so that every
/// `<input list="...">` in any editor page can reference the datalists by
/// their well-known IDs.
#[component]
pub fn EditorDataLists(input_diagram: Memo<InputDiagram<'static>>) -> Element {
    let diagram = input_diagram.read();

    // === Collect IDs === //

    let thing_ids: Vec<String> = diagram
        .things
        .keys()
        .map(|id| id.as_str().to_owned())
        .collect();

    let edge_group_ids: Vec<String> = diagram
        .thing_dependencies
        .keys()
        .chain(diagram.thing_interactions.keys())
        .map(|id| id.as_str().to_owned())
        .collect();

    let tag_ids: Vec<String> = diagram
        .tags
        .keys()
        .map(|id| id.as_str().to_owned())
        .collect();

    let process_ids: Vec<String> = diagram
        .processes
        .keys()
        .map(|id| id.as_str().to_owned())
        .collect();

    let process_step_ids: Vec<String> = diagram
        .processes
        .values()
        .flat_map(|proc| proc.steps.keys())
        .map(|id| id.as_str().to_owned())
        .collect();

    // Entity IDs = union of all the above.
    let entity_ids: Vec<String> = thing_ids
        .iter()
        .chain(edge_group_ids.iter())
        .chain(tag_ids.iter())
        .chain(process_ids.iter())
        .chain(process_step_ids.iter())
        .cloned()
        .collect();

    // Style aliases = builtins + user-defined custom aliases.
    let style_alias_values = {
        let input_diagram_base = InputDiagram::base();
        let base_style_aliases = input_diagram_base.theme_default.style_aliases.keys();
        let input_diagram_style_aliases = diagram.theme_default.style_aliases.keys();

        base_style_aliases.chain(input_diagram_style_aliases).fold(
            Set::<StyleAlias>::new(),
            |mut style_alias_values, style_alias| {
                if !(style_alias_values.contains(style_alias)) {
                    style_alias_values.insert(style_alias.clone());
                }
                style_alias_values
            },
        )
    };

    rsx! {
        // === thing_ids === //
        datalist {
            id: list_ids::THING_IDS,
            for id in thing_ids.iter() {
                option { value: "{id}" }
            }
        }

        // === edge_group_ids === //
        datalist {
            id: list_ids::EDGE_GROUP_IDS,
            for id in edge_group_ids.iter() {
                option { value: "{id}" }
            }
        }

        // === tag_ids === //
        datalist {
            id: list_ids::TAG_IDS,
            for id in tag_ids.iter() {
                option { value: "{id}" }
            }
        }

        // === process_ids === //
        datalist {
            id: list_ids::PROCESS_IDS,
            for id in process_ids.iter() {
                option { value: "{id}" }
            }
        }

        // === process_step_ids === //
        datalist {
            id: list_ids::PROCESS_STEP_IDS,
            for id in process_step_ids.iter() {
                option { value: "{id}" }
            }
        }

        // === entity_ids (union) === //
        datalist {
            id: list_ids::ENTITY_IDS,
            for id in entity_ids.iter() {
                option { value: "{id}" }
            }
        }

        // === style_aliases === //
        datalist {
            id: list_ids::STYLE_ALIASES,
            for alias in style_alias_values.iter() {
                option { value: "{alias}" }
            }
        }
    }
}
