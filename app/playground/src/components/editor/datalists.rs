//! `<datalist>` elements for autocomplete on ID fields.
//!
//! Each datalist is populated from the current [`InputDiagram`] so that
//! `<input list="...">` fields get browser-native autocomplete suggestions.

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{Memo, ReadableExt},
};
use disposition::input_model::InputDiagram;

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

/// Built-in [`StyleAlias`] variant names (snake_case, matching YAML keys).
const BUILTIN_STYLE_ALIASES: &[&str] = &[
    "circle_xs",
    "circle_sm",
    "circle_md",
    "circle_lg",
    "circle_xl",
    "padding_none",
    "padding_tight",
    "padding_normal",
    "padding_wide",
    "rounded_xs",
    "rounded_sm",
    "rounded_md",
    "rounded_lg",
    "rounded_xl",
    "rounded_2xl",
    "rounded_3xl",
    "rounded_4xl",
    "fill_pale",
    "shade_pale",
    "shade_light",
    "shade_medium",
    "shade_dark",
    "stroke_dashed_animated",
    "stroke_dashed_animated_request",
    "stroke_dashed_animated_response",
];

/// Renders all `<datalist>` elements derived from the current
/// [`InputDiagram`].
///
/// Place this component once near the root of the editor so that every
/// `<input list="...">` in any editor page can reference the datalists by
/// their well-known IDs.
#[component]
pub fn EditorDataLists(input_diagram: Memo<InputDiagram<'static>>) -> Element {
    let diagram = input_diagram.read();

    // ── Collect IDs ──────────────────────────────────────────────────────

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
    let mut style_alias_values: Vec<String> = BUILTIN_STYLE_ALIASES
        .iter()
        .map(|s| (*s).to_owned())
        .collect();
    for alias in diagram.theme_default.style_aliases.keys() {
        let alias_str = alias.as_str().to_owned();
        if !style_alias_values.contains(&alias_str) {
            style_alias_values.push(alias_str);
        }
    }

    rsx! {
        // ── thing_ids ────────────────────────────────────────────────
        datalist {
            id: list_ids::THING_IDS,
            for id in thing_ids.iter() {
                option { value: "{id}" }
            }
        }

        // ── edge_group_ids ───────────────────────────────────────────
        datalist {
            id: list_ids::EDGE_GROUP_IDS,
            for id in edge_group_ids.iter() {
                option { value: "{id}" }
            }
        }

        // ── tag_ids ──────────────────────────────────────────────────
        datalist {
            id: list_ids::TAG_IDS,
            for id in tag_ids.iter() {
                option { value: "{id}" }
            }
        }

        // ── process_ids ──────────────────────────────────────────────
        datalist {
            id: list_ids::PROCESS_IDS,
            for id in process_ids.iter() {
                option { value: "{id}" }
            }
        }

        // ── process_step_ids ─────────────────────────────────────────
        datalist {
            id: list_ids::PROCESS_STEP_IDS,
            for id in process_step_ids.iter() {
                option { value: "{id}" }
            }
        }

        // ── entity_ids (union) ───────────────────────────────────────
        datalist {
            id: list_ids::ENTITY_IDS,
            for id in entity_ids.iter() {
                option { value: "{id}" }
            }
        }

        // ── style_aliases ────────────────────────────────────────────
        datalist {
            id: list_ids::STYLE_ALIASES,
            for alias in style_alias_values.iter() {
                option { value: "{alias}" }
            }
        }
    }
}
