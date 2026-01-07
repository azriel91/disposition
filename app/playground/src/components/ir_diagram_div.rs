use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadSignal, ReadableExt},
};

#[component]
pub fn IrDiagramDiv(ir_diagram_string: ReadSignal<String>) -> Element {
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
                    min-h-50
                    rounded-lg
                    border-2
                    border-gray-300
                    p-2
                    font-mono
                ",
                readonly: true,
                value: ir_diagram_string.read().clone(),
            }
        }
    }
}
