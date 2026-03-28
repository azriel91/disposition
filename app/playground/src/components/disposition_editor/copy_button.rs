use std::time::Duration;

use dioxus::{
    hooks::use_signal,
    prelude::{
        component, debug, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props,
    },
    signals::{Memo, WritableExt},
};

use crate::hooks::use_timeout;

/// A button that copies text to the clipboard and displays a "Copied!" message
/// for a short duration.
#[component]
pub fn CopyButton(text_to_copy: Memo<String>) -> Element {
    let mut clipboard = dioxus_clipboard::hooks::use_clipboard();
    let mut copied_signal = use_signal(|| false);
    let mut copied_text_visibility = use_signal(|| "hidden");
    let _copied_timeout = use_timeout(Duration::from_secs(1), copied_signal, move || {
        copied_text_visibility.set("hidden");
    });

    rsx! {
        button {
            class: "\
                flex-none \
                flex \
                justify-center \
                items-center \
                p-2 \
                text-gray-200 \
                rounded-lg \
                bg-gray-800 \
                border-gray-600 \
                hover:bg-gray-700 \
                active:bg-gray-800 \
                focus:outline-none \
                focus:ring-2 \
                focus:ring-blue-600 \
                focus:ring-offset-2 \
                focus:ring-offset-gray-800\
            ",
            tabindex: "0",
            title: "Copy to clipboard",
            onclick: move |_| async move {
                match clipboard.set(text_to_copy().clone()).await {
                    Ok(()) => {
                        copied_text_visibility.set("visible");
                        copied_signal.set(true);
                    }
                    Err(e) => {
                        debug!("Failed to copy text to clipboard: {:?}", e);
                    }
                }
            },

            span {
                class: "\
                    mr-1 \
                    text-sm \
                    font-semibold \
                    {copied_text_visibility}\
                ",
                "Copied!"
            }

            DoubleSquareOutline {}
        },
    }
}

/// Two squares with rounded corners, used as an icon for the copy button.
#[component]
pub fn DoubleSquareOutline() -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            class: "\
                h-5 \
                w-5 \
                [&>*]:stroke-2 \
                [&>*]:stroke-gray-300 \
                hover:[&>*]:stroke-gray-100\
            ",
            width: "24",
            height: "24",
            view_box: "0 0 24 24",
            rect {
                x: "2",
                y: "9",
                width: "13",
                height: "13",
                rx: "2",
                ry: "2",
                fill: "none",
                stroke: "currentColor",
            },
            path {
                d: "\
                    M 10 5 \
                    V 4 \
                    a 2 2 0 0 1 2 -2 \
                    H 20 \
                    a 2 2 0 0 1 2 2 \
                    V 13 \
                    a 2 2 0 0 1 -2 2 \
                    h -1\
                ",
                fill: "none",
                stroke: "currentColor",
            },
        }
    }
}
