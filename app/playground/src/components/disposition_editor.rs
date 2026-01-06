use dioxus::prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element};

use crate::components::{InputDiagramDiv, IrDiagramDiv, TaffyNodeMappingsDiv};

#[component]
pub fn DispositionEditor() -> Element {
    rsx! {
        div {
            id: "disposition_editor",
            class: "
                flex
                flex-col
                gap-2
            ",
            DispositionDataDivs {}
            DispositionStatusMessageDiv {}
        }
    }
}

#[component]
fn DispositionDataDivs() -> Element {
    rsx! {
        div {
            id: "disposition_data_divs",
            class: "
                w-full
                flex
                flex-row
                flex-wrap
                items-center
                justify-center
            ",
            InputDiagramDiv {}
            IrDiagramDiv {}
            TaffyNodeMappingsDiv {}
        }
    }
}

#[component]
fn DispositionStatusMessageDiv() -> Element {
    rsx! {
        div {
            id: "disposition_status_message_div",
            class: "
                w-full
                flex
                flex-col
                gap-1
            ",
            h3 {
                class: "
                    text-sm
                    font-bold
                    text-gray-300
                ",
                "Status Message"
            }
            DispositionStatusMessage {}
        }
    }
}

#[component]
fn DispositionStatusMessage() -> Element {
    rsx! {
        div {
            id: "disposition_status_message",
            class: "
                rounded-lg
                border
                border-gray-300
                bg-gray-800
                font-mono
                p-2
                select-text
            ",
            "Status text"
        }
    }
}
