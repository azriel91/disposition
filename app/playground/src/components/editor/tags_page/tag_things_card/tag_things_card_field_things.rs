//! Things list field for a [`TagThingsCard`].
//!
//! Extracted from [`TagThingsCard`] to keep the parent component concise.
//!
//! [`TagThingsCard`]: super::TagThingsCard

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::Signal,
};
use disposition::input_model::InputDiagram;

use crate::components::editor::{
    common::{FieldNav, ADD_BTN},
    tags_page::{
        tag_things_card::TagThingsCardFieldThingsRow, tags_page_ops::TagsPageOps, DATA_ATTR,
    },
};

/// Things list with per-row editing and an "add" button.
///
/// Displays each thing ID as a [`TagThingsCardFieldThingsRow`] and provides
/// an "+ Add thing" button at the bottom to append a new entry.
#[component]
pub(crate) fn TagThingsCardFieldThings(
    input_diagram: Signal<InputDiagram<'static>>,
    tag_id: String,
    things: Vec<String>,
) -> Element {
    rsx! {
        div {
            class: "flex flex-col gap-1 pl-4",

            for (idx, thing_id) in things.iter().enumerate() {
                {
                    let thing_id = thing_id.clone();
                    let tag_id = tag_id.clone();
                    rsx! {
                        TagThingsCardFieldThingsRow {
                            key: "{tag_id}_{idx}",
                            input_diagram,
                            tag_id,
                            thing_id,
                            index: idx,
                        }
                    }
                }
            }

            button {
                class: ADD_BTN,
                tabindex: -1,
                onclick: {
                    let tag_id = tag_id.clone();
                    move |_| {
                        TagsPageOps::tag_things_thing_add(input_diagram, &tag_id);
                    }
                },
                onkeydown: FieldNav::value_onkeydown(DATA_ATTR),
                "+ Add thing"
            }
        }
    }
}
