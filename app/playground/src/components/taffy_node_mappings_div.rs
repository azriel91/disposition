use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadSignal, ReadableExt},
};

#[component]
pub fn TaffyNodeMappingsDiv(taffy_node_mappings_string: ReadSignal<String>) -> Element {
    rsx! {
        div {
            id: "taffy_node_mappings_div",
            class: "
                flex-1
                flex
                flex-col
            ",
            label {
                for: "taffy_node_mappings_text",
                class: "
                    text-gray-300
                    font-bold
                    text-sm
                    mb-2
                ",
                "Taffy Node Mappings"
            }
            textarea {
                id: "taffy_node_mappings_text",
                class: "
                    flex-1
                    min-h-50
                    rounded-lg
                    border-2
                    border-gray-300
                    p-2
                    font-mono
                ",
                readonly: true,
                value: taffy_node_mappings_string.read().clone(),
            }
        }
    }
}
