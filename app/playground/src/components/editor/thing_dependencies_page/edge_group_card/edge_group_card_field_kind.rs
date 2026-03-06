//! Edge kind selector field.
//!
//! Extracted from [`EdgeGroupCard`] to keep the parent component concise.
//!
//! [`EdgeGroupCard`]: super::EdgeGroupCard

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::Signal,
};
use disposition::input_model::{edge::EdgeKind, thing::ThingId, InputDiagram};

use crate::components::editor::{
    common::{FieldNav, LABEL_CLASS, SELECT_CLASS},
    thing_dependencies_page::{edge_group_card_ops::EdgeGroupCardOps, MapTarget, DATA_ATTR},
};

/// Edge kind selector (Cyclic / Sequence / Symmetric).
///
/// Displays a labelled `<select>` for the edge kind. On change, calls
/// [`EdgeGroupCardOps::edge_kind_change`] to update the `InputDiagram`.
#[component]
pub(crate) fn EdgeGroupCardFieldKind(
    input_diagram: Signal<InputDiagram<'static>>,
    target: MapTarget,
    edge_group_id: String,
    edge_kind: EdgeKind,
    things: Vec<ThingId<'static>>,
) -> Element {
    rsx! {
        div {
            class: "flex flex-col items-start gap-1 pl-4",

            label { class: LABEL_CLASS, "kind" }

            select {
                class: SELECT_CLASS,
                tabindex: "-1",
                value: "{edge_kind}",
                onchange: {
                    let edge_group_id = edge_group_id.clone();
                    let current_things = things.clone();
                    move |evt: dioxus::events::FormEvent| {
                        let kind_str = evt.value();
                        if let Ok(edge_kind_new) = kind_str.parse::<EdgeKind>() {
                            EdgeGroupCardOps::edge_kind_change(
                                input_diagram,
                                target,
                                &edge_group_id,
                                edge_kind_new,
                                &current_things,
                            );
                        }
                    }
                },
                onkeydown: FieldNav::value_onkeydown(DATA_ATTR),
                option { value: "cyclic", "Cyclic" }
                option { value: "sequence", "Sequence" }
                option { value: "symmetric", "Symmetric" }
            }
        }
    }
}
