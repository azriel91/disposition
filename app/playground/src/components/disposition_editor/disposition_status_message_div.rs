//! Disposition status message components.
//!
//! Renders the current pipeline status messages (info, warnings, errors)
//! in the editor sidebar.

use std::hash::{DefaultHasher, Hasher};

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{Memo, ReadableExt},
};

/// Outer container for the status message section.
#[component]
pub fn DispositionStatusMessageDiv(status_messages: Memo<Vec<String>>) -> Element {
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
            DispositionStatusMessage {
                status_messages,
            }
        }
    }
}

/// Renders the list of status messages inside a styled card.
#[component]
fn DispositionStatusMessage(status_messages: Memo<Vec<String>>) -> Element {
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
            ul {
                class: "
                    list-disc
                    list-inside
                ",
                for message in status_messages.read().iter() {
                    {
                        let message: &String = message;
                        let mut hasher = DefaultHasher::new();
                        hasher.write(message.as_bytes());
                        let key = hasher.finish();

                        rsx! {
                            li {
                                key: "{key}",
                                class: "
                                    text-sm
                                    text-gray-300
                                ",
                                "{message}"
                            }
                        }
                    }
                }
            }
        }
    }
}
