use dioxus::prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element};

#[component]
pub fn IrDiagramDiv() -> Element {
    rsx! {
        div {
            id: "ir_diagram_div",
            class: "
                flex-1
                flex
                flex-col
            ",
            label {
                for: "ir_diagram_text",
                class: "
                    text-gray-300
                    font-bold
                    text-sm
                    mb-2
                ",
                "IR Diagram"
            }
            textarea {
                id: "ir_diagram_text",
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
