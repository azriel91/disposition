//! Things list field with reorderable rows and an add button.
//!
//! Extracted from [`EdgeGroupCard`] to keep the parent component concise.
//!
//! [`EdgeGroupCard`]: super::EdgeGroupCard

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::Signal,
};
use disposition::input_model::{thing::ThingId, InputDiagram};

use crate::components::editor::{
    common::{FieldNav, ADD_BTN, LABEL_CLASS},
    reorderable::ReorderableContainer,
    thing_dependencies_page::{
        edge_group_card::edge_group_card_field_things_row::EdgeGroupCardFieldThingsRow,
        edge_group_card_ops::EdgeGroupCardOps, MapTarget, DATA_ATTR,
    },
};

/// Things list field inside an edge group card.
///
/// Displays a "things" label, a [`ReorderableContainer`] of
/// [`EdgeGroupCardFieldThingsRow`] entries, and an "+ Add thing" button.
#[component]
pub(crate) fn EdgeGroupCardFieldThings(
    input_diagram: Signal<InputDiagram<'static>>,
    target: MapTarget,
    edge_group_id: String,
    things: Vec<ThingId<'static>>,
    thing_focus_idx: Signal<Option<usize>>,
) -> Element {
    let thing_count = things.len();

    rsx! {
        div {
            class: "flex flex-col gap-1 pl-4",

            label { class: LABEL_CLASS, "things" }

            ReorderableContainer {
                data_attr: "data-edge-thing-row".to_owned(),
                section_id: format!("edge_things_{edge_group_id}"),
                focus_index: thing_focus_idx,
                focus_inner_selector: Some("input".to_owned()),

                for (idx, thing_id) in things.iter().enumerate() {
                    {
                        let thing_id = thing_id.clone();
                        let edge_group_id = edge_group_id.clone();
                        rsx! {
                            EdgeGroupCardFieldThingsRow {
                                key: "{edge_group_id}_{idx}",
                                input_diagram,
                                target,
                                edge_group_id,
                                thing_id: thing_id.to_string(),
                                index: idx,
                                thing_count,
                                thing_focus_idx,
                            }
                        }
                    }
                }
            }

            button {
                class: ADD_BTN,
                tabindex: -1,
                onclick: {
                    let edge_group_id = edge_group_id.clone();
                    move |_| {
                        EdgeGroupCardOps::edge_thing_add(input_diagram, target, &edge_group_id);
                    }
                },
                onkeydown: FieldNav::value_onkeydown(DATA_ATTR),
                "+ Add thing"
            }
        }
    }
}
