//! Raw YAML text editor page.
//!
//! Shows the full serialized YAML of the [`InputDiagram`] in a [`CodeMirror`]
//! editor with LSP-backed completion. The user can edit the YAML directly; if it
//! deserializes successfully the structured [`InputDiagram`] signal is updated
//! immediately. If deserialization fails, the parse error is displayed and a
//! "Revert to last good state" button lets the user roll back to the last
//! successfully deserialized YAML.
//!
//! [`CodeMirror`]: dioxus_codemirror::CodeMirror

use dioxus::{
    hooks::{use_effect, use_memo, use_signal},
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal, WritableExt},
};
use dioxus_codemirror::{CodeMirror, Language, LspBridge, Theme};
use disposition::input_model::InputDiagram;

use crate::{hooks::use_dark_mode, lsp_server::DispositionLspServer};

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
/// Displays the full [`InputDiagram`] as YAML in a [`CodeMirror`] editor, with
/// LSP-backed key / value completion served by an in-page `disposition_lsp`
/// language server (see [`DispositionLspServer`]).
///
/// ## Data-flow
///
/// ```text
/// InputDiagram signal  --serialize-->  text_buffer signal  <-->  CodeMirror
///       ^                                    |
///       |                                    | (user edits)
///       |                                    v
///       +---- if parse OK <-- try deserialize
///                                if parse ERR --> show error + revert button
/// ```
///
/// `last_good_yaml` tracks the most recent YAML string that was successfully
/// deserialized. If the user's edits break parsing, they can click "Revert"
/// to restore the text buffer to that last-good value.
///
/// A `self_update` flag breaks the cyclic propagation: when the text -> diagram
/// effect sets `input_diagram`, the flag is raised so the `use_memo` that
/// watches `input_diagram` knows the change originated here and skips
/// re-serializing back into the text buffer (which would reformat the user's
/// in-progress text and move their cursor).
#[component]
pub fn TextPage(input_diagram: Signal<InputDiagram<'static>>) -> Element {
    // The YAML text that is currently shown in the editor.
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

    // Flag: when `true`, the most recent `input_diagram` change was caused by
    // *this* component's text -> diagram effect. The `use_memo` below checks the
    // flag and skips re-serialization when the change originated locally.
    let mut self_update: Signal<bool> = use_signal(|| false);

    // === Theme === //
    // Track the shared dark-mode signal so the editor's palette follows the
    // rest of the app.
    let is_dark = use_dark_mode();

    // === In-page language server === //
    // Must be created unconditionally from the component body (the bridge uses
    // `use_hook` / `use_callback` internally).
    let lsp_server = use_signal(DispositionLspServer::new);
    let lsp = LspBridge::lsp_bridge_from_server_async("file:///diagram.yaml", lsp_server);

    // === text -> diagram === //
    // When the editor content changes, try to deserialize it. Updating
    // `input_diagram` only when the parsed diagram actually differs keeps
    // memo-driven re-serializations (diagram -> text) from looping back here.
    use_effect(move || {
        let text = text_buffer.read().clone();

        match serde_saphyr::from_str::<InputDiagram<'static>>(&text) {
            Ok(diagram) => {
                parse_error.set(None);
                last_good_yaml.set(text);

                if diagram != *input_diagram.peek() {
                    // Raise the flag *before* setting input_diagram so the memo
                    // sees it and skips re-serialization.
                    self_update.set(true);
                    input_diagram.set(diagram);
                }
            }
            Err(e) => {
                // Failed to parse -- show error but don't touch the diagram.
                parse_error.set(Some(e.to_string()));
            }
        }
    });

    // === diagram -> text === //
    // When the InputDiagram changes from *outside* this page (e.g. another
    // editor tab modified it, or the URL round-trips the state), re-serialize
    // into the text buffer -- but only when:
    //
    // 1. The change did NOT originate from this component (`self_update` is false).
    // 2. The text buffer currently parses (no parse error), so we don't stomp
    //    over the user's in-progress edits.
    use_memo(move || {
        let d = input_diagram.read();

        if *self_update.peek() {
            self_update.set(false);
            return;
        }

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

    let current_error = parse_error.read().clone();

    let theme = if *is_dark.read() {
        Theme::Dark
    } else {
        Theme::Light
    };

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

            div {
                class: "\
                    flex-1 \
                    min-h-80 \
                    overflow-auto \
                    rounded-lg \
                    border-2 \
                    border-gray-600 \
                    bg-gray-800 \
                    text-sm \
                    focus-within:border-blue-400\
                ",
                CodeMirror {
                    value: text_buffer,
                    language: Language::Yaml,
                    line_numbers: true,
                    theme,
                    lsp: Some(lsp),
                }
            }

            // === Error banner + revert button === //
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
