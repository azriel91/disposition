use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadSignal, ReadableExt},
};

#[component]
pub fn EdgeRoutingDiagnosticsDiv(edge_routing_diagnostics_string: ReadSignal<String>) -> Element {
    rsx! {
        div {
            id: "edge_routing_diagnostics_div",
            class: "
                flex
                flex-col
            ",
            label {
                for: "edge_routing_diagnostics_text",
                class: "
                    text-gray-300
                    font-bold
                    text-sm
                    mb-2
                ",
                "Edge Routing"
            }
            textarea {
                id: "edge_routing_diagnostics_text",
                class: "
                    flex-1
                    min-h-50
                    rounded-lg
                    border-2
                    border-gray-300
                    p-2
                    font-mono
                    text-nowrap
                ",
                readonly: true,
                value: edge_routing_diagnostics_string.read().clone(),
            }
        }
    }
}
