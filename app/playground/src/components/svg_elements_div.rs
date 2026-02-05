use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadSignal, ReadableExt},
};

#[component]
pub fn SvgElementsDiv(svg_elements_string: ReadSignal<String>) -> Element {
    rsx! {
        div {
            id: "svg_elements_div",
            class: "
                flex
                flex-col
            ",
            label {
                for: "svg_elements_text",
                class: "
                    text-gray-300
                    font-bold
                    text-sm
                    mb-2
                ",
                "SVG Elements"
            }
            textarea {
                id: "svg_elements_text",
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
                value: svg_elements_string.read().clone(),
            }
        }
    }
}
