use dioxus::prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element};

#[component]
pub fn DispositionEditor() -> Element {
    rsx! {
        div {
            // Attributes should be defined in the element before any children
            id: "disposition_editor",
            class: "
                flex
                flex-col
                items-center
                justify-center
            ",
            // components
            div {
                id: "input_diagram_panel",
                class: "flex flex-col",
                // components
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
                    class: "
                        w-150
                        h-full
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
}
