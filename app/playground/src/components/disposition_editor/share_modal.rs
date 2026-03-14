//! Share modal component for the disposition editor.
//!
//! Renders a modal dialog with a shareable URL. The URL includes the full
//! editor state serialized into the hash fragment. An optional checkbox
//! controls whether the `focus_field` is included so the recipient's
//! browser focuses the same field the sharer was editing.

use std::time::Duration;

use dioxus::{
    core::spawn,
    hooks::{use_effect, use_memo, use_signal},
    prelude::{
        component, debug, dioxus_core, dioxus_elements, dioxus_signals, document, rsx, Element,
        Key, Props,
    },
    signals::{ReadableExt, Signal, WritableExt},
};

use crate::{editor_state::EditorState, hooks::use_timeout};

// === CSS constants === //

/// Backdrop overlay covering the full viewport.
const BACKDROP_CLASS: &str = "\
    fixed \
    inset-0 \
    z-50 \
    flex \
    items-center \
    justify-center \
    bg-black/50\
";

/// The modal panel itself.
const MODAL_CLASS: &str = "\
    relative \
    w-full \
    max-w-lg \
    rounded-lg \
    border \
    border-gray-600 \
    bg-gray-800 \
    p-4 \
    shadow-xl \
    flex \
    flex-col \
    gap-3\
";

/// Header row: title + close button.
const HEADER_CLASS: &str = "flex flex-row items-center";

/// Close button in the top-right corner.
const CLOSE_BTN_CLASS: &str = "\
    text-gray-500 \
    hover:text-gray-300 \
    cursor-pointer \
    text-sm \
    px-1 \
    select-none\
";

/// The URL input field.
const URL_INPUT_CLASS: &str = "\
    w-full \
    rounded \
    border \
    border-gray-600 \
    bg-gray-900 \
    text-gray-200 \
    px-2 \
    py-1 \
    text-sm \
    font-mono \
    focus:border-blue-400 \
    focus:outline-none \
    select-all\
";

/// Row containing the checkbox + label.
const CHECKBOX_ROW_CLASS: &str = "flex flex-row items-center gap-2 text-sm text-gray-300";

/// Copy button inside the modal.
const COPY_BTN_CLASS: &str = "\
    flex-none \
    flex \
    justify-center \
    items-center \
    px-3 \
    py-1 \
    text-sm \
    text-gray-200 \
    rounded \
    bg-blue-600 \
    hover:bg-blue-500 \
    active:bg-blue-700 \
    focus:outline-none \
    focus:ring-2 \
    focus:ring-blue-400 \
    cursor-pointer \
    select-none\
";

/// A modal dialog that displays a shareable URL for the current editor state.
///
/// # Props
///
/// * `show`: signal controlling visibility. Set to `false` to close.
/// * `editor_state`: the current editor state (page + diagram).
/// * `last_focused_field`: the `data-input-diagram-field` value of the last
///   focused editor field, if any. Used to populate `focus_field` in the URL
///   when the checkbox is ticked.
#[component]
pub fn ShareModal(
    mut show: Signal<bool>,
    editor_state: Signal<EditorState>,
    last_focused_field: Signal<Option<String>>,
) -> Element {
    if !*show.read() {
        return rsx! {};
    }

    // Whether the "Include focused field" checkbox is checked.
    let mut include_focus = use_signal(|| true);

    // Build the share URL reactively.
    let share_url = use_memo(move || {
        let state = editor_state.read().clone();
        let focus = if *include_focus.read() {
            last_focused_field.read().clone()
        } else {
            None
        };
        let share_state = EditorState {
            page: state.page,
            focus_field: focus,
            input_diagram: state.input_diagram,
        };
        // The URL is `/#<yaml>`. We build it by serializing the state
        // and prepending the origin + path.
        let yaml = share_state.to_string();

        // Modern browsers don't need the full URL encoding, but we have to encode
        // newlines.
        //
        // ```rust
        // let yaml_encoded = urlencoding::encode(&yaml).to_string();
        // ```
        let yaml_encoded = yaml.replace('\n', "%0A");
        format!("#{yaml_encoded}")
    });

    // Full URL including origin -- computed via JS since we don't have
    // access to `window.location.origin` directly in Rust.
    let mut full_url: Signal<String> = use_signal(|| String::new());
    use_effect(move || {
        let fragment = share_url.read().clone();
        spawn(async move {
            let origin =
                document::eval("dioxus.send(window.location.origin + window.location.pathname);")
                    .recv::<String>()
                    .await
                    .ok()
                    .unwrap_or_default();
            full_url.set(format!("{origin}{fragment}"));
        });
    });

    // Copy-to-clipboard feedback.
    let mut clipboard = dioxus_clipboard::hooks::use_clipboard();
    let mut copied_signal = use_signal(|| false);
    let mut copied_label = use_signal(|| "Copy");
    let _copied_timeout = use_timeout(Duration::from_secs(2), copied_signal, move || {
        copied_label.set("Copy");
    });

    // Whether the last focused field is available (controls checkbox
    // enabled state and label).
    let has_focus_field = last_focused_field.read().is_some();

    // Auto-focus and select the URL input when the modal appears.
    use_effect(move || {
        let _show = show.read(); // Run this each time show changes, not only the first time.
        document::eval(
            "requestAnimationFrame(() => {\
                var el = document.querySelector('[data-share-url-input]');\
                if (el) { el.focus(); el.select(); }\
            })",
        );
    });

    rsx! {
        div {
            class: BACKDROP_CLASS,
            // Clicking the backdrop closes the modal.
            onclick: move |_| show.set(false),

            div {
                class: MODAL_CLASS,
                // Prevent clicks inside the modal from closing it.
                onclick: |evt| evt.stop_propagation(),
                // Escape key closes the modal.
                onkeydown: move |evt| {
                    if evt.key() == Key::Escape {
                        evt.prevent_default();
                        evt.stop_propagation();
                        show.set(false);
                    }
                },

                // === Header === //
                div {
                    class: HEADER_CLASS,
                    span {
                        class: "flex-1 font-bold text-gray-200 text-sm",
                        "Share"
                    }
                    span {
                        class: CLOSE_BTN_CLASS,
                        onclick: move |_| show.set(false),
                        "❌"
                    }
                }

                // === URL input === //
                input {
                    class: URL_INPUT_CLASS,
                    r#type: "text",
                    readonly: true,
                    "data-share-url-input": "",
                    value: "{full_url}",
                    // Select all text on focus for easy copying.
                    onfocus: |_| {
                        document::eval(
                            "requestAnimationFrame(() => {\
                                var el = document.querySelector('[data-share-url-input]');\
                                if (el) el.select();\
                            })"
                        );
                    },
                }

                // === Include focus field checkbox === //
                label {
                    class: CHECKBOX_ROW_CLASS,
                    input {
                        r#type: "checkbox",
                        checked: *include_focus.read() && has_focus_field,
                        disabled: !has_focus_field,
                        onchange: move |evt| {
                            include_focus.set(evt.checked());
                        },
                    }
                    if has_focus_field {
                        "Include focused field"
                    } else {
                        span {
                            class: "text-gray-500",
                            "Include focused field (none focused)"
                        }
                    }
                }

                // === Copy button === //
                div {
                    class: "flex flex-row justify-end",
                    button {
                        class: COPY_BTN_CLASS,
                        onclick: move |_| async move {
                            let url = full_url.read().clone();
                            match clipboard.set(url).await {
                                Ok(()) => {
                                    copied_label.set("Copied!");
                                    copied_signal.set(true);
                                }
                                Err(e) => {
                                    debug!("Failed to copy share URL: {:?}", e);
                                    copied_label.set("Failed");
                                    copied_signal.set(true);
                                }
                            }
                        },
                        "{copied_label}"
                    }
                }
            }
        }
    }
}
