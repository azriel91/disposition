//! Raw YAML text editor page.
//!
//! Shows the full serialized YAML of the [`InputDiagram`] in a `<textarea>`.
//! The user can edit the YAML directly; if it deserializes successfully the
//! structured [`InputDiagram`] signal is updated immediately. If
//! deserialization fails, the parse error is displayed and a "Revert to last
//! good state" button lets the user roll back to the last successfully
//! deserialized YAML.

use dioxus::{
    hooks::{use_memo, use_signal},
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::input_model::InputDiagram;

/// CSS classes for the textarea.
const TEXTAREA_CLASS: &str = "\
    w-full \
    min-h-80 \
    flex-1 \
    rounded-lg \
    border-2 \
    border-gray-600 \
    bg-gray-800 \
    text-gray-200 \
    p-2 \
    font-mono \
    text-sm \
    text-nowrap \
    focus:border-blue-400 \
    focus:outline-none\
";

/// CSS classes for the error banner.
const ERROR_BANNER: &str = "\
    rounded-lg \
    border \
    border-red-700 \
    bg-red-950 \
    text-red-300 \
    p-2 \
    text-sm \
    font-mono \
    whitespace-pre-wrap\
";

/// CSS classes for the revert button.
const REVERT_BTN: &str = "\
    mt-1 \
    px-3 py-1 \
    rounded \
    bg-yellow-700 \
    hover:bg-yellow-600 \
    text-yellow-100 \
    text-sm \
    font-semibold \
    cursor-pointer \
    select-none\
";

/// The **Text** editor page.
///
/// Displays the full [`InputDiagram`] as YAML in a `<textarea>`.
///
/// ## Data-flow
///
/// ```text
/// InputDiagram signal  ──serialize──►  text_buffer signal
///       ▲                                    │
///       │                                    │ (user edits)
///       │                                    ▼
///       └──── if parse OK ◄── try deserialize
///                                if parse ERR ──► show error + revert button
/// ```
///
/// `last_good_yaml` tracks the most recent YAML string that was successfully
/// deserialized. If the user's edits break parsing, they can click "Revert"
/// to restore the text buffer to that last-good value.
#[component]
pub fn TextPage(input_diagram: Signal<InputDiagram<'static>>) -> Element {
    // The YAML text that is currently shown in the textarea.
    // Initialised from the current InputDiagram.
    let initial_yaml = {
        let d = input_diagram.read();
        serde_saphyr::to_string(&*d)
            .unwrap_or_default()
            .trim()
            .to_owned()
    };
    let mut text_buffer = use_signal(|| initial_yaml.clone());

    // The last YAML string that was successfully deserialized.
    let mut last_good_yaml = use_signal(|| initial_yaml);

    // Current parse error (if any).
    let mut parse_error: Signal<Option<String>> = use_signal(|| None);

    // When the InputDiagram changes from *outside* this page (e.g. another
    // editor tab modified it), we re-serialize into the text buffer -- but only
    // when the text buffer currently represents a valid (i.e. non-errored)
    // state, so we don't stomp over the user's in-progress edits that have a
    // parse error.
    use_memo(move || {
        let d = input_diagram.read();
        if parse_error.peek().is_none() {
            let yaml = serde_saphyr::to_string(&*d)
                .unwrap_or_default()
                .trim()
                .to_owned();
            if *text_buffer.peek() != yaml {
                text_buffer.set(yaml.clone());
                last_good_yaml.set(yaml);
            }
        }
    });

    let current_text = text_buffer.read().clone();
    let current_error = parse_error.read().clone();

    rsx! {
        div {
            class: "flex flex-col gap-2 flex-1",

            h3 {
                class: "text-sm font-bold text-gray-300",
                "Input Diagram (YAML)"
            }
            p {
                class: "text-xs text-gray-500 mb-1",
                "Edit the full InputDiagram as YAML. Changes are applied live when the YAML is valid."
            }

            textarea {
                class: TEXTAREA_CLASS,
                value: "{current_text}",
                oninput: move |evt| {
                    let new_text = evt.value();
                    text_buffer.set(new_text.clone());

                    // Try to deserialize.
                    match serde_saphyr::from_str::<InputDiagram<'static>>(&new_text) {
                        Ok(diagram) => {
                            // Successful parse -- update the diagram signal and
                            // record this as the last good YAML.
                            parse_error.set(None);
                            last_good_yaml.set(new_text);
                            input_diagram.set(diagram);
                        }
                        Err(e) => {
                            // Failed to parse -- show error but don't touch the
                            // diagram signal.
                            parse_error.set(Some(e.to_string()));
                        }
                    }
                },
            }

            // ── Error banner + revert button ─────────────────────────
            if let Some(err) = current_error {
                div {
                    class: "flex flex-col gap-1",

                    div {
                        class: ERROR_BANNER,
                        "⚠️ YAML parse error:\n{err}"
                    }

                    button {
                        class: REVERT_BTN,
                        onclick: move |_| {
                            let good = last_good_yaml.read().clone();
                            text_buffer.set(good.clone());
                            parse_error.set(None);
                            // Re-deserialize the last-good YAML to make sure
                            // the diagram is in sync (it should already be, but
                            // this is defensive).
                            if let Ok(diagram) = serde_saphyr::from_str::<InputDiagram<'static>>(&good) {
                                input_diagram.set(diagram);
                            }
                        },
                        "⮌ Revert to last good state"
                    }
                }
            }
        }
    }
}
