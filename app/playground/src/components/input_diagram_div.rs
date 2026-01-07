use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, info, rsx, Element, Props},
    signals::{Signal, WritableExt},
};

#[component]
pub fn InputDiagramDiv(input_diagram_string: Signal<String>) -> Element {
    rsx! {
        div {
            id: "input_diagram_div",
            class: "
                flex-1
                flex
                flex-col
            ",
            label {
                for: "input_diagram_text",
                class: "
                    text-gray-300
                    font-bold
                    text-sm
                    mb-2
                ",
                "Input Diagram"
            }
            textarea {
                id: "input_diagram_text",
                oninput: move |event| {
                    let event_value = event.value();
                    info!("changing value! len: {}", event_value.len());
                    input_diagram_string.set(event_value);
                },
                class: "
                    flex-1
                    min-h-50
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
