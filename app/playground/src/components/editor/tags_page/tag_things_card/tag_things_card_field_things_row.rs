//! A single thing row within the things list of a [`TagThingsCard`].
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
    common::{FieldNav, REMOVE_BTN, ROW_CLASS_SIMPLE},
    datalists::list_ids,
    tags_page::{tags_page_ops::TagsPageOps, DATA_ATTR, FIELD_INPUT_CLASS},
};

/// A single thing row within the things list of a tag-things card.
///
/// Displays an index label, a thing ID input (with datalist), and a remove
/// button for one entry in the tag's thing list.
#[component]
pub(crate) fn TagThingsCardFieldThingsRow(
    input_diagram: Signal<InputDiagram<'static>>,
    tag_id: String,
    thing_id: String,
    index: usize,
) -> Element {
    rsx! {
        div {
            class: ROW_CLASS_SIMPLE,

            span {
                class: "text-xs text-gray-500 w-6 text-right",
                "{index}."
            }

            input {
                class: FIELD_INPUT_CLASS,
                style: "max-width:14rem",
                tabindex: "-1",
                list: list_ids::THING_IDS,
                placeholder: "thing_id",
                value: "{thing_id}",
                onchange: {
                    let tag_id = tag_id.clone();
                    move |evt: dioxus::events::FormEvent| {
                        TagsPageOps::tag_things_thing_update(
                            input_diagram,
                            &tag_id,
                            index,
                            &evt.value(),
                        );
                    }
                },
                onkeydown: FieldNav::value_onkeydown(DATA_ATTR),
            }

            button {
                class: REMOVE_BTN,
                tabindex: "-1",
                "data-action": "remove",
                onclick: {
                    let tag_id = tag_id.clone();
                    move |_| {
                        TagsPageOps::tag_things_thing_remove(
                            input_diagram,
                            &tag_id,
                            index,
                        );
                    }
                },
                onkeydown: FieldNav::value_onkeydown(DATA_ATTR),
                "\u{2715}"
            }
        }
    }
}
