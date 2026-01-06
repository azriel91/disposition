use dioxus::prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element};

#[component]
pub fn TaffyNodeMappingsDiv() -> Element {
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
                    h-100
                    rounded-lg
                    border-2
                    border-gray-300
                    p-2
                    font-mono
                "
            }
        }
    }
}
